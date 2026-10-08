use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use serve::{router, App};
use tower::ServiceExt;

fn app(rate: u32) -> (axum::Router, Arc<Mutex<Vec<String>>>) {
    app_with_research(rate, |_| unreachable!("research path not used"))
}

fn app_with_research(
    rate: u32,
    research: impl Fn(&str) -> Result<Value, serve::research::ResearchError> + Send + Sync + 'static,
) -> (axum::Router, Arc<Mutex<Vec<String>>>) {
    let logs = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&logs);
    let state = Arc::new(
        App::load(
            &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("catalogue.json"),
            0.8,
            4_000,
            rate,
        )
        .unwrap()
        .with_log_sink(move |line| sink.lock().unwrap().push(line))
        .with_research(research),
    );
    (
        router(
            state,
            &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static"),
        ),
        logs,
    )
}

async fn post(app: axum::Router, path: &str, body: Value) -> axum::response::Response {
    app.oneshot(
        Request::builder()
            .method("POST")
            .uri(path)
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap(),
    )
    .await
    .unwrap()
}

#[tokio::test]
async fn oversized_json_is_rejected_at_the_boundary() {
    let response = post(
        app(10).0,
        "/api/pipeline",
        json!({"backend":"catalogue", "request":"x".repeat(70_000)}),
    )
    .await;
    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
}

