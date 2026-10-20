use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures_util::StreamExt;
use reqwest::{redirect::Policy, StatusCode};
use serde_json::{json, Map, Value};
use tokio::sync::Semaphore;

use crate::catalogue::Catalogue;
use crate::decide::{decide, DecisionError};

const TYPESAFE_HOST: &str = "https://api.typesafe.ai";
const MAX_RESPONSE_BYTES: usize = 256 * 1024;
const MAX_ATTEMPTS: usize = 3;
const MAX_RETRY_AFTER: Duration = Duration::from_secs(2);
static RETRY_NONCE: AtomicU64 = AtomicU64::new(1);
const ACQUIRE_TIMEOUT: Duration = Duration::from_millis(100);

type DecisionFuture<'a> = Pin<Box<dyn Future<Output = Result<Value, DecisionError>> + Send + 'a>>;

#[derive(Clone)]
pub enum DecisionBackendConfig {
    Catalogue,
    Jev {
        api_key: String,
        model: String,
        timeout: Duration,
        price_per_mtok: Option<f64>,
        max_concurrency: usize,
        pipeline_max_calls: usize,
        pipeline_timeout: Duration,
    },
}

impl fmt::Debug for DecisionBackendConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Catalogue => f.debug_tuple("Catalogue").finish(),
            Self::Jev {
                model,
                timeout,
                price_per_mtok,
                max_concurrency,
                pipeline_max_calls,
                pipeline_timeout,
                ..
            } => f
                .debug_struct("Jev")
                .field("api_key", &"[REDACTED]")
                .field("model", model)
                .field("timeout", timeout)
                .field("price_per_mtok", price_per_mtok)
                .field("max_concurrency", max_concurrency)
                .field("pipeline_max_calls", pipeline_max_calls)
                .field("pipeline_timeout", pipeline_timeout)
                .finish(),
        }
    }
}

impl DecisionBackendConfig {
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Result<Self, String> {
        let get = |key: &str| lookup(key).filter(|value| !value.is_empty());
        match get("DECISION_BACKEND").as_deref().unwrap_or("catalogue") {
            "catalogue" => Ok(Self::Catalogue),
            "jev" => {
                if get("SERVICE_AUTH_MODE").as_deref() != Some("cloud-run-iam") {
                    return Err(
                        "SERVICE_AUTH_MODE=cloud-run-iam is required when DECISION_BACKEND=jev."
                            .into(),
                    );
                }
                let api_key = get("TYPESAFE_API_KEY")
                    .ok_or("TYPESAFE_API_KEY is required when DECISION_BACKEND=jev.")?;
                let model = get("JEV_MODEL").ok_or(
                    "JEV_MODEL is required when DECISION_BACKEND=jev; use an exact pinned model.",
                )?;
                let lower_model = model.to_ascii_lowercase();
                let plausible = lower_model.starts_with("jev-")
                    && lower_model.chars().all(|c| {
                        c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '-' | '.' | '_')
                    })
                    && lower_model.chars().any(|c| c.is_ascii_digit());
                if !plausible || lower_model.contains("latest") || lower_model.contains("preview") {
                    return Err(
                        "JEV_MODEL must be a plausible exact pinned model, not latest or preview."
                            .into(),
                    );
                }
                let timeout_ms = get("JEV_TIMEOUT_MS")
                    .unwrap_or_else(|| "10000".into())
                    .parse::<u64>()
                    .ok()
                    .filter(|value| (100..=60_000).contains(value))
                    .ok_or("JEV_TIMEOUT_MS must be between 100 and 60000.")?;
                let price_per_mtok = get("JEV_PRICE_PER_MTOK")
                    .map(|value| {
                        value
                            .parse::<f64>()
                            .ok()
                            .filter(|price| {
                                price.is_finite() && (0.0..=1_000_000.0).contains(price)
                            })
                            .ok_or(
                                "JEV_PRICE_PER_MTOK must be a finite number between 0 and 1000000.",
                            )
                    })
                    .transpose()?;
                let bounded = |key: &str, default: &str, min, max| {
                    get(key)
                        .unwrap_or_else(|| default.into())
                        .parse::<usize>()
                        .ok()
                        .filter(|value| (min..=max).contains(value))
                        .ok_or_else(|| format!("{key} must be between {min} and {max}."))
                };
                let max_concurrency = bounded("JEV_MAX_CONCURRENCY", "4", 1, 64)?;
                let pipeline_max_calls = bounded("JEV_PIPELINE_MAX_CALLS", "16", 1, 100)?;
                let pipeline_timeout_ms =
                    bounded("JEV_PIPELINE_TIMEOUT_MS", "30000", 100, 120_000)?;
                Ok(Self::Jev {
                    api_key,
                    model,
                    timeout: Duration::from_millis(timeout_ms),
                    price_per_mtok,
                    max_concurrency,
                    pipeline_max_calls,
                    pipeline_timeout: Duration::from_millis(pipeline_timeout_ms as u64),
                })
            }
            _ => Err("DECISION_BACKEND must be \"catalogue\" or \"jev\".".into()),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Catalogue => "catalogue",
            Self::Jev { .. } => "jev",
        }
    }
}

