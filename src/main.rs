mod auth;
mod catalogue;
mod decide;
mod graph;
mod pipeline;
mod request;
mod research;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

use axum::body::Body;
use axum::extract::{DefaultBodyLimit, State};
use axum::http::{HeaderName, HeaderValue, Request, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use tower_http::services::ServeDir;

use crate::auth::{error, guard, Limiter};
use crate::catalogue::Catalogue;

const VERSION: &str = "0.1.0";
const MAX_BODY_BYTES: usize = 64 * 1024;
const MAX_REQUEST_CHARS: usize = 500;
const MAX_PREFERENCE_CHARS: usize = 300;
static REQUEST_SEQUENCE: AtomicU64 = AtomicU64::new(1);

struct App {
    cat: Catalogue,
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

struct ValidationError {
    status: StatusCode,
    detail: String,
}

impl ValidationError {
    fn new(status: StatusCode, detail: impl Into<String>) -> Self {
        Self {
            status,
            detail: detail.into(),
        }
    }

    fn into_response(self) -> Response {
        error(self.status, &self.detail)
    }
}

impl DecideIn {
    fn validate(&self, max_state: usize) -> Result<(), ValidationError> {
        let state_size = serde_json::to_string(&self.state).map_or(0, |value| value.len());
        if state_size > max_state {
            return Err(ValidationError::new(
                StatusCode::PAYLOAD_TOO_LARGE,
                format!("State is longer than {max_state} characters."),
            ));
        }
        if let Some(state) = self.state.as_object() {
            if state
                .get("request")
                .and_then(Value::as_str)
                .is_some_and(|value| value.chars().count() > MAX_REQUEST_CHARS)
            {
                return Err(ValidationError::new(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "Decide request must be at most 500 characters.",
                ));
            }
            if state
                .get("preference")
                .and_then(Value::as_str)
                .is_some_and(|value| value.chars().count() > MAX_PREFERENCE_CHARS)
            {
                return Err(ValidationError::new(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "Decide preference must be at most 300 characters.",
                ));
            }
        }
        for value in self.questions.values() {
            validate_text_values(value)?;
        }
        Ok(())
    }
}

impl PipelineIn {
    fn validate(&self) -> Result<(&str, Option<&str>), ValidationError> {
        let request = self.request.trim();
        if request.is_empty() || request.chars().count() > MAX_REQUEST_CHARS {
            return Err(ValidationError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "Request must be 1 to 500 characters.",
            ));
        }
        let preference = self
            .preference
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        if preference.is_some_and(|value| value.chars().count() > MAX_PREFERENCE_CHARS) {
            return Err(ValidationError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "Preference must be at most 300 characters.",
            ));
        }
        Ok((request, preference))
    }
}

fn validate_text_values(value: &Value) -> Result<(), ValidationError> {
    match value {
        Value::String(value) if value.chars().count() > MAX_REQUEST_CHARS => {
            Err(ValidationError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "Decide text fields must be at most 500 characters.",
            ))
        }
        Value::Array(values) => values.iter().try_for_each(validate_text_values),
        Value::Object(values) => values.values().try_for_each(validate_text_values),
        _ => Ok(()),
    }
}

fn check(app: &App) -> Option<Response> {
    // ponytail: process-local limiting is only a useful ceiling through two instances;
    // move this bucket to a shared edge limiter before scaling beyond that.
    (!guard(&app.limiter)).then(|| {
        error(
            StatusCode::TOO_MANY_REQUESTS,
            "Too many requests. Wait a minute and try again.",
        )
    })
}

