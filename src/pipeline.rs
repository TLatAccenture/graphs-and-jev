use std::time::Instant;

use serde_json::{json, Map, Value};

use crate::catalogue::Catalogue;
use crate::decide::{decide, DecisionError};
use crate::graph::{discover, subgraph};
use crate::request::{re, requested_groups, resolve};

const RELEVANCE_THRESHOLD: f64 = 0.5;
const MAX_CANDIDATES: usize = 8;
const MAX_RANKED: usize = 6;

fn norm_lex(s: &str) -> String {
    let s = s.to_lowercase().replace("sulfur", "sulphur");
    re(r"pm\s*2[.,]?\s*5").replace_all(&s, "pm2.5").into_owned()
}

fn lexical_candidates(cat: &Catalogue, request: &str) -> Vec<(String, f64)> {
    let text = norm_lex(request);
    let words: Vec<String> = re(r"[a-z0-9.]+")
        .find_iter(&text)
        .flatten()
        .map(|m| m.as_str().to_string())
        .collect();
    let stop = [
        "of",
        "and",
        "the",
        "matter",
        "oxides",
        "oxide",
        "carbon",
        "organic",
        "compounds",
        "particles",
    ];
    let mut scored = Vec::new();
    for p in &cat.pollutants {
        let mut best: f64 = 0.0;
        let names = std::iter::once(p.label.to_lowercase())
            .chain(std::iter::once(p.id.to_lowercase()))
            .chain(p.aliases.iter().map(|a| a.to_lowercase()));
        for name in names {
            let n = norm_lex(&name);
            let pattern = re(&format!(
                r"(?<![a-z0-9]){}(?![a-z0-9])",
                fancy_regex::escape(&n)
            ));
            if pattern.is_match(&text).unwrap_or(false) {
                best = 1.0;
                break;
            }
            let toks: Vec<&str> = re(r"[a-z0-9.]+")
                .find_iter(&n)
                .flatten()
                .map(|m| m.as_str())
                .filter(|w| !stop.contains(w))
                .collect();
            if toks.len() > 1 || toks.first().is_some_and(|t| t.len() > 3) {
                let share = if toks.is_empty() {
                    0.0
                } else {
                    toks.iter()
                        .filter(|w| words.iter().any(|x| x == *w))
                        .count() as f64
                        / toks.len() as f64
                };
                if share >= 0.5 {
                    best = best.max(0.5 * share);
                }
            }
        }
        if best > 0.0 {
            scored.push((p.id.clone(), best));
        }
    }
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    scored.truncate(MAX_CANDIDATES);
    scored
}

fn gate_q() -> Value {
    json!({"type": "choice", "criteria": {"answer": "answer", "clarify": "clarify", "reject": "reject"}})
}

fn noul_q(text: &str) -> Value {
    json!({"type": "noul", "instructions": text, "criteria": {"true": "yes", "false": "no"}})
}

fn level_q(dim: &str) -> Value {
    let criteria = match dim {
        "GEO" => {
            json!({"region": "region", "country": "country", "continent": "continent", "none": "none"})
        }
        "TIME" => json!({"month": "month", "year": "year", "none": "none"}),
        _ => json!({"subsector": "subsector", "macrosector": "macrosector", "none": "none"}),
    };
    json!({"type": "choice", "criteria": criteria})
}

fn fit_q() -> Value {
    json!({"type": "score", "criteria": ["none", "part", "most", "full"]})
}

fn uncertain(min_confidence: f64, answer: &Value) -> bool {
    answer["confidence"].as_f64().unwrap_or(0.0) < min_confidence
}

