use std::collections::HashMap;
use std::path::PathBuf;

use serve::DecisionBackendConfig;

#[test]
fn defaults_to_catalogue_and_jev_requires_key_and_model() {
    let empty = HashMap::<String, String>::new();
    let config = DecisionBackendConfig::from_lookup(|key| empty.get(key).cloned()).unwrap();
    assert_eq!(config.name(), "catalogue");

    let jev = HashMap::from([
        ("DECISION_BACKEND".to_string(), "jev".to_string()),
        ("SERVICE_AUTH_MODE".to_string(), "cloud-run-iam".to_string()),
    ]);
    let error = DecisionBackendConfig::from_lookup(|key| jev.get(key).cloned()).unwrap_err();
    assert!(error.contains("TYPESAFE_API_KEY"));

    let no_model = HashMap::from([
        ("DECISION_BACKEND".to_string(), "jev".to_string()),
        ("TYPESAFE_API_KEY".to_string(), "secret".to_string()),
        ("SERVICE_AUTH_MODE".to_string(), "cloud-run-iam".to_string()),
    ]);
    let error = DecisionBackendConfig::from_lookup(|key| no_model.get(key).cloned()).unwrap_err();
    assert!(error.contains("JEV_MODEL"));
}

use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::body::{to_bytes, Body};
use axum::extract::State;
use axum::http::{HeaderMap, Request, StatusCode};
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Json, Router};
use serde_json::{json, Value};
use serve::{DecisionBackend, HostedJevBackend, HostedJevLimits, HostedJevSettings};
use tower::ServiceExt;

#[derive(Clone)]
struct Fake {
    calls: Arc<Mutex<Vec<(HeaderMap, Value)>>>,
    replies: Arc<Mutex<Vec<(StatusCode, Value)>>>,
}

async fn fake_handler(
    State(fake): State<Fake>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    fake.calls.lock().unwrap().push((headers, body));
    let (status, reply) = fake.replies.lock().unwrap().remove(0);
    (status, Json(reply))
}

async fn server(replies: Vec<(StatusCode, Value)>) -> (String, Fake) {
    let fake = Fake {
        calls: Arc::new(Mutex::new(Vec::new())),
        replies: Arc::new(Mutex::new(replies)),
    };
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = Router::new()
        .route("/v1/systemone", post(fake_handler))
        .with_state(fake.clone());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (format!("http://{address}"), fake)
}

fn choice_question() -> serde_json::Map<String, Value> {
    json!({"gate":{"type":"choice","instructions":"choose","criteria":{"answer":"ok","review":"not ok"}}}).as_object().unwrap().clone()
}

fn success() -> Value {
    json!({"model":"jev-1.2.3","answers":{"gate":{"type":"choice","probabilities":{"answer":0.9,"review":0.1},"choice":"answer","confidence":0.8}},"usage":{"input_tokens":100,"output_tokens":17}})
}

#[tokio::test]
async fn sends_exact_wire_and_returns_hosted_provenance() {
    let (base, fake) = server(vec![(StatusCode::OK, success())]).await;
    let backend = HostedJevBackend::for_test(
        "secret-key".into(),
        "jev-1.2.3".into(),
        base,
        Duration::from_secs(1),
        Some(10.0),
    )
    .unwrap();
    let state = json!({"request":"private state"});
    let questions = choice_question();
    let result = backend.decide(&state, &questions).await.unwrap();
    let calls = fake.calls.lock().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0["authorization"], "Bearer secret-key");
    assert_eq!(
        calls[0].1,
        json!({"model":"jev-1.2.3","state":state,"questions":questions})
    );
    assert_eq!(result["backend"], "jev");
    assert_eq!(result["residency"], "hosted");
    assert_eq!(result["input_tokens"], 100);
    assert_eq!(result["cost_usd"], 0.001);
    let latency_ms = result["latency_ms"].as_f64().unwrap();
    assert!(latency_ms.is_finite() && latency_ms >= 0.0);
    assert_eq!(result["detail"]["served_by"], "TypeSafe hosted Jev");
    assert_eq!(result["detail"]["usage"]["output_tokens"], 17);
    assert_eq!(
        result["answers"]["gate"],
        json!({"type":"choice","probabilities":{"answer":0.9,"review":0.1},"top":"answer","choice":"answer","confidence":0.8})
    );
}

#[tokio::test]
async fn choice_confidence_is_concentration_not_max_probability() {
    let answer = json!({"model":"jev-1.2.3","usage":{"input_tokens":1,"output_tokens":1},"answers":{"gate":{"type":"choice","probabilities":{"answer":0.32,"review":0.64,"reject":0.04},"choice":"review","confidence":0.46}}});
    let questions = json!({"gate":{"type":"choice","instructions":"choose","criteria":{"answer":"ok","review":"check","reject":"no"}}}).as_object().unwrap().clone();
    let (base, _) = server(vec![(StatusCode::OK, answer)]).await;
    let backend = HostedJevBackend::for_test(
        "key".into(),
        "jev-1.2.3".into(),
        base,
        Duration::from_secs(1),
        None,
    )
    .unwrap();
    let result = backend.decide(&json!({}), &questions).await.unwrap();
    assert_eq!(result["answers"]["gate"]["top"], "review");
    assert_eq!(result["answers"]["gate"]["confidence"], 0.46);
}

