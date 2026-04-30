pub fn messages_received() {
    metrics::counter!("messages_received_total").increment(1);
}

pub fn messages_of_interest() {
    metrics::counter!("messages_of_interest_total").increment(1);
}

pub fn posts_indexed() {
    metrics::counter!("posts_indexed_total").increment(1);
}

pub fn posts_deleted() {
    metrics::counter!("posts_deleted_total").increment(1);
}
