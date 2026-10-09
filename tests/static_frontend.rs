use std::fs;
use std::path::PathBuf;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;
use tower_http::services::ServeDir;

const PAGES: [&str; 4] = [
    "index.html",
    "dcceew/index.html",
    "parkinsons/index.html",
    "little-bunnies/index.html",
];
const USE_CASES: [&str; 3] = [
    "dcceew/index.html",
    "parkinsons/index.html",
    "little-bunnies/index.html",
];

fn page(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("static")
        .join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

#[tokio::test]
async fn static_routes_serve_pages_and_assets() {
    let service = || ServeDir::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static"));
    for uri in [
        "/",
        "/dcceew/",
        "/parkinsons/",
        "/little-bunnies/",
        "/guide.css",
        "/favicon.svg",
        "/assets/small-world.svg",
    ] {
        let response = service()
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK, "{uri}");
    }

    let response = service()
        .oneshot(
            Request::builder()
                .uri("/assets/does-not-exist.svg")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

fn tags<'a>(html: &'a str, name: &str) -> Vec<&'a str> {
    let open = format!("<{name}");
    let mut found = Vec::new();
    let mut rest = html;
    while let Some(i) = rest.find(&open) {
        let after = &rest[i + open.len()..];
        if after.starts_with(|c: char| c.is_whitespace() || c == '>' || c == '/') {
            let end = after.find('>').map_or(after.len(), |e| e + 1);
            found.push(&rest[i..i + open.len() + end]);
        }
        rest = after;
    }
    found
}

#[test]
fn every_page_has_document_basics() {
    for rel in PAGES {
        let html = page(rel);
        assert!(html.contains("<html lang=\"en\""), "{rel}: html lang");
        assert!(html.contains("name=\"viewport\""), "{rel}: viewport meta");
        assert_eq!(tags(&html, "h1").len(), 1, "{rel}: exactly one h1");
        assert!(
            html.contains("class=\"skip\" href=\"#main\""),
            "{rel}: skip link to #main"
        );
        assert!(html.contains("id=\"main\""), "{rel}: main target");
    }
}

#[test]
fn every_page_shares_primary_navigation() {
    for rel in PAGES {
        let html = page(rel);
        assert!(
            html.contains("<nav class=\"site-nav\" aria-label=\"Primary\">"),
            "{rel}: shared primary nav"
        );
        for href in ["/#how-it-works", "/#use-cases", "/#technical-guide"] {
            assert!(html.contains(&format!("href=\"{href}\"")), "{rel}: {href}");
        }
    }
}

#[test]
fn homepage_has_story_anchors() {
    let html = page("index.html");
    for id in ["how-it-works", "use-cases", "technical-guide"] {
        assert!(html.contains(&format!("id=\"{id}\"")), "homepage: #{id}");
    }
}

#[test]
fn use_case_pages_announce_demo_status() {
    for rel in USE_CASES {
        let html = page(rel);
        let status = tags(&html, "p")
            .into_iter()
            .chain(tags(&html, "div"))
            .any(|t| t.contains("class=\"demo-status\"") && t.contains("aria-live=\"polite\""));
        assert!(status, "{rel}: demo status region with aria-live=polite");
    }
}

#[test]
fn every_image_declares_alt_text() {
    for rel in PAGES {
        let html = page(rel);
        for img in tags(&html, "img") {
            assert!(img.contains(" alt=\""), "{rel}: image without alt: {img}");
        }
    }
}

#[test]
fn legacy_chrome_tabs_are_gone() {
    for rel in PAGES {
        assert!(
            !page(rel).contains("class=\"chrome\""),
            "{rel}: legacy chrome tabs"
        );
    }
    assert!(!page("guide.css").contains(".chrome"), "guide.css: .chrome");
}

fn engine_rows(html: &str) -> Vec<&str> {
    let start = html.find("const E=[").expect("homepage: engine metadata");
    let block = &html[start..];
    let end = block
        .find("\n];")
        .expect("homepage: end of engine metadata");
    block[..end]
        .lines()
        .filter(|l| l.trim_start().starts_with("{n:"))
        .collect()
}

fn quoted_after<'a>(row: &'a str, key: &str) -> Option<&'a str> {
    let rest = &row[row.find(key)? + key.len()..];
    let rest = rest.strip_prefix('"')?;
    Some(&rest[..rest.find('"')?])
}

#[test]
fn homepage_has_spanner_graph_spotlight() {
    let html = page("index.html");
    assert!(
        html.contains("id=\"spanner-graph-spotlight\""),
        "spotlight section"
    );
    assert!(
        html.contains("Google Cloud Spanner Graph"),
        "full product name"
    );
    for url in [
        "https://cloud.google.com/spanner/docs/graph/overview",
        "https://cloud.google.com/spanner/docs/graph/queries-overview",
        "https://cloud.google.com/spanner/pricing",
    ] {
        assert!(
            html.contains(&format!("href=\"{url}\"")),
            "official link {url}"
        );
    }
    assert!(html.contains(">Strong fit<"), "Strong fit heading");
    assert!(
        html.contains(">Consider carefully<"),
        "Consider carefully heading"
    );
    let lower = html.to_lowercase();
    for phrase in ["sponsor", "upgraded", "grade raised", "raised its grade"] {
        assert!(
            !lower.contains(phrase),
            "grade must not change for the event: {phrase}"
        );
    }
}