#[tokio::test]
async fn pipeline_rejects_overlong_request_and_preference() {
    let response = post(
        app(10).0,
        "/api/pipeline",
        json!({"backend":"catalogue", "request":"x".repeat(501)}),
    )
    .await;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let response = post(
        app(10).0,
        "/api/pipeline",
        json!({"backend":"catalogue", "request":"ok", "preference":"x".repeat(301)}),
    )
    .await;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn decide_rejects_overlong_request_preference_and_question_fields() {
    for body in [
        json!({"backend":"catalogue", "state":{"request":"x".repeat(501)}, "questions":{}}),
        json!({"backend":"catalogue", "state":{"preference":"x".repeat(301)}, "questions":{}}),
        json!({"backend":"catalogue", "state":{}, "questions":{"gate":{"instructions":"x".repeat(501)}}}),
    ] {
        let response = post(app(10).0, "/api/decide", body).await;
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}

#[tokio::test]
async fn expensive_routes_share_a_global_rate_bucket() {
    let (app, _) = app(1);
    let first = post(
        app.clone(),
        "/api/decide",
        json!({"backend":"catalogue", "state":{"request":"ok"}, "questions":{}}),
    )
    .await;
    assert_eq!(first.status(), StatusCode::OK);
    let second = post(
        app,
        "/api/pipeline",
        json!({"backend":"catalogue", "request":"ok"}),
    )
    .await;
    assert_eq!(second.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn health_and_readiness_bypass_expensive_route_limit() {
    let (app, _) = app(0);
    for path in ["/api/health", "/api/ready"] {
        let response = app
            .clone()
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK, "{path}");
    }
    let response = post(
        app,
        "/api/decide",
        json!({"backend":"catalogue", "state":{}, "questions":{}}),
    )
    .await;
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn quota_rejection_precedes_json_extraction() {
    let (router, logs) = app(0);
    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/pipeline")
                .header("content-type", "application/json")
                .body(Body::from("not-json"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    assert!(logs
        .lock()
        .unwrap()
        .join("\n")
        .contains(r#""outcome":"rate_limited""#));
}

#[tokio::test]
async fn decide_rejects_oversized_question_name_and_excess_count() {
    let response = post(
        app(10).0,
        "/api/decide",
        json!({"backend":"catalogue", "state":{}, "questions": {"x".repeat(101): {"type":"choice","labels":["yes","no"]}}}),
    )
    .await;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let questions = (0..101)
        .map(|index| {
            (
                format!("q{index}"),
                json!({"type":"choice","labels":["yes","no"]}),
            )
        })
        .collect::<serde_json::Map<_, _>>();
    let response = post(
        app(10).0,
        "/api/decide",
        json!({"backend":"catalogue", "state":{}, "questions": questions}),
    )
    .await;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn pipeline_log_captures_decision_outcome_without_raw_inputs() {
    let request = "SENTINEL_REQUEST annual CO2 for Australia in 2024";
    let preference = "SENTINEL_PREFERENCE prefer national data";
    let (router, logs) = app(10);
    let response = post(
        router,
        "/api/pipeline",
        json!({"backend":"catalogue", "request":request, "preference":preference}),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.headers().contains_key("x-request-id"));
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let outcome = serde_json::from_slice::<Value>(&body).unwrap()["outcome"]
        .as_str()
        .unwrap()
        .to_owned();
    let captured = logs.lock().unwrap().join("\n");
    assert!(captured.contains(&format!(r#""outcome":"{outcome}""#)));
    assert!(!captured.contains(request));
    assert!(!captured.contains(preference));
}

#[tokio::test]
async fn middleware_rejections_have_meaningful_outcomes() {
    let (router, logs) = app(10);
    let response = post(
        router,
        "/api/pipeline",
        json!({"backend":"catalogue", "request":"x".repeat(70_000)}),
    )
    .await;
    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    assert!(logs
        .lock()
        .unwrap()
        .join("\n")
        .contains(r#""outcome":"payload_too_large""#));

    let (router, logs) = app(0);
    let response = post(
        router,
        "/api/decide",
        json!({"backend":"catalogue", "state":{}, "questions":{}}),
    )
    .await;
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    assert!(logs
        .lock()
        .unwrap()
        .join("\n")
        .contains(r#""outcome":"rate_limited""#));
}

#[tokio::test]
async fn retrieval_timeout_returns_bounded_unavailable_body_and_typed_log() {
    let (app, logs) = app_with_research(10, |_| {
        Err(serve::research::ResearchError::timeout(
            "internal timeout detail",
        ))
    });
    let response = post(
        app,
        "/api/pipeline",
        json!({"backend":"catalogue", "domain":"parkinsons", "request":"research question"}),
    )
    .await;
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body: Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 4_096).await.unwrap()).unwrap();
    assert_eq!(body["outcome"], "unavailable");
    assert_eq!(body["error_class"], "timeout");
    assert_eq!(
        body["message"],
        "Research sources are temporarily unavailable."
    );
    assert!(body["citations"].as_array().unwrap().is_empty());
    assert!(body["graph"]["nodes"].as_array().unwrap().is_empty());
    assert!(body["graph"]["edges"].as_array().unwrap().is_empty());
    assert!(serde_json::to_vec(&body).unwrap().len() < 1_024);
    let captured = logs.lock().unwrap().join("\n");
    assert!(captured.contains(r#""outcome":"unavailable""#));
    assert!(captured.contains(r#""upstream_error_class":"timeout""#));
    assert!(!captured.contains("internal timeout detail"));
}

#[tokio::test]
async fn concrete_upstream_classes_reach_structured_logs() {
    for class in ["timeout", "fetch", "invalid"] {
        let message = match class {
            "timeout" => "request timed out",
            "invalid" => "invalid json response",
            _ => "connection refused",
        };
        let (app, logs) = app_with_research(10, move |_| {
            let error = match class {
                "timeout" => serve::research::ResearchError::timeout(message),
                "invalid" => serve::research::ResearchError::invalid(message),
                _ => serve::research::ResearchError::fetch(message),
            };
            Err(error)
        });
        let response = post(
            app,
            "/api/pipeline",
            json!({"backend":"catalogue", "domain":"parkinsons", "request":"research question"}),
        )
        .await;
        let expected = if class == "timeout" {
            StatusCode::SERVICE_UNAVAILABLE
        } else {
            StatusCode::BAD_GATEWAY
        };
        assert_eq!(response.status(), expected);
        let captured = logs.lock().unwrap().join("\n");
        assert!(captured.contains(&format!(r#""upstream_error_class":"{class}""#)));
        assert!(!captured.contains(message));
    }
}
