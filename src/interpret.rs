//! Optional reading of typed catalogue requests by Gemini on Vertex AI.
//!
//! Gemini only maps the reader's words to catalogue terms. The service writes those terms back
//! as a plain request and the catalogue rules decide on it, so Gemini never chooses an outcome,
//! a dataset or a number. Any failure, or any term Gemini finds outside the catalogue, leaves
//! the rules to decide on the reader's own words, and the response says so.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::Deserialize;
use serde_json::{json, Value};

use crate::catalogue::Catalogue;

pub(crate) const DEFAULT_MODEL: &str = "gemini-3.8-flash";
const DEFAULT_LOCATION: &str = "global";
const DEFAULT_TIMEOUT_MS: u64 = 8_000;
const DEFAULT_THINKING: &str = "low";
const METADATA_HOST: &str = "metadata.google.internal";
const MAX_UNSUPPORTED: usize = 5;
const MAX_UNSUPPORTED_CHARS: usize = 60;

/// Who reads typed requests before the rules decide: the rules alone, or Gemini first.
pub enum Interpreter {
    Rules,
    Gemini(Box<Gemini>),
}

pub struct Gemini {
    model: String,
    location: String,
    endpoint: String,
    project: Option<String>,
    thinking: Option<String>,
    access_token: Option<String>,
    metadata_host: String,
    http: reqwest::Client,
    metadata: reqwest::Client,
    cached_token: Mutex<Option<(String, Instant)>>,
    cached_project: Mutex<Option<String>>,
}

/// What the rules decided on, and who read the request.
pub struct Interpretation {
    pub by: &'static str,
    pub model: String,
    pub request: String,
    pub note: Option<String>,
    log: String,
}

impl Interpretation {
    pub fn to_json(&self) -> Value {
        json!({"by": self.by, "model": self.model, "request": self.request, "note": self.note})
    }

    /// A short label for the request log. It never carries request text.
    pub fn log_label(&self) -> &str {
        &self.log
    }
}

enum Failure {
    Credentials,
    Project,
    Timeout,
    Unreachable,
    Status(u16),
    Unreadable,
    OffCatalogue,
}

impl Failure {
    fn reason(&self) -> String {
        match self {
            Self::Credentials => "no Google Cloud access token".into(),
            Self::Project => "no Google Cloud project".into(),
            Self::Timeout => "Gemini did not respond in time".into(),
            Self::Unreachable => "Vertex AI could not be reached".into(),
            Self::Status(code) => format!("Vertex AI returned HTTP {code}"),
            Self::Unreadable => "Gemini returned an unreadable reply".into(),
            Self::OffCatalogue => "Gemini returned terms outside the catalogue".into(),
        }
    }
}

fn from_reqwest(error: reqwest::Error) -> Failure {
    if error.is_timeout() {
        Failure::Timeout
    } else {
        Failure::Unreachable
    }
}

