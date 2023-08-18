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
        let commit = match parse_commit_message(&message) {
            Ok(Some(commit)) => commit,
            Ok(None) => continue,
            Err(e) => {
                println!("Couldn't parse commit: {:?}", e);
                continue;
            }
        };

        let post_messages = extract_post_messages(&commit).await;
        match post_messages {
            Ok(post_messages) => {
                if !post_messages.is_empty() {
                    println!("{:?}", post_messages);
                }
            }
            Err(e) => {
                println!("Coudln't extract post messages: {:?}", e);
            }
        }
    }

    Ok(())
}

fn parse_commit_message(message: &[u8]) -> Result<Option<Commit>> {
    match Frame::try_from(message)? {
        Frame::Message(message) => match message.body {
            Message::Commit(commit) => Ok(Some(*commit)),
            _ => Ok(None),
        },
        Frame::Error(err) => panic!("Frame error: {err:?}"),
    }
}

#[derive(Debug)]
enum Action {
    Create,
    Delete,
}

#[derive(Debug)]
struct PostMessage {
    action: Action,
    author_did: String,
    cid: String,
    uri: String,
    languages: Vec<String>,
    text: String,
}

async fn extract_post_messages(commit: &Commit) -> Result<Vec<PostMessage>> {
    let mut posts = Vec::new();

    let (items, _) = rs_car::car_read_all(&mut commit.blocks.as_slice(), true).await?;
    for op in &commit.ops {
        let collection = op.path.split('/').next().expect("op.path is empty");
        if (op.action != "create" && op.action != "delete") || collection != "app.bsky.feed.post" {
            continue;
        }

        if let Some((_, item)) = items.iter().find(|(cid, _)| Some(*cid) == op.cid) {
            let record: Record = ciborium::from_reader(&mut item.as_slice())?;

            posts.push(PostMessage {
                action: if op.action == "create" {
                    Action::Create
                } else {
                    Action::Delete
                },
                languages: record.langs.unwrap_or_else(Vec::new),
                text: record.text,
                author_did: commit.repo.clone(),
                cid: op.cid.expect("cid is not there, what").to_string(),
                uri: format!("at://{}/{}", commit.repo, op.path),
            })
        }
    }

    Ok(posts)
}
