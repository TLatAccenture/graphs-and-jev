use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use serve::{router, App, Site};
use tower::ServiceExt;

fn app(rate: u32) -> (axum::Router, Arc<Mutex<Vec<String>>>) {
    app_with_site(rate, Site::default())
}

fn site(vars: &[(&str, &str)]) -> Result<Site, String> {
    let vars: HashMap<String, String> = vars
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    Site::from_lookup(|key| vars.get(key).cloned())
}

fn app_with_site(rate: u32, site: Site) -> (axum::Router, Arc<Mutex<Vec<String>>>) {
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
        .with_site(site),
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

async fn get(app: axum::Router, path: &str) -> axum::response::Response {
    app.oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
        .await
        .unwrap()
}

async fn json_body(response: axum::response::Response) -> Value {
    serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap()).unwrap()
}

fn header<'a>(response: &'a axum::response::Response, name: &str) -> &'a str {
    response
        .headers()
        .get(name)
        .unwrap_or_else(|| panic!("missing header {name}"))
        .to_str()
        .unwrap()
}

#[tokio::test]
async fn pipeline_accepts_the_site_api_body_and_air_alias() {
    let request = "annual CO2 for Australia in 2024";
    let mut answers = Vec::new();
    for body in [
        json!({"domain":"catalogue", "request":request, "preference":null}),
        json!({"domain":"air", "request":request, "preference":null}),
        json!({"request":request}),
        json!({"backend":"catalogue", "domain":"catalogue", "request":request}),
    ] {
        let response = post(app(10).0, "/api/pipeline", body.clone()).await;
        assert_eq!(response.status(), StatusCode::OK, "{body}");
        let value = json_body(response).await;
        assert_eq!(value["outcome"], "answer", "{body}");
        answers.push(json!([value["explanation"], value["solutions"]]));
    }
    assert!(answers.windows(2).all(|pair| pair[0] == pair[1]));
}

