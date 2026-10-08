//! Parkinson's research path: reject clinical asks, otherwise fetch a few
//! public pages and read them as a labelled graph. Citations are only URLs
//! this request fetched.

use std::time::{SystemTime, UNIX_EPOCH};

use fancy_regex::Regex;
use serde_json::{json, Value};

use crate::request::re;

#[derive(Clone)]
pub struct Page {
    pub url: String,
    pub title: String,
    pub text: String,
}

pub trait Retrieve {
    fn search(&self, query: &str) -> Result<Vec<Page>, String>;
}

#[derive(Debug)]
pub struct ResearchError(pub String);

impl ResearchError {
    pub fn class(&self) -> &'static str {
        let message = self.0.to_ascii_lowercase();
        if message.contains("timed out") || message.contains("timeout") {
            "timeout"
        } else if message.contains("json") || message.contains("decode") {
            "invalid"
        } else {
            "fetch"
        }
    }
}

struct Hit {
    from: String,
    from_label: &'static str,
    ty: &'static str,
    to: String,
    to_label: &'static str,
    url: String,
    title: String,
}

pub fn run(request: &str, retrieve: &impl Retrieve) -> Result<Value, ResearchError> {
    let gate = if clinical(request) {
        "reject"
    } else {
        "answer"
    };
    if gate == "reject" {
        return Ok(body(
            request,
            "reject",
            "This path does not diagnose, prescribe, or give a dose.",
            "The question asks for a dose, a diagnosis, or a prescription. No pages were retrieved.",
            &[],
            &[],
        ));
    }
    let pages = retrieve.search(request).map_err(ResearchError)?;
    if pages.is_empty() {
        return Ok(body(
            request,
            "no_data",
            "No public pages came back for this question.",
            "The search returned nothing to cite.",
            &[],
            &[],
        ));
    }
    let hits = paths(&pages);
    if hits.is_empty() {
        return Ok(body(
            request,
            "clarify",
            "Pages were retrieved, and none of them stated a typed relationship.",
            "No targets, binds, associates, treats, or studied-in path was stated in the retrieved text.",
            &pages,
            &[],
        ));
    }
    let explanation = explain(&hits);
    Ok(body(
        request,
        "answer",
        "A retrieved page states this relationship.",
        &explanation,
        &pages,
        &hits,
    ))
}

fn clinical(request: &str) -> bool {
    let text = request.to_lowercase();
    [
        "dose",
        "doses",
        "dosage",
        "diagnosis",
        "diagnose",
        "prescription",
        "prescribe",
        "should i take",
    ]
    .iter()
    .any(|term| term_in(&text, term))
        || re(r"(?i)\bmg\b").is_match(&text).unwrap_or(false)
}

fn term_in(text: &str, term: &str) -> bool {
    re(&format!(r"(?i)\b{}\b", fancy_regex::escape(term)))
        .is_match(text)
        .unwrap_or(false)
}

fn paths(pages: &[Page]) -> Vec<Hit> {
    let mut hits = Vec::new();
    for page in pages {
        if !page.url.starts_with("https://") {
            continue;
        }
        let text: String = page.text.chars().take(2000).collect();
        for hit in scan(&text) {
            hits.push(Hit {
                from: hit.0,
                from_label: hit.1,
                ty: hit.2,
                to: hit.3,
                to_label: hit.4,
                url: page.url.clone(),
                title: page.title.clone(),
            });
            if hits.len() == 6 {
                return hits;
            }
        }
    }
    hits
}

fn scan(text: &str) -> Vec<(String, &'static str, &'static str, String, &'static str)> {
    let mut out = Vec::new();
    take(
        &mut out,
        text,
        r"\b([A-Z][A-Za-z0-9-]{1,40})\b(?:\s+[A-Za-z0-9-]{1,30}){0,4}\s+(?i:targets|binds)\s+([A-Za-z0-9][A-Za-z0-9-]{2,40})",
        |verb, from| {
            let ty = if verb.to_lowercase().contains("binds") {
                "binds"
            } else {
                "targets"
            };
            (label_for(from), ty, "Protein")
        },
    );
    take(
        &mut out,
        text,
        r"\b([A-Z][A-Z0-9]{2,7})\b(?:\s+[A-Za-z0-9-]{1,20}){0,3}\s+(?i:associated\s+with)\s+([A-Za-z][A-Za-z-]{2,30}(?:\s+[A-Za-z-]{2,20}){0,2})",
        |_, _| ("Gene", "associates", "Disease"),
    );
    take(
        &mut out,
        text,
        r"\b([A-Z][A-Za-z0-9-]{1,40})\b(?:\s+[A-Za-z0-9-]{1,30}){0,5}\s+(?i:approved\s+for)\s+([A-Za-z][A-Za-z-]{2,20}(?:\s+[A-Za-z-]{2,20}){0,3})",
        |_, _| ("Compound", "treats", "Symptom"),
    );
    take(
        &mut out,
        text,
        r"\b([A-Z][A-Za-z0-9-]{1,40})\b(?:\s+[A-Za-z0-9-]{1,20}){0,4}\s+(?i:studied\s+in)\s+([A-Z][A-Za-z0-9-]{1,40})",
        |_, from| (label_for(from), "studied_in", "Trial"),
    );
    out
}

