use std::collections::{HashMap, HashSet};

use serde_json::{json, Value};

use crate::catalogue::Catalogue;

fn provides<'a>(cat: &'a Catalogue, pollutants: &[String]) -> HashMap<String, HashSet<String>> {
    let need: HashSet<&str> = pollutants.iter().map(String::as_str).collect();
    let mut out = HashMap::new();
    for s in &cat.sources {
        let held: HashSet<String> = s
            .pollutants
            .iter()
            .filter(|p| need.contains(p.as_str()))
            .cloned()
            .collect();
        if !held.is_empty() {
            out.insert(s.id.clone(), held);
        }
    }
    out
}

pub fn profile(
    cat: &Catalogue,
    source_id: &str,
    dim: &str,
    level: &str,
    constraint: Option<&Value>,
) -> HashMap<String, i32> {
    let source = cat.source(source_id);
    let Some(native) = source.levels.get(dim) else {
        return HashMap::new();
    };
    let rows = source.rows.get(dim).cloned().unwrap_or_default();
    if let Some(constraint) = constraint {
        let clevel = constraint["level"].as_str().unwrap_or("");
        if !cat.level_rolls_up(dim, native, clevel) {
            return HashMap::new();
        }
        let members: HashSet<String> = constraint["members"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        let mut filtered = HashMap::new();
        for (member, count) in rows {
            let Some(rolled) = cat.roll_up(dim, native, &member, clevel) else {
                continue;
            };
            if !members.contains(&rolled) {
                continue;
            }
            if let Some(target) = cat.roll_up(dim, native, &member, level) {
                *filtered.entry(target).or_insert(0) += count;
            }
        }
        return filtered;
    }
    if native == level {
        return rows;
    }
    let mut out = HashMap::new();
    for (member, count) in rows {
        if let Some(target) = cat.roll_up(dim, native, &member, level) {
            *out.entry(target).or_insert(0) += count;
        }
    }
    out
}

pub fn sources_for(
    cat: &Catalogue,
    pollutants: &[String],
    levels: &serde_json::Map<String, Value>,
    filters: Option<&serde_json::Map<String, Value>>,
) -> HashMap<String, HashSet<String>> {
    let provides = provides(cat, pollutants);
    if levels.is_empty() {
        return provides;
    }
    let mut ok = HashMap::new();
    for (sid, ps) in provides {
        let mut fine = true;
        for (dim, lvl) in levels {
            let lvl = lvl.as_str().unwrap_or("");
            let source = cat.source(&sid);
            let same = source.levels.get(dim).is_some_and(|n| n == lvl);
            let finer = source
                .levels
                .get(dim)
                .is_some_and(|n| cat.level_rolls_up(dim, n, lvl) && n != lvl);
            if !(same || finer) {
                fine = false;
                break;
            }
        }
        if fine {
            if let Some(filters) = filters {
                for (dim, constraint) in filters {
                    let native = cat.source(&sid).levels.get(dim);
                    let clevel = constraint["level"].as_str().unwrap_or("");
                    if native.is_none_or(|n| !cat.level_rolls_up(dim, n, clevel)) {
                        fine = false;
                        break;
                    }
                    let ask = levels.get(dim).and_then(|v| v.as_str()).unwrap_or(clevel);
                    if profile(cat, &sid, dim, ask, Some(constraint)).is_empty() {
                        fine = false;
                        break;
                    }
                }
            }
        }
        if fine {
            ok.insert(sid, ps);
        }
    }
    ok
}

fn combinations(items: &[String], k: usize) -> Vec<Vec<String>> {
    fn rec(
        items: &[String],
        k: usize,
        start: usize,
        cur: &mut Vec<String>,
        out: &mut Vec<Vec<String>>,
    ) {
        if cur.len() == k {
            out.push(cur.clone());
            return;
        }
        for i in start..items.len() {
            cur.push(items[i].clone());
            rec(items, k, i + 1, cur, out);
            cur.pop();
        }
    }
    let mut out = Vec::new();
    rec(items, k, 0, &mut Vec::new(), &mut out);
    out
}

fn ordered_profile(
    cat: &Catalogue,
    dim: &str,
    level: &str,
    shared: &HashSet<String>,
    per_source: &[HashMap<String, i32>],
) -> serde_json::Map<String, Value> {
    let order = cat.members_of(dim, level);
    let mut names: Vec<String> = shared.iter().cloned().collect();
    names.sort_by_key(|m| order.iter().position(|x| x == m).unwrap_or(usize::MAX));
    let mut map = serde_json::Map::new();
    for m in names {
        let n = per_source
            .iter()
            .filter_map(|p| p.get(&m))
            .copied()
            .min()
            .unwrap_or(0);
        map.insert(m, json!(n));
    }
    map
}

fn solution(
    cat: &Catalogue,
    combo: &[String],
    pollutants: &[String],
    levels: &serde_json::Map<String, Value>,
    candidates: &HashMap<String, HashSet<String>>,
    filters: Option<&serde_json::Map<String, Value>>,
) -> Value {
    let mut profiles = serde_json::Map::new();
    for (dim, lvl) in levels {
        let lvl = lvl.as_str().unwrap_or("");
        let constraint = filters.and_then(|f| f.get(dim));
        let per_source: Vec<HashMap<String, i32>> = combo
            .iter()
            .map(|s| profile(cat, s, dim, lvl, constraint))
            .collect();
        let shared = if per_source.is_empty() {
            HashSet::new()
        } else {
            let mut acc: HashSet<String> = per_source[0].keys().cloned().collect();
            for p in &per_source[1..] {
                acc.retain(|k| p.contains_key(k));
            }
            acc
        };
        profiles.insert(
            format!("{dim}.{lvl}"),
            Value::Object(ordered_profile(cat, dim, lvl, &shared, &per_source)),
        );
    }
    let mut cells: i64 = 1;
    for prof in profiles.values() {
        cells *= prof.as_object().map(|o| o.len() as i64).unwrap_or(0);
    }
    if levels.is_empty() {
        cells = 0;
    }
    let mut notes = Vec::new();
    for s in combo {
        let src = cat.source(s);
        for dim in src.levels.keys() {
            if levels.contains_key(dim) {
                continue;
            }
            let held = src.coverage.get(dim).cloned().unwrap_or_default();
            let native = &src.levels[dim];
            let total = cat.members_of(dim, native);
            if held.len() < total.len() {
                let what = if held.len() <= 2 {
                    held.join(" and ")
                } else {
                    let plural = cat.plural(native, total.len());
                    let rest = plural.split_once(' ').map(|(_, r)| r).unwrap_or(native);
                    format!("{} of {} {rest}", held.len(), total.len())
                };
                notes.push(format!(
                    "{} is summed over {} but covers only {what}",
                    src.name,
                    dim.to_lowercase()
                ));
            }
        }
    }
    let sources: Vec<Value> = combo.iter().map(|s| {
        let src = cat.source(s);
        let mut provides: Vec<String> = candidates[s].iter().filter(|p| pollutants.contains(p)).cloned().collect();
        provides.sort();
        json!({"id": s, "name": src.name, "publisher": src.publisher, "updated": src.updated, "provides": provides})
    }).collect();
    json!({"sources": sources, "profiles": profiles, "cells": cells, "notes": notes})
}

pub fn discover(
    cat: &Catalogue,
    pollutants: &[String],
    levels: &serde_json::Map<String, Value>,
    filters: Option<&serde_json::Map<String, Value>>,
) -> Vec<Value> {
    let candidates = sources_for(cat, pollutants, levels, filters);
    let need: HashSet<String> = pollutants.iter().cloned().collect();
    let mut ids: Vec<String> = candidates.keys().cloned().collect();
    ids.sort();
    let mut solutions: Vec<Vec<String>> = Vec::new();
    for k in 1..=3.min(ids.len()) {
        for combo in combinations(&ids, k) {
            let mut covered = HashSet::new();
            for s in &combo {
                covered.extend(candidates[s].iter().cloned());
            }
            if !need.is_subset(&covered) {
                continue;
            }
            if solutions
                .iter()
                .any(|prev| prev.iter().all(|p| combo.contains(p)))
            {
                continue;
            }
            solutions.push(combo);
        }
    }
    let mut out = Vec::new();
    for combo in solutions {
        let sol = solution(cat, &combo, pollutants, levels, &candidates, filters);
        if sol["cells"].as_i64().unwrap_or(0) <= 0 {
            continue;
        }
        let mut complete = true;
        if let Some(filters) = filters {
            for (dim, constraint) in filters {
                let clevel = constraint["level"].as_str().unwrap_or("");
                let profiles: Vec<HashMap<String, i32>> = combo
                    .iter()
                    .map(|s| profile(cat, s, dim, clevel, Some(constraint)))
                    .collect();
                if profiles.is_empty() {
                    complete = false;
                    break;
                }
                let mut shared: HashSet<String> = profiles[0].keys().cloned().collect();
                for p in &profiles[1..] {
                    shared.retain(|k| p.contains_key(k));
                }
                let members: HashSet<String> = constraint["members"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str().map(str::to_string))
                            .collect()
                    })
                    .unwrap_or_default();
                if !members.is_subset(&shared) {
                    complete = false;
                    break;
                }
            }
        }
        if !complete {
            continue;
        }
        out.push(sol);
        if out.len() >= 8 {
            break;
        }
    }
    out
}

