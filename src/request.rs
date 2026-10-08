use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};

use fancy_regex::Regex;

pub(crate) fn re(pat: &str) -> Regex {
    static CACHE: OnceLock<Mutex<HashMap<String, Regex>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = cache.lock().expect("regex cache");
    if let Some(found) = guard.get(pat) {
        return found.clone();
    }
    let compiled = Regex::new(pat).expect("pattern");
    guard.insert(pat.to_string(), compiled.clone());
    compiled
}
use serde_json::{json, Value};
use unicode_normalization::UnicodeNormalization;

use crate::catalogue::Catalogue;

fn combining(c: char) -> bool {
    unicode_normalization::char::canonical_combining_class(c) != 0
}

pub fn norm(text: &str) -> String {
    let stripped: String = text.nfkd().filter(|c| !combining(*c)).collect();
    let lower = stripped.to_lowercase().replace("sulfur", "sulphur");
    re(r"pm\s*2[.,]?\s*5")
        .replace_all(&lower, "pm2.5")
        .into_owned()
}

fn bounded(term: &str) -> Regex {
    re(&format!(
        r"(?<![a-z0-9]){}(?![a-z0-9])",
        regex_escape(&norm(term))
    ))
}

fn regex_escape(s: &str) -> String {
    fancy_regex::escape(s).into_owned()
}

pub fn contains(text: &str, term: &str) -> bool {
    bounded(term).is_match(&norm(text)).unwrap_or(false)
}

fn spans_cover(spans: &[(usize, usize)], start: usize, end: usize) -> bool {
    spans.iter().any(|&(a, b)| a <= start && end <= b)
}

pub fn named_pollutants(cat: &Catalogue, request: &str) -> Vec<String> {
    let text = norm(request);
    let mut spans = Vec::new();
    let mut found = HashSet::new();
    let mut terms = Vec::new();
    for p in &cat.pollutants {
        let mut names: Vec<&str> = std::iter::once(p.label.as_str())
            .chain(std::iter::once(p.id.as_str()))
            .chain(p.aliases.iter().map(String::as_str))
            .collect();
        if p.label.chars().count() <= 2 && !matches!(p.id.as_str(), "CO" | "BC" | "OC") {
            names.retain(|n| n.chars().count() > 2);
            let raw = re(&format!(
                r"(?<![A-Za-z0-9]){}(?![A-Za-z0-9])",
                regex_escape(&p.label)
            ));
            if raw.is_match(request).unwrap_or(false) {
                found.insert(p.id.clone());
            }
        }
        for name in names {
            terms.push((norm(name), p.id.clone()));
        }
    }
    terms.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
    for (term, pid) in terms {
        let re = re(&format!(
            r"(?<![a-z0-9]){}(?![a-z0-9])",
            regex_escape(&term)
        ));
        for m in re.find_iter(&text).flatten() {
            if !spans_cover(&spans, m.start(), m.end()) {
                found.insert(pid.clone());
                spans.push((m.start(), m.end()));
            }
        }
    }
    cat.pollutants
        .iter()
        .filter(|p| found.contains(&p.id))
        .map(|p| p.id.clone())
        .collect()
}

pub fn requested_groups(cat: &Catalogue, request: &str) -> Vec<String> {
    let text = norm(request);
    let mut groups = Vec::new();
    if re(r"\b(greenhouse gas(?:es)?|ghg)\b")
        .is_match(&text)
        .unwrap_or(false)
    {
        groups.push("ghg".into());
    }
    if contains(&text, "particulate matter")
        && !re(r"\b(fine|coarse) particulate")
            .is_match(&text)
            .unwrap_or(false)
    {
        groups.push("pm".into());
    }
    if re(r"\bheavy metals?\b").is_match(&text).unwrap_or(false) {
        groups.push("metal".into());
    }
    let specific = named_pollutants(cat, request);
    let broad = re(r"\b(pollution|emissions?|data|everything)\b")
        .is_match(&text)
        .unwrap_or(false);
    if contains(&text, "all pollutants") || (groups.is_empty() && specific.is_empty() && broad) {
        groups.insert(0, "all".into());
    }
    groups
}