fn take(
    out: &mut Vec<(String, &'static str, &'static str, String, &'static str)>,
    text: &str,
    pat: &str,
    kind: fn(&str, &str) -> (&'static str, &'static str, &'static str),
) {
    let pattern: Regex = re(pat);
    for caps in pattern.find_iter(text).flatten().take(3) {
        let whole = caps.as_str();
        let Some(groups) = pattern.captures(whole).ok().flatten() else {
            continue;
        };
        let verb = groups.get(0).map(|m| m.as_str()).unwrap_or("");
        let Some(from) = clean(groups.get(1).map(|m| m.as_str()).unwrap_or("")) else {
            continue;
        };
        let Some(to) = clean(groups.get(2).map(|m| m.as_str()).unwrap_or("")) else {
            continue;
        };
        let (from_label, ty, to_label) = kind(verb, &from);
        out.push((from, from_label, ty, to, to_label));
    }
}

fn label_for(name: &str) -> &'static str {
    let gene = name.len() >= 3
        && name.len() <= 8
        && name
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit());
    if gene {
        "Gene"
    } else {
        "Compound"
    }
}

fn clean(raw: &str) -> Option<String> {
    let name = raw
        .trim()
        .trim_matches(|c: char| !c.is_alphanumeric() && c != '-');
    if name.chars().count() < 3 || name.chars().count() > 60 {
        return None;
    }
    let lower = name.to_lowercase();
    if matches!(
        lower.as_str(),
        "the" | "this" | "that" | "with" | "from" | "which" | "what" | "approved" | "and"
    ) {
        return None;
    }
    Some(name.to_string())
}

