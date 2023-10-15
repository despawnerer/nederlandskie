use std::collections::HashMap;

use anyhow::{anyhow, Error, Result};

use crate::services::bluesky::internals::cbor::CborValue;

#[derive(Debug)]
pub struct PostRecord {
    pub text: String,
    pub langs: Option<Vec<String>>,
    pub reply: Option<ReplyRef>,
}

impl TryFrom<CborValue> for PostRecord {
    type Error = Error;

    fn try_from(root: CborValue) -> Result<Self> {
        let mut map: HashMap<_, _> = root.try_into()?;

        Ok(PostRecord {
            text: map
                .remove("text")
                .ok_or_else(|| anyhow!("Missing field: text"))?
                .try_into()?,
            langs: map
                .remove("langs")
                .map(|value| value.try_into())
                .transpose()?,
            reply: map.remove("reply")
                .map(|value| value.try_into())
                .transpose()?,
        })
    }
}

#[derive(Debug)]
pub struct ReplyRef {
    pub parent: Ref,
    pub root: Ref,
}

impl TryFrom<CborValue> for ReplyRef {
    type Error = Error;

    fn try_from(root: CborValue) -> Result<Self> {
        let mut map: HashMap<_, _> = root.try_into()?;

        Ok(ReplyRef {
            parent: map
                .remove("parent")
                .ok_or_else(|| anyhow!("Missing field: parent"))?
                .try_into()?,
            root: map
                .remove("root")
                .ok_or_else(|| anyhow!("Missing field: root"))?
                .try_into()?,
        })
    }
}

#[derive(Debug)]
pub struct Ref {
    pub cid: String,
    pub uri: String,
}

impl TryFrom<CborValue> for Ref {
    type Error = Error;

    fn try_from(root: CborValue) -> Result<Self> {
        let mut map: HashMap<_, _> = root.try_into()?;

        Ok(Ref {
            cid: map
                .remove("cid")
                .ok_or_else(|| anyhow!("Missing field: cid"))?
                .try_into()?,
            uri: map
                .remove("uri")
                .ok_or_else(|| anyhow!("Missing field: uri"))?
                .try_into()?,
        })
    }
}
