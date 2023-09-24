use anyhow::{anyhow, Error, Result};
use sk_cbor::Value;

type CborMap = Vec<(Value, Value)>;

pub struct PostRecord {
    pub langs: Option<Vec<String>>,
    pub text: String,
}

impl TryFrom<&CborMap> for PostRecord {
    type Error = Error;

    fn try_from(root: &CborMap) -> Result<Self> {
        let mut text: Option<&str> = None;
        let mut langs: Option<Vec<&str>> = None;

        for (key, value) in iter_string_keys(root) {
            match key {
                "text" => text = Some(string(value)?),
                "langs" => langs = Some(array_of_strings(value)?),
                _ => continue,
            }
        }

        Ok(PostRecord {
            text: text
                .ok_or_else(|| anyhow!("Missing field: text"))?
                .to_owned(),
            langs: langs.map(|v| v.into_iter().map(str::to_owned).collect()),
        })
    }
}

pub struct LikeRecord {
    pub subject: Subject,
}

impl TryFrom<&CborMap> for LikeRecord {
    type Error = Error;

    fn try_from(root: &CborMap) -> Result<Self> {
        let mut subject = None;

        for (key, value) in iter_string_keys(root) {
            match key {
                "subject" => subject = Some(map(value)?.try_into()?),
                _ => continue,
            }
        }

        Ok(LikeRecord {
            subject: subject.ok_or_else(|| anyhow!("Missing field: subject"))?,
        })
    }
}

pub struct Subject {
    pub cid: String,
    pub uri: String,
}

impl TryFrom<&CborMap> for Subject {
    type Error = Error;

    fn try_from(root: &CborMap) -> Result<Self> {
        let mut cid = None;
        let mut uri = None;

        for (key, value) in iter_string_keys(root) {
            match key {
                "cid" => cid = Some(string(value)?),
                "uri" => uri = Some(string(value)?),
                _ => continue,
            }
        }

        Ok(Subject {
            cid: cid.ok_or_else(|| anyhow!("Missing field: cid"))?.to_owned(),
            uri: uri.ok_or_else(|| anyhow!("Missing field: uri"))?.to_owned(),
        })
    }
}

pub struct FollowRecord {
    pub subject: String,
}

impl TryFrom<&CborMap> for FollowRecord {
    type Error = Error;

    fn try_from(root: &CborMap) -> Result<Self> {
        let mut subject = None;

        for (key, value) in iter_string_keys(root) {
            match key {
                "subject" => subject = Some(string(value)?),
                _ => continue,
            }
        }

        Ok(FollowRecord {
            subject: subject
                .ok_or_else(|| anyhow!("Missing field: subject"))?
                .to_owned(),
        })
    }
}

pub fn read_record<T: for<'a> TryFrom<&'a CborMap, Error = Error>>(bytes: &[u8]) -> Result<T> {
    let root = match sk_cbor::read(bytes) {
        Err(_) => return Err(anyhow!("Could not decode anything")),
        Ok(v) => v,
    };

    let root_map = match root {
        Value::Map(m) => m,
        _ => return Err(anyhow!("Expected root object to be a map")),
    };

    (&root_map).try_into()
}

fn iter_string_keys(map: &CborMap) -> impl Iterator<Item = (&str, &Value)> {
    map.iter().flat_map(|(k, v)| match k {
        Value::TextString(k) => Some((k.as_str(), v)),
        _ => None,
    })
}

fn map(value: &Value) -> Result<&CborMap> {
    match value {
        Value::Map(m) => Ok(m),
        _ => Err(anyhow!("Expected a map")),
    }
}

fn string(value: &Value) -> Result<&str> {
    match value {
        Value::TextString(value) => Ok(value.as_str()),
        _ => Err(anyhow!("Expected string")),
    }
}

fn array_of_strings(value: &Value) -> Result<Vec<&str>> {
    match value {
        Value::Array(vec) => {
            let mut res = Vec::with_capacity(vec.len());
            for vec_value in vec {
                res.push(string(vec_value)?)
            }
            Ok(res)
        }
        _ => Err(anyhow!("Expected array")),
    }
}
