//! Parkinson's research path: reject clinical asks, otherwise fetch a few
//! public pages and read them as a labelled graph. Citations are only URLs
//! this request fetched.

use std::io::Read;
use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use fancy_regex::Regex;
use reqwest::Url;
use serde_json::{json, Value};

use crate::request::re;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);
const TOTAL_TIMEOUT: Duration = Duration::from_secs(8);
const MAX_RESPONSE_BYTES: usize = 1_000_000;
const MAX_PAGES: usize = 4;
const MAX_RESULTS: usize = 3;
const USER_AGENT: &str =
    "GraphsAndJevResearch/0.1 (+https://github.com/TLatAccenture/graphs-and-jev)";

#[derive(Clone)]
pub struct Page {
    pub url: String,
    pub title: String,
    pub text: String,
}

pub trait Retrieve {
    fn search(&self, query: &str) -> Result<Vec<Page>, ResearchError>;
}

#[derive(Debug)]
pub struct ResearchError {
    class: &'static str,
    detail: String,
}

impl ResearchError {
    pub fn fetch(detail: impl Into<String>) -> Self {
        Self {
            class: "fetch",
            detail: detail.into(),
        }
    }

    pub fn invalid(detail: impl Into<String>) -> Self {
        Self {
            class: "invalid",
            detail: detail.into(),
        }
    }

    pub fn timeout(_detail: impl Into<String>) -> Self {
        Self {
            class: "timeout",
            detail: "Source retrieval timed out.".into(),
        }
    }

    fn from_reqwest(error: reqwest::Error) -> Self {
        if error.is_timeout() {
            Self::timeout(error.to_string())
        } else {
            Self::fetch("Source retrieval failed.")
        }
    }

    pub fn class(&self) -> &'static str {
        self.class
    }

    pub fn detail(&self) -> &str {
        &self.detail
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
    let pages = bounded_pages(retrieve.search(request)?);
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

fn live_client() -> Result<&'static reqwest::blocking::Client, ResearchError> {
    static CLIENT: OnceLock<Result<reqwest::blocking::Client, String>> = OnceLock::new();
    CLIENT
        .get_or_init(|| {
            reqwest::blocking::Client::builder()
                .connect_timeout(CONNECT_TIMEOUT)
                .timeout(TOTAL_TIMEOUT)
                .redirect(reqwest::redirect::Policy::custom(|attempt| {
                    if attempt.previous().len() >= 5 {
                        attempt.error("too many redirects")
                    } else if attempt.url().scheme() == "https" {
                        attempt.follow()
                    } else {
                        attempt.stop()
                    }
                }))
                .https_only(true)
                .user_agent(USER_AGENT)
                .build()
                .map_err(|error| error.to_string())
        })
        .as_ref()
        .map_err(|_| ResearchError::fetch("Could not configure source retrieval."))
}

impl Retrieve for Live {
    fn search(&self, query: &str) -> Result<Vec<Page>, ResearchError> {
        let client = live_client()?;
        let mut pages = Vec::new();
        let mut failures = Vec::new();
        match wikipedia(client, query) {
            Ok(found) => pages.extend(found),
            Err(error) => failures.push(error),
        }
        match europe_pmc(client, query) {
            Ok(found) => pages.extend(found),
            Err(error) => failures.push(error),
        }
        if pages.is_empty() && failures.len() == 2 {
            return Err(combine_failures(failures));
        }
        Ok(bounded_pages(pages))
    }
}

fn valid_https(url: &str) -> bool {
    Url::parse(url).is_ok_and(|url| url.scheme() == "https" && url.host_str().is_some())
}

fn bounded_pages(pages: Vec<Page>) -> Vec<Page> {
    pages
        .into_iter()
        .filter(|page| valid_https(&page.url) && !page.title.trim().is_empty())
        .take(MAX_PAGES)
        .collect()
}

fn combine_failures(failures: Vec<ResearchError>) -> ResearchError {
    let class = if failures.iter().any(|error| error.class() == "timeout") {
        "timeout"
    } else if failures.iter().any(|error| error.class() == "invalid") {
        "invalid"
    } else {
        "fetch"
    };
    ResearchError {
        class,
        detail: "Could not reach the web to retrieve sources.".into(),
    }
}

