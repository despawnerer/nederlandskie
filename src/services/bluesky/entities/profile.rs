use atrium_api::app::bsky::actor::profile::Record as ProfileRecord;

#[derive(Debug)]
pub struct ProfileDetails {
    pub display_name: String,
    pub description: String,
}

impl From<ProfileRecord> for ProfileDetails {
    fn from(value: ProfileRecord) -> Self {
        Self {
            display_name: value.display_name.unwrap_or_default(),
            description: value.description.unwrap_or_default()
        }
    }
}
