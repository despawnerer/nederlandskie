pub fn profiles_classified() {
    metrics::counter!("profiles_classified_total").increment(1);
}

pub fn profiles_pending(n: usize) {
    metrics::gauge!("profiles_pending").set(n as f64);
}