fn strs(v: &Value) -> Vec<String> {
    v.as_array()
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn summarise(cat: &Catalogue, sol: &Value, pollutants: &[String]) -> String {
    let mut parts = Vec::new();
    let sources = sol["sources"].as_array().cloned().unwrap_or_default();
    let srcs = sources
        .iter()
        .map(|s| {
            format!(
                "{} ({}, updated {})",
                s["name"].as_str().unwrap_or(""),
                s["publisher"].as_str().unwrap_or(""),
                s["updated"].as_i64().unwrap_or(0)
            )
        })
        .collect::<Vec<_>>()
        .join("; ");
    parts.push(format!("Sources: {srcs}."));
    if let Some(profiles) = sol["profiles"].as_object() {
        for (key, prof) in profiles {
            let mut bits = key.split('.');
            let dim = bits.next().unwrap_or("");
            let lvl = bits.next().unwrap_or("");
            let mems: Vec<String> = prof
                .as_object()
                .map(|o| o.keys().cloned().collect())
                .unwrap_or_default();
            if mems.is_empty() {
                parts.push(format!("No {lvl} is covered by every source."));
            } else if dim == "TIME" {
                let range = if mems.len() == 1 {
                    mems[0].clone()
                } else {
                    format!("{} to {}", mems[0], mems[mems.len() - 1])
                };
                parts.push(format!(
                    "{}s: {range} ({} covered).",
                    lvl[0..1].to_uppercase() + &lvl[1..],
                    mems.len()
                ));
            } else {
                let shown = if mems.len() > 8 {
                    format!("{} and {} more", mems[..8].join(", "), mems.len() - 8)
                } else {
                    mems.join(", ")
                };
                parts.push(format!("{}: {shown}.", cat.plural(lvl, mems.len())));
            }
        }
    }
    let labels: Vec<&str> = pollutants
        .iter()
        .take(8)
        .map(|id| cat.pollutant(id).label.as_str())
        .collect();
    let extra = if pollutants.len() > 8 {
        format!(" and {} more", pollutants.len() - 8)
    } else {
        String::new()
    };
    parts.push(format!("Pollutants: {}{extra}.", labels.join(", ")));
    for n in sol["notes"].as_array().into_iter().flatten() {
        if let Some(s) = n.as_str() {
            parts.push(format!("{s}."));
        }
    }
    parts.join(" ")
}

fn explain(
    cat: &Catalogue,
    pollutants: &[String],
    levels: &Map<String, Value>,
    ranked: &[Value],
) -> Value {
    let mut names: Vec<&str> = pollutants
        .iter()
        .take(6)
        .map(|id| cat.pollutant(id).label.as_str())
        .collect();
    let name = if pollutants.len() > 6 {
        format!("{} and {} more", names.join(", "), pollutants.len() - 6)
    } else {
        names.drain(..).collect::<Vec<_>>().join(", ")
    };
    let by = levels
        .values()
        .filter_map(|v| v.as_str())
        .collect::<Vec<_>>()
        .join(" and ");
    let mut lines = vec![format!(
        "{} solution{} hold {name} by {by}.",
        ranked.len(),
        if ranked.len() == 1 { "" } else { "s" }
    )];
    for (i, sol) in ranked.iter().enumerate() {
        let srcs = sol["sources"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|s| s["name"].as_str())
                    .collect::<Vec<_>>()
                    .join(" + ")
            })
            .unwrap_or_default();
        let mut line = format!(
            "{}: {}, {srcs}, covering {}.",
            if i == 0 { "First" } else { "Then" },
            sol["id"].as_str().unwrap_or(""),
            cover(cat, sol)
        );
        if let Some(fit) = sol.get("fit") {
            line.push_str(&format!(
                " Preference fit {:.1} of 3 ({:.0}% confident in its most likely level).",
                fit["score"].as_f64().unwrap_or(0.0),
                fit["confidence"].as_f64().unwrap_or(0.0) * 100.0
            ));
        }
        let notes: Vec<&str> = sol["notes"]
            .as_array()
            .map(|a| a.iter().filter_map(|n| n.as_str()).collect())
            .unwrap_or_default();
        if !notes.is_empty() {
            line.push_str(&format!(" Caution: {}.", notes.join("; ")));
        }
        lines.push(line);
        if i == 2 && ranked.len() > 3 {
            lines.push(format!(
                "{} more solution{} rank lower.",
                ranked.len() - 3,
                if ranked.len() - 3 == 1 { "" } else { "s" }
            ));
            break;
        }
    }
    json!({"by": "template", "text": lines.join("\n")})
}

fn cover(cat: &Catalogue, sol: &Value) -> String {
    let mut bits = Vec::new();
    if let Some(profiles) = sol["profiles"].as_object() {
        for (key, prof) in profiles {
            let mut split = key.split('.');
            let dim = split.next().unwrap_or("");
            let lvl = split.next().unwrap_or("");
            let mems: Vec<&str> = prof
                .as_object()
                .map(|o| o.keys().map(String::as_str).collect())
                .unwrap_or_default();
            if dim == "TIME" && !mems.is_empty() {
                bits.push(if mems.len() > 1 {
                    format!("{} to {}", mems[0], mems[mems.len() - 1])
                } else {
                    mems[0].to_string()
                });
            } else {
                bits.push(cat.plural(lvl, mems.len()));
            }
        }
    }
    bits.join(", ")
}

