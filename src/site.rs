use axum::http::HeaderValue;
use serde_json::json;

const DISCLAIMER: &str = "A personal research thought piece on agentic migration and modernisation. It is not an official Accenture or Google Cloud product, a service offer or a statement of capability. Product names and marks belong to their owners.";

/// Browser config and CSP, rendered once at start-up from environment variables.
pub struct Site {
    config_js: String,
    csp: HeaderValue,
}

impl Default for Site {
    fn default() -> Self {
        Self::from_lookup(|_| None).expect("default site config")
    }
}

impl Site {
    /// Builds the config from `lookup`; empty values count as unset.
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Result<Self, String> {
        let var = |key: &str, default: &str| {
            lookup(key)
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| default.to_string())
        };
        let mode = var("API_MODE", "live");
        for (key, value) in [
            ("API_MODE", &mode),
            ("API_MODE_CATALOGUE", &var("API_MODE_CATALOGUE", &mode)),
            ("API_MODE_LINEAGE", &var("API_MODE_LINEAGE", "fixtures")),
        ] {
            if value != "live" && value != "fixtures" {
                return Err(format!("{key} must be \"live\" or \"fixtures\"."));
            }
        }
        let timeout: u32 = var("API_TIMEOUT_MS", "15000")
            .parse()
            .ok()
            .filter(|ms| *ms > 0)
            .ok_or("API_TIMEOUT_MS must be a positive integer.")?;
        let base_url = var("API_BASE_URL", "");
        let connect = if base_url.is_empty() {
            "'self'".to_string()
        } else {
            format!("'self' {}", origin(&base_url)?)
        };
        let config = json!({
            "brand": {
                "org": var("BRAND_ORG", "Accenture"),
                "name": var("BRAND_NAME", "The MCP Shed"),
                "tagline": var("BRAND_TAGLINE", "Agentic migration and modernisation"),
                "kind": var("BRAND_KIND", "Thought piece"),
                "disclaimer": var("BRAND_DISCLAIMER", DISCLAIMER),
            },
            "api": {
                "mode": mode,
                "baseUrl": base_url,
                "timeoutMs": timeout,
                "routes": {
                    "health": var("API_ROUTE_HEALTH", "/api/health"),
                    "catalogue": var("API_ROUTE_CATALOGUE", "/api/catalogue"),
                    "pipeline": var("API_ROUTE_PIPELINE", "/api/pipeline"),
                },
                "domains": {
                    "catalogue": var("API_MODE_CATALOGUE", &mode),
                    "lineage": var("API_MODE_LINEAGE", "fixtures"),
                },
            },
        });
        // '<' only occurs inside JSON strings, so escaping it keeps `</script>` and `<!--` inert.
        let body = config
            .to_string()
            .replace('<', "\\u003c")
            .replace('\u{2028}', "\\u2028")
            .replace('\u{2029}', "\\u2029");
        let csp = format!(
            "default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval'; \
             style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; \
             font-src https://fonts.gstatic.com; img-src 'self' data:; connect-src {connect}; \
             object-src 'none'; base-uri 'self'; form-action 'self'; frame-ancestors 'none'"
        );
        Ok(Self {
            config_js: format!("window.SITE_CONFIG = {body};\n"),
            csp: HeaderValue::from_str(&csp).map_err(|_| "API_BASE_URL is not a valid origin.")?,
        })
    }

    pub fn config_js(&self) -> &str {
        &self.config_js
    }

    pub fn csp(&self) -> &HeaderValue {
        &self.csp
    }
}

/// Reduces an http(s) URL to `scheme://host[:port]` for the CSP, rejecting anything else.
fn origin(url: &str) -> Result<String, String> {
    let invalid = || "API_BASE_URL must be an http or https URL with a plain host.".to_string();
    let (scheme, rest) = url
        .strip_prefix("https://")
        .map(|rest| ("https://", rest))
        .or_else(|| url.strip_prefix("http://").map(|rest| ("http://", rest)))
        .ok_or_else(invalid)?;
    let host = rest.split(['/', '?', '#']).next().unwrap_or("");
    let plain = |c: char| c.is_ascii_alphanumeric() || "-.:[]".contains(c);
    if host.is_empty() || !host.chars().all(plain) || url.chars().any(char::is_control) {
        return Err(invalid());
    }
    Ok(format!("{scheme}{host}"))
}
