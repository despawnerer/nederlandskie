use std::collections::HashMap;

use anyhow::{anyhow, Error, Result};
use sk_cbor::Value;

pub struct CborValue {
    inner: Value,
}

impl CborValue {
    pub fn new(inner: Value) -> CborValue {
        CborValue { inner }
    }
}

impl TryFrom<CborValue> for HashMap<String, CborValue> {
    type Error = Error;

    fn try_from(value: CborValue) -> Result<Self> {
        match value.inner {
            Value::Map(entries) => {
                let mut result = HashMap::with_capacity(entries.len());
                for (key, value) in entries {
                    result.insert(CborValue::new(key).try_into()?, CborValue::new(value));
                }
                Ok(result)
            }
            _ => Err(anyhow!("Not a map")),
        }
    }
}

impl TryFrom<CborValue> for String {
    type Error = Error;

    fn try_from(value: CborValue) -> Result<Self> {
        match value.inner {
            Value::TextString(value) => Ok(value),
            _ => Err(anyhow!("Expected string")),
        }
    }
}

impl TryFrom<CborValue> for Vec<String> {
    type Error = Error;

    fn try_from(value: CborValue) -> Result<Self> {
        match value.inner {
            Value::Array(vec) => {
                let mut res = Vec::with_capacity(vec.len());
                for vec_value in vec {
                    res.push(CborValue::new(vec_value).try_into()?)
                }
                Ok(res)
            }
            _ => Err(anyhow!("Expected array")),
        }
    }
}

pub fn read_record<T: TryFrom<CborValue, Error = Error>>(bytes: &[u8]) -> Result<T> {
    let root = match sk_cbor::read(bytes) {
        Err(_) => return Err(anyhow!("Could not decode anything")),
        Ok(v) => v,
    };

    CborValue::new(root).try_into()
}