async fn health(State(app): State<Arc<App>>) -> Json<Value> {
    Json(json!({
        "ok": true,
        "version": VERSION,
        "auth_required": false,
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

async fn decide(State(app): State<Arc<App>>, Json(body): Json<DecideIn>) -> Response {
    if let Some(response) = check(&app) {
        return response;
    }
    if body.backend != "catalogue" {
        return error(
            StatusCode::NOT_FOUND,
            "Unknown backend. Available: catalogue",
        );
    }
    if let Err(failure) = body.validate(app.max_state) {
        return failure.into_response();
    }
    match crate::decide::decide(&app.cat, &body.state, &body.questions) {
        Ok(v) => (StatusCode::OK, Json(v)).into_response(),
        Err(e) => error(StatusCode::UNPROCESSABLE_ENTITY, &e.0),
    }
}

async fn run_pipeline(State(app): State<Arc<App>>, Json(body): Json<PipelineIn>) -> Response {
    if let Some(response) = check(&app) {
        return response;
    }
    if body.backend != "catalogue" {
        return error(
            StatusCode::NOT_FOUND,
            "Unknown backend. Available: catalogue",
        );
    }
    let (request, preference) = match body.validate() {
        Ok(validated) => validated,
        Err(failure) => return failure.into_response(),
    };
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

fn router(app: Arc<App>, static_dir: &std::path::Path) -> Router {
    let expensive = Router::new()
        .route("/api/decide", post(decide))
        .route("/api/pipeline", post(run_pipeline))
        .layer(DefaultBodyLimit::max(MAX_BODY_BYTES));
    Router::new()
        .route("/api/health", get(health))
        .route("/api/ready", get(ready))
        .route("/api/catalogue", get(catalogue))
        .merge(expensive)
        .fallback_service(static_service(static_dir))
        .with_state(app)
        .layer(middleware::from_fn(log_request))
}

fn log_fields(
    request_id: &str,
    route: &str,
    status: StatusCode,
    latency_ms: u128,
    outcome: &str,
    upstream_error_class: &str,
) -> Value {
    json!({
        "request_id": request_id,
        "route": route,
        "status": status.as_u16(),
        "latency_ms": latency_ms,
        "outcome": outcome,
        "upstream_error_class": upstream_error_class,
    })
}

async fn log_request(request: Request<Body>, next: Next) -> Response {
    let started = Instant::now();
    let route = request.uri().path().to_owned();
    let request_id = format!("r-{}", REQUEST_SEQUENCE.fetch_add(1, Ordering::Relaxed));
    let mut response = next.run(request).await;
    let status = response.status();
    let outcome = if status.is_success() {
        "success"
    } else {
        "error"
    };
    let upstream_error_class = if status == StatusCode::BAD_GATEWAY {
        "upstream"
    } else {
        "none"
    };
    println!(
        "{}",
        log_fields(
            &request_id,
            &route,
            status,
            started.elapsed().as_millis(),
            outcome,
            upstream_error_class,
        )
    );
    response.headers_mut().insert(
        HeaderName::from_static("x-request-id"),
        HeaderValue::from_str(&request_id).expect("request id header"),
    );
    response
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
    let router = router(app, &static_dir);
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
mod api_security_tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    fn test_app(rate: u32) -> Arc<App> {
        Arc::new(App {
            cat: Catalogue::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("catalogue.json"))
                .expect("catalogue"),
            min_confidence: 0.8,
            max_state: 4_000,
            limiter: Limiter::new(rate),
        })
    }

    fn test_router(rate: u32) -> Router {
        let static_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static");
        router(test_app(rate), &static_dir)
    }

    async fn post_request(path: &str, body: String, rate: u32) -> axum::response::Response {
        test_router(rate)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(path)
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .expect("request"),
            )
            .await
            .expect("response")
    }

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

    #[tokio::test]
    async fn responses_include_request_ids() {
        let response = test_router(10)
            .oneshot(
                Request::builder()
                    .uri("/api/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(response.headers().contains_key("x-request-id"));
    }

    #[test]
    fn structured_log_fields_exclude_sensitive_payloads() {
        let secret_request = "raw-request-secret";
        let secret_preference = "raw-preference-secret";
        let fields = log_fields(
            "r-1",
            "/api/pipeline",
            StatusCode::OK,
            12,
            "success",
            "none",
        );
        let encoded = fields.to_string();
        assert!(!encoded.contains(secret_request));
        assert!(!encoded.contains(secret_preference));
        for key in [
            "request_id",
            "route",
            "status",
            "latency_ms",
            "outcome",
            "upstream_error_class",
        ] {
            assert!(fields.get(key).is_some(), "missing {key}");
        }
    }

    #[tokio::test]
    async fn oversized_json_is_rejected_before_deserialization() {
        let response = post_request(
            "/api/pipeline",
            format!(
                r#"{{"backend":"catalogue","request":"{}"}}"#,
                "x".repeat(70_000)
            ),
            10,
        )
        .await;
        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }

    #[tokio::test]
    async fn pipeline_rejects_overlong_request_and_preference() {
        let request = post_request(
            "/api/pipeline",
            serde_json::to_string(&json!({"backend":"catalogue","request":"x".repeat(501)}))
                .unwrap(),
            10,
        )
        .await;
        assert_eq!(request.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let preference = post_request(
            "/api/pipeline",
            serde_json::to_string(
                &json!({"backend":"catalogue","request":"ok","preference":"x".repeat(301)}),
            )
            .unwrap(),
            10,
        )
        .await;
        assert_eq!(preference.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn decide_rejects_overlong_nested_fields() {
        let response = post_request(
            "/api/decide",
            serde_json::to_string(&json!({
                "backend":"catalogue",
                "state":{"request":"x".repeat(501)},
                "questions":{}
            }))
            .unwrap(),
            10,
        )
        .await;
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn decide_rejects_overlong_question_fields() {
        let response = post_request(
            "/api/decide",
            serde_json::to_string(&json!({
                "backend":"catalogue",
                "state":{},
                "questions":{"gate":{"instructions":"x".repeat(501)}}
            }))
            .unwrap(),
            10,
        )
        .await;
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn decide_rejects_overlong_preference() {
        let response = post_request(
            "/api/decide",
            serde_json::to_string(&json!({
                "backend":"catalogue",
                "state":{"preference":"x".repeat(301)},
                "questions":{}
            }))
            .unwrap(),
            10,
        )
        .await;
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn expensive_routes_share_a_global_rate_bucket() {
        let app = test_router(1);
        let first = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/decide")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"backend":"catalogue","state":{"request":"ok"},"questions":{}}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(first.status(), StatusCode::OK);
        let second = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/pipeline")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"backend":"catalogue","request":"ok"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(second.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[tokio::test]
    async fn health_and_readiness_bypass_expensive_route_limit() {
        let app = test_router(0);
        for path in ["/api/health", "/api/ready"] {
            let response = app
                .clone()
                .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK, "{path}");
        }
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/decide")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"backend":"catalogue","state":{},"questions":{}}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    }
}