fn mask_terms(cat: &Catalogue) -> Vec<String> {
    let mut known: Vec<String> = cat.aliases.keys().cloned().collect();
    for dim in ["GEO", "SECTOR"] {
        for lvl in cat.dim_levels(dim) {
            known.extend(cat.members_of(dim, lvl).iter().cloned());
        }
    }
    for p in &cat.pollutants {
        known.push(p.label.clone());
        known.push(p.id.clone());
        known.extend(p.aliases.clone());
    }
    for terms in cat.level_terms.values() {
        known.extend(terms.clone());
    }
    known.extend(
        [
            "greenhouse gas",
            "greenhouse gases",
            "particulate matter",
            "heavy metals",
            "heavy metal",
            "sector",
            "sectors",
            "industry",
        ]
        .into_iter()
        .map(str::to_string),
    );
    known.sort_by(|a, b| b.len().cmp(&a.len()));
    known
}

fn mask_catalogue_terms(cat: &Catalogue, text: &str) -> String {
    let mut text = norm(text);
    for term in mask_terms(cat) {
        let re = re(&format!(
            r"(?<![a-z0-9]){}(?![a-z0-9])",
            regex_escape(&norm(&term))
        ));
        text = re.replace_all(&text, " ").into_owned();
    }
    text
}

fn unresolved_constraints(cat: &Catalogue, text: &str) -> Vec<String> {
    let text = mask_catalogue_terms(cat, text);
    let ignored = [
        "all",
        "the",
        "a",
        "an",
        "and",
        "or",
        "each",
        "every",
        "last",
        "past",
        "one",
        "two",
        "three",
        "four",
        "five",
        "data",
        "dataset",
        "datasets",
        "source",
        "sources",
        "emission",
        "emissions",
        "pollution",
        "pollutants",
        "air",
        "countries",
        "continents",
        "geography",
        "temporal",
        "dimensions",
        "to",
        "through",
        "until",
        "between",
        "in",
        "for",
        "across",
        "from",
    ];
    let clause_re =
        re(r"\b(?:in|for|across|from)\s+?(.*?)(?=\b(?:by|per|with|where|aggregated)\b|[;.!?]|$)");
    let word_re = re(r"[a-z]+");
    let mut unknown = Vec::new();
    for c in clause_re.captures_iter(&text).flatten() {
        let clause = c.get(1).map(|m| m.as_str()).unwrap_or("");
        let words: Vec<String> = word_re
            .find_iter(clause)
            .flatten()
            .map(|m| m.as_str().to_string())
            .filter(|w| !ignored.contains(&w.as_str()))
            .collect();
        if !words.is_empty() {
            let phrase = words.join(" ");
            if !unknown.contains(&phrase) {
                unknown.push(phrase);
            }
        }
    }
    unknown
}

fn unresolved_request_terms(cat: &Catalogue, text: &str) -> Vec<String> {
    let allowed = [
        "i",
        "we",
        "you",
        "would",
        "like",
        "wish",
        "want",
        "to",
        "can",
        "could",
        "please",
        "me",
        "gimme",
        "give",
        "gather",
        "collect",
        "find",
        "show",
        "compare",
        "analyse",
        "analyze",
        "aanlyse",
        "obtain",
        "join",
        "pull",
        "get",
        "break",
        "it",
        "down",
        "what",
        "about",
        "how",
        "much",
        "did",
        "emit",
        "emitted",
        "measure",
        "measures",
        "measured",
        "containing",
        "contain",
        "contains",
        "which",
        "where",
        "with",
        "also",
        "information",
        "are",
        "is",
        "of",
        "the",
        "a",
        "an",
        "and",
        "or",
        "in",
        "for",
        "across",
        "from",
        "by",
        "per",
        "all",
        "each",
        "every",
        "last",
        "past",
        "one",
        "two",
        "three",
        "four",
        "five",
        "through",
        "until",
        "between",
        "data",
        "dataset",
        "datasets",
        "source",
        "sources",
        "pollution",
        "pollutant",
        "pollutants",
        "emission",
        "emissions",
        "air",
        "aggregated",
        "aggregate",
        "aggregation",
        "level",
        "levels",
        "value",
        "values",
        "indicator",
        "indicators",
        "figures",
        "numbers",
        "results",
        "everything",
        "have",
        "ghg",
        "geography",
        "geographic",
        "temporal",
        "time",
        "dimensions",
        "official",
        "only",
        "most",
        "recent",
        "latest",
        "newest",
        "available",
        "updated",
        "prefer",
    ];
    let masked = mask_catalogue_terms(cat, text);
    let word_re = re(r"[a-z][a-z0-9]*");
    let mut out = Vec::new();
    for m in word_re.find_iter(&masked).flatten() {
        let w = m.as_str();
        if !allowed.contains(&w) && !out.contains(&w.to_string()) {
            out.push(w.to_string());
        }
    }
    out
}

