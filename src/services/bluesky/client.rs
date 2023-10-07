use std::matches;
use std::sync::Arc;
use std::sync::Mutex;

use anyhow::{anyhow, Result};
use atrium_api::blob::BlobRef;
use atrium_api::client::AtpServiceClient;
use atrium_api::client::AtpServiceWrapper;
use atrium_api::records::Record;
use axum::http::StatusCode;
use chrono::Utc;
use futures::StreamExt;
use log::error;
use tokio_tungstenite::{connect_async, tungstenite};

use super::session::Session;
use super::streaming::{handle_message, CommitProcessor};
use super::xrpc_client::AuthenticateableXrpcClient;

#[derive(Debug)]
pub struct ProfileDetails {
    pub display_name: String,
    pub description: String,
}

pub struct Bluesky {
    client: AtpServiceClient<AtpServiceWrapper<AuthenticateableXrpcClient>>,
    session: Option<Arc<Mutex<Session>>>,
}

impl Bluesky {
    pub fn unauthenticated(host: &str) -> Self {
        Self {
            client: AtpServiceClient::new(AuthenticateableXrpcClient::new(host.to_owned())),
            session: None,
        }
    }

    pub async fn login(host: &str, handle: &str, password: &str) -> Result<Self> {
        use atrium_api::com::atproto::server::create_session::Input;

        let client = AtpServiceClient::new(AuthenticateableXrpcClient::new(host.to_owned()));

        let result = client
            .service
            .com
            .atproto
            .server
            .create_session(Input {
                identifier: handle.to_owned(),
                password: password.to_owned(),
            })
            .await?;

        let session = Arc::new(Mutex::new(result.try_into()?));

        let authenticated_client = AtpServiceClient::new(AuthenticateableXrpcClient::with_session(
            host.to_owned(),
            session.clone(),
        ));

        Ok(Self {
            client: authenticated_client,
            session: Some(session),
        })
    }

    pub fn session(&self) -> Option<Session> {
        self.session
            .as_ref()
            .and_then(|s| s.lock().ok())
            .map(|s| s.clone())
    }

    pub async fn upload_blob(&self, blob: Vec<u8>) -> Result<BlobRef> {
        self.ensure_token_valid().await?;

        let result = self
            .client
            .service
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

        self.ensure_token_valid().await?;

        self.client
            .service
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
            .client
            .service
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

        let profile = match profile_data.value {
            Record::AppBskyActorProfile(profile) => profile,
            _ => return Err(anyhow!("Big bad, no such profile")),
        };

        Ok(Some(ProfileDetails {
            display_name: profile.display_name.unwrap_or_default(),
            description: profile.description.unwrap_or_default(),
        }))
    }

    pub async fn resolve_handle(&self, handle: &str) -> Result<Option<String>> {
        use atrium_api::com::atproto::identity::resolve_handle::Parameters;

        let result = self
            .client
            .service
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
                "wss://bsky.social/xrpc/com.atproto.sync.subscribeRepos?cursor={}",
                cursor
            ),
            None => "wss://bsky.social/xrpc/com.atproto.sync.subscribeRepos".to_owned(),
        };

        let (mut stream, _) = connect_async(url).await?;

        while let Some(Ok(tungstenite::Message::Binary(message))) = stream.next().await {
            if let Err(e) = handle_message(&message, processor).await {
                error!("Error handling a message: {:?}", e);
            }
        }

        Ok(())
    }

    async fn ensure_token_valid(&self) -> Result<()> {
        let access_jwt_exp = self
            .session
            .as_ref()
            .ok_or_else(|| anyhow!("Not authenticated"))?
            .lock()
            .map_err(|e| anyhow!("session mutex is poisoned: {e}"))?
            .access_jwt_exp;

        let jwt_expired = Utc::now() > access_jwt_exp;

        if jwt_expired {
            let refreshed = self
                .client
                .service
                .com
                .atproto
                .server
                .refresh_session()
                .await?;

            let mut session = self
                .session
                .as_ref()
                .ok_or_else(|| anyhow!("Not authenticated"))?
                .lock()
                .map_err(|e| anyhow!("session mutex is poisoned: {e}"))?;

            *session = refreshed.try_into()?;
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