#[test]
fn engine_marks_are_local_and_documented() {
    let html = page("index.html");
    let rows = engine_rows(&html);
    assert!(rows.len() >= 11, "engine metadata rows: {}", rows.len());
    for row in rows {
        assert!(
            row.contains("initials:\""),
            "neutral fallback initials: {row}"
        );
        if row.contains("logo:null") {
            continue;
        }
        let path = quoted_after(row, "logo:").unwrap_or_else(|| panic!("logo path or null: {row}"));
        assert!(
            path.starts_with("/assets/engine-logos/") && !path.contains(".."),
            "logo below assets/engine-logos: {path}"
        );
        let file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("static")
            .join(path.strip_prefix("/").unwrap());
        assert!(file.is_file(), "logo file exists: {path}");
    }
    for img in tags(&html, "img") {
        assert!(!img.contains("src=\"http"), "remote image: {img}");
    }
}

#[test]
fn engine_marks_are_inert() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static/assets/engine-logos");
    let Ok(entries) = fs::read_dir(&dir) else {
        panic!("{}: missing", dir.display())
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("svg") {
            continue;
        }
        let svg = fs::read_to_string(&path).unwrap().to_lowercase();
        for bad in [
            "<script",
            "onload=",
            "onclick=",
            "href=\"http",
            "href='http",
            "url(http",
        ] {
            assert!(!svg.contains(bad), "{}: {bad}", path.display());
        }
    }
}

#[test]
fn sticky_header_anchor_offsets_cover_desktop_and_mobile() {
    let css = page("guide.css");
    assert!(
        css.contains("--header-offset:84px"),
        "desktop header offset token"
    );
    assert!(
        css.contains("@media (max-width:820px){:root{--header-offset:132px}}"),
        "mobile header offset token"
    );
    assert!(
        css.contains("scroll-padding-top:var(--header-offset)"),
        "document anchor offset"
    );
}

#[test]
fn live_use_cases_call_only_same_origin_api_routes() {
    let dcceew = page("dcceew/index.html");
    assert!(
        dcceew.contains("fetch(\"/api/catalogue\")"),
        "DCCEEW loads the Rust catalogue"
    );
    assert!(
        dcceew.contains("fetch(\"/api/pipeline\""),
        "DCCEEW calls the Rust pipeline"
    );

    let parkinsons = page("parkinsons/index.html");
    assert!(
        parkinsons.contains("fetch(\"/api/pipeline\""),
        "Parkinson's calls the Rust pipeline"
    );

    for rel in PAGES {
        let html = page(rel);
        assert!(
            !html.contains("demo-data.js"),
            "{rel}: static fixture script"
        );
        assert!(!html.contains("fetch(\"http"), "{rel}: cross-origin fetch");
    }
    assert!(!PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("static/demo-data.js")
        .exists());
}

#[test]
fn live_use_cases_keep_accessible_state_contracts() {
    for rel in ["dcceew/index.html", "parkinsons/index.html"] {
        let html = page(rel);
        assert!(
            html.contains("setStatus(\"Working:") || html.contains("setStatus(\"Working"),
            "{rel}: working state"
        );
        assert!(html.contains("\"error\")"), "{rel}: error state");
        assert!(
            html.contains("\"unavailable\")"),
            "{rel}: unavailable state"
        );
        assert!(html.contains("aria-busy"), "{rel}: busy state");
        assert!(
            html.contains("disabled = true"),
            "{rel}: submit disabled while working"
        );
    }
}

#[test]
fn publication_copy_matches_live_backends_and_disclosures() {
    let home = page("index.html");
    let dcceew = page("dcceew/index.html");
    let parkinsons = page("parkinsons/index.html");

    assert!(dcceew.contains("deterministic Rust catalogue rules"));
    assert!(dcceew.to_lowercase().contains("illustrative"));
    assert!(parkinsons.contains("Wikipedia") && parkinsons.contains("Europe PMC"));
    assert!(parkinsons.contains("live public-source retrieval"));
    assert!(parkinsons.to_lowercase().contains("not clinical advice"));
    assert!(parkinsons.contains("refused before retrieval"));
    assert!(home.contains("deterministic Rust catalogue rules"));
    assert!(home.contains("Wikipedia and Europe PMC"));
}

#[test]
fn browser_assets_contain_no_runtime_secrets_analytics_or_raw_personal_data() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static");
    let mut pending = vec![root];
    while let Some(path) = pending.pop() {
        for entry in fs::read_dir(path).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
                continue;
            }
            let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
                continue;
            };
            if !["html", "js", "css", "svg"].contains(&ext) {
                continue;
            }
            let text = fs::read_to_string(&path).unwrap();
            for forbidden in [
                "API_TOKEN",
                "ninth-airship-386815",
                "Authorization: Bearer",
                "googletagmanager",
                "google-analytics",
                "gtag(",
                "John Doe",
            ] {
                assert!(!text.contains(forbidden), "{}: {forbidden}", path.display());
            }
        }
    }
}

#[test]
fn browser_assets_use_cloud_run_root_paths() {
    for rel in PAGES {
        let html = page(rel);
        assert!(
            !html.contains("/graphs-and-jev/"),
            "{rel}: stale GitHub Pages base path"
        );
    }
}

#[test]
fn every_page_references_local_favicon() {
    for rel in PAGES {
        assert!(
            page(rel).contains(r#"<link rel="icon" href="/favicon.svg" type="image/svg+xml">"#),
            "{rel}: local favicon"
        );
    }
}
