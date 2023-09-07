mod processes;
mod services;

use anyhow::Result;

use crate::processes::post_saver::PostSaver;
use crate::processes::profile_classifier::ProfileClassifier;
use crate::services::ai::AI;
use crate::services::bluesky::Bluesky;
use crate::services::database::Database;

#[tokio::main]
async fn main() -> Result<()> {
    // TODO: Use env vars
    let ai = AI::new("fake-api-key", "https://api.openai.com");
    let bluesky = Bluesky::new("https://bsky.social");
    let database =
        Database::connect("postgres://postgres:password@localhost/nederlandskie").await?;

    let post_saver = PostSaver::new(&database, &bluesky);
    let profile_classifier = ProfileClassifier::new(&database, &ai, &bluesky);

    tokio::try_join!(post_saver.start(), profile_classifier.start())?;

    Ok(())
}
