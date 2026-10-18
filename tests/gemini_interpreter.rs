//! The Gemini interpreter against a fake metadata server and a fake Vertex AI endpoint.
//! Nothing here leaves the machine.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use serde_json::{json, Value};
use serve::{router, App, Interpreter};
use tower::ServiceExt;

const TOKEN: &str = "fake-token";
const PROJECT: &str = "demo-project";
const GENERATE: &str =
    "/v1/projects/demo-project/locations/global/publishers/google/models/gemini-3.8-flash:generateContent";

#[derive(Clone)]
enum Reply {
    Reading(Value),
    Status(u16),
    Slow,
}

#[derive(Default)]
struct Seen {
    metadata_flavour: Vec<String>,
    authorisation: Vec<String>,
    bodies: Vec<Value>,
}

/// Serves the metadata token, the project id and generateContent on one local port.
async fn fake_google(reply: Reply) -> (String, Arc<Mutex<Seen>>) {
    let seen = Arc::new(Mutex::new(Seen::default()));
    let log = Arc::clone(&seen);
    let handler = move |request: Request<Body>| {
        let log = Arc::clone(&log);
        let reply = reply.clone();
        async move {
            let (parts, body) = request.into_parts();
            let header = |name: &str| {
                parts
                    .headers
                    .get(name)
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("")
                    .to_string()
            };
            let path = parts.uri.path().to_string();
            if path.starts_with("/computeMetadata/") {
                let flavour = header("metadata-flavor");
                log.lock().unwrap().metadata_flavour.push(flavour.clone());
                if flavour != "Google" {
                    return StatusCode::FORBIDDEN.into_response();
                }
                return match path.as_str() {
                    "/computeMetadata/v1/instance/service-accounts/default/token" => axum::Json(
                        json!({"access_token": TOKEN, "expires_in": 3599, "token_type": "Bearer"}),
                    )
                    .into_response(),
                    "/computeMetadata/v1/project/project-id" => PROJECT.into_response(),
                    _ => StatusCode::NOT_FOUND.into_response(),
                };
            }
            if path != GENERATE {
                return StatusCode::NOT_FOUND.into_response();
            }
            let auth = header("authorization");
            let body: Value =
                serde_json::from_slice(&to_bytes(body, usize::MAX).await.unwrap()).unwrap();
            {
                let mut log = log.lock().unwrap();
                log.authorisation.push(auth);
                log.bodies.push(body);
            }
            match reply {
                Reply::Reading(reading) => axum::Json(
                    json!({"candidates": [{"content": {"role": "model", "parts": [
                        {"text": "Mapping the request.", "thought": true},
                        {"text": reading.to_string()}
                    ]}}]}),
                )
                .into_response(),
                Reply::Status(code) => StatusCode::from_u16(code).unwrap().into_response(),
                Reply::Slow => {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    StatusCode::OK.into_response()
                }
            }
        }
    };
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let origin = listener.local_addr().unwrap().to_string();
    tokio::spawn(async move {
        axum::serve(listener, axum::Router::new().fallback(handler))
            .await
            .unwrap()
    });
    (origin, seen)
}

fn app(vars: &[(&str, &str)]) -> (axum::Router, Arc<Mutex<Vec<String>>>) {
    let vars: HashMap<String, String> = vars
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    let logs = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&logs);
    let state = Arc::new(
        App::load(
            &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("catalogue.json"),
            0.8,
            4_000,
            60,
        )
        .unwrap()
        .with_log_sink(move |line| sink.lock().unwrap().push(line))
        .with_interpreter(Interpreter::from_lookup(|key| vars.get(key).cloned()).unwrap()),
    );
    (
        router(
            state,
            &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static"),
        ),
        logs,
    )
}

fn gemini_app(origin: &str, extra: &[(&str, &str)]) -> (axum::Router, Arc<Mutex<Vec<String>>>) {
    let endpoint = format!("http://{origin}");
    let mut vars = vec![
        ("CATALOGUE_INTERPRETER", "gemini"),
        ("GEMINI_ENDPOINT", endpoint.as_str()),
        ("GCE_METADATA_HOST", origin),
    ];
    vars.extend_from_slice(extra);
    app(&vars)
}

async fn ask(app: axum::Router, request: &str) -> Value {
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/pipeline")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"domain": "catalogue", "request": request, "preference": null})
                        .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    body(response).await
}