pub trait DecisionBackend: Send + Sync {
    fn decide<'a>(
        &'a self,
        state: &'a Value,
        questions: &'a Map<String, Value>,
    ) -> DecisionFuture<'a>;
    fn name(&self) -> &'static str;
    fn label(&self) -> &'static str;
    fn model(&self) -> &str;
    fn residency(&self) -> &'static str;
    fn auth_required(&self) -> bool {
        false
    }
    fn dependency_status(&self) -> &'static str {
        "local"
    }
    fn pipeline_max_calls(&self) -> usize {
        usize::MAX
    }
    fn pipeline_timeout(&self) -> Duration {
        Duration::from_secs(120)
    }
}

pub struct CatalogueBackend {
    cat: Arc<Catalogue>,
}

impl CatalogueBackend {
    pub fn new(cat: Arc<Catalogue>) -> Self {
        Self { cat }
    }
}

impl DecisionBackend for CatalogueBackend {
    fn decide<'a>(
        &'a self,
        state: &'a Value,
        questions: &'a Map<String, Value>,
    ) -> DecisionFuture<'a> {
        Box::pin(async move { decide(&self.cat, state, questions) })
    }
    fn name(&self) -> &'static str {
        "catalogue"
    }
    fn label(&self) -> &'static str {
        "Catalogue rules"
    }
    fn model(&self) -> &str {
        "catalogue-rules-v1"
    }
    fn residency(&self) -> &'static str {
        "local"
    }
}

#[derive(Clone)]
pub struct HostedJevSettings {
    pub api_key: String,
    pub model: String,
    pub base_url: String,
    pub timeout: Duration,
    pub price_per_mtok: Option<f64>,
}

#[derive(Clone, Copy)]
pub struct HostedJevLimits {
    pub max_concurrency: usize,
    pub pipeline_max_calls: usize,
    pub pipeline_timeout: Duration,
}

impl Default for HostedJevLimits {
    fn default() -> Self {
        Self {
            max_concurrency: 4,
            pipeline_max_calls: 16,
            pipeline_timeout: Duration::from_secs(30),
        }
    }
}

pub struct HostedJevBackend {
    http: reqwest::Client,
    api_key: String,
    model: String,
    endpoint: String,
    timeout: Duration,
    price_per_mtok: Option<f64>,
    semaphore: Arc<Semaphore>,
    pipeline_max_calls: usize,
    pipeline_timeout: Duration,
}

impl HostedJevBackend {
    pub fn production(
        settings: HostedJevSettings,
        limits: HostedJevLimits,
    ) -> Result<Self, String> {
        Self::new(
            HostedJevSettings {
                base_url: TYPESAFE_HOST.into(),
                ..settings
            },
            limits,
        )
    }

    #[doc(hidden)]
    pub fn for_test(
        api_key: String,
        model: String,
        base_url: String,
        timeout: Duration,
        price_per_mtok: Option<f64>,
    ) -> Result<Self, String> {
        Self::new(
            HostedJevSettings {
                api_key,
                model,
                base_url,
                timeout,
                price_per_mtok,
            },
            HostedJevLimits::default(),
        )
    }

    #[doc(hidden)]
    pub fn for_test_with_limits(
        settings: HostedJevSettings,
        limits: HostedJevLimits,
    ) -> Result<Self, String> {
        Self::new(settings, limits)
    }

    fn new(settings: HostedJevSettings, limits: HostedJevLimits) -> Result<Self, String> {
        let endpoint = format!("{}/v1/systemone", settings.base_url.trim_end_matches('/'));
        let http = reqwest::Client::builder()
            .redirect(Policy::none())
            .build()
            .map_err(|_| "Unable to configure hosted Jev client.")?;
        Ok(Self {
            http,
            api_key: settings.api_key,
            model: settings.model,
            endpoint,
            timeout: settings.timeout,
            price_per_mtok: settings.price_per_mtok,
            semaphore: Arc::new(Semaphore::new(limits.max_concurrency)),
            pipeline_max_calls: limits.pipeline_max_calls,
            pipeline_timeout: limits.pipeline_timeout,
        })
    }