fn wikipedia(client: &reqwest::blocking::Client, query: &str) -> Result<Vec<Page>, ResearchError> {
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

fn europe_pmc(client: &reqwest::blocking::Client, query: &str) -> Result<Vec<Page>, ResearchError> {
    let url = format!(
        "https://www.ebi.ac.uk/europepmc/webservices/rest/search?format=json&pageSize=3&resultType=core&query={}",
        encode(query)
    );
    let found = get_json(client, &url)?;
    let Some(results) = found["resultList"]["result"].as_array() else {
        return Ok(Vec::new());
    };
    Ok(europe_pmc_pages(results))
}

fn europe_pmc_pages(results: &[Value]) -> Vec<Page> {
    results
        .iter()
        .filter_map(|row| {
            let title = row["title"].as_str()?.trim();
            let url = citation_url(row);
            (valid_https(&url) && !title.is_empty()).then(|| Page {
                url,
                title: title.to_string(),
                text: row["abstractText"].as_str().unwrap_or("").to_string(),
            })
        })
        .take(MAX_RESULTS)
        .collect()
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

fn get_json(client: &reqwest::blocking::Client, url: &str) -> Result<Value, ResearchError> {
    let response = client
        .get(url)
        .send()
        .map_err(ResearchError::from_reqwest)?;
    if !response.status().is_success() {
        return Err(ResearchError::fetch(format!("HTTP {}", response.status())));
    }
    let content_length = response.content_length();
    let bytes = read_bounded(response, content_length)?;
    parse_json(&bytes)
}

fn read_bounded(reader: impl Read, content_length: Option<u64>) -> Result<Vec<u8>, ResearchError> {
    if content_length.is_some_and(|length| length > MAX_RESPONSE_BYTES as u64) {
        return Err(ResearchError::invalid("response too large"));
    }
    let mut bytes = Vec::new();
    reader
        .take(MAX_RESPONSE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| ResearchError::fetch(error.to_string()))?;
    if bytes.len() > MAX_RESPONSE_BYTES {
        return Err(ResearchError::invalid("response too large"));
    }
    Ok(bytes)
}

fn parse_json(bytes: &[u8]) -> Result<Value, ResearchError> {
    serde_json::from_slice(bytes).map_err(|error| ResearchError::invalid(error.to_string()))
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
    use std::io::Cursor;

    struct Fixture {
        pages: Vec<Page>,
        calls: Cell<u32>,
        fail: bool,
    }

    impl Retrieve for Fixture {
        fn search(&self, _query: &str) -> Result<Vec<Page>, ResearchError> {
            self.calls.set(self.calls.get() + 1);
            if self.fail {
                return Err(ResearchError::fetch(
                    "Could not reach the web to retrieve sources.",
                ));
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
    fn clinical_requests_are_rejected_without_a_fetch() {
        for request in [
            "What dose of levodopa is appropriate?",
            "Can you diagnose these tremors?",
            "Please write a prescription for levodopa",
        ] {
            let fixture = Fixture {
                calls: Cell::new(0),
                fail: false,
                pages: vec![],
            };
            let out = run(request, &fixture).unwrap();
            assert_eq!(out["outcome"], "reject", "{request}");
            assert!(out["citations"].as_array().unwrap().is_empty());
            assert_eq!(fixture.calls.get(), 0, "{request}");
        }
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
    fn source_failures_preserve_the_most_specific_class() {
        let error = combine_failures(vec![
            ResearchError::fetch("connection refused"),
            ResearchError::timeout("request timed out"),
        ]);
        assert_eq!(error.class(), "timeout");

        let error = combine_failures(vec![
            ResearchError::fetch("http 502"),
            ResearchError::invalid("invalid json"),
        ]);
        assert_eq!(error.class(), "invalid");
    }

    fn withholding_server() -> String {
        let listener = std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).unwrap();
        let address = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            let (_connection, _) = listener.accept().unwrap();
            std::thread::sleep(std::time::Duration::from_millis(100));
        });
        format!("http://{address}")
    }

    #[test]
    fn transport_error_classes_are_typed() {
        let timeout = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_millis(1))
            .build()
            .unwrap()
            .get(withholding_server())
            .send()
            .unwrap_err();
        assert_eq!(ResearchError::from_reqwest(timeout).class(), "timeout");

        let invalid = parse_json(b"not-json").unwrap_err();
        assert_eq!(invalid.class(), "invalid");
    }

    #[test]
    fn connect_read_and_total_timeouts_map_to_explicit_unavailable_failure() {
        for error in [
            ResearchError::timeout("connect timeout"),
            ResearchError::from_reqwest(
                reqwest::blocking::Client::builder()
                    .timeout(Duration::from_millis(1))
                    .build()
                    .unwrap()
                    .get(withholding_server())
                    .send()
                    .unwrap_err(),
            ),
            ResearchError::timeout("total timeout"),
        ] {
            assert_eq!(error.class(), "timeout");
            assert_eq!(error.detail(), "Source retrieval timed out.");
        }
    }

    #[test]
    fn non_https_pages_are_rejected_before_citation_metadata_is_preserved() {
        let fixture = Fixture {
            calls: Cell::new(0),
            fail: false,
            pages: vec![Page {
                url: "http://example.org/not-secure".into(),
                title: "Untrusted title".into(),
                text: "LRRK2 associated with Parkinson disease".into(),
            }],
        };
        let out = run("What is associated with Parkinson disease?", &fixture).unwrap();
        assert_eq!(out["outcome"], "no_data");
        assert!(out["citations"].as_array().unwrap().is_empty());
        assert!(out["graph"]["nodes"].as_array().unwrap().is_empty());
    }

    #[test]
    fn retrieved_pages_and_source_results_are_bounded() {
        let pages: Vec<Page> = (0..MAX_PAGES + 3)
            .map(|n| Page {
                url: format!("https://example.org/{n}"),
                title: format!("Page {n}"),
                text: "A general note that names no relationship.".into(),
            })
            .collect();
        assert_eq!(bounded_pages(pages).len(), MAX_PAGES);

        let rows: Vec<Value> = (0..MAX_RESULTS + 3)
            .map(|n| json!({"title": format!("Result {n}"), "pmid": n.to_string()}))
            .collect();
        assert_eq!(europe_pmc_pages(&rows).len(), MAX_RESULTS);
    }

    #[test]
    fn response_reader_stops_at_the_byte_limit() {
        let bytes = vec![b' '; MAX_RESPONSE_BYTES + 1];
        let error = read_bounded(Cursor::new(bytes), None).unwrap_err();
        assert_eq!(error.class(), "invalid");
        assert_eq!(error.detail(), "response too large");
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
        assert!(err.detail().contains("Could not reach the web"));
    }
}
