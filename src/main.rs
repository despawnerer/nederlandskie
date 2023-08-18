mod database;
mod frames;
mod streaming;

use crate::streaming::start_stream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    start_stream().await?;

    Ok(())
}