pub fn subgraph(
    cat: &Catalogue,
    pollutants: &[String],
    levels: &serde_json::Map<String, Value>,
    source_ids: &[String],
) -> Value {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    for p in &cat.pollutants {
        if pollutants.iter().any(|id| id == &p.id) {
            nodes.push(json!({"id": format!("P:{}", p.id), "kind": "pollutant", "label": p.label}));
        }
    }
    for dim in ["GEO", "TIME", "SECTOR"] {
        let lvls = cat.dim_levels(dim);
        for lvl in lvls {
            nodes.push(json!({"id": format!("L:{dim}.{lvl}"), "kind": "level", "label": lvl, "dim": dim, "selected": levels.get(dim).and_then(|v| v.as_str()) == Some(lvl)}));
        }
        for pair in lvls.windows(2) {
            edges.push(json!({"from": format!("L:{dim}.{}", pair[0]), "to": format!("L:{dim}.{}", pair[1]), "kind": "rolls_up"}));
        }
    }
    for sid in source_ids {
        let s = cat.source(sid);
        nodes.push(json!({"id": format!("S:{sid}"), "kind": "source", "label": s.name}));
        for p in &s.pollutants {
            if pollutants.iter().any(|id| id == p) {
                edges.push(
                    json!({"from": format!("S:{sid}"), "to": format!("P:{p}"), "kind": "provides"}),
                );
            }
        }
        for (dim, lvl) in &s.levels {
            edges.push(json!({"from": format!("S:{sid}"), "to": format!("L:{dim}.{lvl}"), "kind": "at_level"}));
        }
    }
    json!({"nodes": nodes, "edges": edges})
}
