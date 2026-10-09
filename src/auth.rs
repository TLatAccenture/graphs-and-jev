use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

pub struct Limiter {
    per_minute: u32,
    hits: Mutex<VecDeque<Instant>>,
}

impl Limiter {
    pub fn new(per_minute: u32) -> Self {
        Self {
            per_minute,
            hits: Mutex::new(VecDeque::new()),
        }
    }
}

pub fn guard(limiter: &Limiter) -> bool {
    let now = Instant::now();
    let mut hits = limiter.hits.lock().expect("rate limit lock");
    while hits
        .front()
        .is_some_and(|t| now.duration_since(*t) > Duration::from_secs(60))
    {
        hits.pop_front();
    }
    if hits.len() as u32 >= limiter.per_minute {
        return false;
    }
    hits.push_back(now);
    true
}

pub fn error(status: StatusCode, detail: &str) -> Response {
    (status, Json(json!({ "detail": detail }))).into_response()
}
