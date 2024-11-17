use std::fmt::Debug;
use std::matches;
use std::time::Duration;

use anyhow::Result;
use atrium_api::agent::{store::MemorySessionStore, AtpAgent};
use atrium_api::types::string::Datetime;
use atrium_api::types::{BlobRef, Collection, Object, TryIntoUnknown};
use atrium_xrpc_client::reqwest::ReqwestClient;
use http::StatusCode;
use log::error;
use tokio_stream::StreamExt;
use tokio_tungstenite::{connect_async, tungstenite};

use super::entities::ProfileDetails;
use super::streaming::{handle_message, CommitProcessor};

pub struct Bluesky {
    agent: AtpAgent<MemorySessionStore, ReqwestClient>,
}

impl Bluesky {
    pub const XRPC_HOST: &'static str = "https://bsky.social";
    pub const FIREHOSE_HOST: &'static str = "wss://bsky.network";
    pub const STREAMING_TIMEOUT: Duration = Duration::from_secs(60);

    pub fn unauthenticated() -> Self {
        Self {
            agent: AtpAgent::new(
                ReqwestClient::new(Self::XRPC_HOST),
                MemorySessionStore::default(),
            ),
        }
    }

    pub async fn login(handle: &str, password: &str) -> Result<Self> {
        let agent = AtpAgent::new(
            ReqwestClient::new(Self::XRPC_HOST),
            MemorySessionStore::default(),
        );
        agent.login(handle, password).await?;

        Ok(Self { agent })
    }

    pub async fn upload_blob(&self, blob: Vec<u8>) -> Result<BlobRef> {
        let result = self.agent.api.com.atproto.repo.upload_blob(blob).await?;

        Ok(result.data.blob)
    }

    pub async fn publish_feed(
        &self,
        publisher_did: &str,
        feed_generator_did: &str,
        name: &str,
        display_name: &str,
        description: &str,
        avatar: Option<BlobRef>,
    ) -> Result<()> {
        use atrium_api::com::atproto::repo::put_record::InputData;

        self.agent
            .api
            .com
            .atproto
            .repo
            .put_record(
                InputData {
                    collection: atrium_api::app::bsky::feed::Generator::nsid(),
                    record: atrium_api::app::bsky::feed::generator::RecordData {
                        avatar,
                        created_at: Datetime::now(),
                        description: Some(description.to_owned()),
                        description_facets: None,
                        did: feed_generator_did.parse().map_err(anyhow::Error::msg)?,
                        display_name: display_name.to_owned(),
                        labels: None,
                        accepts_interactions: None,
                    }
                    .try_into_unknown()?,
                    repo: publisher_did.parse().map_err(anyhow::Error::msg)?,
                    rkey: name.to_owned(),
                    swap_commit: None,
                    swap_record: None,
                    validate: None,
                }
                .into(),
            )
            .await?;

        Ok(())
    }

    pub async fn fetch_profile_details(&self, did: &str) -> Result<Option<ProfileDetails>> {
        use atrium_api::com::atproto::repo::get_record::ParametersData;

        let result = self
            .agent
            .api
            .com
            .atproto
            .repo
            .get_record(
                ParametersData {
                    collection: atrium_api::app::bsky::actor::Profile::nsid(),
                    cid: None,
                    repo: did.parse().map_err(anyhow::Error::msg)?,
                    rkey: "self".to_owned(),
                }
                .into(),
            )
            .await;

        let profile_output = match result {
            Ok(profile_output) => profile_output,
            Err(e) if is_missing_repo_error(&e) => return Ok(None),
            Err(e) if is_record_not_found_error(&e) => return Ok(None),
            Err(e) => return Err(e.into()),
        };

        Ok(Some(ProfileDetails::try_from(profile_output.data.value)?))
    }

    pub async fn resolve_handle(&self, handle: &str) -> Result<Option<String>> {
        use atrium_api::com::atproto::identity::resolve_handle::ParametersData;

        let result = self
            .agent
            .api
            .com
            .atproto
            .identity
            .resolve_handle(Object::from(ParametersData {
                handle: handle.parse().map_err(anyhow::Error::msg)?,
            }))
            .await;

        match result {
            Ok(result) => Ok(Some(result.did.to_string())),
            Err(e) if is_unable_to_resolve_handle_error(&e) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn subscribe_to_operations<P: CommitProcessor>(
        &self,
        processor: &P,
        cursor: Option<i64>,
    ) -> Result<()> {
        let url = match cursor {
            Some(cursor) => format!(
                "{}/xrpc/com.atproto.sync.subscribeRepos?cursor={}",
                Self::FIREHOSE_HOST,
                cursor
            ),
            None => format!(
                "{}/xrpc/com.atproto.sync.subscribeRepos",
                Self::FIREHOSE_HOST
            ),
        };

        let (stream, _) = connect_async(url).await?;
        let stream = stream.timeout(Self::STREAMING_TIMEOUT);
        let mut stream = Box::pin(stream);

        while let Some(Ok(tungstenite::Message::Binary(message))) = stream.try_next().await? {
            if let Err(e) = handle_message(&message, processor).await {
                error!("Error handling a message: {:?}", e);
            }
        }

        Ok(())
    }
}

fn is_missing_repo_error<T>(error: &atrium_xrpc::error::Error<T>) -> bool
where
    T: Debug,
{
    use atrium_xrpc::error::{Error, ErrorResponseBody, XrpcError, XrpcErrorKind};

    matches!(error,
        Error::XrpcResponse(XrpcError {
            status,
            error:
                Some(XrpcErrorKind::Undefined(ErrorResponseBody {
                    error: Some(error_code),
                    message: Some(error_message),
                })),
        }) if
            // FIXME: This is this way instead of pattern matching because atrium's
            //        version of http is pegged at like 0.2.x and it does not
            //        re-export it so we have no way of referencing the real type
            status.as_u16() == StatusCode::BAD_REQUEST.as_u16()
            && error_code == "InvalidRequest"
            && error_message.starts_with("Could not find repo")
    )
}

fn is_record_not_found_error<T>(error: &atrium_xrpc::error::Error<T>) -> bool
where
    T: Debug,
{
    use atrium_xrpc::error::{Error, ErrorResponseBody, XrpcError, XrpcErrorKind};

    matches!(error,
        Error::XrpcResponse(XrpcError {
            status,
            error:
                Some(XrpcErrorKind::Undefined(ErrorResponseBody {
                    error: Some(error_code),
                    message: Some(error_message),
                })),
        }) if
            // FIXME: This is this way instead of pattern matching because atrium's
            //        version of http is pegged at like 0.2.x and it does not
            //        re-export it so we have no way of referencing the real type
            status.as_u16() == StatusCode::BAD_REQUEST.as_u16()
            && error_code == "RecordNotFound"
    )
}

fn is_unable_to_resolve_handle_error<T>(error: &atrium_xrpc::error::Error<T>) -> bool
where
    T: Debug,
{
    use atrium_xrpc::error::{Error, ErrorResponseBody, XrpcError, XrpcErrorKind};

    matches!(error,
        Error::XrpcResponse(XrpcError {
            status,
            error:
                Some(XrpcErrorKind::Undefined(ErrorResponseBody {
                    error: Some(error_code),
                    message: Some(error_message),
                })),
        }) if
            // FIXME: This is this way instead of pattern matching because atrium's
            //        version of http is pegged at like 0.2.x and it does not
            //        re-export it so we have no way of referencing the real type
            status.as_u16() == StatusCode::BAD_REQUEST.as_u16()
            && error_code == "InvalidRequest"
            && error_message.starts_with("Unable to resolve handle")
    )
}
