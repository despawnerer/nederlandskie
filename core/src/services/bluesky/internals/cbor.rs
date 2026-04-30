use anyhow::Result;
use serde::de::DeserializeOwned;

pub fn read_record<T: DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    Ok(serde_ipld_dagcbor::from_slice(bytes)?)
}
