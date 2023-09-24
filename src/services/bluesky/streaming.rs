use std::collections::{HashMap, HashSet};

use anyhow::Result;
use async_trait::async_trait;
use atrium_api::com::atproto::sync::subscribe_repos::{Commit, Message};

use super::{
    decode::{read_record, FollowRecord, LikeRecord, PostRecord},
    proto::Frame,
};

const COLLECTION_POST: &str = "app.bsky.feed.post";
const COLLECTION_LIKE: &str = "app.bsky.feed.like";
const COLLECTION_FOLLOW: &str = "app.bsky.graph.follow";

const ACTION_CREATE: &str = "create";
const ACTION_DELETE: &str = "delete";

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
    CreateLike {
        author_did: String,
        cid: String,
        uri: String,
        subject_cid: String,
        subject_uri: String,
    },
    CreateFollow {
        author_did: String,
        cid: String,
        uri: String,
        subject: String,
    },
    DeletePost {
        uri: String,
    },
    DeleteLike {
        uri: String,
    },
    DeleteFollow {
        uri: String,
    },
}

pub async fn handle_message<P: CommitProcessor>(message: &[u8], processor: &P) -> Result<()> {
    let commit = match parse_commit_from_message(message)? {
        Some(commit) => commit,
        None => return Ok(()),
    };

    let operations = extract_operations(&commit).await?;

    processor
        .process_commit(&CommitDetails {
            seq: commit.seq,
            operations,
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

    let (blocks, _header) = rs_car::car_read_all(&mut commit.blocks.as_slice(), true).await?;
    let blocks_by_cid: HashMap<_, _> = blocks.into_iter().collect();

    for op in &commit.ops {
        let collection = op.path.split('/').next().expect("op.path is empty");
        let action = op.action.as_str();
        let uri = format!("at://{}/{}", commit.repo, op.path);

        let operation = match action {
            ACTION_CREATE => {
                let cid = match op.cid {
                    Some(cid) => cid,
                    None => continue,
                };

                let block = match blocks_by_cid.get(&cid) {
                    Some(block) => block,
                    None => continue,
                };

                match collection {
                    COLLECTION_POST => {
                        let record: PostRecord = read_record(block)?;

                        Operation::CreatePost {
                            author_did: commit.repo.clone(),
                            cid: cid.to_string(),
                            uri,
                            languages: record.langs.unwrap_or_default().iter().cloned().collect(),
                            text: record.text,
                        }
                    }
                    COLLECTION_LIKE => {
                        let record: LikeRecord = read_record(block)?;

                        Operation::CreateLike {
                            author_did: commit.repo.clone(),
                            cid: cid.to_string(),
                            uri,
                            subject_cid: record.subject.cid,
                            subject_uri: record.subject.uri,
                        }
                    }
                    COLLECTION_FOLLOW => {
                        let record: FollowRecord = read_record(block)?;

                        Operation::CreateFollow {
                            author_did: commit.repo.clone(),
                            cid: cid.to_string(),
                            uri,
                            subject: record.subject,
                        }
                    }
                    _ => continue,
                }
            }
            ACTION_DELETE => match collection {
                COLLECTION_POST => Operation::DeletePost { uri },
                COLLECTION_LIKE => Operation::DeleteLike { uri },
                COLLECTION_FOLLOW => Operation::DeleteFollow { uri },
                _ => continue,
            },
            _ => continue,
        };

        operations.push(operation)
    }

    Ok(operations)
}
