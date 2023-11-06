use std::matches;

use anyhow::{anyhow, Result};
use atrium_api::blob::BlobRef;
use atrium_api::records::Record;
use atrium_api::agent::{AtpAgent, Session};
use atrium_xrpc::client::reqwest::ReqwestClient;
use axum::http::StatusCode;
use chrono::Utc;
use futures::StreamExt;
use log::error;
use tokio_tungstenite::{connect_async, tungstenite};

use super::entities::{ProfileDetails};
use super::streaming::{handle_message, CommitProcessor};

pub struct Bluesky {
    agent: AtpAgent<ReqwestClient>,
}

impl Bluesky {
    pub const XRPC_HOST: &str = "https://bsky.social";
    pub const FIREHOSE_HOST: &str = "wss://bsky.network";

    pub fn unauthenticated() -> Self {
        Self {
            agent: AtpAgent::new(ReqwestClient::new(Self::XRPC_HOST.to_owned()))
        }
    }

    pub async fn login(handle: &str, password: &str) -> Result<Self> {
        let agent = AtpAgent::new(ReqwestClient::new(Self::XRPC_HOST.to_owned()));
        agent.login(handle, password).await?;

        Ok(Self { agent })
    }

    pub fn session(&self) -> Option<Session> {
        self.agent.get_session()
    }

    pub async fn upload_blob(&self, blob: Vec<u8>) -> Result<BlobRef> {
        let result = self
            .agent
            .api
            .com
            .atproto
            .repo
            .upload_blob(blob)
            .await?;

        Ok(result.blob)
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
        use atrium_api::com::atproto::repo::put_record::Input;

        self.agent
            .api
            .com
            .atproto
            .repo
            .put_record(Input {
                collection: "app.bsky.feed.generator".to_owned(),
                record: Record::AppBskyFeedGenerator(Box::new(
                    atrium_api::app::bsky::feed::generator::Record {
                        avatar,
                        created_at: Utc::now().to_rfc3339(),
                        description: Some(description.to_owned()),
                        description_facets: None,
                        did: feed_generator_did.to_owned(),
                        display_name: display_name.to_owned(),
                        labels: None,
                    },
                )),
                repo: publisher_did.to_owned(),
                rkey: name.to_owned(),
                swap_commit: None,
                swap_record: None,
                validate: None,
            })
            .await?;

        Ok(())
    }

    pub async fn fetch_profile_details(&self, did: &str) -> Result<Option<ProfileDetails>> {
        let result = self
            .agent
            .api
            .com
            .atproto
            .repo
            .get_record(atrium_api::com::atproto::repo::get_record::Parameters {
                collection: "app.bsky.actor.profile".to_owned(),
                cid: None,
                repo: did.to_owned(),
                rkey: "self".to_owned(),
            })
            .await;

        let profile_data = match result {
            Ok(profile_data) => profile_data,
            Err(e) if is_missing_record_error(&e) => return Ok(None),
            Err(e) => return Err(e.into()),
        };

        match profile_data.value {
            Record::AppBskyActorProfile(profile) => Ok(Some(ProfileDetails::from(*profile))),
            _ => Err(anyhow!("Wrong type of record")),
        }
    }

    pub async fn resolve_handle(&self, handle: &str) -> Result<Option<String>> {
        use atrium_api::com::atproto::identity::resolve_handle::Parameters;

        let result = self
            .agent
            .api
            .com
            .atproto
            .identity
            .resolve_handle(Parameters {
                handle: handle.to_owned(),
            })
            .await;

        match result {
            Ok(result) => Ok(Some(result.did)),
            Err(e) if is_unable_to_resolve_handle_error(&e) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn subscribe_to_operations<P: CommitProcessor>(
        &self,
        processor: &P,
        cursor: Option<i32>,
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

        let (mut stream, _) = connect_async(url).await?;

        while let Some(Ok(tungstenite::Message::Binary(message))) = stream.next().await {
            if let Err(e) = handle_message(&message, processor).await {
                error!("Error handling a message: {:?}", e);
            }
        }

        Ok(())
    }
}

fn is_missing_record_error<T>(error: &atrium_xrpc::error::Error<T>) -> bool {
    use atrium_xrpc::error::{Error, ErrorResponseBody, XrpcError, XrpcErrorKind};

    matches!(error,
        Error::XrpcResponse(XrpcError {
            status: StatusCode::BAD_REQUEST,
            error:
                Some(XrpcErrorKind::Undefined(ErrorResponseBody {
                    error: Some(error_code),
                    message: Some(error_message),
                })),
        }) if error_code == "InvalidRequest"
            && error_message.starts_with("Could not locate record")
    )
}

fn is_unable_to_resolve_handle_error<T>(error: &atrium_xrpc::error::Error<T>) -> bool {
    use atrium_xrpc::error::{Error, ErrorResponseBody, XrpcError, XrpcErrorKind};

    matches!(error,
        Error::XrpcResponse(XrpcError {
            status: StatusCode::BAD_REQUEST,
            error:
                Some(XrpcErrorKind::Undefined(ErrorResponseBody {
                    error: Some(error_code),
                    message: Some(error_message),
                })),
        }) if error_code == "InvalidRequest"
            && error_message.starts_with("Unable to resolve handle")
    )
}
