mod auth;
mod catalogue;
mod decide;
mod graph;
mod pipeline;
mod request;
pub mod research;

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

type ResearchFn = dyn Fn(&str) -> Result<Value, research::ResearchError> + Send + Sync;

pub struct App {
    cat: Catalogue,
    min_confidence: f64,
    max_state: usize,
    limiter: Limiter,
    log: Arc<dyn Fn(String) + Send + Sync>,
    research: Arc<ResearchFn>,
}

impl App {
    pub fn load(
        catalogue_path: &std::path::Path,
        min_confidence: f64,
        max_state: usize,
        rate: u32,
    ) -> Result<Self, String> {
        if !(0.0..=1.0).contains(&min_confidence) {
            return Err("MIN_CONFIDENCE must be in [0, 1].".into());
        }
        Ok(Self {
            cat: Catalogue::load(catalogue_path)?,
            min_confidence,
            max_state,
            limiter: Limiter::new(rate),
            log: Arc::new(|line| println!("{line}")),
            research: Arc::new(|request| research::run(request, &research::Live)),
        })
    }

    pub fn with_log_sink(mut self, log: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.log = Arc::new(log);
        self
    }

    pub fn with_research(
        mut self,
        research: impl Fn(&str) -> Result<Value, research::ResearchError> + Send + Sync + 'static,
    ) -> Self {
        self.research = Arc::new(research);
        self
    }

    pub fn warm(&self) {
        let _ = pipeline::run(
            &self.cat,
            "Annual CO2 for Australia in 2024",
            None,
            self.min_confidence,
        );
    }
}

#[derive(Clone, Debug)]
pub struct LogMetadata {
    pub outcome: &'static str,
    pub upstream_error_class: &'static str,
}

fn with_log_metadata(
    mut response: Response,
    outcome: &'static str,
    upstream_error_class: &'static str,
) -> Response {
    response.extensions_mut().insert(LogMetadata {
        outcome,
        upstream_error_class,
    });
    response
}

pub fn classified_error(
    status: StatusCode,
    detail: &str,
    upstream_error_class: &'static str,
) -> Response {
    with_log_metadata(
        error(status, detail),
        "upstream_error",
        upstream_error_class,
    )
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
        if self.questions.len() > 100 {
            return Err(ValidationError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "At most 100 questions are allowed.",
            ));
        }
        if self.questions.keys().any(|name| name.chars().count() > 100) {
            return Err(ValidationError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "Question names must be at most 100 characters.",
            ));
        }
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
        Ok(value) => with_log_metadata(
            (StatusCode::OK, Json(value)).into_response(),
            "decided",
            "none",
        ),
        Err(e) => error(StatusCode::UNPROCESSABLE_ENTITY, &e.0),
    }
}

async fn run_pipeline(State(app): State<Arc<App>>, Json(body): Json<PipelineIn>) -> Response {
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
        let research = Arc::clone(&app.research);
        let joined = tokio::task::spawn_blocking(move || research(&owned)).await;
        return match joined {
            Ok(Ok(v)) => decision_response(v),
            Ok(Err(e)) => research_unavailable(e.class()),
            Err(_) => classified_error(
                StatusCode::BAD_GATEWAY,
                "Could not complete the retrieval task.",
                "task",
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
        Ok(v) => decision_response(v),
        Err(e) => error(StatusCode::UNPROCESSABLE_ENTITY, &e.0),
    }
}

fn research_unavailable(class: &'static str) -> Response {
    let status = if class == "timeout" {
        StatusCode::SERVICE_UNAVAILABLE
    } else {
        StatusCode::BAD_GATEWAY
    };
    with_log_metadata(
        (
            status,
            Json(json!({
                "outcome": "unavailable",
                "error_class": class,
                "message": "Research sources are temporarily unavailable.",
                "citations": [],
                "graph": {"nodes": [], "edges": []},
            })),
        )
            .into_response(),
        "unavailable",
        class,
    )
}

fn decision_response(value: Value) -> Response {
    let outcome = match value.get("outcome").and_then(Value::as_str) {
        Some("answer") => "answer",
        Some("clarify") => "clarify",
        Some("review") => "review",
        Some("reject") => "reject",
        Some("no_data") => "no_data",
        _ => "answer",
    };
    with_log_metadata(
        (StatusCode::OK, Json(value)).into_response(),
        outcome,
        "none",
    )
}

fn static_service(static_dir: &std::path::Path) -> ServeDir {
    ServeDir::new(static_dir).append_index_html_on_directories(true)
}

pub fn router(app: Arc<App>, static_dir: &std::path::Path) -> Router {
    let expensive = Router::new()
        .route("/api/decide", post(decide))
        .route("/api/pipeline", post(run_pipeline))
        .layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
        .route_layer(middleware::from_fn_with_state(app.clone(), limit_expensive));
    Router::new()
        .route("/api/health", get(health))
        .route("/api/ready", get(ready))
        .route("/api/catalogue", get(catalogue))
        .merge(expensive)
        .fallback_service(static_service(static_dir))
        .with_state(app.clone())
        .layer(middleware::from_fn_with_state(app, log_request))
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

async fn limit_expensive(
    State(app): State<Arc<App>>,
    request: Request<Body>,
    next: Next,
) -> Response {
    match check(&app) {
        Some(response) => response,
        None => next.run(request).await,
    }
}

async fn log_request(State(app): State<Arc<App>>, request: Request<Body>, next: Next) -> Response {
    let started = Instant::now();
    let route = request.uri().path().to_owned();
    let request_id = format!("r-{}", REQUEST_SEQUENCE.fetch_add(1, Ordering::Relaxed));
    let mut response = next.run(request).await;
    let status = response.status();
    let metadata = response.extensions().get::<LogMetadata>().cloned();
    let outcome = metadata.as_ref().map_or_else(
        || match status {
            StatusCode::PAYLOAD_TOO_LARGE => "payload_too_large",
            StatusCode::UNPROCESSABLE_ENTITY => "invalid_request",
            StatusCode::TOO_MANY_REQUESTS => "rate_limited",
            StatusCode::NOT_FOUND => "not_found",
            _ if status.is_success() => "success",
            _ => "error",
        },
        |metadata| metadata.outcome,
    );
    let upstream_error_class = metadata
        .as_ref()
        .map_or("none", |metadata| metadata.upstream_error_class);
    (app.log)(
        log_fields(
            &request_id,
            &route,
            status,
            started.elapsed().as_millis(),
            outcome,
            upstream_error_class,
        )
        .to_string(),
    );
    response.headers_mut().insert(
        HeaderName::from_static("x-request-id"),
        HeaderValue::from_str(&request_id).expect("request id header"),
    );
    response
}