#[tokio::test]
async fn observed_jev_1_13_choice_fixtures_use_official_concentration() {
    let cases = [
        (
            "clarify",
            0.64,
            json!({"answer":0.06,"clarify":0.76,"reject":0.18}),
            329,
            40,
        ),
        (
            "answer",
            0.28,
            json!({"answer":0.52,"clarify":0.42,"reject":0.06}),
            335,
            0,
        ),
        (
            "clarify",
            0.62,
            json!({"answer":0.07,"clarify":0.75,"reject":0.18}),
            329,
            40,
        ),
    ];
    let replies = cases
        .iter()
        .map(|(choice, confidence, probabilities, input_tokens, output_tokens)| {
            (
                StatusCode::OK,
                json!({"model":"jev-1.13.0","answers":{"gate":{"type":"choice","probabilities":probabilities,"choice":choice,"confidence":confidence}},"usage":{"input_tokens":input_tokens,"output_tokens":output_tokens}}),
            )
        })
        .collect();
    let (base, _) = server(replies).await;
    let backend = HostedJevBackend::for_test(
        "key".into(),
        "jev-1.13.0".into(),
        base,
        Duration::from_secs(1),
        Some(10.0),
    )
    .unwrap();
    let questions = json!({"gate":{"type":"choice","instructions":"choose","criteria":{"answer":"ok","clarify":"ask","reject":"no"}}})
        .as_object()
        .unwrap()
        .clone();

    for (choice, confidence, probabilities, input_tokens, output_tokens) in cases {
        let result = backend.decide(&json!({}), &questions).await.unwrap();
        assert_eq!(
            result["answers"]["gate"],
            json!({"type":"choice","probabilities":probabilities,"top":choice,"choice":choice,"confidence":confidence})
        );
        assert_eq!(result["input_tokens"], input_tokens);
        assert_eq!(result["detail"]["usage"]["output_tokens"], output_tokens);
        let latency_ms = result["latency_ms"].as_f64().unwrap();
        assert!(latency_ms.is_finite() && latency_ms >= 0.0);
    }
}

#[tokio::test]
async fn confidence_rounding_tolerance_is_bounded_for_choice_and_score() {
    let cases = [
        (
            json!({"gate":{"type":"choice","probabilities":{"answer":0.07,"clarify":0.75,"reject":0.18},"choice":"clarify","confidence":0.619}}),
            json!({"gate":{"type":"choice","criteria":{"answer":"answer","clarify":"clarify","reject":"reject"}}}),
        ),
        (
            json!({"fit":{"type":"score","probabilities":{"0":0.0,"1":0.57,"2":0.43},"score":1.43,"confidence":0.349,"legend":{"0":"poor","1":"good","2":"best"}}}),
            json!({"fit":{"type":"score","criteria":["poor","good","best"]}}),
        ),
    ];
    for (answers, questions) in cases {
        let (base, _) = server(vec![(
            StatusCode::OK,
            json!({"model":"jev-1.13.0","usage":{"input_tokens":1,"output_tokens":1},"answers":answers}),
        )])
        .await;
        let backend = HostedJevBackend::for_test(
            "key".into(),
            "jev-1.13.0".into(),
            base,
            Duration::from_secs(1),
            None,
        )
        .unwrap();
        let questions = questions.as_object().unwrap().clone();
        assert!(backend.decide(&json!({}), &questions).await.is_err());
    }
}

#[tokio::test]
async fn official_score_confidence_examples_use_first_modal_index() {
    let examples = [
        (json!({"0":0.0,"1":0.5,"2":0.5}), 1.5, 0.25, "1"),
        (json!({"0":0.5,"1":0.0,"2":0.5}), 1.0, 0.0, "0"),
        (json!({"0":0.1,"1":0.2,"2":0.7}), 1.6, 0.4, "2"),
        (json!({"0":0.0,"1":0.57,"2":0.43}), 1.43, 0.355, "1"),
    ];
    let replies = examples.iter().map(|(probabilities, score, confidence, _)| (
        StatusCode::OK,
        json!({"model":"jev-1.2.3","usage":{"input_tokens":1,"output_tokens":1},"answers":{"fit":{"type":"score","probabilities":probabilities,"score":score,"confidence":confidence,"legend":{"0":"poor","1":"good","2":"best"}}}}),
    )).collect();
    let (base, _) = server(replies).await;
    let backend = HostedJevBackend::for_test(
        "key".into(),
        "jev-1.2.3".into(),
        base,
        Duration::from_secs(1),
        None,
    )
    .unwrap();
    for (_, _, confidence, top) in examples {
        let answer = backend
            .decide(&json!({}), &score_questions())
            .await
            .unwrap();
        assert_eq!(answer["answers"]["fit"]["top"], top);
        assert_eq!(answer["answers"]["fit"]["confidence"], confidence);
    }
}

#[tokio::test]
async fn noul_concentration_covers_flat_and_peaked() {
    let replies = [0.5, 0.9].into_iter().map(|noul| (
        StatusCode::OK,
        json!({"model":"jev-1.2.3","usage":{"input_tokens":1,"output_tokens":1},"answers":{"relevant":{"type":"noul","noul":noul}}}),
    )).collect();
    let (base, _) = server(replies).await;
    let backend = HostedJevBackend::for_test(
        "key".into(),
        "jev-1.2.3".into(),
        base,
        Duration::from_secs(1),
        None,
    )
    .unwrap();
    assert_eq!(
        backend.decide(&json!({}), &noul_questions()).await.unwrap()["answers"]["relevant"]
            ["confidence"],
        0.0
    );
    assert_eq!(
        backend.decide(&json!({}), &noul_questions()).await.unwrap()["answers"]["relevant"]
            ["confidence"],
        0.8
    );
}

#[tokio::test]
async fn single_label_choice_and_score_are_rejected_by_provider_normalization() {
    let cases = [
        (
            json!({"gate":{"type":"choice","instructions":"choose","criteria":{"answer":"only"}}}),
            json!({"gate":{"type":"choice","probabilities":{"answer":1.0},"choice":"answer","confidence":1.0}}),
        ),
        (
            json!({"fit":{"type":"score","instructions":"score","criteria":["only"]}}),
            json!({"fit":{"type":"score","probabilities":{"0":1.0},"score":0.0,"confidence":1.0,"legend":{"0":"only"}}}),
        ),
    ];
    for (questions, answers) in cases {
        let reply = json!({"model":"jev-1.2.3","usage":{"input_tokens":1,"output_tokens":1},"answers":answers});
        let (base, _) = server(vec![(StatusCode::OK, reply)]).await;
        let backend = HostedJevBackend::for_test(
            "key".into(),
            "jev-1.2.3".into(),
            base,
            Duration::from_secs(1),
            None,
        )
        .unwrap();
        assert!(backend
            .decide(&json!({}), questions.as_object().unwrap())
            .await
            .is_err());
    }
}

#[tokio::test]
async fn rejects_provider_confidence_that_is_not_concentration() {
    let reply = json!({"model":"jev-1.2.3","usage":{"input_tokens":1,"output_tokens":1},"answers":{"gate":{"type":"choice","probabilities":{"answer":0.9,"review":0.1},"choice":"answer","confidence":0.9}}});
    let (base, _) = server(vec![(StatusCode::OK, reply)]).await;
    let backend = HostedJevBackend::for_test(
        "key".into(),
        "jev-1.2.3".into(),
        base,
        Duration::from_secs(1),
        None,
    )
    .unwrap();
    assert!(backend
        .decide(&json!({}), &choice_question())
        .await
        .is_err());
}

