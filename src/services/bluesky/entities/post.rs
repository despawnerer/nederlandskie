use std::collections::HashMap;

use anyhow::{anyhow, Error, Result};

use crate::services::bluesky::internals::cbor::CborValue;

pub struct PostRecord {
    pub langs: Option<Vec<String>>,
    pub text: String,
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
            langs: map.remove("langs").map(|value| value.try_into()).transpose()?,
        })
    }
}