fn explain(hits: &[Hit]) -> String {
    hits.iter()
        .take(2)
        .map(|h| {
            format!(
                "{} {} {}, as reported by {}.",
                h.from,
                h.ty.replace('_', " "),
                h.to,
                h.title
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn body(
    request: &str,
    outcome: &str,
    message: &str,
    explanation: &str,
    pages: &[Page],
    hits: &[Hit],
) -> Value {
    let citations: Vec<Value> = pages
        .iter()
        .filter(|p| p.url.starts_with("https://"))
        .map(|p| json!({"url": p.url, "title": p.title}))
        .collect();
    let mut nodes = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for page in pages.iter().filter(|p| p.url.starts_with("https://")) {
        let id = format!("source:{}", page.url);
        if seen.insert(id.clone()) {
            nodes.push(json!({"id": id, "label": "Source", "name": page.title}));
        }
    }
    let edges: Vec<Value> = hits
        .iter()
        .map(|h| {
            let retrieved_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            json!({
                "from": h.from,
                "from_label": h.from_label,
                "ty": h.ty,
                "to": h.to,
                "to_label": h.to_label,
                "url": h.url,
                "title": h.title,
                "retrieved_at": retrieved_at,
            })
        })
        .collect();
    json!({
        "request": request,
        "backend": "catalogue",
        "backend_label": "Catalogue rules",
        "model": "catalogue-rules-v1",
        "residency": "local",
        "outcome": outcome,
        "message": message,
        "explanation": {"text": explanation},
        "citations": citations,
        "graph": {"nodes": nodes, "edges": edges},
        "stages": [{
            "stage": "gate",
            "title": "Answer, clarify or reject",
            "question": {"type": "choice", "criteria": {"answer": "A typed path is stated by a retrieved page.", "clarify": "Pages were retrieved but state no typed path.", "reject": "The question asks for a dose, diagnosis, or prescription.", "no_data": "No page was retrieved."}},
            "answer": {"top": outcome, "confidence": 1.0}
        }]
    })
}

pub struct Live;

impl Retrieve for Live {
    fn search(&self, query: &str) -> Result<Vec<Page>, String> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(8))
            .user_agent("GraphJevResearch/0.1")
            .build()
            .map_err(|e| e.to_string())?;
        let mut pages = Vec::new();
        let mut failures = 0;
        match wikipedia(&client, query) {
            Ok(p) => pages.extend(p),
            Err(_) => failures += 1,
        }
        match europe_pmc(&client, query) {
            Ok(p) => pages.extend(p),
            Err(_) => failures += 1,
        }
        if pages.is_empty() && failures == 2 {
            return Err("Could not reach the web to retrieve sources.".into());
        }
        pages.truncate(4);
        Ok(pages)
    }
}

fn wikipedia(client: &reqwest::blocking::Client, query: &str) -> Result<Vec<Page>, String> {
    let search = format!(
        "https://en.wikipedia.org/w/api.php?action=query&list=search&srlimit=1&utf8=1&format=json&srsearch={}",
        encode(query)
    );
    let found = get_json(client, &search)?;
    let title = found["query"]["search"]
        .as_array()
        .and_then(|a| a.first())
        .and_then(|h| h["title"].as_str())
        .unwrap_or("")
        .to_string();
    if title.is_empty() {
        return Ok(Vec::new());
    }
    let summary = get_json(
        client,
        &format!(
            "https://en.wikipedia.org/api/rest_v1/page/summary/{}",
            encode(&title)
        ),
    )?;
    let url = summary["content_urls"]["desktop"]["page"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let text = summary["extract"].as_str().unwrap_or("").to_string();
    let name = summary["title"].as_str().unwrap_or(&title).to_string();
    if url.is_empty() {
        return Ok(Vec::new());
    }
    Ok(vec![Page {
        url,
        title: name,
        text,
    }])
}

fn europe_pmc(client: &reqwest::blocking::Client, query: &str) -> Result<Vec<Page>, String> {
    let url = format!(
        "https://www.ebi.ac.uk/europepmc/webservices/rest/search?format=json&pageSize=3&resultType=core&query={}",
        encode(query)
    );
    let found = get_json(client, &url)?;
    let mut pages = Vec::new();
    let Some(results) = found["resultList"]["result"].as_array() else {
        return Ok(pages);
    };
    for row in results {
        let title = row["title"].as_str().unwrap_or("").trim().to_string();
        if title.is_empty() {
            continue;
        }
        let url = citation_url(row);
        if url.is_empty() {
            continue;
        }
        let text = row["abstractText"].as_str().unwrap_or("").to_string();
        pages.push(Page { url, title, text });
    }
    Ok(pages)
}

fn citation_url(row: &Value) -> String {
    if let Some(doi) = row["doi"].as_str().filter(|s| !s.is_empty()) {
        if doi.starts_with("https://") {
            return doi.to_string();
        }
        if !doi.starts_with("http") {
            return format!("https://doi.org/{doi}");
        }
    }
    if let Some(pmid) = row["pmid"].as_str().filter(|s| !s.is_empty()) {
        return format!("https://europepmc.org/article/MED/{pmid}");
    }
    String::new()
}

fn get_json(client: &reqwest::blocking::Client, url: &str) -> Result<Value, String> {
    let response = client.get(url).send().map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }
    let bytes = response.bytes().map_err(|e| e.to_string())?;
    if bytes.len() > 1_000_000 {
        return Err("response too large".into());
    }
    serde_json::from_slice(&bytes).map_err(|e| e.to_string())
}

fn encode(text: &str) -> String {
    let mut out = String::new();
    for b in text.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    struct Fixture {
        pages: Vec<Page>,
        calls: Cell<u32>,
        fail: bool,
    }

    impl Retrieve for Fixture {
        fn search(&self, _query: &str) -> Result<Vec<Page>, String> {
            self.calls.set(self.calls.get() + 1);
            if self.fail {
                return Err("Could not reach the web to retrieve sources.".into());
            }
            Ok(self.pages.clone())
        }
    }

    #[test]
    fn fixture_page_is_the_citation() {
        let fixture = Fixture {
            calls: Cell::new(0),
            fail: false,
            pages: vec![Page {
                url: "https://example.org/papers/fixture".into(),
                title: "Fixture note".into(),
                text: "SK-129 targets alpha-synuclein oligomers in preclinical models.".into(),
            }],
        };
        let out = run(
            "Which published work targets alpha-synuclein clumping?",
            &fixture,
        )
        .unwrap();
        assert_eq!(out["outcome"], "answer");
        let text = out["explanation"]["text"].as_str().unwrap().to_lowercase();
        assert!(text.contains("sk-129"));
        assert!(text.contains("targets"));
        assert!(text.contains("alpha-synuclein"));
        assert!(!text.contains("mg"));
        let urls: Vec<&str> = out["citations"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|c| c["url"].as_str())
            .collect();
        assert_eq!(urls, vec!["https://example.org/papers/fixture"]);
        assert_eq!(fixture.calls.get(), 1);
    }

    #[test]
    fn dose_request_is_rejected_without_a_fetch() {
        let fixture = Fixture {
            calls: Cell::new(0),
            fail: false,
            pages: vec![],
        };
        let out = run("What dose of levodopa should I prescribe", &fixture).unwrap();
        assert_eq!(out["outcome"], "reject");
        assert!(out["citations"].as_array().unwrap().is_empty());
        assert_eq!(fixture.calls.get(), 0);
    }

    #[test]
    fn page_without_a_path_is_clarify() {
        let fixture = Fixture {
            calls: Cell::new(0),
            fail: false,
            pages: vec![Page {
                url: "https://example.org/notes/overview".into(),
                title: "Overview".into(),
                text: "A general note that names no relationship.".into(),
            }],
        };
        let out = run("What has been reported about motor symptoms?", &fixture).unwrap();
        assert_eq!(out["outcome"], "clarify");
        let urls: Vec<&str> = out["citations"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|c| c["url"].as_str())
            .collect();
        assert_eq!(urls, vec!["https://example.org/notes/overview"]);
    }

    #[test]
    fn a_failed_fetch_is_an_error() {
        let fixture = Fixture {
            calls: Cell::new(0),
            fail: true,
            pages: vec![],
        };
        let err = run(
            "What published work targets alpha-synuclein clumping?",
            &fixture,
        )
        .unwrap_err();
        assert!(err.0.contains("Could not reach the web"));
    }
}
