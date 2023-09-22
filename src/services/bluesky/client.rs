use anyhow::{anyhow, Result};
use atrium_api::blob::BlobRef;
use atrium_api::client::AtpServiceClient;
use atrium_api::client::AtpServiceWrapper;
use atrium_api::records::Record;
use atrium_xrpc::client::reqwest::ReqwestClient;
use chrono::Utc;
use futures::StreamExt;
use log::error;
use tokio_tungstenite::{connect_async, tungstenite};

use super::streaming::{handle_message, CommitProcessor};

#[derive(Debug)]
pub struct ProfileDetails {
    pub display_name: String,
    pub description: String,
}

#[derive(Debug)]
pub struct SessionDetails {
    pub did: String,
}

pub struct Bluesky {
    client: AtpServiceClient<AtpServiceWrapper<ReqwestClient>>,
}

impl Bluesky {
    pub fn new(host: &str) -> Self {
        Self {
            client: AtpServiceClient::new(ReqwestClient::new(host.to_owned())),
        }
    }

    pub async fn login(&self, handle: &str, password: &str) -> Result<SessionDetails> {
        use atrium_api::com::atproto::server::create_session::Input;

        let result = self
            .client
            .service
            .com
            .atproto
            .server
            .create_session(Input {
                identifier: handle.to_owned(),
                password: password.to_owned(),
            })
            .await?;

        Ok(SessionDetails { did: result.did })
    }

    pub async fn upload_blob(&self, blob: Vec<u8>) -> Result<BlobRef> {
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
                        created_at: Utc::now().to_string(),
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

    pub async fn fetch_profile_details(&self, did: &str) -> Result<ProfileDetails> {
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
            .await?;

        let profile = match result.value {
            Record::AppBskyActorProfile(profile) => profile,
            _ => return Err(anyhow!("Big bad, no such profile")),
        };

        Ok(ProfileDetails {
            display_name: profile.display_name.unwrap_or_default(),
            description: profile.description.unwrap_or_default(),
        })
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
}