#[tokio::test]
async fn lineage_domain_is_not_implemented_and_logged_as_such() {
    let (router, logs) = app(10);
    let response = post(
        router,
        "/api/pipeline",
        json!({"domain":"lineage", "request":"Drop table KNA1 after cutover", "preference":null}),
    )
    .await;
    assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
    assert_eq!(
        json_body(response).await,
        json!({"detail": "Lineage domain not implemented on this service."})
    );
    let captured = logs.lock().unwrap().join("\n");
    assert!(captured.contains(r#""outcome":"not_implemented""#));
    assert!(!captured.contains("KNA1"));
}

#[tokio::test]
async fn removed_and_unknown_domains_and_backends_are_rejected() {
    for domain in ["parkinsons", "other"] {
        let response = post(
            app(10).0,
            "/api/pipeline",
            json!({"domain":domain, "request":"LRRK2 research question"}),
        )
        .await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND, "{domain}");
        assert_eq!(
            json_body(response).await["detail"],
            "Unknown domain. Available: catalogue, lineage"
        );
    }
    let response = post(
        app(10).0,
        "/api/pipeline",
        json!({"backend":"other", "domain":"catalogue", "request":"ok"}),
    )
    .await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

async fn config(app: axum::Router) -> (axum::response::Response, String, Value) {
    let response = get(app, "/config.js").await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        header(&response, "content-type"),
        "text/javascript; charset=utf-8"
    );
    assert_eq!(header(&response, "cache-control"), "no-cache");
    let (parts, body) = response.into_parts();
    let text = String::from_utf8(to_bytes(body, usize::MAX).await.unwrap().to_vec()).unwrap();
    let json = text
        .strip_prefix("window.SITE_CONFIG = ")
        .and_then(|rest| rest.strip_suffix(";\n"))
        .unwrap_or_else(|| panic!("config.js shape: {text}"));
    let value = serde_json::from_str(json).unwrap();
    (
        axum::response::Response::from_parts(parts, Body::empty()),
        text,
        value,
    )
}

#[tokio::test]
async fn config_js_renders_rust_defaults_over_the_static_file() {
    let (_, _, value) = config(app(10).0).await;
    assert_eq!(
        value,
        json!({
            "brand": {
                "org": "Accenture",
                "name": "The MCP Shed",
                "tagline": "Agentic migration and modernisation",
                "kind": "Thought piece",
                "disclaimer": "A personal research thought piece on agentic migration and modernisation. It is not an official Accenture or Google Cloud product, a service offer or a statement of capability. Product names and marks belong to their owners."
            },
            "api": {
                "mode": "live",
                "baseUrl": "",
                "timeoutMs": 15000,
                "routes": {"health": "/api/health", "catalogue": "/api/catalogue", "pipeline": "/api/pipeline"},
                "domains": {"catalogue": "live", "lineage": "fixtures"}
            }
        })
    );
}

#[tokio::test]
async fn config_js_applies_overrides_and_escapes_hostile_values() {
    let hostile = "Shed \"test\" \\ </script><script>alert(1)</script>\u{2028}";
    let site = site(&[
        ("BRAND_NAME", hostile),
        ("BRAND_ORG", "Example"),
        ("API_MODE", "fixtures"),
        ("API_BASE_URL", "https://api.example.run.app/v1"),
        ("API_TIMEOUT_MS", "9000"),
        ("API_ROUTE_PIPELINE", "/v1/pipeline"),
        ("API_MODE_LINEAGE", "live"),
        ("BRAND_TAGLINE", ""),
    ])
    .unwrap();
    let (_, text, value) = config(app_with_site(10, site).0).await;
    assert!(!text.contains('<'), "raw '<' in config.js: {text}");
    assert!(text.contains(r"\u003c/script>"));
    assert!(!text.contains('\u{2028}'));
    assert_eq!(value["brand"]["name"], hostile);
    assert_eq!(value["brand"]["org"], "Example");
    assert_eq!(
        value["brand"]["tagline"],
        "Agentic migration and modernisation"
    );
    assert_eq!(value["api"]["mode"], "fixtures");
    assert_eq!(value["api"]["baseUrl"], "https://api.example.run.app/v1");
    assert_eq!(value["api"]["timeoutMs"], 9000);
    assert_eq!(value["api"]["routes"]["pipeline"], "/v1/pipeline");
    assert_eq!(value["api"]["domains"]["catalogue"], "fixtures");
    assert_eq!(value["api"]["domains"]["lineage"], "live");
}

#[test]
fn invalid_site_settings_fail_at_startup() {
    for vars in [
        vec![("API_MODE", "Live")],
        vec![("API_MODE_LINEAGE", "remote")],
        vec![("API_TIMEOUT_MS", "soon")],
        vec![("API_TIMEOUT_MS", "0")],
        vec![("API_BASE_URL", "javascript:alert(1)")],
        vec![("API_BASE_URL", "https://api.example.com; script-src *")],
        vec![("API_BASE_URL", "https://user@api.example.com")],
        vec![("API_BASE_URL", "https://")],
    ] {
        assert!(site(&vars).is_err(), "{vars:?}");
    }
}

#[tokio::test]
async fn responses_carry_security_and_cache_headers() {
    let (router, _) = app_with_site(
        10,
        site(&[("API_BASE_URL", "https://api.example.run.app/v1")]).unwrap(),
    );
    for path in [
        "/",
        "/technical-guide/",
        "/assets/small-world.svg",
        "/api/health",
        "/config.js",
    ] {
        let response = get(router.clone(), path).await;
        assert_eq!(response.status(), StatusCode::OK, "{path}");
        assert_eq!(
            header(&response, "x-content-type-options"),
            "nosniff",
            "{path}"
        );
        assert_eq!(
            header(&response, "referrer-policy"),
            "strict-origin-when-cross-origin",
            "{path}"
        );
        let csp = header(&response, "content-security-policy");
        for directive in [
            "default-src 'self'",
            "connect-src 'self' https://api.example.run.app;",
            "https://fonts.googleapis.com",
            "font-src https://fonts.gstatic.com",
            "frame-ancestors 'none'",
            "object-src 'none'",
        ] {
            assert!(csp.contains(directive), "{path}: {directive} in {csp}");
        }
        assert_eq!(
            csp.contains("'unsafe-eval'"),
            path == "/technical-guide/",
            "{path}: 'unsafe-eval' only on the technical guide"
        );
        let cache = response
            .headers()
            .get("cache-control")
            .map(|v| v.to_str().unwrap());
        match path {
            "/" | "/technical-guide/" | "/config.js" => {
                assert_eq!(cache, Some("no-cache"), "{path}")
            }
            "/assets/small-world.svg" => assert_eq!(cache, Some("public, max-age=604800")),
            _ => assert_eq!(cache, None, "{path}"),
        }
    }
    let csp = header(&get(app(10).0, "/").await, "content-security-policy").to_owned();
    assert!(csp.contains("connect-src 'self';"), "{csp}");
}
