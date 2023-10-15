use std::collections::HashMap;

use anyhow::{anyhow, Error, Result};

use crate::services::bluesky::internals::cbor::CborValue;

pub struct FollowRecord {
    pub subject: String,
}

impl TryFrom<CborValue> for FollowRecord {
    type Error = Error;

    fn try_from(root: CborValue) -> Result<Self> {
        let mut map: HashMap<_, _> = root.try_into()?;

        Ok(FollowRecord {
            subject: map
                .remove("subject")
                .ok_or_else(|| anyhow!("Missing field: subject"))?
                .try_into()?,
        })
    }
}