#[tokio::test]
async fn retries_only_retryable_statuses_with_bounds() {
    let (base, fake) = server(vec![
        (StatusCode::SERVICE_UNAVAILABLE, json!({})),
        (StatusCode::BAD_GATEWAY, json!({})),
        (StatusCode::OK, success()),
    ])
    .await;
    let backend = HostedJevBackend::for_test(
        "key".into(),
        "jev-1.2.3".into(),
        base,
        Duration::from_secs(1),
        None,
    )
    .unwrap();
    backend
        .decide(&json!({}), &choice_question())
        .await
        .unwrap();
    assert_eq!(fake.calls.lock().unwrap().len(), 3);

    let (base, fake) = server(vec![(StatusCode::BAD_REQUEST, json!({"raw":"secret"}))]).await;
    let backend = HostedJevBackend::for_test(
        "key".into(),
        "jev-1.2.3".into(),
        base,
        Duration::from_secs(1),
        None,
    )
    .unwrap();
    assert!(backend
        .decide(&json!({}), &choice_question())
        .await
        .is_err());
    assert_eq!(fake.calls.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn malformed_answers_are_rejected() {
    let cases = [
        json!({"model":"jev-1.2.3","usage":{"input_tokens":1,"output_tokens":1},"answers":{}}),
        json!({"model":"jev-1.2.3","usage":{"input_tokens":1,"output_tokens":1},"answers":{"gate":{"type":"noul","noul":0.9}}}),
        json!({"model":"jev-1.2.3","usage":{"input_tokens":1,"output_tokens":1},"answers":{"gate":{"type":"choice","probabilities":{"answer":0.8,"review":0.1},"choice":"answer","confidence":0.8}}}),
        json!({"model":"jev-1.2.3","usage":{"input_tokens":1,"output_tokens":1},"answers":{"gate":{"type":"choice","probabilities":{"answer":0.8,"review":0.2,"extra":0.0},"choice":"answer","confidence":0.8}}}),
        json!({"model":"jev-1.2.3","usage":{"input_tokens":1,"output_tokens":1},"answers":{"gate":{"type":"choice","probabilities":{"answer":0.8,"review":0.2},"choice":"review","confidence":0.8}}}),
        json!({"model":"jev-1.2.3","usage":{"input_tokens":1,"output_tokens":1},"answers":{"gate":{"type":"choice","probabilities":{"answer":0.8,"review":0.2},"choice":"answer","confidence":1.2}}}),
    ];
    for reply in cases {
        let (base, _) = server(vec![(StatusCode::OK, reply)]).await;
        let backend = HostedJevBackend::for_test(
            "key".into(),
            "jev-1.2.3".into(),
            base,
            Duration::from_secs(1),
            None,
        )
        .unwrap();
        assert!(backend
            .decide(&json!({}), &choice_question())
            .await
            .is_err());
    }
}

#[tokio::test]
async fn decide_route_uses_the_configured_backend_and_logs_no_payloads() {
    let (base, _) = server(vec![(StatusCode::OK, success())]).await;
    let backend = Arc::new(
        HostedJevBackend::for_test(
            "SENTINEL_KEY".into(),
            "jev-1.2.3".into(),
            base,
            Duration::from_secs(1),
            None,
        )
        .unwrap(),
    );
    let logs = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&logs);
    let app = Arc::new(
        serve::App::load(
            &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("catalogue.json"),
            0.8,
            4000,
            10,
        )
        .unwrap()
        .with_backend(backend)
        .with_log_sink(move |line| sink.lock().unwrap().push(line)),
    );
    let response = serve::router(app, &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static")).oneshot(Request::builder().method("POST").uri("/api/decide").header("content-type","application/json").body(Body::from(json!({"backend":"jev","state":{"request":"SENTINEL_STATE"},"questions":choice_question()}).to_string())).unwrap()).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body: Value =
        serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(body["backend"], "jev");
    let logs = logs.lock().unwrap().join("\n");
    for secret in ["SENTINEL_KEY", "SENTINEL_STATE", "choose"] {
        assert!(!logs.contains(secret));
    }
}

struct FailingBackend;

impl DecisionBackend for FailingBackend {
    fn decide<'a>(
        &'a self,
        _state: &'a Value,
        _questions: &'a serde_json::Map<String, Value>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Value, serve::DecisionError>> + Send + 'a>,
    > {
        Box::pin(async { Err(serve::DecisionError::unavailable()) })
    }
    fn name(&self) -> &'static str {
        "jev"
    }
    fn label(&self) -> &'static str {
        "TypeSafe hosted Jev"
    }
    fn model(&self) -> &str {
        "jev-1.2.3"
    }
    fn residency(&self) -> &'static str {
        "hosted"
    }
}

#[tokio::test]
async fn pipeline_backend_failure_returns_review_before_discovery() {
    let app = Arc::new(
        serve::App::load(
            &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("catalogue.json"),
            0.8,
            4000,
            10,
        )
        .unwrap()
        .with_backend(Arc::new(FailingBackend)),
    );
    let response = serve::router(
        app,
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static"),
    )
    .oneshot(
        Request::builder()
            .method("POST")
            .uri("/api/pipeline")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({"request":"annual CO2 for Australia in 2024"}).to_string(),
            ))
            .unwrap(),
    )
    .await
    .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body: Value =
        serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(body["outcome"], "review");
    assert_eq!(body["backend"], "jev");
    assert_eq!(body["stages"], json!([]));
    assert!(body.get("solutions").is_none());
    assert!(body.get("subgraph").is_none());
}

#[test]
fn jev_requires_private_service_auth_and_debug_redacts_key() {
    let vars = HashMap::from([
        ("DECISION_BACKEND".to_string(), "jev".to_string()),
        (
            "TYPESAFE_API_KEY".to_string(),
            "SENTINEL_SECRET".to_string(),
        ),
        ("JEV_MODEL".to_string(), "jev-1.2.3".to_string()),
    ]);
    let error = DecisionBackendConfig::from_lookup(|key| vars.get(key).cloned()).unwrap_err();
    assert!(error.contains("SERVICE_AUTH_MODE"));
    assert!(!error.contains("SENTINEL_SECRET"));

    let mut private = vars;
    private.insert("SERVICE_AUTH_MODE".into(), "cloud-run-iam".into());
    let config = DecisionBackendConfig::from_lookup(|key| private.get(key).cloned()).unwrap();
    let debug = format!("{config:?}");
    assert!(!debug.contains("SENTINEL_SECRET"));
    assert!(debug.contains("[REDACTED]"));
}