fn finish(
    mut out: Value,
    outcome: &str,
    message: Option<&str>,
    totals: &mut Map<String, Value>,
    started: Instant,
) -> Value {
    totals.insert(
        "latency_ms".into(),
        json!((totals["latency_ms"].as_f64().unwrap_or(0.0) * 10.0).round() / 10.0),
    );
    totals.insert(
        "wall_ms".into(),
        json!((started.elapsed().as_secs_f64() * 1000.0 * 10.0).round() / 10.0),
    );
    out["outcome"] = json!(outcome);
    out["message"] = json!(message);
    out["totals"] = Value::Object(totals.clone());
    out
}

pub fn run(
    cat: &Catalogue,
    request: &str,
    preference: Option<&str>,
    min_confidence: f64,
) -> Result<Value, DecisionError> {
    let started = Instant::now();
    let mut stages = Vec::new();
    let mut totals = json!({"decisions": 0, "calls": 0, "latency_ms": 0.0, "cost_usd": 0.0})
        .as_object()
        .unwrap()
        .clone();
    let preference = preference.unwrap_or("");
    let parsed = resolve(cat, request);
    let mut out = json!({
        "request": request,
        "preference": preference,
        "backend": "catalogue",
        "backend_label": "Catalogue rules",
        "model": "catalogue-rules-v1",
        "residency": "local",
        "stages": [],
        "policy": {
            "min_confidence": min_confidence,
            "validated_risk_guarantee": false,
            "calibration_loaded": false,
            "time_anchor": parsed["time_anchor"],
        },
    });

    let mut ask = |_name: &str,
                   state: Value,
                   questions: Map<String, Value>|
     -> Result<Value, DecisionError> {
        let res = decide(cat, &state, &questions)?;
        *totals.get_mut("calls").unwrap() = json!(totals["calls"].as_i64().unwrap_or(0) + 1);
        *totals.get_mut("decisions").unwrap() =
            json!(totals["decisions"].as_i64().unwrap_or(0) + questions.len() as i64);
        Ok(res)
    };

    let mut qs = Map::new();
    qs.insert("gate".into(), gate_q());
    let r = ask("gate", json!({"request": request}), qs)?;
    let gate = &r["answers"]["gate"];
    stages.push(json!({"stage": "gate", "title": "Answer, clarify or reject", "answer": gate}));
    if parsed["gate"] == "reject" {
        out["stages"] = json!(stages);
        return Ok(finish(
            out,
            "reject",
            Some("The request requires unsupported data, granularity or years outside 2015-2025."),
            &mut totals,
            started,
        ));
    }
    let issues = strs(&parsed["issues"]);
    if !issues.is_empty() {
        out["stages"] = json!(stages);
        return Ok(finish(
            out,
            "clarify",
            Some(&issues.join(" ")),
            &mut totals,
            started,
        ));
    }
    if uncertain(min_confidence, gate) {
        out["stages"] = json!(stages);
        return Ok(finish(
            out,
            "review",
            Some(
                "Gate confidence is below the acceptance threshold. Review or clarify the request.",
            ),
            &mut totals,
            started,
        ));
    }
    if gate["top"] != "answer" {
        let msg = if gate["top"] == "clarify" {
            "The request needs more detail: name a pollutant (or ask for emissions in general) and a breakdown such as country and year."
        } else {
            "The catalogue does not hold what this request needs."
        };
        out["stages"] = json!(stages);
        return Ok(finish(
            out,
            gate["top"].as_str().unwrap_or("reject"),
            Some(msg),
            &mut totals,
            started,
        ));
    }

    let mut cands: Vec<(String, String)> = cat
        .groups
        .iter()
        .map(|(g, meta)| (format!("group:{g}"), meta.describe.clone()))
        .collect();
    let lex = lexical_candidates(cat, request);
    for (pid, _) in &lex {
        cands.push((
            format!("pollutant:{pid}"),
            cat.pollutant(pid).describe.clone(),
        ));
    }
    let mut qs = Map::new();
    for (name, text) in &cands {
        qs.insert(name.clone(), noul_q(text));
    }
    let r = ask("relevance", json!({"request": request}), qs)?;
    let explicit = requested_groups(cat, request);
    let mut rel = Vec::new();
    let mut groups = Vec::new();
    let mut singles = Vec::new();
    for (name, _) in &cands {
        let yes = r["answers"][name]["probabilities"]["yes"]
            .as_f64()
            .unwrap_or(0.0);
        let (kind, id) = name.split_once(':').unwrap_or(("", ""));
        let selected = if kind == "group" {
            explicit.iter().any(|g| g == id)
        } else {
            yes >= RELEVANCE_THRESHOLD
        };
        let label = if kind == "group" {
            cat.groups[id].label.clone()
        } else {
            cat.pollutant(id).label.clone()
        };
        rel.push(json!({"node": name, "label": label, "kind": kind, "p": yes, "selected": selected, "by": if kind == "group" { "explicit catalogue group" } else { "model" }}));
        if selected && kind == "group" {
            groups.push(id.to_string());
        }
        if selected && kind == "pollutant" {
            singles.push(id.to_string());
        }
    }
    let mut pollutants: Vec<String> = singles;
    for g in &groups {
        for id in &cat.groups[g].members {
            if !pollutants.contains(id) {
                pollutants.push(id.clone());
            }
        }
    }
    pollutants = cat
        .pollutants
        .iter()
        .filter(|p| pollutants.contains(&p.id))
        .map(|p| p.id.clone())
        .collect();
    stages.push(json!({"stage": "enrich", "title": "Pollutants", "threshold": RELEVANCE_THRESHOLD, "graph_candidates": lex.iter().map(|(id, _)| id).collect::<Vec<_>>(), "nodes": rel, "groups_expanded": groups, "pollutants": pollutants}));
    let parsed_pollutants = strs(&parsed["pollutants"]);
    if !parsed_pollutants.is_empty()
        && parsed_pollutants
            .iter()
            .collect::<std::collections::HashSet<_>>()
            != pollutants.iter().collect()
    {
        out["stages"] = json!(stages);
        return Ok(finish(
            out,
            "review",
            Some("Model pollutants conflict with explicit catalogue terms; review the query."),
            &mut totals,
            started,
        ));
    }

    let mut qs = Map::new();
    for dim in ["GEO", "TIME", "SECTOR"] {
        qs.insert(dim.into(), level_q(dim));
    }
    let r = ask("levels", json!({"request": request}), qs)?;
    let mut levels = Map::new();
    if let Some(answers) = r["answers"].as_object() {
        for (dim, a) in answers {
            if a["top"] != "none" {
                levels.insert(dim.clone(), a["top"].clone());
            }
        }
    }
    let filters = parsed["filters"].as_object().cloned().unwrap_or_default();
    stages.push(json!({"stage": "query", "title": "Breakdown", "answers": r["answers"], "query": {"pollutants": pollutants, "levels": levels, "filters": filters}}));
    out["query"] = json!({"pollutants": pollutants, "levels": levels, "filters": filters});
    if r["answers"]
        .as_object()
        .is_some_and(|o| o.values().any(|a| uncertain(min_confidence, a)))
    {
        out["stages"] = json!(stages);
        return Ok(finish(
            out,
            "review",
            Some("A breakdown-level decision is below the acceptance threshold."),
            &mut totals,
            started,
        ));
    }
    if filters.keys().any(|dim| !levels.contains_key(dim)) {
        out["stages"] = json!(stages);
        return Ok(finish(
            out,
            "review",
            Some("The extracted levels omit a named member constraint."),
            &mut totals,
            started,
        ));
    }
    if let Some(parsed_levels) = parsed["levels"].as_object() {
        if !parsed_levels.is_empty()
            && parsed_levels
                .iter()
                .any(|(dim, lvl)| levels.get(dim) != Some(lvl))
        {
            out["stages"] = json!(stages);
            return Ok(finish(
                out,
                "review",
                Some("Model levels conflict with explicit catalogue terms; review the query."),
                &mut totals,
                started,
            ));
        }
    }
    if pollutants.is_empty() || levels.is_empty() {
        let missing = if pollutants.is_empty() {
            "a pollutant"
        } else {
            "a breakdown"
        };
        out["stages"] = json!(stages);
        return Ok(finish(out, "clarify", Some(&format!("The gate said answer, but no {missing} passed the threshold. Ask the user for {missing}.")), &mut totals, started));
    }

    let sols = discover(cat, &pollutants, &levels, Some(&filters));
    stages.push(json!({"stage": "discover", "title": "Discovery", "solutions": sols.len()}));
    if sols.is_empty() {
        out["subgraph"] = subgraph(cat, &pollutants, &levels, &[]);
        out["stages"] = json!(stages);
        return Ok(finish(
            out,
            "no_data",
            Some("No combination of sources holds these pollutants at these levels."),
            &mut totals,
            started,
        ));
    }

    let mut ranked = Vec::new();
    let mut rank_uncertain = false;
    for (i, mut sol) in sols.into_iter().enumerate() {
        sol["id"] = json!(((b'A' + i as u8) as char).to_string());
        sol["summary"] = json!(summarise(cat, &sol, &pollutants));
        if !preference.is_empty() {
            let mut qs = Map::new();
            qs.insert("fit".into(), fit_q());
            let state = json!({"request": request, "preference": preference, "solution": {"sources": sol["sources"], "profiles": sol["profiles"], "cells": sol["cells"]}});
            let r = ask("fit", state, qs)?;
            sol["fit"] = r["answers"]["fit"].clone();
            rank_uncertain |= uncertain(min_confidence, &sol["fit"]);
        }
        ranked.push(sol);
    }
    if preference.is_empty() {
        ranked.sort_by(|a, b| {
            b["cells"]
                .as_i64()
                .unwrap_or(0)
                .cmp(&a["cells"].as_i64().unwrap_or(0))
        });
    } else {
        ranked.sort_by(|a, b| {
            let fs = b["fit"]["score"]
                .as_f64()
                .unwrap_or(0.0)
                .partial_cmp(&a["fit"]["score"].as_f64().unwrap_or(0.0))
                .unwrap();
            fs.then(
                b["cells"]
                    .as_i64()
                    .unwrap_or(0)
                    .cmp(&a["cells"].as_i64().unwrap_or(0)),
            )
        });
    }
    ranked.truncate(MAX_RANKED);
    let order: Vec<Value> = ranked.iter().map(|s| s["id"].clone()).collect();
    stages.push(json!({"stage": "rank", "title": "Ranking", "by": if preference.is_empty() { "coverage (no preference given)" } else { "preference fit" }, "order": order}));
    let mut source_ids = Vec::new();
    for sol in &ranked {
        if let Some(sources) = sol["sources"].as_array() {
            for s in sources {
                if let Some(id) = s["id"].as_str() {
                    if !source_ids.contains(&id.to_string()) {
                        source_ids.push(id.to_string());
                    }
                }
            }
        }
    }
    source_ids.sort();
    out["solutions"] = json!(ranked);
    out["subgraph"] = subgraph(cat, &pollutants, &levels, &source_ids);
    if rank_uncertain {
        out["stages"] = json!(stages);
        return Ok(finish(out, "review", Some("Preference scores are uncertain. Candidate datasets are returned for review, not approved automatically."), &mut totals, started));
    }
    out["explanation"] = explain(cat, &pollutants, &levels, &ranked);
    stages.push(json!({"stage": "explain", "title": "Explanation", "by": "template"}));
    out["stages"] = json!(stages);
    Ok(finish(out, "answer", None, &mut totals, started))
}

#[cfg(test)]
mod tests {
    use super::run;
    use crate::catalogue::Catalogue;
    use std::path::Path;

    fn cat() -> Catalogue {
        Catalogue::load(Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/catalogue.json"
        )))
        .unwrap()
    }

    #[test]
    fn annual_co2_for_australia() {
        let out = run(&cat(), "Annual CO2 for Australia in 2024", None, 0.8).unwrap();
        assert_eq!(out["outcome"], "answer");
        let pollutants = out["query"]["pollutants"].as_array().unwrap();
        assert!(pollutants.iter().any(|p| p == "CO2"));
    }

    #[test]
    fn particulate_matter_by_region() {
        let out = run(&cat(), "Particulate matter by region", None, 0.8).unwrap();
        assert_eq!(out["outcome"], "answer");
        assert_eq!(
            out["query"]["pollutants"],
            serde_json::json!(["PM2_5", "PM10"])
        );
        assert_eq!(out["query"]["levels"]["GEO"], "region");
    }
}