    async fn call(
        &self,
        state: &Value,
        questions: &Map<String, Value>,
    ) -> Result<Value, DecisionError> {
        let nonce = RETRY_NONCE.fetch_add(1, Ordering::Relaxed);
        let started = Instant::now();
        let operation = async {
            let _permit =
                tokio::time::timeout(ACQUIRE_TIMEOUT.min(self.timeout), self.semaphore.acquire())
                    .await
                    .map_err(|_| DecisionError::unavailable())?
                    .map_err(|_| DecisionError::unavailable())?;
            for attempt in 0..MAX_ATTEMPTS {
                let response = self
                    .http
                    .post(&self.endpoint)
                    .bearer_auth(&self.api_key)
                    .json(&json!({"model": self.model, "state": state, "questions": questions}))
                    .send()
                    .await
                    .map_err(|_| DecisionError::unavailable())?;
                let status = response.status();
                if status.is_redirection() {
                    return Err(DecisionError::unavailable());
                }
                if retryable(status) && attempt + 1 < MAX_ATTEMPTS {
                    let wait =
                        retry_after(&response).unwrap_or_else(|| retry_delay(attempt, nonce));
                    tokio::time::sleep(wait.min(MAX_RETRY_AFTER)).await;
                    continue;
                }
                if !status.is_success() {
                    return Err(
                        if status.is_server_error() || status == StatusCode::TOO_MANY_REQUESTS {
                            DecisionError::unavailable()
                        } else {
                            DecisionError::malformed()
                        },
                    );
                }
                if response
                    .content_length()
                    .is_some_and(|size| size > MAX_RESPONSE_BYTES as u64)
                {
                    return Err(DecisionError::malformed());
                }
                let mut bytes = Vec::new();
                let mut stream = response.bytes_stream();
                while let Some(chunk) = stream.next().await {
                    let chunk = chunk.map_err(|_| DecisionError::unavailable())?;
                    if bytes.len() + chunk.len() > MAX_RESPONSE_BYTES {
                        return Err(DecisionError::malformed());
                    }
                    bytes.extend_from_slice(&chunk);
                }
                let value: Value =
                    serde_json::from_slice(&bytes).map_err(|_| DecisionError::malformed())?;
                return validate_response(value, questions, &self.model, self.price_per_mtok);
            }
            Err(DecisionError::unavailable())
        };
        let mut value = tokio::time::timeout(self.timeout, operation)
            .await
            .map_err(|_| DecisionError::unavailable())??;
        value["latency_ms"] = json!(started.elapsed().as_secs_f64() * 1_000.0);
        Ok(value)
    }
}

impl DecisionBackend for HostedJevBackend {
    fn decide<'a>(
        &'a self,
        state: &'a Value,
        questions: &'a Map<String, Value>,
    ) -> DecisionFuture<'a> {
        Box::pin(self.call(state, questions))
    }
    fn name(&self) -> &'static str {
        "jev"
    }
    fn label(&self) -> &'static str {
        "TypeSafe hosted Jev"
    }
    fn model(&self) -> &str {
        &self.model
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
        self.pipeline_max_calls
    }
    fn pipeline_timeout(&self) -> Duration {
        self.pipeline_timeout
    }
}

pub fn build_backend(
    config: DecisionBackendConfig,
    cat: Arc<Catalogue>,
) -> Result<Arc<dyn DecisionBackend>, String> {
    match config {
        DecisionBackendConfig::Catalogue => Ok(Arc::new(CatalogueBackend::new(cat))),
        DecisionBackendConfig::Jev {
            api_key,
            model,
            timeout,
            price_per_mtok,
            max_concurrency,
            pipeline_max_calls,
            pipeline_timeout,
        } => Ok(Arc::new(HostedJevBackend::production(
            HostedJevSettings {
                api_key,
                model,
                base_url: TYPESAFE_HOST.into(),
                timeout,
                price_per_mtok,
            },
            HostedJevLimits {
                max_concurrency,
                pipeline_max_calls,
                pipeline_timeout,
            },
        )?)),
    }
}

fn retryable(status: StatusCode) -> bool {
    status == StatusCode::TOO_MANY_REQUESTS || matches!(status.as_u16(), 500 | 502 | 503 | 504)
}

