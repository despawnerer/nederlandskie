use anyhow::anyhow;
use std::time::Duration;

use anyhow::Result;
use atrium_api::client::AtpServiceClient;
use atrium_api::xrpc::client::reqwest::ReqwestClient;

use crate::ai::{infer_country_of_living, AI};
use crate::database::{fetch_unprocessed_profile_dids, store_profile_details, ConnectionPool};

#[derive(Debug)]
struct ProfileDetails {
    display_name: String,
    description: String,
}

pub async fn classify_unclassified_profiles(db: ConnectionPool, ai: AI) -> Result<()> {
    loop {
        // TODO: Maybe streamify this so that each thing is processed in parallel
        // TODO: Also don't just exit this function when an error happens, just wait a minute or so?
        let dids = fetch_unprocessed_profile_dids(&db).await?;
        if dids.is_empty() {
            println!("No profiles to process: waiting 10 seconds");
            tokio::time::sleep(Duration::from_secs(10)).await;
        } else {
            for did in &dids {
                fill_in_profile_details(&db, &ai, did).await?;
            }
        }
    }
}

async fn fill_in_profile_details(db: &ConnectionPool, ai: &AI, did: &str) -> Result<()> {
    let details = fetch_profile_details(did).await?;
    let country = infer_country_of_living(ai, &details.display_name, &details.description).await?;
    store_profile_details(db, did, &country).await?;
    println!("Stored inferred country of living for {did}: {country}");
    Ok(())
}

async fn fetch_profile_details(did: &str) -> Result<ProfileDetails> {
    let client = AtpServiceClient::new(ReqwestClient::new("https://bsky.social".into()));

    let result = client
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