async fn body(response: Response) -> Value {
    serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap()).unwrap()
}

fn reading(fields: Value) -> Value {
    let mut base = json!({
        "in_scope": true, "pollutants": [], "groups": [], "geo_level": "none",
        "time_level": "none", "sector_level": "none", "places": [], "sectors": [],
        "year_from": null, "year_to": null, "unsupported": []
    });
    for (key, value) in fields.as_object().unwrap() {
        base[key] = value.clone();
    }
    base
}

#[tokio::test]
async fn rules_read_requests_by_default_and_nothing_is_called() {
    let (origin, seen) = fake_google(Reply::Status(500)).await;
    let (router, logs) = app(&[
        ("GEMINI_ENDPOINT", &format!("http://{origin}")),
        ("GCE_METADATA_HOST", &origin),
    ]);
    let value = ask(router, "Annual PM2.5 by country, 2019 to 2024").await;
    assert_eq!(value["outcome"], "answer");
    assert!(value.get("interpretation").is_none(), "{value}");
    let seen = seen.lock().unwrap();
    assert!(seen.metadata_flavour.is_empty() && seen.bodies.is_empty());
    assert!(!logs.lock().unwrap().join("\n").contains("interpreter"));
}

#[tokio::test]
async fn gemini_reads_free_text_and_the_rules_decide_on_its_catalogue_terms() {
    let words = "SENTINEL could I see fine particles for each nation, yearly, 2019 through 2024?";
    let (origin, seen) = fake_google(Reply::Reading(reading(json!({
        "pollutants": ["PM2_5"], "geo_level": "country", "time_level": "year",
        "year_from": 2019, "year_to": 2024
    }))))
    .await;
    let (router, logs) = gemini_app(&origin, &[]);
    let value = ask(router, words).await;
    assert_eq!(value["outcome"], "answer", "{value}");
    assert_eq!(value["request"], words);
    assert_eq!(
        value["interpretation"],
        json!({
            "by": "gemini",
            "model": "gemini-3.8-flash",
            "request": "PM2.5 by country and year from 2019 to 2024",
            "note": null
        })
    );
    let rules_alone = ask(app(&[]).0, "PM2.5 by country and year from 2019 to 2024").await;
    assert_eq!(value["solutions"], rules_alone["solutions"]);

    let seen = seen.lock().unwrap();
    assert_eq!(seen.metadata_flavour, vec!["Google", "Google"]);
    assert_eq!(seen.authorisation, vec![format!("Bearer {TOKEN}")]);
    let sent = &seen.bodies[0];
    assert_eq!(sent["contents"][0]["parts"][0]["text"], words);
    assert!(!sent["systemInstruction"].to_string().contains("SENTINEL"));
    assert_eq!(
        sent["generationConfig"]["responseMimeType"],
        "application/json"
    );
    assert_eq!(
        sent["generationConfig"]["thinkingConfig"]["thinkingLevel"],
        "LOW"
    );
    assert!(
        sent["generationConfig"]["responseSchema"]["properties"]["places"]["items"]["enum"]
            .as_array()
            .unwrap()
            .contains(&json!("Bavaria"))
    );

    let captured = logs.lock().unwrap().join("\n");
    assert!(captured.contains(r#""interpreter":"gemini""#), "{captured}");
    assert!(!captured.contains("SENTINEL"));
}

#[tokio::test]
async fn the_token_is_reused_across_requests() {
    let (origin, seen) = fake_google(Reply::Reading(reading(json!({
        "pollutants": ["CO2"], "geo_level": "country", "places": ["Australia"],
        "year_from": 2024, "year_to": 2024
    }))))
    .await;
    let (router, _) = gemini_app(&origin, &[]);
    for _ in 0..3 {
        let value = ask(router.clone(), "carbon dioxide down under, 2024").await;
        assert_eq!(value["interpretation"]["by"], "gemini");
    }
    let seen = seen.lock().unwrap();
    assert_eq!(
        seen.metadata_flavour.len(),
        2,
        "one token and one project lookup"
    );
    assert_eq!(seen.bodies.len(), 3);
}

#[tokio::test]
async fn a_vertex_error_falls_back_to_the_rules_and_says_so() {
    let words = "Annual PM2.5 by country, 2019 to 2024";
    let (origin, _) = fake_google(Reply::Status(404)).await;
    let (router, logs) = gemini_app(&origin, &[]);
    let value = ask(router, words).await;
    let rules_alone = ask(app(&[]).0, words).await;
    assert_eq!(value["outcome"], rules_alone["outcome"]);
    assert_eq!(value["solutions"], rules_alone["solutions"]);
    assert_eq!(value["interpretation"]["by"], "rules");
    assert_eq!(value["interpretation"]["request"], words);
    assert_eq!(
        value["interpretation"]["note"],
        "Gemini unavailable: Vertex AI returned HTTP 404. The rules read your words."
    );
    assert!(logs
        .lock()
        .unwrap()
        .join("\n")
        .contains(r#""interpreter":"rules_fallback: Vertex AI returned HTTP 404""#));
}

#[tokio::test]
async fn a_slow_reply_falls_back_within_the_timeout() {
    let (origin, _) = fake_google(Reply::Slow).await;
    let (router, _) = gemini_app(&origin, &[("GEMINI_TIMEOUT_MS", "300")]);
    let started = std::time::Instant::now();
    let value = ask(router, "Emissions").await;
    assert!(started.elapsed() < Duration::from_secs(3));
    assert_eq!(value["outcome"], "clarify");
    assert_eq!(
        value["interpretation"]["note"],
        "Gemini unavailable: Gemini did not respond in time. The rules read your words."
    );
}

#[tokio::test]
async fn missing_credentials_fall_back_to_the_rules() {
    let (origin, seen) = fake_google(Reply::Status(500)).await;
    let endpoint = format!("http://{origin}");
    let (router, _) = app(&[
        ("CATALOGUE_INTERPRETER", "gemini"),
        ("GEMINI_ENDPOINT", &endpoint),
        ("GCE_METADATA_HOST", "127.0.0.1:9"),
    ]);
    let value = ask(router, "Emissions").await;
    assert_eq!(value["interpretation"]["by"], "rules");
    assert_eq!(
        value["interpretation"]["note"],
        "Gemini unavailable: no Google Cloud access token. The rules read your words."
    );
    assert!(seen.lock().unwrap().bodies.is_empty());
}

#[tokio::test]
async fn terms_outside_the_catalogue_leave_the_rules_to_decide_on_the_readers_words() {
    let words = "Hourly ozone by street in 1990";
    let (origin, _) = fake_google(Reply::Reading(reading(json!({
        "year_from": 1990, "year_to": 1990, "unsupported": ["hourly", "ozone", "street"]
    }))))
    .await;
    let (router, logs) = gemini_app(&origin, &[]);
    let value = ask(router, words).await;
    assert_eq!(value["outcome"], "reject");
    assert_eq!(value["interpretation"]["by"], "gemini");
    assert_eq!(value["interpretation"]["request"], words);
    assert_eq!(
        value["interpretation"]["note"],
        "Outside the catalogue: hourly, ozone, street. The rules decided on your words."
    );
    assert!(logs
        .lock()
        .unwrap()
        .join("\n")
        .contains(r#""interpreter":"gemini_outside_catalogue""#));
}

#[tokio::test]
async fn an_invented_term_is_treated_as_a_failure_not_a_reading() {
    let (origin, _) = fake_google(Reply::Reading(reading(json!({
        "pollutants": ["O3"], "geo_level": "country"
    }))))
    .await;
    let (router, _) = gemini_app(&origin, &[]);
    let value = ask(router, "ozone by country").await;
    assert_eq!(value["interpretation"]["by"], "rules");
    assert_eq!(
        value["interpretation"]["note"],
        "Gemini unavailable: Gemini returned terms outside the catalogue. The rules read your words."
    );
}

#[tokio::test]
async fn a_set_project_and_access_token_skip_the_metadata_server() {
    let (origin, seen) = fake_google(Reply::Reading(reading(json!({
        "pollutants": ["CH4"], "geo_level": "continent"
    }))))
    .await;
    let (router, _) = gemini_app(
        &origin,
        &[
            ("GOOGLE_CLOUD_PROJECT", PROJECT),
            ("GEMINI_ACCESS_TOKEN", "local-token"),
        ],
    );
    let value = ask(router, "methane per continent").await;
    assert_eq!(value["interpretation"]["request"], "CH4 by continent");
    let seen = seen.lock().unwrap();
    assert!(seen.metadata_flavour.is_empty());
    assert_eq!(seen.authorisation, vec!["Bearer local-token"]);
}