fn retry_after(response: &reqwest::Response) -> Option<Duration> {
    response
        .headers()
        .get(reqwest::header::RETRY_AFTER)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok())
        .map(Duration::from_secs)
}

fn retry_delay(attempt: usize, nonce: u64) -> Duration {
    let base = 100_u64 << attempt.min(4);
    let mixed = nonce
        .wrapping_mul(0x9E3779B97F4A7C15)
        .rotate_left(attempt as u32);
    Duration::from_millis(base + mixed % (base / 2 + 1))
}

fn labels(question: &Value) -> Result<Vec<String>, DecisionError> {
    match question.get("type").and_then(Value::as_str) {
        Some("noul") => Ok(vec!["yes".into(), "no".into()]),
        Some("choice") => question
            .get("criteria")
            .and_then(Value::as_object)
            .map(|v| v.keys().cloned().collect())
            .ok_or_else(DecisionError::malformed),
        Some("score") => question
            .get("criteria")
            .and_then(Value::as_array)
            .map(|v| (0..v.len()).map(|i| i.to_string()).collect())
            .ok_or_else(DecisionError::malformed),
        _ => Err(DecisionError::malformed()),
    }
}

fn exact_fields(object: &Map<String, Value>, allowed: &[&str]) -> Result<(), DecisionError> {
    (object.len() == allowed.len() && object.keys().all(|key| allowed.contains(&key.as_str())))
        .then_some(())
        .ok_or_else(DecisionError::malformed)
}

fn validate_response(
    mut value: Value,
    questions: &Map<String, Value>,
    configured_model: &str,
    price_per_mtok: Option<f64>,
) -> Result<Value, DecisionError> {
    let object = value.as_object().ok_or_else(DecisionError::malformed)?;
    exact_fields(object, &["model", "answers", "usage"])?;
    if object.get("model").and_then(Value::as_str) != Some(configured_model) {
        return Err(DecisionError::malformed());
    }
    let answers = object
        .get("answers")
        .and_then(Value::as_object)
        .ok_or_else(DecisionError::malformed)?;
    if answers.len() != questions.len() || !questions.keys().all(|key| answers.contains_key(key)) {
        return Err(DecisionError::malformed());
    }
    let normalized_answers: Map<String, Value> = questions
        .iter()
        .map(|(name, question)| {
            normalize_answer(answers.get(name).unwrap(), question)
                .map(|answer| (name.clone(), answer))
        })
        .collect::<Result<_, _>>()?;
    let usage = object
        .get("usage")
        .and_then(Value::as_object)
        .ok_or_else(DecisionError::malformed)?;
    exact_fields(usage, &["input_tokens", "output_tokens"])?;
    let input_tokens = usage
        .get("input_tokens")
        .and_then(Value::as_u64)
        .ok_or_else(DecisionError::malformed)?;
    let output_tokens = usage
        .get("output_tokens")
        .and_then(Value::as_u64)
        .ok_or_else(DecisionError::malformed)?;
    let cost = price_per_mtok.map(|price| input_tokens as f64 * price / 1_000_000.0);
    value["answers"] = Value::Object(normalized_answers);
    value.as_object_mut().unwrap().remove("usage");
    value["backend"] = json!("jev");
    value["residency"] = json!("hosted");
    value["cost_usd"] = json!(cost);
    value["input_tokens"] = json!(input_tokens);
    value["detail"] = json!({
        "served_by": "TypeSafe hosted Jev",
        "usage": {"output_tokens": output_tokens},
    });
    Ok(value)
}

