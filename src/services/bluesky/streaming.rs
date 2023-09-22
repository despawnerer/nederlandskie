use std::collections::HashSet;

use anyhow::Result;
use async_trait::async_trait;

use super::proto::Frame;
use anyhow::anyhow;
use atrium_api::app::bsky::feed::post::Record;
use atrium_api::com::atproto::sync::subscribe_repos::{Commit, Message};

#[async_trait]
pub trait CommitProcessor {
    async fn process_commit(&self, commit: &CommitDetails) -> Result<()>;
}

pub struct CommitDetails {
    pub seq: i32,
    pub operations: Vec<Operation>,
}

#[derive(Debug)]
pub enum Operation {
    CreatePost {
        author_did: String,
        cid: String,
        uri: String,
        languages: HashSet<String>,
        text: String,
    },
    DeletePost {
        uri: String,
    },
}

pub async fn handle_message<P: CommitProcessor>(message: &[u8], processor: &P) -> Result<()> {
    let commit = match parse_commit_from_message(&message)? {
        Some(commit) => commit,
        None => return Ok(()),
    };

    let operations = extract_operations(&commit).await?;

    processor
        .process_commit(&CommitDetails {
            seq: commit.seq,
            operations: operations,
        })
        .await?;

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

async fn extract_operations(commit: &Commit) -> Result<Vec<Operation>> {
    let mut operations = Vec::new();

    let (items, _) = rs_car::car_read_all(&mut commit.blocks.as_slice(), true).await?;
    for op in &commit.ops {
        let collection = op.path.split('/').next().expect("op.path is empty");
        if (op.action != "create" && op.action != "delete") || collection != "app.bsky.feed.post" {
            continue;
        }

        let uri = format!("at://{}/{}", commit.repo, op.path);

        if let Some((_, item)) = items.iter().find(|(cid, _)| Some(*cid) == op.cid) {
            let record: Record = ciborium::from_reader(&mut item.as_slice())?;

            operations.push(match op.action.as_str() {
                "create" => Operation::CreatePost {
                    languages: record
                        .langs
                        .unwrap_or_else(Vec::new)
                        .iter()
                        .cloned()
                        .collect(),
                    text: record.text,
                    author_did: commit.repo.clone(),
                    cid: op
                        .cid
                        .ok_or(anyhow!(
                            "cid is not present for a post create operation, how is that possible"
                        ))?
                        .to_string(),
                    uri,
                },
                "delete" => Operation::DeletePost { uri },
                _ => unreachable!(),
            });
        }
    }

    Ok(operations)
}