impl Interpreter {
    /// Builds the interpreter from `lookup`; empty values count as unset.
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Result<Self, String> {
        let var = |key: &str| lookup(key).filter(|value| !value.trim().is_empty());
        match var("CATALOGUE_INTERPRETER").as_deref().unwrap_or("rules") {
            "rules" => return Ok(Self::Rules),
            "gemini" => {}
            _ => return Err("CATALOGUE_INTERPRETER must be \"rules\" or \"gemini\".".into()),
        }
        let model = var("GEMINI_MODEL").unwrap_or_else(|| DEFAULT_MODEL.into());
        let location = var("GEMINI_LOCATION").unwrap_or_else(|| DEFAULT_LOCATION.into());
        let name = |value: &str| {
            !value.is_empty()
                && value
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"-._".contains(&b))
        };
        if !name(&model) {
            return Err("GEMINI_MODEL must be a model id such as gemini-3.8-flash.".into());
        }
        if !name(&location) {
            return Err("GEMINI_LOCATION must be a location such as global.".into());
        }
        let project = var("GOOGLE_CLOUD_PROJECT");
        if project.as_deref().is_some_and(|p| !name(p)) {
            return Err("GOOGLE_CLOUD_PROJECT must be a project id.".into());
        }
        let timeout: u64 = var("GEMINI_TIMEOUT_MS")
            .map_or(Ok(DEFAULT_TIMEOUT_MS), |ms| ms.parse())
            .ok()
            .filter(|ms| (1..=60_000).contains(ms))
            .ok_or("GEMINI_TIMEOUT_MS must be an integer from 1 to 60000.")?;
        let thinking =
            match var("GEMINI_THINKING_LEVEL")
                .as_deref()
                .unwrap_or(DEFAULT_THINKING)
            {
                "model-default" => None,
                level @ ("minimal" | "low" | "medium" | "high") => Some(level.to_uppercase()),
                _ => return Err(
                    "GEMINI_THINKING_LEVEL must be minimal, low, medium, high or model-default."
                        .into(),
                ),
            };
        let endpoint = match var("GEMINI_ENDPOINT") {
            Some(endpoint) => {
                let origin = endpoint.trim_end_matches('/');
                let rest = origin
                    .strip_prefix("https://")
                    .or_else(|| origin.strip_prefix("http://"));
                if rest.is_none_or(|r| r.is_empty() || r.contains(['/', '?', '#', ' '])) {
                    return Err("GEMINI_ENDPOINT must be an http(s) origin with no path.".into());
                }
                origin.to_string()
            }
            None if location == "global" => "https://aiplatform.googleapis.com".into(),
            None => format!("https://{location}-aiplatform.googleapis.com"),
        };
        let metadata_host = var("GCE_METADATA_HOST").unwrap_or_else(|| METADATA_HOST.into());
        if metadata_host.contains(['/', '?', '#', ' ']) {
            return Err("GCE_METADATA_HOST must be host[:port].".into());
        }
        let timeout = Duration::from_millis(timeout);
        let http = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| format!("Gemini client: {e}"))?;
        // The metadata server is link-local, so it never goes through a proxy.
        let metadata = reqwest::Client::builder()
            .timeout(timeout.min(Duration::from_secs(2)))
            .no_proxy()
            .build()
            .map_err(|e| format!("metadata client: {e}"))?;
        Ok(Self::Gemini(Box::new(Gemini {
            model,
            location,
            endpoint,
            project,
            thinking,
            access_token: var("GEMINI_ACCESS_TOKEN"),
            metadata_host,
            http,
            metadata,
            cached_token: Mutex::new(None),
            cached_project: Mutex::new(None),
        })))
    }

    /// Reads `request` with Gemini. Returns `None` when the rules read requests alone.
    pub async fn interpret(&self, cat: &Catalogue, request: &str) -> Option<Interpretation> {
        let Self::Gemini(gemini) = self else {
            return None;
        };
        let reading = match gemini.read(cat, request).await {
            Ok(reading) => reading,
            Err(failure) => {
                return Some(Interpretation {
                    by: "rules",
                    model: gemini.model.clone(),
                    request: request.to_string(),
                    note: Some(format!(
                        "Gemini unavailable: {}. The rules read your words.",
                        failure.reason()
                    )),
                    log: format!("rules_fallback: {}", failure.reason()),
                })
            }
        };
        if !reading.in_scope || !reading.unsupported.is_empty() {
            let outside = if reading.unsupported.is_empty() {
                "the request is not about emissions data".to_string()
            } else {
                reading.unsupported.join(", ")
            };
            return Some(Interpretation {
                by: "gemini",
                model: gemini.model.clone(),
                request: request.to_string(),
                note: Some(format!(
                    "Outside the catalogue: {outside}. The rules decided on your words."
                )),
                log: "gemini_outside_catalogue".into(),
            });
        }
        let composed = compose(cat, &reading);
        if composed.is_empty() {
            return Some(Interpretation {
                by: "gemini",
                model: gemini.model.clone(),
                request: request.to_string(),
                note: Some(
                    "Gemini found no catalogue terms. The rules decided on your words.".into(),
                ),
                log: "gemini_no_terms".into(),
            });
        }
        Some(Interpretation {
            by: "gemini",
            model: gemini.model.clone(),
            request: composed,
            note: None,
            log: "gemini".into(),
        })
    }
}

/// Catalogue terms Gemini found in a request. Empty means the reader did not say.
#[derive(Debug, Default, Deserialize, PartialEq)]
pub(crate) struct Reading {
    pub in_scope: bool,
    #[serde(default)]
    pub pollutants: Vec<String>,
    #[serde(default)]
    pub groups: Vec<String>,
    #[serde(default)]
    pub geo_level: Option<String>,
    #[serde(default)]
    pub time_level: Option<String>,
    #[serde(default)]
    pub sector_level: Option<String>,
    #[serde(default)]
    pub places: Vec<String>,
    #[serde(default)]
    pub sectors: Vec<String>,
    #[serde(default)]
    pub year_from: Option<i32>,
    #[serde(default)]
    pub year_to: Option<i32>,
    #[serde(default)]
    pub unsupported: Vec<String>,
}

