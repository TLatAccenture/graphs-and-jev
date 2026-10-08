use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

pub struct Limiter {
    per_minute: u32,
    hits: Mutex<HashMap<String, VecDeque<Instant>>>,
}

impl Limiter {
    pub fn new(per_minute: u32) -> Self {
        Self {
            per_minute,
            hits: Mutex::new(HashMap::new()),
        }
    }
}

pub fn guard(
    token: Option<&str>,
    headers: &HeaderMap,
    addr: Option<SocketAddr>,
    limiter: &Limiter,
) -> Result<(), Response> {
    if let Some(expected) = token {
        let ok = headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            == Some(&format!("Bearer {expected}"));
        if !ok {
            return Err(error(
                StatusCode::UNAUTHORIZED,
                "Missing or wrong token. Send Authorization: Bearer <API_TOKEN>.",
            ));
        }
    }
    let ip = addr
        .map(|a| a.ip().to_string())
        .unwrap_or_else(|| "unknown".into());
    let now = Instant::now();
    let mut hits = limiter.hits.lock().expect("rate limit lock");
    let q = hits.entry(ip).or_default();
    while q
        .front()
        .is_some_and(|t| now.duration_since(*t) > Duration::from_secs(60))
    {
        q.pop_front();
    }
    if q.len() as u32 >= limiter.per_minute {
        return Err(error(
            StatusCode::TOO_MANY_REQUESTS,
            "Too many requests. Wait a minute and try again.",
        ));
    }
    q.push_back(now);
    Ok(())
}

pub fn error(status: StatusCode, detail: &str) -> Response {
    (status, Json(json!({ "detail": detail }))).into_response()
}