#[test]
fn hosted_limits_are_bounded() {
    let base = [
        ("DECISION_BACKEND", "jev"),
        ("SERVICE_AUTH_MODE", "cloud-run-iam"),
        ("TYPESAFE_API_KEY", "key"),
        ("JEV_MODEL", "jev-1.2.3"),
    ];
    for (key, value) in [
        ("JEV_MAX_CONCURRENCY", "0"),
        ("JEV_PIPELINE_MAX_CALLS", "0"),
        ("JEV_PIPELINE_TIMEOUT_MS", "999999"),
    ] {
        let vars: HashMap<String, String> = base
            .into_iter()
            .chain([(key, value)])
            .map(|(k, v)| (k.into(), v.into()))
            .collect();
        assert!(DecisionBackendConfig::from_lookup(|name| vars.get(name).cloned()).is_err());
    }
}

fn noul_questions() -> serde_json::Map<String, Value> {
    json!({"relevant":{"type":"noul","instructions":"sensitive instructions"}})
        .as_object()
        .unwrap()
        .clone()
}

fn score_questions() -> serde_json::Map<String, Value> {
    json!({"fit":{"type":"score","instructions":"score","criteria":["poor","good","best"]}})
        .as_object()
        .unwrap()
        .clone()
}

#[tokio::test]
async fn exact_noul_and_score_wires_and_semantics() {
    let replies = vec![
        (
            StatusCode::OK,
            json!({"model":"jev-1.2.3","usage":{"input_tokens":1,"output_tokens":1},"answers":{"relevant":{"type":"noul","noul":0.7}}}),
        ),
        (
            StatusCode::OK,
            json!({"model":"jev-1.2.3","usage":{"input_tokens":1,"output_tokens":1},"answers":{"fit":{"type":"score","probabilities":{"0":0.1,"1":0.2,"2":0.7},"score":1.6,"confidence":0.4,"legend":{"0":"poor","1":"good","2":"best"}}}}),
        ),
    ];
    let (base, fake) = server(replies).await;
    let backend = HostedJevBackend::for_test(
        "key".into(),
        "jev-1.2.3".into(),
        base,
        Duration::from_secs(1),
        None,
    )
    .unwrap();
    let noul = backend
        .decide(&json!({"one":1}), &noul_questions())
        .await
        .unwrap();
    let score = backend
        .decide(&json!({"two":2}), &score_questions())
        .await
        .unwrap();
    let calls = fake.calls.lock().unwrap();
    assert_eq!(
        calls[0].1,
        json!({"model":"jev-1.2.3","state":{"one":1},"questions":noul_questions()})
    );
    assert_eq!(
        calls[1].1,
        json!({"model":"jev-1.2.3","state":{"two":2},"questions":score_questions()})
    );
    assert_eq!(
        noul["answers"]["relevant"],
        json!({"type":"noul","probabilities":{"yes":0.7,"no":0.3},"top":"yes","noul":0.7,"confidence":0.4})
    );
    assert_eq!(
        score["answers"]["fit"],
        json!({"type":"score","probabilities":{"0":0.1,"1":0.2,"2":0.7},"top":"2","score":1.6,"confidence":0.4,"legend":{"0":"poor","1":"good","2":"best"}})
    );
    assert!(score["cost_usd"].is_null());
    let latency_ms = score["latency_ms"].as_f64().unwrap();
    assert!(latency_ms.is_finite() && latency_ms >= 0.0);
}

#[tokio::test]
async fn provider_native_ties_are_deterministic() {
    let replies = vec![
        (
            StatusCode::OK,
            json!({"model":"jev-1.2.3","usage":{"input_tokens":1,"output_tokens":1},"answers":{"gate":{"type":"choice","probabilities":{"answer":0.5,"review":0.5},"choice":"review","confidence":0.0}}}),
        ),
        (
            StatusCode::OK,
            json!({"model":"jev-1.2.3","usage":{"input_tokens":1,"output_tokens":1},"answers":{"relevant":{"type":"noul","noul":0.5}}}),
        ),
        (
            StatusCode::OK,
            json!({"model":"jev-1.2.3","usage":{"input_tokens":1,"output_tokens":1},"answers":{"fit":{"type":"score","probabilities":{"0":0.5,"1":0.5,"2":0.0},"score":0.5,"confidence":0.25,"legend":{"0":"poor","1":"good","2":"best"}}}}),
        ),
    ];
    let (base, _) = server(replies).await;
    let backend = HostedJevBackend::for_test(
        "key".into(),
        "jev-1.2.3".into(),
        base,
        Duration::from_secs(1),
        None,
    )
    .unwrap();
    let choice = backend
        .decide(&json!({}), &choice_question())
        .await
        .unwrap();
    let noul = backend.decide(&json!({}), &noul_questions()).await.unwrap();
    let score = backend
        .decide(&json!({}), &score_questions())
        .await
        .unwrap();
    assert_eq!(choice["answers"]["gate"]["top"], "review");
    assert_eq!(noul["answers"]["relevant"]["top"], "no");
    assert_eq!(score["answers"]["fit"]["top"], "0");
}

#[tokio::test]
async fn score_rejects_missing_extra_and_wrong_provider_legend() {
    let answers = [
        json!({"type":"score","probabilities":{"0":0.1,"1":0.2,"2":0.7},"score":1.6,"confidence":0.4}),
        json!({"type":"score","probabilities":{"0":0.1,"1":0.2,"2":0.7},"score":1.6,"confidence":0.4,"legend":{"0":"poor","1":"good","2":"best"},"top":"2"}),
        json!({"type":"score","probabilities":{"0":0.1,"1":0.2,"2":0.7},"score":1.6,"confidence":0.4,"legend":{"0":"poor","1":"best","2":"good"}}),
        json!({"type":"score","probabilities":{"0":0.1,"1":0.2,"2":0.7},"score":1.6,"confidence":0.4,"legend":{"0":"poor","1":"good"}}),
    ];
    for answer in answers {
        let reply = json!({"model":"jev-1.2.3","usage":{"input_tokens":1,"output_tokens":1},"answers":{"fit":answer}});
        let (base, _) = server(vec![(StatusCode::OK, reply)]).await;
        let backend = HostedJevBackend::for_test(
            "key".into(),
            "jev-1.2.3".into(),
            base,
            Duration::from_secs(1),
            None,
        )
        .unwrap();
        assert!(backend
            .decide(&json!({}), &score_questions())
            .await
            .is_err());
    }
}