fn normalize_answer(answer: &Value, question: &Value) -> Result<Value, DecisionError> {
    let object = answer.as_object().ok_or_else(DecisionError::malformed)?;
    let qtype = question
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(DecisionError::malformed)?;
    let allowed: &[&str] = match qtype {
        "choice" => &["type", "probabilities", "choice", "confidence"],
        "noul" => &["type", "noul"],
        "score" => &["type", "probabilities", "score", "confidence", "legend"],
        _ => return Err(DecisionError::malformed()),
    };
    exact_fields(object, allowed)?;
    if object.get("type").and_then(Value::as_str) != Some(qtype) {
        return Err(DecisionError::malformed());
    }

    if qtype == "noul" {
        let noul = object
            .get("noul")
            .and_then(Value::as_f64)
            .filter(|value| value.is_finite() && (0.0..=1.0).contains(value))
            .ok_or_else(DecisionError::malformed)?;
        let no = ((1.0 - noul) * 100_000.0).round() / 100_000.0;
        let top = if noul > no { "yes" } else { "no" };
        let confidence = ((2.0 * noul - 1.0).abs() * 100_000.0).round() / 100_000.0;
        return Ok(json!({
            "type": "noul",
            "probabilities": {"yes": noul, "no": no},
            "top": top,
            "noul": noul,
            "confidence": confidence,
        }));
    }

    let labels = labels(question)?;
    if labels.len() < 2 {
        return Err(DecisionError::malformed());
    }
    let probabilities = object
        .get("probabilities")
        .and_then(Value::as_object)
        .ok_or_else(DecisionError::malformed)?;
    if probabilities.len() != labels.len()
        || !labels.iter().all(|label| probabilities.contains_key(label))
    {
        return Err(DecisionError::malformed());
    }
    let mut sum = 0.0;
    let mut high = -1.0;
    for label in &labels {
        let probability = probabilities[label]
            .as_f64()
            .filter(|value| value.is_finite() && (0.0..=1.0).contains(value))
            .ok_or_else(DecisionError::malformed)?;
        sum += probability;
        if probability > high {
            high = probability;
        }
    }
    if (sum - 1.0).abs() > 0.001 {
        return Err(DecisionError::malformed());
    }
    let confidence = object
        .get("confidence")
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite() && (0.0..=1.0).contains(value))
        .ok_or_else(DecisionError::malformed)?;

    if qtype == "choice" {
        let provider_choice = object
            .get("choice")
            .and_then(Value::as_str)
            .filter(|choice| labels.iter().any(|label| label == choice))
            .ok_or_else(DecisionError::malformed)?;
        if (probabilities[provider_choice].as_f64().unwrap() - high).abs() > f64::EPSILON {
            return Err(DecisionError::malformed());
        }
        let uniform = 1.0 / labels.len() as f64;
        let expected_confidence = (high - uniform) / (1.0 - uniform);
        if (confidence - expected_confidence).abs() > 0.001 {
            return Err(DecisionError::malformed());
        }
        return Ok(json!({
            "type": "choice",
            "probabilities": probabilities,
            "top": provider_choice,
            "choice": provider_choice,
            "confidence": confidence,
        }));
    }

    let modal_index = labels
        .iter()
        .position(|label| probabilities[label].as_f64().unwrap() == high)
        .ok_or_else(DecisionError::malformed)?;
    let spread: f64 = labels
        .iter()
        .enumerate()
        .map(|(index, label)| {
            probabilities[label].as_f64().unwrap() * index.abs_diff(modal_index) as f64
        })
        .sum();
    let midpoint = (labels.len() - 1) as f64 / 2.0;
    let uniform_mad = (0..labels.len())
        .map(|index| (index as f64 - midpoint).abs())
        .sum::<f64>()
        / labels.len() as f64;
    let expected_confidence = (1.0 - spread / uniform_mad).max(0.0);
    if (confidence - expected_confidence).abs() > 0.001 {
        return Err(DecisionError::malformed());
    }
    let top = labels[modal_index].as_str();

    let expected = (labels
        .iter()
        .zip(0..)
        .map(|(label, index)| index as f64 * probabilities[label].as_f64().unwrap())
        .sum::<f64>()
        * 10_000.0)
        .round()
        / 10_000.0;
    if object
        .get("score")
        .and_then(Value::as_f64)
        .is_none_or(|value| !value.is_finite() || (value - expected).abs() > 0.001)
    {
        return Err(DecisionError::malformed());
    }
    let expected_legend: Map<String, Value> = question["criteria"]
        .as_array()
        .ok_or_else(DecisionError::malformed)?
        .iter()
        .enumerate()
        .map(|(index, criterion)| (index.to_string(), criterion.clone()))
        .collect();
    if object.get("legend") != Some(&Value::Object(expected_legend.clone())) {
        return Err(DecisionError::malformed());
    }
    Ok(json!({
        "type": "score",
        "probabilities": probabilities,
        "top": top,
        "score": expected,
        "confidence": confidence,
        "legend": expected_legend,
    }))
}

#[cfg(test)]
mod tests {
    use super::retry_delay;
    use std::collections::HashSet;

    #[test]
    fn retry_jitter_is_bounded_and_dispersed() {
        for attempt in 0..3 {
            let base = 100_u64 << attempt;
            let values: HashSet<u128> = (1..32)
                .map(|nonce| retry_delay(attempt, nonce).as_millis())
                .collect();
            assert!(values.len() > 8);
            assert!(values
                .iter()
                .all(|value| *value >= base as u128 && *value <= (base + base / 2) as u128));
        }
    }
}
