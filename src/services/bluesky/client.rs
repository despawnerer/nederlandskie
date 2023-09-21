use anyhow::{anyhow, Result};
use atrium_api::client::AtpServiceClient;
use atrium_api::client::AtpServiceWrapper;
use atrium_xrpc::client::reqwest::ReqwestClient;
use futures::StreamExt;
use tokio_tungstenite::{connect_async, tungstenite};
use log::error;

use super::streaming::{handle_message, OperationProcessor};

#[derive(Debug)]
pub struct ProfileDetails {
    pub display_name: String,
    pub description: String,
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
            atrium_api::records::Record::AppBskyActorProfile(profile) => profile,
            _ => return Err(anyhow!("Big bad, no such profile")),
        };

        Ok(ProfileDetails {
            display_name: profile.display_name.unwrap_or_else(String::new),
            description: profile.description.unwrap_or_else(String::new),
        })
    }

    pub async fn subscribe_to_operations<P: OperationProcessor>(
        &self,
        processor: &P,
    ) -> Result<()> {
        let (mut stream, _) =
            connect_async("wss://bsky.social/xrpc/com.atproto.sync.subscribeRepos").await?;

        while let Some(Ok(tungstenite::Message::Binary(message))) = stream.next().await {
            if let Err(e) = handle_message(&message, processor).await {
                error!("Error handling a message: {:?}", e);
            }
        }

        Ok(())
    }
}
