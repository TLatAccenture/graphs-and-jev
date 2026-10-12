use std::net::{Ipv4Addr, Ipv6Addr};

use axum::http::HeaderValue;
use serde_json::json;

const DISCLAIMER: &str = "A personal research thought piece on agentic migration and modernisation. It is not an official Accenture or Google Cloud product or a service offer. Product names and marks belong to their owners.";

/// Browser config and CSP, rendered once at start-up from environment variables.
pub struct Site {
    config_js: String,
    csp: HeaderValue,
    csp_labs: HeaderValue,
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
                "name": var("BRAND_NAME", "The MCP Tool Shed"),
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
        // The technical-guide labs run edited code with `new Function`, so only that page gets 'unsafe-eval'.
        let policy = |eval: &str| {
            format!(
                "default-src 'self'; script-src 'self' 'unsafe-inline'{eval}; \
                 style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; \
                 font-src https://fonts.gstatic.com; img-src 'self' data:; connect-src {connect}; \
                 object-src 'none'; base-uri 'self'; form-action 'self'; frame-ancestors 'none'"
            )
        };
        let header = |value: String| {
            HeaderValue::from_str(&value)
                .map_err(|_| "API_BASE_URL is not a valid origin.".to_string())
        };
        Ok(Self {
            config_js: format!("window.SITE_CONFIG = {body};\n"),
            csp: header(policy(""))?,
            csp_labs: header(policy(" 'unsafe-eval'"))?,
        })
    }

    pub fn config_js(&self) -> &str {
        &self.config_js
    }

    pub fn csp(&self, path: &str) -> &HeaderValue {
        if matches!(path, "/technical-guide/" | "/technical-guide/index.html") {
            &self.csp_labs
        } else {
            &self.csp
        }
    }
}

/// Reduces an http(s) URL to `scheme://host[:port]` for the CSP. The host must be a DNS name,
/// IPv4 address or bracketed IPv6 address, and the port 0-65535, so browsers can parse it too.
fn origin(url: &str) -> Result<String, String> {
    let invalid = || {
        "API_BASE_URL must be an http or https URL with a DNS name, IPv4 or [IPv6] host and an optional port."
            .to_string()
    };
    let (scheme, rest) = url
        .strip_prefix("https://")
        .map(|rest| ("https://", rest))
        .or_else(|| url.strip_prefix("http://").map(|rest| ("http://", rest)))
        .ok_or_else(invalid)?;
    if url.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return Err(invalid());
    }
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    let port = if let Some(v6) = authority.strip_prefix('[') {
        let (addr, after) = v6.split_once(']').ok_or_else(invalid)?;
        addr.parse::<Ipv6Addr>().map_err(|_| invalid())?;
        match after {
            "" => None,
            _ => Some(after.strip_prefix(':').ok_or_else(invalid)?),
        }
    } else {
        let (host, port) = match authority.split_once(':') {
            Some((host, port)) => (host, Some(port)),
            None => (authority, None),
        };
        let label = |l: &str| {
            (1..=63).contains(&l.len())
                && l.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
                && !l.starts_with('-')
                && !l.ends_with('-')
        };
        let numeric_tail = host
            .rsplit('.')
            .next()
            .is_some_and(|l| l.bytes().all(|b| b.is_ascii_digit()));
        if !host.split('.').all(label) || (numeric_tail && host.parse::<Ipv4Addr>().is_err()) {
            return Err(invalid());
        }
        port
    };
    if let Some(port) = port {
        if port.is_empty()
            || !port.bytes().all(|b| b.is_ascii_digit())
            || port.parse::<u16>().is_err()
        {
            return Err(invalid());
        }
    }
    Ok(format!("{scheme}{authority}"))
}