pub fn resolve(cat: &Catalogue, request: &str) -> Value {
    let text = norm(request);
    let mut levels: serde_json::Map<String, Value> = serde_json::Map::new();
    let mut filters: serde_json::Map<String, Value> = serde_json::Map::new();
    let mut issues = Vec::new();
    let mut pollutants = named_pollutants(cat, request);
    let groups = requested_groups(cat, request);
    for g in &groups {
        if let Some(group) = cat.groups.get(g) {
            for id in &group.members {
                if !pollutants.contains(id) {
                    pollutants.push(id.clone());
                }
            }
        }
    }
    pollutants = cat
        .pollutants
        .iter()
        .filter(|p| pollutants.contains(&p.id))
        .map(|p| p.id.clone())
        .collect();

    for dim in ["GEO", "TIME", "SECTOR"] {
        let mut explicit: Vec<&str> = cat
            .dim_levels(dim)
            .iter()
            .map(String::as_str)
            .filter(|lvl| {
                cat.level_terms
                    .get(*lvl)
                    .is_some_and(|terms| terms.iter().any(|t| contains(&text, t)))
            })
            .collect();
        if dim == "TIME" && explicit.contains(&"month") {
            explicit = vec!["month"];
        }
        if explicit.len() > 1 {
            issues.push(format!(
                "Choose one {} aggregation level.",
                dim.to_lowercase()
            ));
        }
        if let Some(lvl) = explicit.first() {
            levels.insert(dim.into(), json!(lvl));
        } else if dim == "GEO"
            && re(r"\b(geography|geographic)\b")
                .is_match(&text)
                .unwrap_or(false)
        {
            levels.insert(dim.into(), json!("country"));
        } else if dim == "TIME" && re(r"\b(temporal|time)\b").is_match(&text).unwrap_or(false) {
            levels.insert(dim.into(), json!("year"));
        } else if dim == "SECTOR"
            && re(r"\b(sectors?|industry)\b")
                .is_match(&text)
                .unwrap_or(false)
        {
            levels.insert(dim.into(), json!("macrosector"));
        }
    }

    for dim in ["GEO", "SECTOR"] {
        let mut terms: Vec<(String, String, String)> = Vec::new();
        for lvl in cat.dim_levels(dim) {
            for m in cat.members_of(dim, lvl) {
                terms.push((m.clone(), lvl.clone(), m.clone()));
            }
        }
        if dim == "GEO" {
            for (alias, member) in &cat.aliases {
                if let Some(lvl) = cat
                    .dim_levels(dim)
                    .iter()
                    .find(|l| cat.members_of(dim, l).iter().any(|m| m == member))
                {
                    terms.push((alias.clone(), lvl.clone(), member.clone()));
                }
            }
        }
        terms.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
        let mut matches = Vec::new();
        let mut occupied = Vec::new();
        for (term, lvl, member) in &terms {
            let re = re(&format!(
                r"(?<![a-z0-9]){}(?![a-z0-9])",
                regex_escape(&norm(term))
            ));
            for m in re.find_iter(&text).flatten() {
                if !spans_cover(&occupied, m.start(), m.end()) {
                    occupied.push((m.start(), m.end()));
                    matches.push((lvl.clone(), member.clone()));
                }
            }
        }
        if !matches.is_empty() {
            let ranks: HashSet<usize> = matches
                .iter()
                .filter_map(|(l, _)| cat.dim_levels(dim).iter().position(|x| x == l))
                .collect();
            if ranks.len() > 1 {
                issues.push(format!(
                    "Mixed {} member levels need clarification.",
                    dim.to_lowercase()
                ));
            }
            let lvl = matches
                .iter()
                .min_by_key(|(l, _)| {
                    cat.dim_levels(dim)
                        .iter()
                        .position(|x| x == l)
                        .unwrap_or(99)
                })
                .unwrap()
                .0
                .clone();
            let mut members = Vec::new();
            for (l, m) in &matches {
                if l == &lvl && !members.contains(m) {
                    members.push(m.clone());
                }
            }
            filters.insert(dim.into(), json!({"level": lvl, "members": members}));
            levels.entry(dim.to_string()).or_insert(json!(lvl));
        }
    }

    let year_re = re(r"(?<![0-9])(?:18|19|20|21)[0-9]{2}(?![0-9])");
    let mut year_tokens: Vec<i32> = year_re
        .find_iter(&text)
        .flatten()
        .filter_map(|m| m.as_str().parse().ok())
        .collect();
    let relative = re(r"\b(?:last|past) (\d+|one|two|three|four|five) years?\b")
        .captures(&text)
        .ok()
        .flatten();
    if let Some(rel) = &relative {
        let number = rel.get(1).unwrap().as_str();
        let count: i32 = number.parse().unwrap_or(match number {
            "one" => 1,
            "two" => 2,
            "three" => 3,
            "four" => 4,
            "five" => 5,
            _ => 0,
        });
        let max_year = *cat.years.iter().max().unwrap_or(&2025);
        year_tokens = vec![max_year - count + 1, max_year];
    }
    if !year_tokens.is_empty() {
        levels.entry("TIME".to_string()).or_insert(json!("year"));
        let start = *year_tokens.iter().min().unwrap();
        let end = *year_tokens.iter().max().unwrap();
        let date = r"\d{4}(?:-\d{2})?";
        let continuous = relative.is_some() || re(&format!(r"\bbetween\s+{date}\s+and\s+{date}|{date}\s*(?:-|\u{{2013}}|to|through|until)\s*{date}")).is_match(&text).unwrap_or(false);
        let mut selected: Vec<i32> = if continuous {
            (start..=end).collect()
        } else {
            year_tokens.clone()
        };
        selected.sort();
        selected.dedup();
        let iso = re(r"\b(\d{4}-(?:0[1-9]|1[0-2]))\b");
        let iso_months: Vec<String> = iso
            .captures_iter(&text)
            .flatten()
            .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
            .collect();
        if !iso_months.is_empty() {
            let selected_months = if continuous {
                months(
                    iso_months.iter().min().unwrap(),
                    iso_months.iter().max().unwrap(),
                )
            } else {
                let mut m = iso_months.clone();
                m.sort();
                m.dedup();
                m
            };
            filters.insert(
                "TIME".into(),
                json!({"level": "month", "members": selected_months}),
            );
            if !cat
                .level_terms
                .get("year")
                .is_some_and(|t| t.iter().any(|term| contains(&text, term)))
            {
                levels.insert("TIME".into(), json!("month"));
            } else {
                levels.entry("TIME".to_string()).or_insert(json!("month"));
            }
        } else {
            let members: Vec<String> = selected.iter().map(|y| y.to_string()).collect();
            filters.insert("TIME".into(), json!({"level": "year", "members": members}));
        }
    }
    for qualifier in unresolved_constraints(cat, request) {
        issues.push(format!(
            "Unrecognised constraint '{qualifier}'; use catalogue members or clarify the request."
        ));
    }
    let unknown = unresolved_request_terms(cat, request);
    if !unknown.is_empty() && issues.is_empty() {
        issues.push(format!(
            "Unrecognised request terms: {}. Use catalogue terms or clarify the request.",
            unknown.join(", ")
        ));
    }
    let mut unsupported = re(r"\b(cities|city|streets?|stations?|daily|days?|hourly|hours?|weeks?|forecasts?|water|noise|radon|poem|world cup)\b").is_match(&text).unwrap_or(false);
    let outside = year_tokens.iter().any(|y| !cat.years.contains(y));
    let unheld = re(r"\b(ozone|no2|sf6|sulphur hexafluoride)\b")
        .is_match(&text)
        .unwrap_or(false);
    if unheld && named_pollutants(cat, request).is_empty() {
        unsupported = true;
        pollutants.clear();
    }
    let off_topic = pollutants.is_empty()
        && levels.is_empty()
        && !re(r"\b(pollution|emissions?|data|everything)\b")
            .is_match(&text)
            .unwrap_or(false);
    let gate = if unsupported || outside || off_topic {
        "reject"
    } else if !issues.is_empty() || pollutants.is_empty() || levels.is_empty() {
        "clarify"
    } else {
        "answer"
    };
    let time_anchor = if relative.is_some() {
        cat.years.iter().max().copied()
    } else {
        None
    };
    json!({
        "gate": gate,
        "pollutants": pollutants,
        "groups": groups,
        "levels": levels,
        "filters": filters,
        "issues": issues,
        "time_anchor": time_anchor,
    })
}