#[tokio::test]
async fn rejects_wrong_model_usage_latency_and_unknown_fields() {
    let answer = json!({"gate":{"type":"choice","probabilities":{"answer":0.9,"review":0.1},"choice":"answer","confidence":0.8}});
    let cases = [
        json!({"model":"other","answers":answer,"usage":{"input_tokens":1,"output_tokens":1}}),
        json!({"model":"jev-1.2.3","answers":answer}),
        json!({"model":"jev-1.2.3","answers":answer,"usage":{"input_tokens":-1,"output_tokens":1}}),
        json!({"model":"jev-1.2.3","answers":answer,"usage":{"input_tokens":1}}),
        json!({"model":"jev-1.2.3","answers":answer,"usage":{"input_tokens":1,"output_tokens":-1}}),
        json!({"model":"jev-1.2.3","answers":answer,"usage":{"input_tokens":1,"output_tokens":1,"unknown":true}}),
        json!({"model":"jev-1.2.3","answers":answer,"usage":{"input_tokens":1,"output_tokens":1},"unknown":true}),
        json!({"model":"jev-1.2.3","usage":{"input_tokens":1,"output_tokens":1},"answers":{"gate":{"type":"choice","probabilities":{"answer":0.9,"review":0.1},"choice":"answer","confidence":0.8,"unknown":true}}}),
    ];
    for reply in cases {
        let (base, _) = server(vec![(StatusCode::OK, reply)]).await;
        let backend = HostedJevBackend::for_test(
            "key".into(),
            "jev-1.2.3".into(),
            base,
            Duration::from_secs(1),
            None,
        )
        .unwrap();
        assert_eq!(
            backend
                .decide(&json!({}), &choice_question())
                .await
                .unwrap_err()
                .kind,
            serve::DecisionErrorKind::BadGateway
        );
    }
}

#[tokio::test]
async fn semaphore_saturation_is_unavailable_without_extra_upstream_call() {
    async fn slow(
        State(fake): State<Fake>,
        headers: HeaderMap,
        Json(body): Json<Value>,
    ) -> impl IntoResponse {
        fake.calls.lock().unwrap().push((headers, body));
        tokio::time::sleep(Duration::from_millis(400)).await;
        (StatusCode::OK, Json(success()))
    }
    let fake = Fake {
        calls: Arc::new(Mutex::new(Vec::new())),
        replies: Arc::new(Mutex::new(Vec::new())),
    };
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let app = Router::new()
        .route("/v1/systemone", post(slow))
        .with_state(fake.clone());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let backend = Arc::new(
        HostedJevBackend::for_test_with_limits(
            HostedJevSettings {
                api_key: "key".into(),
                model: "jev-1.2.3".into(),
                base_url: base,
                timeout: Duration::from_secs(1),
                price_per_mtok: None,
            },
            HostedJevLimits {
                max_concurrency: 1,
                pipeline_max_calls: 16,
                pipeline_timeout: Duration::from_secs(2),
            },
        )
        .unwrap(),
    );
    let first = {
        let backend = Arc::clone(&backend);
        tokio::spawn(async move { backend.decide(&json!({}), &choice_question()).await })
    };
    tokio::time::sleep(Duration::from_millis(20)).await;
    let second = backend
        .decide(&json!({}), &choice_question())
        .await
        .unwrap_err();
    assert_eq!(second.kind, serve::DecisionErrorKind::Unavailable);
    first.await.unwrap().unwrap();
    assert_eq!(fake.calls.lock().unwrap().len(), 1);
}

#[derive(Clone)]
struct ScriptBackend {
    replies: Arc<Mutex<Vec<Result<Value, serve::DecisionError>>>>,
    max_calls: usize,
    deadline: Duration,
}

impl DecisionBackend for ScriptBackend {
    fn decide<'a>(
        &'a self,
        _state: &'a Value,
        questions: &'a serde_json::Map<String, Value>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Value, serve::DecisionError>> + Send + 'a>,
    > {
        Box::pin(async move {
            let mut reply = self.replies.lock().unwrap().remove(0)?;
            if reply["answers"]
                .as_object()
                .is_some_and(|answer| answer.is_empty())
            {
                let answers = questions.iter().map(|(name, question)| {
                    let value = match question["type"].as_str().unwrap() {
                        "noul" => json!({"type":"noul","probabilities":{"yes":0.0,"no":1.0},"top":"no","noul":0.0,"confidence":1.0}),
                        "choice" => { let top=question["criteria"].as_object().unwrap().keys().next().unwrap(); json!({"type":"choice","probabilities":question["criteria"].as_object().unwrap().keys().map(|label|(label.clone(),json!(if label==top {1.0}else{0.0}))).collect::<serde_json::Map<_,_>>(),"top":top,"choice":top,"confidence":1.0}) },
                        _ => json!({}),
                    };
                    (name.clone(), value)
                }).collect::<serde_json::Map<_,_>>();
                reply["answers"] = Value::Object(answers);
            }
            Ok(reply)
        })
    }
    fn name(&self) -> &'static str {
        "jev"
    }
    fn label(&self) -> &'static str {
        "TypeSafe hosted Jev"
    }
    fn model(&self) -> &str {
        "jev-1.2.3"
    }
    fn residency(&self) -> &'static str {
        "hosted"
    }
    fn auth_required(&self) -> bool {
        true
    }
    fn dependency_status(&self) -> &'static str {
        "configuration-only; not probed"
    }
    fn pipeline_max_calls(&self) -> usize {
        self.max_calls
    }
    fn pipeline_timeout(&self) -> Duration {
        self.deadline
    }
}

