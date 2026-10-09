use serde_json::{json, Map, Value};

use crate::catalogue::Catalogue;
use crate::request::{preference_score, resolve};

#[derive(Debug)]
pub struct DecisionError(pub String);

fn labels(q: &Value) -> Vec<String> {
    match q.get("type").and_then(|t| t.as_str()) {
        Some("noul") => vec!["yes".into(), "no".into()],
        Some("choice") => q
            .get("criteria")
            .and_then(|c| c.as_object())
            .map(|o| o.keys().cloned().collect())
            .unwrap_or_default(),
        Some("score") => {
            let n = q
                .get("criteria")
                .and_then(|c| c.as_array())
                .map(|a| a.len())
                .unwrap_or(0);
            (0..n).map(|i| i.to_string()).collect()
        }
        _ => Vec::new(),
    }
}

fn one_hot(labs: &[String], gold: &str) -> Map<String, Value> {
    let mut probs = Map::new();
    for lab in labs {
        probs.insert(lab.clone(), json!(if lab == gold { 1.0 } else { 0.0 }));
    }
    probs
}

fn uniform(labs: &[String]) -> Map<String, Value> {
    let p = if labs.is_empty() {
        0.0
    } else {
        1.0 / labs.len() as f64
    };
    let mut probs = Map::new();
    for lab in labs {
        probs.insert(lab.clone(), json!(p));
    }
    probs
}

fn answer_value(q: &Value, probs: Map<String, Value>) -> Value {
    let qtype = q.get("type").and_then(|t| t.as_str()).unwrap_or("");
    let top = probs
        .iter()
        .max_by(|a, b| {
            a.1.as_f64()
                .unwrap_or(0.0)
                .partial_cmp(&b.1.as_f64().unwrap_or(0.0))
                .unwrap()
        })
        .map(|(k, _)| k.clone())
        .unwrap_or_default();
    let confidence = probs
        .values()
        .filter_map(|v| v.as_f64())
        .fold(0.0_f64, f64::max);
    let mut out = json!({"type": qtype, "probabilities": probs, "top": top, "confidence": (confidence * 100000.0).round() / 100000.0});
    if qtype == "noul" {
        out["noul"] = json!(
            ((out["probabilities"]["yes"].as_f64().unwrap_or(0.0)) * 100000.0).round() / 100000.0
        );
    }
    if qtype == "score" {
        let score: f64 = out["probabilities"]
            .as_object()
            .map(|o| {
                o.iter()
                    .map(|(k, v)| k.parse::<f64>().unwrap_or(0.0) * v.as_f64().unwrap_or(0.0))
                    .sum()
            })
            .unwrap_or(0.0);
        out["score"] = json!((score * 10000.0).round() / 10000.0);
        if let Some(criteria) = q.get("criteria").and_then(|c| c.as_array()) {
            let mut legend = Map::new();
            for (i, lvl) in criteria.iter().enumerate() {
                legend.insert(i.to_string(), lvl.clone());
            }
            out["legend"] = Value::Object(legend);
        }
    }
    out
}

pub fn decide(
    cat: &Catalogue,
    state: &Value,
    questions: &Map<String, Value>,
) -> Result<Value, DecisionError> {
    let request = state.get("request").and_then(|v| v.as_str()).unwrap_or("");
    let parsed = resolve(cat, request);
    let mut answers = Map::new();
    for (name, q) in questions {
        let labs = labels(q);
        if labs.is_empty() {
            return Err(DecisionError(format!("Unsupported question {name}.")));
        }
        let probs = if name == "gate" {
            one_hot(&labs, parsed["gate"].as_str().unwrap_or("clarify"))
        } else if cat.dimensions.contains_key(name) {
            let gold = parsed["levels"]
                .get(name)
                .and_then(|v| v.as_str())
                .unwrap_or("none");
            one_hot(&labs, gold)
        } else if let Some(id) = name.strip_prefix("group:") {
            let yes = parsed["groups"]
                .as_array()
                .is_some_and(|g| g.iter().any(|x| x.as_str() == Some(id)));
            one_hot(&labs, if yes { "yes" } else { "no" })
        } else if let Some(id) = name.strip_prefix("pollutant:") {
            let yes = parsed["pollutants"]
                .as_array()
                .is_some_and(|g| g.iter().any(|x| x.as_str() == Some(id)));
            one_hot(&labs, if yes { "yes" } else { "no" })
        } else if name == "relevant" {
            let instructions = q.get("instructions").and_then(|v| v.as_str()).unwrap_or("");
            let yes = parsed["pollutants"].as_array().is_some_and(|ps| {
                ps.iter().filter_map(|p| p.as_str()).any(|id| {
                    cat.pollutants
                        .iter()
                        .any(|p| p.id == id && instructions.contains(&p.describe))
                })
            });
            one_hot(&labs, if yes { "yes" } else { "no" })
        } else if name == "fit" && state.get("solution").is_some() {
            match preference_score(
                cat,
                state
                    .get("preference")
                    .and_then(|v| v.as_str())
                    .unwrap_or(""),
                &state["solution"],
            ) {
                None => uniform(&labs),
                Some(score) => one_hot(&labs, &score.to_string()),
            }
        } else {
            return Err(DecisionError(
                "Catalogue rules support pipeline extraction and explicit preferences only.".into(),
            ));
        };
        answers.insert(name.clone(), answer_value(q, probs));
    }
    Ok(json!({
        "backend": "catalogue",
        "model": "catalogue-rules-v1",
        "latency_ms": 0.0,
        "input_tokens": null,
        "cost_usd": 0.0,
        "residency": "local",
        "detail": {"by": "catalogue rules", "calibration": "not a model probability"},
        "answers": answers,
    }))
}