fn members(cat: &Catalogue, dim: &str) -> Vec<String> {
    cat.dim_levels(dim)
        .iter()
        .flat_map(|level| cat.members_of(dim, level).iter().cloned())
        .collect()
}

fn group_ids(cat: &Catalogue) -> Vec<String> {
    let mut ids: Vec<String> = cat.groups.keys().cloned().collect();
    ids.sort();
    ids
}

fn with_none(levels: &[String]) -> Vec<String> {
    std::iter::once("none".to_string())
        .chain(levels.iter().cloned())
        .collect()
}

impl Reading {
    /// Checks every term against the catalogue, since a model reply is a claim, not a fact.
    fn check(mut self, cat: &Catalogue) -> Result<Self, Failure> {
        let known = |value: &String, list: &[String]| list.contains(value);
        let pollutants: Vec<String> = cat.pollutants.iter().map(|p| p.id.clone()).collect();
        let level = |value: &mut Option<String>, dim: &str| {
            if value.as_deref() == Some("none") {
                *value = None;
            }
            value
                .as_ref()
                .is_none_or(|v| cat.dim_levels(dim).contains(v))
        };
        let ok = self.pollutants.iter().all(|p| known(p, &pollutants))
            && self.groups.iter().all(|g| known(g, &group_ids(cat)))
            && self.places.iter().all(|p| known(p, &members(cat, "GEO")))
            && self
                .sectors
                .iter()
                .all(|s| known(s, &members(cat, "SECTOR")))
            && level(&mut self.geo_level, "GEO")
            && level(&mut self.time_level, "TIME")
            && level(&mut self.sector_level, "SECTOR")
            && [self.year_from, self.year_to]
                .iter()
                .flatten()
                .all(|y| (1000..=9999).contains(y));
        if !ok {
            return Err(Failure::OffCatalogue);
        }
        self.unsupported = self
            .unsupported
            .iter()
            .map(|term| term.split_whitespace().collect::<Vec<_>>().join(" "))
            .filter(|term| !term.is_empty())
            .map(|term| term.chars().take(MAX_UNSUPPORTED_CHARS).collect())
            .take(MAX_UNSUPPORTED)
            .collect();
        Ok(self)
    }
}