fn pipeline_app(backend: Arc<dyn DecisionBackend>) -> Router {
    let app = Arc::new(
        serve::App::load(
            &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("catalogue.json"),
            0.8,
            4000,
            20,
        )
        .unwrap()
        .with_backend(backend),
    );
    serve::router(
        app,
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static"),
    )
}

async fn pipeline_request(app: Router, request: &str) -> Value {
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/pipeline")
                .header("content-type", "application/json")
                .body(Body::from(json!({"request":request}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap()).unwrap()
}

fn gate(top: &str, confidence: f64) -> Value {
    let mut probs = serde_json::Map::new();
    for label in ["answer", "clarify", "reject"] {
        probs.insert(
            label.into(),
            json!(if label == top {
                confidence
            } else {
                (1.0 - confidence) / 2.0
            }),
        );
    }
    json!({"backend":"jev","model":"jev-1.2.3","input_tokens":10,"cost_usd":0.01,"latency_ms":2.0,"answers":{"gate":{"type":"choice","probabilities":probs,"top":top,"choice":top,"confidence":confidence}}})
}

#[tokio::test]
async fn low_confidence_gate_precedes_deterministic_reject() {
    let backend = ScriptBackend {
        replies: Arc::new(Mutex::new(vec![Ok(gate("reject", 0.6))])),
        max_calls: 5,
        deadline: Duration::from_secs(1),
    };
    let value = pipeline_request(pipeline_app(Arc::new(backend)), "tell me a joke").await;
    assert_eq!(value["outcome"], "review");
    assert_eq!(value["totals"]["calls"], 1);
}

#[tokio::test]
async fn later_failure_preserves_stages_and_totals() {
    let backend = ScriptBackend {
        replies: Arc::new(Mutex::new(vec![
            Ok(gate("answer", 1.0)),
            Err(serve::DecisionError::unavailable()),
        ])),
        max_calls: 5,
        deadline: Duration::from_secs(1),
    };
    let value = pipeline_request(
        pipeline_app(Arc::new(backend)),
        "annual CO2 for Australia in 2024",
    )
    .await;
    assert_eq!(value["outcome"], "review");
    assert_eq!(value["stages"].as_array().unwrap().len(), 1);
    assert_eq!(value["totals"]["calls"], 1);
    assert_eq!(value["totals"]["input_tokens"], 10);
    assert_eq!(value["totals"]["cost_usd"], 0.01);
    assert_eq!(value["totals"]["backend_errors"], 1);
}

#[tokio::test]
async fn pipeline_call_budget_stops_before_next_backend_call() {
    let backend = ScriptBackend {
        replies: Arc::new(Mutex::new(vec![Ok(gate("answer", 1.0))])),
        max_calls: 1,
        deadline: Duration::from_secs(1),
    };
    let replies = Arc::clone(&backend.replies);
    let value = pipeline_request(
        pipeline_app(Arc::new(backend)),
        "annual CO2 for Australia in 2024",
    )
    .await;
    assert_eq!(value["outcome"], "review");
    assert!(replies.lock().unwrap().is_empty());
    assert_eq!(value["totals"]["calls"], 1);
}

#[tokio::test]
async fn health_reports_private_auth_and_configuration_only_readiness() {
    let backend = ScriptBackend {
        replies: Arc::new(Mutex::new(vec![])),
        max_calls: 1,
        deadline: Duration::from_secs(1),
    };
    let app = pipeline_app(Arc::new(backend));
    for (path, auth) in [("/api/health", true), ("/api/ready", false)] {
        let response = app
            .clone()
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        let value: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        if path.ends_with("health") {
            assert_eq!(value["auth_required"], auth);
        }
        assert_eq!(
            if path.ends_with("health") {
                &value["backend_detail"]["dependency_check"]
            } else {
                &value["dependency_check"]
            },
            "configuration-only; not probed"
        );
    }
}

#[tokio::test]
async fn decide_maps_explicit_error_kinds() {
    struct ErrorBackend(serve::DecisionErrorKind);
    impl DecisionBackend for ErrorBackend {
        fn decide<'a>(
            &'a self,
            _: &'a Value,
            _: &'a serde_json::Map<String, Value>,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<Value, serve::DecisionError>> + Send + 'a>,
        > {
            Box::pin(async move {
                Err(match self.0 {
                    serve::DecisionErrorKind::InvalidRequest => {
                        serve::DecisionError::invalid("invalid")
                    }
                    serve::DecisionErrorKind::BadGateway => serve::DecisionError::malformed(),
                    serve::DecisionErrorKind::Unavailable => serve::DecisionError::unavailable(),
                })
            })
        }
        fn name(&self) -> &'static str {
            "jev"
        }
        fn label(&self) -> &'static str {
            "Jev"
        }
        fn model(&self) -> &str {
            "m"
        }
        fn residency(&self) -> &'static str {
            "hosted"
        }
    }
    for (kind, status) in [
        (
            serve::DecisionErrorKind::InvalidRequest,
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (
            serve::DecisionErrorKind::BadGateway,
            StatusCode::BAD_GATEWAY,
        ),
        (
            serve::DecisionErrorKind::Unavailable,
            StatusCode::SERVICE_UNAVAILABLE,
        ),
    ] {
        let response = pipeline_app(Arc::new(ErrorBackend(kind)))
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/decide")
                    .header("content-type", "application/json")
                    .body(Body::from(json!({"state":{},"questions":{}}).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), status);
    }
}

#[tokio::test]
async fn pipeline_deadline_returns_review() {
    struct SlowBackend;
    impl DecisionBackend for SlowBackend {
        fn decide<'a>(
            &'a self,
            _: &'a Value,
            _: &'a serde_json::Map<String, Value>,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<Value, serve::DecisionError>> + Send + 'a>,
        > {
            Box::pin(async {
                tokio::time::sleep(Duration::from_millis(200)).await;
                Ok(gate("answer", 1.0))
            })
        }
        fn name(&self) -> &'static str {
            "jev"
        }
        fn label(&self) -> &'static str {
            "Jev"
        }
        fn model(&self) -> &str {
            "m"
        }
        fn residency(&self) -> &'static str {
            "hosted"
        }
        fn pipeline_timeout(&self) -> Duration {
            Duration::from_millis(50)
        }
    }
    let value = pipeline_request(
        pipeline_app(Arc::new(SlowBackend)),
        "annual CO2 for Australia in 2024",
    )
    .await;
    assert_eq!(value["outcome"], "review");
    assert_eq!(value["totals"]["calls"], 0);
}

