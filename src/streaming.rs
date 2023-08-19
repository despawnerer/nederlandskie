use anyhow::Result;

use crate::frames::Frame;
use anyhow::anyhow;
use atrium_api::app::bsky::feed::post::Record;
use atrium_api::com::atproto::sync::subscribe_repos::Commit;
use atrium_api::com::atproto::sync::subscribe_repos::Message;
use futures::StreamExt;
use tokio_tungstenite::{connect_async, tungstenite};

pub async fn start_stream() -> Result<()> {
    let (mut stream, _) =
        connect_async("wss://bsky.social/xrpc/com.atproto.sync.subscribeRepos").await?;

    while let Some(Ok(tungstenite::Message::Binary(message))) = stream.next().await {
        if let Err(e) = handle_message(&message).await {
            println!("Error handling a message: {:?}", e);
        }
    }

    Ok(())
}

async fn handle_message(message: &[u8]) -> Result<()> {
    let commit = match parse_commit_from_message(&message)? {
        Some(commit) => commit,
        None => return Ok(()),
    };

    let post_operations = extract_post_operations(&commit).await?;
    for operation in &post_operations {
        println!("{:?}", operation);
    }

    Ok(())
}

fn parse_commit_from_message(message: &[u8]) -> Result<Option<Commit>> {
    match Frame::try_from(message)? {
        Frame::Message(message) => match message.body {
            Message::Commit(commit) => Ok(Some(*commit)),
            _ => Ok(None),
        },
        Frame::Error(err) => panic!("Frame error: {err:?}"),
    }
}

#[derive(Debug)]
enum PostOperation {
    Create {
        author_did: String,
        cid: String,
        uri: String,
        languages: Vec<String>,
        text: String,
    },
    Delete {
        cid: String,
    },
}

async fn extract_post_operations(commit: &Commit) -> Result<Vec<PostOperation>> {
    let mut operations = Vec::new();

    let (items, _) = rs_car::car_read_all(&mut commit.blocks.as_slice(), true).await?;
    for op in &commit.ops {
        let collection = op.path.split('/').next().expect("op.path is empty");
        if (op.action != "create" && op.action != "delete") || collection != "app.bsky.feed.post" {
            continue;
        }

        let cid = op.cid.expect("cid is not there, what").to_string();

        if let Some((_, item)) = items.iter().find(|(cid, _)| Some(*cid) == op.cid) {
            let record: Record = ciborium::from_reader(&mut item.as_slice())?;

            operations.push(match op.action.as_str() {
                "create" => PostOperation::Create {
                    languages: record.langs.unwrap_or_else(Vec::new),
                    text: record.text,
                    author_did: commit.repo.clone(),
                    cid,
                    uri: format!("at://{}/{}", commit.repo, op.path),
                },
                "delete" => PostOperation::Delete { cid },
                _ => unreachable!(),
            });
        }
    }

    Ok(operations)
}
