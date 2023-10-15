use std::collections::HashMap;

use anyhow::{anyhow, Error, Result};

use crate::services::bluesky::internals::cbor::CborValue;

pub struct LikeRecord {
    pub subject: Subject,
}

impl TryFrom<CborValue> for LikeRecord {
    type Error = Error;

    fn try_from(root: CborValue) -> Result<Self> {
        let mut map: HashMap<_, _> = root.try_into()?;

        Ok(LikeRecord {
            subject: map
                .remove("subject")
                .ok_or_else(|| anyhow!("Missing field: subject"))?
                .try_into()?,
        })
    }
}

pub struct Subject {
    pub cid: String,
    pub uri: String,
}

impl TryFrom<CborValue> for Subject {
    type Error = Error;

    fn try_from(root: CborValue) -> Result<Self> {
        let mut map: HashMap<_, _> = root.try_into()?;

        Ok(Subject {
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