#[tokio::test]
async fn hosted_answer_conflicting_with_deterministic_reject_is_review() {
    let backend = ScriptBackend {
        replies: Arc::new(Mutex::new(vec![Ok(gate("answer", 1.0))])),
        max_calls: 5,
        deadline: Duration::from_secs(1),
    };
    let value = pipeline_request(pipeline_app(Arc::new(backend)), "tell me a joke").await;
    assert_eq!(value["outcome"], "review");
}

#[test]
fn moving_and_implausible_models_are_rejected_case_insensitively() {
    for model in ["JEV-LATEST", "jev-Preview", "latest", "jev", "jev 1.2"] {
        let vars: HashMap<String, String> = HashMap::from([
            ("DECISION_BACKEND".into(), "jev".into()),
            ("SERVICE_AUTH_MODE".into(), "cloud-run-iam".into()),
            ("TYPESAFE_API_KEY".into(), "key".into()),
            ("JEV_MODEL".into(), model.into()),
        ]);
        assert!(
            DecisionBackendConfig::from_lookup(|key| vars.get(key).cloned()).is_err(),
            "{model}"
        );
    }
}

#[tokio::test]
async fn every_nonretryable_4xx_is_bad_gateway_without_retry() {
    for status in [400, 401, 403, 404, 409, 422] {
        let (base, fake) = server(vec![(
            StatusCode::from_u16(status).unwrap(),
            json!({"raw":"do not expose"}),
        )])
        .await;
        let backend = HostedJevBackend::for_test(
            "key".into(),
            "jev-1.2.3".into(),
            base,
            Duration::from_secs(1),
            None,
        )
        .unwrap();
        assert_eq!(
            backend
                .decide(&json!({}), &choice_question())
                .await
                .unwrap_err()
                .kind,
            serve::DecisionErrorKind::BadGateway
        );
        assert_eq!(fake.calls.lock().unwrap().len(), 1);
    }
}

#[tokio::test]
async fn aggregate_usage_becomes_null_when_any_success_omits_metric() {
    let first = gate("answer", 1.0);
    let second = json!({"backend":"jev","model":"jev-1.2.3","latency_ms":null,"input_tokens":null,"cost_usd":null,"answers":{}});
    let backend = ScriptBackend {
        replies: Arc::new(Mutex::new(vec![Ok(first), Ok(second)])),
        max_calls: 5,
        deadline: Duration::from_secs(1),
    };
    let value = pipeline_request(
        pipeline_app(Arc::new(backend)),
        "annual CO2 for Australia in 2024",
    )
    .await;
    assert_eq!(value["outcome"], "review");
    assert!(value["totals"]["input_tokens"].is_null());
    assert!(value["totals"]["cost_usd"].is_null());
    assert!(value["totals"]["latency_ms"].is_null());
    assert_eq!(value["totals"]["usage_complete"], false);
}

#[tokio::test]
async fn explicit_group_requires_jev_yes_or_reviews() {
    let relevance = json!({"backend":"jev","model":"jev-1.2.3","input_tokens":5,"cost_usd":0.01,"latency_ms":1.0,"answers":{}});
    let backend = ScriptBackend {
        replies: Arc::new(Mutex::new(vec![Ok(gate("answer", 1.0)), Ok(relevance)])),
        max_calls: 5,
        deadline: Duration::from_secs(1),
    };
    let value = pipeline_request(
        pipeline_app(Arc::new(backend)),
        "annual greenhouse gases by country in 2024",
    )
    .await;
    assert_eq!(value["outcome"], "review");
    assert!(value["message"]
        .as_str()
        .unwrap()
        .contains("explicit catalogue group"));
}

#[tokio::test]
async fn redirect_is_refused_and_authorization_not_forwarded() {
    let received = Arc::new(Mutex::new(Vec::<HeaderMap>::new()));
    let capture = Arc::clone(&received);
    let target_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let target = format!("http://{}/capture", target_listener.local_addr().unwrap());
    let target_app = Router::new().route(
        "/capture",
        post(move |headers: HeaderMap| {
            let capture = Arc::clone(&capture);
            async move {
                capture.lock().unwrap().push(headers);
                StatusCode::OK
            }
        }),
    );
    tokio::spawn(async move { axum::serve(target_listener, target_app).await.unwrap() });
    let source_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let source = format!("http://{}", source_listener.local_addr().unwrap());
    let source_app = Router::new().route(
        "/v1/systemone",
        post(move || {
            let target = target.clone();
            async move { (StatusCode::FOUND, [("location", target)]) }
        }),
    );
    tokio::spawn(async move { axum::serve(source_listener, source_app).await.unwrap() });
    let backend = HostedJevBackend::for_test(
        "SENTINEL_KEY".into(),
        "jev-1.2.3".into(),
        source,
        Duration::from_secs(1),
        None,
    )
    .unwrap();
    assert_eq!(
        backend
            .decide(&json!({}), &choice_question())
            .await
            .unwrap_err()
            .kind,
        serve::DecisionErrorKind::Unavailable
    );
    assert!(received.lock().unwrap().is_empty());
}

#[tokio::test]
async fn declared_and_streamed_oversized_bodies_are_rejected() {
    async fn declared() -> impl IntoResponse {
        (StatusCode::OK, vec![b'x'; 300000])
    }
    async fn streamed() -> impl IntoResponse {
        let body = Body::from(vec![b'x'; 300000]);
        (StatusCode::OK, body)
    }
    for handler in [
        Router::new().route("/v1/systemone", post(declared)),
        Router::new().route("/v1/systemone", post(streamed)),
    ] {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        tokio::spawn(async move { axum::serve(listener, handler).await.unwrap() });
        let backend = HostedJevBackend::for_test(
            "key".into(),
            "jev-1.2.3".into(),
            base,
            Duration::from_secs(1),
            None,
        )
        .unwrap();
        assert_eq!(
            backend
                .decide(&json!({}), &choice_question())
                .await
                .unwrap_err()
                .kind,
            serve::DecisionErrorKind::BadGateway
        );
    }
}