fn months(start: &str, end: &str) -> Vec<String> {
    let mut parts = start.split('-');
    let mut y: i32 = parts.next().unwrap_or("2015").parse().unwrap_or(2015);
    let mut m: i32 = parts.next().unwrap_or("1").parse().unwrap_or(1);
    let mut end_parts = end.split('-');
    let ye: i32 = end_parts.next().unwrap_or("2025").parse().unwrap_or(2025);
    let me: i32 = end_parts.next().unwrap_or("12").parse().unwrap_or(12);
    let mut out = Vec::new();
    while (y, m) <= (ye, me) {
        out.push(format!("{y}-{m:02}"));
        m += 1;
        if m == 13 {
            y += 1;
            m = 1;
        }
    }
    out
}

pub fn preference_score(cat: &Catalogue, preference: &str, solution: &Value) -> Option<i32> {
    let text = norm(preference);
    if text.contains("as many") || text.contains("coverage") {
        return None;
    }
    let allowed = [
        "official",
        "sources",
        "source",
        "only",
        "the",
        "most",
        "recent",
        "latest",
        "newest",
        "data",
        "available",
        "updated",
        "prefer",
        "please",
        "in",
        "for",
        "across",
        "from",
        "and",
        "with",
        "by",
        "per",
        "all",
        "each",
        "every",
        "last",
        "past",
        "one",
        "two",
        "three",
        "four",
        "five",
        "to",
        "through",
        "until",
        "between",
        "countries",
        "regions",
        "continents",
    ];
    let masked = mask_catalogue_terms(cat, preference);
    let word_re = re(r"[a-z]+");
    if word_re
        .find_iter(&masked)
        .flatten()
        .any(|m| !allowed.contains(&m.as_str()))
    {
        return None;
    }
    let mut scores = Vec::new();
    if contains(&text, "official") {
        let sources = solution["sources"].as_array().cloned().unwrap_or_default();
        let official: Vec<bool> = sources
            .iter()
            .map(|s| s["publisher"] == "official")
            .collect();
        let score = if official.iter().all(|v| *v) {
            3
        } else if official.iter().any(|v| *v) {
            1
        } else {
            0
        };
        scores.push(score);
    }
    if re(r"\b(recent|latest|newest)\b")
        .is_match(&text)
        .unwrap_or(false)
    {
        let latest = solution["sources"]
            .as_array()
            .and_then(|s| s.iter().filter_map(|x| x["updated"].as_i64()).min())
            .unwrap_or(0) as i32;
        let max_year = *cat.years.iter().max().unwrap_or(&2025);
        scores.push((3 - (max_year - latest)).max(0));
    }
    let parsed = resolve(cat, preference);
    let groups = parsed["groups"].as_array().cloned().unwrap_or_default();
    if !parsed["issues"].as_array().is_none_or(|a| a.is_empty())
        || !named_pollutants(cat, preference).is_empty()
        || groups.iter().any(|g| g != "all")
    {
        return None;
    }
    let constraints = parsed["filters"].as_object().cloned().unwrap_or_default();
    let profiles = solution["profiles"]
        .as_object()
        .cloned()
        .unwrap_or_default();
    for (dim, constraint) in constraints {
        let key = profiles
            .keys()
            .find(|k| k.starts_with(&(dim.clone() + ".")))
            .cloned();
        if let Some(key) = key {
            let native = key.split('.').nth(1).unwrap_or("");
            let level = constraint["level"].as_str().unwrap_or("");
            let wanted: Vec<String> = constraint["members"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default();
            let mut covered = HashSet::new();
            if let Some(prof) = profiles[&key].as_object() {
                for member in prof.keys() {
                    if let Some(up) = cat.roll_up(&dim, native, member, level) {
                        covered.insert(up);
                    }
                }
            }
            let share = if wanted.is_empty() {
                0.0
            } else {
                wanted.iter().filter(|m| covered.contains(*m)).count() as f64 / wanted.len() as f64
            };
            scores.push((3.0 * share) as i32);
        } else {
            scores.push(0);
        }
    }
    if scores.is_empty() {
        None
    } else {
        scores.into_iter().min()
    }
}
