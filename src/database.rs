use chrono::{DateTime, Utc};

pub struct Post {
    indexed_at: DateTime<Utc>,
    author_did: String,
    cid: String,
    uri: String,
}

pub struct Profile {
    first_seen_at: DateTime<Utc>,
    did: String,
    handle: String,
    likely_country_of_living: String,
}

pub struct SubscriptionState {
    service: String,
    cursor: i64,
}