#[tokio::test]
async fn malformed_json_timeout_and_retry_exhaustion_are_classified() {
    async fn malformed() -> impl IntoResponse {
        (StatusCode::OK, "not-json")
    }
    async fn slow() -> impl IntoResponse {
        tokio::time::sleep(Duration::from_millis(200)).await;
        (StatusCode::OK, Json(success()))
    }
    for (app, timeout, kind) in [
        (
            Router::new().route("/v1/systemone", post(malformed)),
            Duration::from_secs(1),
            serve::DecisionErrorKind::BadGateway,
        ),
        (
            Router::new().route("/v1/systemone", post(slow)),
            Duration::from_millis(50),
            serve::DecisionErrorKind::Unavailable,
        ),
    ] {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let backend =
            HostedJevBackend::for_test("key".into(), "jev-1.2.3".into(), base, timeout, None)
                .unwrap();
        assert_eq!(
            backend
                .decide(&json!({}), &choice_question())
                .await
                .unwrap_err()
                .kind,
            kind
        );
    }
    let (base, fake) = server(vec![(StatusCode::SERVICE_UNAVAILABLE, json!({})); 3]).await;
    let backend = HostedJevBackend::for_test(
        "key".into(),
        "jev-1.2.3".into(),
        base,
        Duration::from_secs(2),
        None,
    )
    .unwrap();
    assert_eq!(
        backend
            .decide(&json!({}), &choice_question())
            .await
            .unwrap_err()
            .kind,
        serve::DecisionErrorKind::Unavailable
    );
    assert_eq!(fake.calls.lock().unwrap().len(), 3);
}

#[derive(Clone)]
struct AdaptiveBackend {
    calls: Arc<Mutex<Vec<Vec<String>>>>,
    low_relevance: bool,
}
impl DecisionBackend for AdaptiveBackend {
    fn decide<'a>(
        &'a self,
        _: &'a Value,
        questions: &'a serde_json::Map<String, Value>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Value, serve::DecisionError>> + Send + 'a>,
    > {
        Box::pin(async move {
            self.calls
                .lock()
                .unwrap()
                .push(questions.keys().cloned().collect());
            let answers=questions.iter().map(|(name,q)| {
                let answer=match q["type"].as_str().unwrap() {
                    "noul" => { let yes=name=="pollutant:CO2"; let confidence=if self.low_relevance{0.6}else{1.0}; let py=if yes{confidence}else{1.0-confidence}; json!({"type":"noul","probabilities":{"yes":py,"no":1.0-py},"top":if py>=0.5{"yes"}else{"no"},"noul":py,"confidence":confidence}) },
                    "choice" => { let desired=match name.as_str(){"gate"=>"answer","GEO"=>"country","TIME"=>"year","SECTOR"=>"macrosector",_=>"none"}; let probs=q["criteria"].as_object().unwrap().keys().map(|label|(label.clone(),json!(if label==desired{1.0}else{0.0}))).collect::<serde_json::Map<_,_>>();json!({"type":"choice","probabilities":probs,"top":desired,"choice":desired,"confidence":1.0}) },
                    "score" => json!({"type":"score","probabilities":{"0":0.0,"1":0.0,"2":0.0,"3":1.0},"top":"3","score":3.0,"confidence":1.0}),
                    _=>unreachable!(),
                };(name.clone(),answer)
            }).collect::<serde_json::Map<_,_>>();
            Ok(
                json!({"backend":"jev","model":"jev-1.2.3","input_tokens":1,"cost_usd":0.01,"latency_ms":1.0,"answers":answers}),
            )
        })
    }
    fn name(&self) -> &'static str {
        "jev"
    }
    fn label(&self) -> &'static str {
        "Jev"
    }
    fn model(&self) -> &str {
        "jev-1.2.3"
    }
    fn residency(&self) -> &'static str {
        "hosted"
    }
}

#[tokio::test]
async fn low_confidence_relevance_stops_before_levels() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let backend = AdaptiveBackend {
        calls: Arc::clone(&calls),
        low_relevance: true,
    };
    let value = pipeline_request(
        pipeline_app(Arc::new(backend)),
        "annual CO2 by country in 2024",
    )
    .await;
    assert_eq!(value["outcome"], "review");
    assert_eq!(calls.lock().unwrap().len(), 2);
}

#[tokio::test]
async fn successful_multistage_pipeline_uses_jev_including_fit() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let backend = AdaptiveBackend {
        calls: Arc::clone(&calls),
        low_relevance: false,
    };
    let app = pipeline_app(Arc::new(backend));
    let response=app.oneshot(Request::builder().method("POST").uri("/api/pipeline").header("content-type","application/json").body(Body::from(json!({"request":"annual CO2 by country in 2024","preference":"prefer broad coverage"}).to_string())).unwrap()).await.unwrap();
    let value: Value =
        serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(value["outcome"], "answer", "{value}");
    let calls = calls.lock().unwrap();
    assert!(calls.len() >= 4);
    assert!(calls.iter().any(|names| names == &vec!["fit".to_string()]));
    assert_eq!(value["backend"], "jev");
    assert_eq!(value["totals"]["usage_complete"], true);
    assert!(value["totals"]["input_tokens"].as_u64().is_some());
    assert!(value["totals"]["cost_usd"].as_f64().is_some());
    assert!(value["totals"]["latency_ms"].as_f64().is_some());
}

#[tokio::test]
async fn bad_noul_and_score_semantics_are_rejected() {
    let cases = [
        (
            noul_questions(),
            json!({"model":"jev-1.2.3","usage":{"input_tokens":1,"output_tokens":1},"answers":{"relevant":{"type":"noul","noul":1.1}}}),
        ),
        (
            score_questions(),
            json!({"model":"jev-1.2.3","usage":{"input_tokens":1,"output_tokens":1},"answers":{"fit":{"type":"score","probabilities":{"0":0.1,"1":0.2,"2":0.7},"score":1.5,"confidence":0.7,"legend":{"0":"poor","1":"good","2":"best"}}}}),
        ),
    ];
    for (questions, reply) in cases {
        let (base, _) = server(vec![(StatusCode::OK, reply)]).await;
        let backend = HostedJevBackend::for_test(
            "key".into(),
            "jev-1.2.3".into(),
            base,
            Duration::from_secs(1),
            None,
        )
        .unwrap();
        assert_eq!(
            backend
                .decide(&json!({}), &questions)
                .await
                .unwrap_err()
                .kind,
            serve::DecisionErrorKind::BadGateway
        );
    }
}
