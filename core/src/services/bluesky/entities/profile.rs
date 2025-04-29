use std::ops::Deref;

use atrium_api::{app::bsky::actor::profile::RecordData as ProfileRecordData, types::Unknown};
use ipld_core::ipld::Ipld;

#[derive(Debug)]
pub struct ProfileDetails {
    pub display_name: String,
    pub description: String,
}

impl From<ProfileRecordData> for ProfileDetails {
    fn from(value: ProfileRecordData) -> Self {
        Self {
            display_name: value.display_name.unwrap_or_default(),
            description: value.description.unwrap_or_default(),
        }
    }
}

impl TryFrom<Unknown> for ProfileDetails {
    type Error = anyhow::Error;

    fn try_from(value: Unknown) -> Result<Self, Self::Error> {
        let string_or_empty = |value: &Unknown, key: &str| match value {
            Unknown::Object(map) => match map.get(key) {
                Some(x) => match x.deref() {
                    Ipld::String(s) => s.clone(),
                    _ => "".to_owned(),
                },
                _ => "".to_owned(),
            },
            _ => "".to_owned(),
        };

        Ok(ProfileDetails {
            display_name: string_or_empty(&value, "displayName"),
            description: string_or_empty(&value, "description"),
        })
    }
}
