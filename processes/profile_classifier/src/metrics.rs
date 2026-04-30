pub fn profiles_classified() {
    metrics::counter!("profiles_classified_total").increment(1);
}

pub fn profiles_pending(n: usize) {
    metrics::gauge!("profiles_pending").set(n as f64);
}

pub fn profiles_classification_failed(kind: &'static str) {
    metrics::counter!("profiles_classification_errors_total", "kind" => kind).increment(1);
}
