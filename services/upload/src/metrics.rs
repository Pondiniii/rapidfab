use once_cell::sync::Lazy;
use prometheus::{register_int_counter_vec, IntCounterVec, Opts};

// Allow dead code until metrics are used in endpoints
#[allow(dead_code)]
pub static UPLOAD_REQUESTS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        Opts::new("upload_requests_total", "Total upload requests"),
        &["scope", "status"]
    )
    .unwrap()
});

// Allow dead code until metrics are used in endpoints
#[allow(dead_code)]
pub static UPLOAD_BYTES: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        Opts::new("upload_bytes_total", "Total bytes uploaded"),
        &["scope"]
    )
    .unwrap()
});

// Allow dead code until metrics are used in endpoints
#[allow(dead_code)]
pub static RATE_LIMIT_HITS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        Opts::new(
            "upload_rate_limit_hits_total",
            "Rate limit hits by quota type"
        ),
        &["type"]
    )
    .unwrap()
});