fn join(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [one] => one.clone(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

/// The name the rules parser reads back as exactly this pollutant. Two labels mention "year",
/// which the parser would take as a time breakdown, so those use the catalogue id.
fn pollutant_term(cat: &Catalogue, id: &str) -> String {
    let p = cat.pollutant(id);
    if p.label.to_lowercase().contains("year") {
        p.id.clone()
    } else {
        p.label.clone()
    }
}

/// Writes a reading back as a plain request in catalogue terms, for the rules to decide on.
pub(crate) fn compose(cat: &Catalogue, reading: &Reading) -> String {
    let group_terms = cat.groups.keys().filter(|g| reading.groups.contains(g));
    let mut what: Vec<String> = group_terms
        .map(|g| match g.as_str() {
            "all" => "all pollutants".to_string(),
            _ => cat.groups[g].label.to_lowercase(),
        })
        .collect();
    what.sort();
    what.extend(
        cat.pollutants
            .iter()
            .filter(|p| reading.pollutants.contains(&p.id))
            .map(|p| pollutant_term(cat, &p.id)),
    );
    let mut parts = Vec::new();
    if !what.is_empty() {
        parts.push(join(&what));
    }
    let levels: Vec<String> = [
        &reading.geo_level,
        &reading.time_level,
        &reading.sector_level,
    ]
    .into_iter()
    .flatten()
    .cloned()
    .collect();
    if !levels.is_empty() {
        parts.push(format!("by {}", join(&levels)));
    }
    let places: Vec<String> = members(cat, "GEO")
        .into_iter()
        .filter(|m| reading.places.contains(m))
        .collect();
    if !places.is_empty() {
        parts.push(format!("in {}", join(&places)));
    }
    let sectors: Vec<String> = members(cat, "SECTOR")
        .into_iter()
        .filter(|m| reading.sectors.contains(m))
        .collect();
    if !sectors.is_empty() {
        parts.push(format!("for {}", join(&sectors)));
    }
    // An open range such as "since 2019" runs to the catalogue's last year, and "until 2020"
    // from its first.
    let first = cat.years.iter().min().copied().unwrap_or(2015);
    let last = cat.years.iter().max().copied().unwrap_or(2025);
    let span = match (reading.year_from, reading.year_to) {
        (Some(a), Some(b)) => Some((a, b)),
        (Some(a), None) => Some((a, last)),
        (None, Some(b)) => Some((first, b)),
        (None, None) => None,
    };
    match span {
        Some((a, b)) if a == b => parts.push(format!("in {a}")),
        Some((a, b)) => parts.push(format!("from {} to {}", a.min(b), a.max(b))),
        None => {}
    }
    parts.join(" ")
}

/// The fixed instruction: the catalogue's terms and what to do with them. It holds no request text.
fn instruction(cat: &Catalogue) -> String {
    let pollutants: Vec<String> = cat
        .pollutants
        .iter()
        .map(|p| format!("{} ({}; {})", p.id, p.label, p.name))
        .collect();
    let groups: Vec<String> = group_ids(cat)
        .iter()
        .map(|g| format!("{g} ({})", cat.groups[g].label))
        .collect();
    let years = format!(
        "{} to {}",
        cat.years.iter().min().unwrap_or(&2015),
        cat.years.iter().max().unwrap_or(&2025)
    );
    format!(
        "You read a typed request for emissions data and map it to the terms of one catalogue. \
         You never answer the request, choose datasets or give figures.\n\
         Pollutants: {pollutants}.\n\
         Pollutant groups: {groups}. Use a group only when the request names the group.\n\
         Geography levels: {geo}. Time levels: {time}. Sector levels: {sector}.\n\
         Places: {places}.\n\
         Sectors: {sectors}.\n\
         The catalogue holds the years {years}. Count relative years such as \"the last five \
         years\" back from the latest year held.\n\
         Rules:\n\
         - Use only the listed values. Spell places and sectors exactly as listed.\n\
         - Set a level only when the request asks for that breakdown, or names places, sectors \
         or years at that level. Otherwise use \"none\". Never guess a breakdown.\n\
         - Put years in year_from and year_to. One year goes in both. For an open range such \
         as \"since 2019\", leave the other end null.\n\
         - List in unsupported the words for anything the catalogue does not hold, such as \
         another pollutant, a city, a station, daily or hourly data, a forecast or another \
         topic. Keep each entry to a few words from the request.\n\
         - Set in_scope to false when the request is not about emissions data at all.",
        pollutants = pollutants.join(", "),
        groups = groups.join(", "),
        geo = cat.dim_levels("GEO").join(", "),
        time = cat.dim_levels("TIME").join(", "),
        sector = cat.dim_levels("SECTOR").join(", "),
        places = members(cat, "GEO").join(", "),
        sectors = members(cat, "SECTOR").join(", "),
    )
}

/// The reply shape, with every catalogue term as an enum so Gemini cannot invent one.
fn schema(cat: &Catalogue) -> Value {
    let list =
        |values: Vec<String>| json!({"type": "ARRAY", "items": {"type": "STRING", "enum": values}});
    let level = |dim: &str| json!({"type": "STRING", "enum": with_none(cat.dim_levels(dim))});
    let year = json!({"type": "INTEGER", "nullable": true});
    let order = [
        "in_scope",
        "pollutants",
        "groups",
        "geo_level",
        "time_level",
        "sector_level",
        "places",
        "sectors",
        "year_from",
        "year_to",
        "unsupported",
    ];
    json!({
        "type": "OBJECT",
        "properties": {
            "in_scope": {"type": "BOOLEAN"},
            "pollutants": list(cat.pollutants.iter().map(|p| p.id.clone()).collect()),
            "groups": list(group_ids(cat)),
            "geo_level": level("GEO"),
            "time_level": level("TIME"),
            "sector_level": level("SECTOR"),
            "places": list(members(cat, "GEO")),
            "sectors": list(members(cat, "SECTOR")),
            "year_from": year,
            "year_to": year,
            "unsupported": {"type": "ARRAY", "items": {"type": "STRING"}},
        },
        "required": order,
        "propertyOrdering": order,
    })
}

#[derive(Deserialize)]
struct Token {
    access_token: String,
    expires_in: u64,
}

impl Gemini {
    async fn read(&self, cat: &Catalogue, request: &str) -> Result<Reading, Failure> {
        let token = self.token().await?;
        let project = self.project().await?;
        let url = format!(
            "{}/v1/projects/{project}/locations/{}/publishers/google/models/{}:generateContent",
            self.endpoint, self.location, self.model
        );
        let mut generation = json!({
            "responseMimeType": "application/json",
            "responseSchema": schema(cat),
            "maxOutputTokens": 8192,
        });
        if let Some(level) = &self.thinking {
            generation["thinkingConfig"] = json!({"thinkingLevel": level});
        }
        let body = json!({
            "systemInstruction": {"parts": [{"text": instruction(cat)}]},
            "contents": [{"role": "user", "parts": [{"text": request}]}],
            "generationConfig": generation,
        });
        let response = self
            .http
            .post(url)
            .bearer_auth(token)
            .json(&body)
            .send()
            .await
            .map_err(from_reqwest)?;
        if !response.status().is_success() {
            return Err(Failure::Status(response.status().as_u16()));
        }
        let reply: Value = response.json().await.map_err(|e| {
            if e.is_timeout() {
                Failure::Timeout
            } else {
                Failure::Unreadable
            }
        })?;
        reading_from(&reply)?.check(cat)
    }

    async fn token(&self) -> Result<String, Failure> {
        if let Some(token) = &self.access_token {
            return Ok(token.clone());
        }
        if let Some((token, expires)) = self.cached_token.lock().expect("token cache").as_ref() {
            if Instant::now() < *expires {
                return Ok(token.clone());
            }
        }
        let url = format!(
            "http://{}/computeMetadata/v1/instance/service-accounts/default/token",
            self.metadata_host
        );
        let response = self
            .metadata
            .get(url)
            .header("Metadata-Flavor", "Google")
            .send()
            .await
            .map_err(|_| Failure::Credentials)?;
        if !response.status().is_success() {
            return Err(Failure::Credentials);
        }
        let token: Token = response.json().await.map_err(|_| Failure::Credentials)?;
        // Refresh a minute early so a token never expires mid-request.
        let expires = Instant::now() + Duration::from_secs(token.expires_in.saturating_sub(60));
        *self.cached_token.lock().expect("token cache") =
            Some((token.access_token.clone(), expires));
        Ok(token.access_token)
    }

    async fn project(&self) -> Result<String, Failure> {
        if let Some(project) = &self.project {
            return Ok(project.clone());
        }
        if let Some(project) = self.cached_project.lock().expect("project cache").as_ref() {
            return Ok(project.clone());
        }
        let url = format!(
            "http://{}/computeMetadata/v1/project/project-id",
            self.metadata_host
        );
        let response = self
            .metadata
            .get(url)
            .header("Metadata-Flavor", "Google")
            .send()
            .await
            .map_err(|_| Failure::Project)?;
        let project = response
            .error_for_status()
            .map_err(|_| Failure::Project)?
            .text()
            .await
            .map_err(|_| Failure::Project)?
            .trim()
            .to_string();
        let valid = !project.is_empty()
            && project
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"-.:".contains(&b));
        if !valid {
            return Err(Failure::Project);
        }
        *self.cached_project.lock().expect("project cache") = Some(project.clone());
        Ok(project)
    }
}

/// Takes the JSON text from the first candidate, skipping any thought parts.
fn reading_from(reply: &Value) -> Result<Reading, Failure> {
    let text: String = reply["candidates"][0]["content"]["parts"]
        .as_array()
        .ok_or(Failure::Unreadable)?
        .iter()
        .filter(|part| part["thought"] != json!(true))
        .filter_map(|part| part["text"].as_str())
        .collect();
    serde_json::from_str(&text).map_err(|_| Failure::Unreadable)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::request::{named_pollutants, resolve};
    use std::path::PathBuf;

    fn cat() -> Catalogue {
        Catalogue::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("catalogue.json")).unwrap()
    }

    fn reading(value: Value) -> Reading {
        serde_json::from_value(value).unwrap()
    }

    #[test]
    fn every_pollutant_term_reads_back_as_that_pollutant_alone() {
        let cat = cat();
        for p in &cat.pollutants {
            let term = pollutant_term(&cat, &p.id);
            assert_eq!(named_pollutants(&cat, &term), vec![p.id.clone()], "{term}");
            let parsed = resolve(&cat, &format!("{term} by country"));
            assert_eq!(parsed["levels"], json!({"GEO": "country"}), "{term}");
        }
    }

    #[test]
    fn a_full_reading_composes_to_a_request_the_rules_answer() {
        let cat = cat();
        let r = reading(json!({
            "in_scope": true, "pollutants": ["PM10", "PM2_5"], "groups": [],
            "geo_level": "region", "time_level": "year", "sector_level": null,
            "places": ["Bavaria", "Lombardy"], "sectors": [],
            "year_from": 2024, "year_to": 2019, "unsupported": []
        }));
        let text = compose(&cat, &r);
        assert_eq!(
            text,
            "PM2.5 and PM10 by region and year in Lombardy and Bavaria from 2019 to 2024"
        );
        let parsed = resolve(&cat, &text);
        assert_eq!(parsed["gate"], "answer", "{parsed}");
        assert_eq!(parsed["pollutants"], json!(["PM2_5", "PM10"]));
        assert_eq!(
            parsed["filters"]["TIME"]["members"],
            json!(["2019", "2020", "2021", "2022", "2023", "2024"])
        );
    }

    #[test]
    fn groups_sectors_and_single_years_compose_in_catalogue_terms() {
        let cat = cat();
        let r = reading(json!({
            "in_scope": true, "pollutants": ["CO2e100"], "groups": ["metal", "ghg"],
            "geo_level": "country", "time_level": null, "sector_level": "subsector",
            "places": [], "sectors": ["Aviation", "Road transport"],
            "year_from": 2020, "year_to": 2020, "unsupported": []
        }));
        let text = compose(&cat, &r);
        assert_eq!(
            text,
            "greenhouse gases, heavy metals and CO2e100 by country and subsector for Road transport and Aviation in 2020"
        );
        let parsed = resolve(&cat, &text);
        assert_eq!(parsed["issues"], json!([]), "{parsed}");
        assert_eq!(parsed["groups"], json!(["ghg", "metal"]));
        assert_eq!(parsed["filters"]["TIME"]["members"], json!(["2020"]));
    }

    #[test]
    fn open_year_ranges_run_to_the_ends_of_the_catalogue() {
        let cat = cat();
        let since = reading(json!({"in_scope": true, "pollutants": ["CH4"], "year_from": 2022}));
        assert_eq!(compose(&cat, &since), "CH4 from 2022 to 2025");
        let until = reading(json!({"in_scope": true, "pollutants": ["CH4"], "year_to": 2016}));
        assert_eq!(compose(&cat, &until), "CH4 from 2015 to 2016");
        let late = reading(json!({"in_scope": true, "pollutants": ["CH4"], "year_from": 2030}));
        assert_eq!(compose(&cat, &late), "CH4 from 2025 to 2030");
        assert_eq!(resolve(&cat, &compose(&cat, &late))["gate"], "reject");
    }

    #[test]
    fn a_reading_with_no_breakdown_leaves_the_rules_to_ask() {
        let cat = cat();
        let r = reading(
            json!({"in_scope": true, "pollutants": ["CH4"], "groups": [],
            "geo_level": "none", "time_level": "none", "sector_level": "none",
            "places": [], "sectors": [], "year_from": null, "year_to": null, "unsupported": []}),
        )
        .check(&cat)
        .ok()
        .unwrap();
        assert_eq!(compose(&cat, &r), "CH4");
        assert_eq!(resolve(&cat, "CH4")["gate"], "clarify");
    }

    #[test]
    fn terms_outside_the_catalogue_are_refused() {
        let cat = cat();
        for bad in [
            json!({"in_scope": true, "pollutants": ["O3"]}),
            json!({"in_scope": true, "places": ["Paris"]}),
            json!({"in_scope": true, "geo_level": "city"}),
            json!({"in_scope": true, "groups": ["voc"]}),
            json!({"in_scope": true, "year_from": 99999}),
        ] {
            assert!(reading(bad.clone()).check(&cat).is_err(), "{bad}");
        }
    }

    #[test]
    fn unsupported_terms_are_trimmed_and_capped() {
        let cat = cat();
        let long = "x".repeat(200);
        let r = reading(json!({"in_scope": true,
            "unsupported": ["  hourly  data ", "", long, "a", "b", "c", "d", "e"]}))
        .check(&cat)
        .ok()
        .unwrap();
        assert_eq!(r.unsupported.len(), MAX_UNSUPPORTED);
        assert_eq!(r.unsupported[0], "hourly data");
        assert_eq!(r.unsupported[1].chars().count(), MAX_UNSUPPORTED_CHARS);
    }

    #[test]
    fn the_reply_text_is_taken_from_non_thought_parts() {
        let reply = json!({"candidates": [{"content": {"parts": [
            {"text": "thinking about it", "thought": true},
            {"text": "{\"in_scope\": true, \"pollutants\": [\"CO2\"]}"}
        ]}}]});
        let r = reading_from(&reply).ok().unwrap();
        assert_eq!(r.pollutants, vec!["CO2"]);
        assert!(reading_from(&json!({"candidates": []})).is_err());
        assert!(
            reading_from(&json!({"candidates": [{"content": {"parts": [{"text": "no"}]}}]}))
                .is_err()
        );
    }

    #[test]
    fn the_schema_and_instruction_list_only_catalogue_terms() {
        let cat = cat();
        let s = schema(&cat);
        assert_eq!(
            s["properties"]["pollutants"]["items"]["enum"]
                .as_array()
                .unwrap()
                .len(),
            24
        );
        assert_eq!(
            s["properties"]["geo_level"]["enum"],
            json!(["none", "region", "country", "continent"])
        );
        assert_eq!(
            s["properties"]["groups"]["items"]["enum"],
            json!(["all", "ghg", "metal", "pm"])
        );
        let text = instruction(&cat);
        assert!(text.contains("2015 to 2025"));
        assert!(text.contains("Bavaria") && text.contains("Road transport"));
    }

    #[test]
    fn settings_default_to_rules_and_are_validated() {
        let none = |_: &str| None;
        assert!(matches!(
            Interpreter::from_lookup(none),
            Ok(Interpreter::Rules)
        ));
        let with = |pairs: &'static [(&'static str, &'static str)]| {
            move |key: &str| {
                pairs
                    .iter()
                    .find(|(k, _)| *k == key)
                    .map(|(_, v)| v.to_string())
            }
        };
        let Ok(Interpreter::Gemini(g)) =
            Interpreter::from_lookup(with(&[("CATALOGUE_INTERPRETER", "gemini")]))
        else {
            panic!("gemini");
        };
        assert_eq!(g.model, "gemini-3.8-flash");
        assert_eq!(g.endpoint, "https://aiplatform.googleapis.com");
        assert_eq!(g.thinking.as_deref(), Some("LOW"));
        let Ok(Interpreter::Gemini(g)) = Interpreter::from_lookup(with(&[
            ("CATALOGUE_INTERPRETER", "gemini"),
            ("GEMINI_LOCATION", "europe-west4"),
            ("GEMINI_THINKING_LEVEL", "model-default"),
        ])) else {
            panic!("regional");
        };
        assert_eq!(g.endpoint, "https://europe-west4-aiplatform.googleapis.com");
        assert_eq!(g.thinking, None);
        for bad in [
            &[("CATALOGUE_INTERPRETER", "llm")][..],
            &[
                ("CATALOGUE_INTERPRETER", "gemini"),
                ("GEMINI_MODEL", "Gemini 3.8"),
            ][..],
            &[
                ("CATALOGUE_INTERPRETER", "gemini"),
                ("GEMINI_TIMEOUT_MS", "0"),
            ][..],
            &[
                ("CATALOGUE_INTERPRETER", "gemini"),
                ("GEMINI_THINKING_LEVEL", "max"),
            ][..],
            &[
                ("CATALOGUE_INTERPRETER", "gemini"),
                ("GEMINI_ENDPOINT", "ftp://x"),
            ][..],
            &[
                ("CATALOGUE_INTERPRETER", "gemini"),
                ("GEMINI_ENDPOINT", "https://x/v1"),
            ][..],
            &[
                ("CATALOGUE_INTERPRETER", "gemini"),
                ("GOOGLE_CLOUD_PROJECT", "a/b"),
            ][..],
        ] {
            let pairs: Vec<(String, String)> = bad
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();
            let lookup = |key: &str| pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone());
            assert!(Interpreter::from_lookup(lookup).is_err(), "{bad:?}");
        }
    }
}
