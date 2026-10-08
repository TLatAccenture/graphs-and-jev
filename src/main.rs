mod auth;
mod catalogue;
mod decide;
mod graph;
mod pipeline;
mod request;
mod research;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::{ConnectInfo, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use tower_http::services::ServeDir;

use crate::auth::{error, guard, Limiter};
use crate::catalogue::Catalogue;

const VERSION: &str = "0.1.0";

struct App {
    cat: Catalogue,
    token: Option<String>,
    min_confidence: f64,
    max_state: usize,
    limiter: Limiter,
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key)
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| default.to_string())
}

#[derive(Deserialize)]
struct DecideIn {
    backend: String,
    state: Value,
    questions: serde_json::Map<String, Value>,
}

#[derive(Deserialize)]
struct PipelineIn {
    backend: String,
    request: String,
    #[serde(default)]
    preference: Option<String>,
    #[serde(default)]
    domain: Option<String>,
    #[serde(default)]
    use_llm: bool,
}

fn check(
    app: &App,
    headers: &HeaderMap,
    addr: Option<SocketAddr>,
) -> Result<(), axum::response::Response> {
    guard(app.token.as_deref(), headers, addr, &app.limiter)
}

async fn health(State(app): State<Arc<App>>) -> Json<Value> {
    Json(json!({
        "ok": true,
        "version": VERSION,
        "auth_required": app.token.is_some(),
        "backend": "catalogue",
        "sources": app.cat.sources.len(),
        "pollutants": app.cat.pollutants.len(),
    }))
}

async fn ready(State(app): State<Arc<App>>) -> Json<Value> {
    Json(json!({"ok": !app.cat.sources.is_empty() && !app.cat.pollutants.is_empty()}))
}

async fn catalogue(State(app): State<Arc<App>>) -> Json<Value> {
    let pollutants: Vec<Value> = app.cat.pollutants.iter().map(|p| json!({"id": p.id, "label": p.label, "name": p.name, "unit": p.unit, "groups": p.groups})).collect();
    let sources: Vec<Value> = app.cat.sources.iter().map(|s| {
        let coverage: serde_json::Map<String, Value> = s.coverage.iter().map(|(k, v)| {
            let span = if v.is_empty() { json!([]) } else { json!([v.first(), v.last(), v.len()]) };
            (k.clone(), span)
        }).collect();
        json!({"id": s.id, "name": s.name, "description": s.description, "publisher": s.publisher, "updated": s.updated, "levels": s.levels, "pollutants": s.pollutants, "coverage": coverage})
    }).collect();
    Json(json!({
        "pollutants": pollutants,
        "groups": app.cat.groups.iter().map(|(k, g)| (k.clone(), json!(g.label))).collect::<serde_json::Map<String, Value>>(),
        "dimensions": app.cat.dimensions,
        "defaults": app.cat.default_level,
        "summary": app.cat.summary,
        "sources": sources,
    }))
}

async fn decide(
    State(app): State<Arc<App>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<DecideIn>,
) -> axum::response::Response {
    if let Err(resp) = check(&app, &headers, Some(addr)) {
        return resp;
    }
    if body.backend != "catalogue" {
        return error(
            StatusCode::NOT_FOUND,
            "Unknown backend. Available: catalogue",
        );
    }
    let size = serde_json::to_string(&body.state)
        .map(|s| s.len())
        .unwrap_or(0);
    if size > app.max_state {
        return error(
            StatusCode::PAYLOAD_TOO_LARGE,
            &format!("State is longer than {} characters.", app.max_state),
        );
    }
    match crate::decide::decide(&app.cat, &body.state, &body.questions) {
        Ok(v) => (StatusCode::OK, Json(v)).into_response(),
        Err(e) => error(StatusCode::UNPROCESSABLE_ENTITY, &e.0),
    }
}

async fn run_pipeline(
    State(app): State<Arc<App>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<PipelineIn>,
) -> axum::response::Response {
    if let Err(resp) = check(&app, &headers, Some(addr)) {
        return resp;
    }
    if body.backend != "catalogue" {
        return error(
            StatusCode::NOT_FOUND,
            "Unknown backend. Available: catalogue",
        );
    }
    let request = body.request.trim();
    if request.is_empty() || request.chars().count() > 500 {
        return error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "Request must be 1 to 500 characters.",
        );
    }
    let preference = body
        .preference
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    if preference.is_some_and(|s| s.chars().count() > 300) {
        return error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "Preference must be at most 300 characters.",
        );
    }
    let _ = body.use_llm;
    let domain = body
        .domain
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("air");
    if domain == "parkinsons" {
        let owned = request.to_string();
        let joined =
            tokio::task::spawn_blocking(move || research::run(&owned, &research::Live)).await;
        return match joined {
            Ok(Ok(v)) => (StatusCode::OK, Json(v)).into_response(),
            Ok(Err(e)) => error(StatusCode::BAD_GATEWAY, &e.0),
            Err(_) => error(
                StatusCode::BAD_GATEWAY,
                "Could not reach the web to retrieve sources.",
            ),
        };
    }
    if domain != "air" {
        return error(
            StatusCode::NOT_FOUND,
            "Unknown domain. Available: air, parkinsons",
        );
    }
    match pipeline::run(&app.cat, request, preference, app.min_confidence) {
        Ok(v) => (StatusCode::OK, Json(v)).into_response(),
        Err(e) => error(StatusCode::UNPROCESSABLE_ENTITY, &e.0),
    }
}

fn static_service(static_dir: &std::path::Path) -> ServeDir {
    ServeDir::new(static_dir).append_index_html_on_directories(true)
}

#[tokio::main]
async fn main() {
    let catalogue_path = PathBuf::from(env_or("CATALOGUE_PATH", "catalogue.json"));
    let static_dir = PathBuf::from(env_or("STATIC_DIR", "static"));
    let cat = Catalogue::load(&catalogue_path).unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(1);
    });
    let min_confidence: f64 = env_or("MIN_CONFIDENCE", "0.8").parse().unwrap_or(0.8);
    if !(0.0..=1.0).contains(&min_confidence) {
        eprintln!("MIN_CONFIDENCE must be in [0, 1].");
        std::process::exit(1);
    }
    let app = Arc::new(App {
        cat,
        token: std::env::var("API_TOKEN").ok().filter(|v| !v.is_empty()),
        min_confidence,
        max_state: env_or("MAX_STATE_CHARS", "4000").parse().unwrap_or(4000),
        limiter: Limiter::new(
            env_or("RATE_LIMIT_PER_MINUTE", "120")
                .parse()
                .unwrap_or(120),
        ),
    });
    let port: u16 = env_or("PORT", "8080").parse().unwrap_or(8080);
    let _ = pipeline::run(
        &app.cat,
        "Annual CO2 for Australia in 2024",
        None,
        app.min_confidence,
    );
    let router = Router::new()
        .route("/api/health", get(health))
        .route("/api/ready", get(ready))
        .route("/api/catalogue", get(catalogue))
        .route("/api/decide", post(decide))
        .route("/api/pipeline", post(run_pipeline))
        .fallback_service(static_service(&static_dir))
        .with_state(app);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    println!("listening on http://127.0.0.1:{port}");
    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("serve");
}

#[cfg(test)]
mod static_route_tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    #[tokio::test]
    async fn missing_static_asset_returns_not_found() {
        let static_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static");
        let response = static_service(&static_dir)
            .oneshot(
                Request::builder()
                    .uri("/assets/does-not-exist.svg")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("static response");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}

use axum::response::IntoResponse;
