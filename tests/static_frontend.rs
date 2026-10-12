use std::fs;
use std::path::{Path, PathBuf};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;
use tower_http::services::ServeDir;

const PAGES: [&str; 5] = [
    "index.html",
    "technical-guide/index.html",
    "environmental-agency/index.html",
    "lineage-migration/index.html",
    "little-bunnies/index.html",
];
const DEMOS: [&str; 3] = [
    "environmental-agency/index.html",
    "lineage-migration/index.html",
    "little-bunnies/index.html",
];

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static")
}

fn page(rel: &str) -> String {
    let path = root().join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// Relative prefix from a page back to the site root.
fn up(rel: &str) -> &'static str {
    if rel.contains('/') {
        "../"
    } else {
        ""
    }
}

fn browser_assets() -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut pending = vec![root()];
    while let Some(path) = pending.pop() {
        for entry in fs::read_dir(path).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|ext| ["html", "js", "css", "svg"].contains(&ext))
            {
                found.push(path);
            }
        }
    }
    found
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

fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let marker = format!(" {name}=\"");
    let start = tag.find(&marker)? + marker.len();
    Some(&tag[start..start + tag[start..].find('"')?])
}

#[tokio::test]
async fn static_routes_serve_the_new_page_set_and_drop_retired_ones() {
    let service = || ServeDir::new(root());
    for uri in [
        "/",
        "/technical-guide/",
        "/environmental-agency/",
        "/lineage-migration/",
        "/little-bunnies/",
        "/guide.css",
        "/favicon.svg",
        "/config.js",
        "/js/api.js",
        "/js/estate-graph.js",
        "/fixtures/catalogue.js",
        "/fixtures/lineage.js",
        "/assets/small-world.svg",
        "/assets/environmental-agency-landscape.svg",
    ] {
        let response = service()
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK, "{uri}");
    }
    for uri in [
        "/dcceew/",
        "/parkinsons/",
        "/demo-data.js",
        "/assets/parkinsons-research.svg",
        "/assets/dcceew-landscape.svg",
        "/assets/does-not-exist.svg",
    ] {
        let response = service()
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND, "{uri}");
    }
}

#[test]
fn every_page_has_document_basics() {
    for rel in PAGES {
        let html = page(rel);
        assert!(html.contains("<html lang=\"en\""), "{rel}: html lang");
        assert!(html.contains("name=\"viewport\""), "{rel}: viewport meta");
        assert!(
            html.contains("<meta name=\"description\""),
            "{rel}: description meta"
        );
        assert_eq!(tags(&html, "h1").len(), 1, "{rel}: exactly one h1");
        assert!(
            html.contains("class=\"skip\" href=\"#main\""),
            "{rel}: skip link to #main"
        );
        assert!(html.contains("id=\"main\""), "{rel}: main target");
    }
}

#[test]
fn every_page_loads_config_then_site_api_with_relative_paths() {
    for rel in PAGES {
        let html = page(rel);
        let up = up(rel);
        let config = html
            .find(&format!("<script src=\"{up}config.js\"></script>"))
            .unwrap_or_else(|| panic!("{rel}: config.js script"));
        let api = html
            .find(&format!("<script src=\"{up}js/api.js\"></script>"))
            .unwrap_or_else(|| panic!("{rel}: js/api.js script"));
        let head_end = html.find("</head>").expect("head");
        assert!(
            config < api && api < head_end,
            "{rel}: config.js then js/api.js in head"
        );
    }
}

#[test]
fn every_page_shares_header_footer_and_brand_bindings() {
    for rel in PAGES {
        let html = page(rel);
        let up = up(rel);
        assert!(
            html.contains("<nav class=\"site-nav\" aria-label=\"Primary\">"),
            "{rel}: shared primary nav"
        );
        let pillars = format!("{up}#pillars");
        let use_cases = format!("{up}#use-cases");
        let guide = format!("{up}technical-guide/");
        for href in [&pillars, &use_cases, &guide] {
            assert!(html.contains(&format!("href=\"{href}\"")), "{rel}: {href}");
        }
        for key in ["name", "org", "kind", "disclaimer"] {
            assert!(
                html.contains(&format!("data-brand=\"{key}\"")),
                "{rel}: data-brand={key}"
            );
        }
        assert!(html.contains("<footer"), "{rel}: shared footer");
    }
}

#[test]
fn every_internal_link_and_asset_is_relative() {
    for rel in PAGES {
        let html = page(rel);
        assert!(
            !html.contains("/graphs-and-jev/"),
            "{rel}: GitHub Pages base path"
        );
        for name in ["a", "link", "script", "img", "source"] {
            for tag in tags(&html, name) {
                for key in ["href", "src"] {
                    if let Some(url) = attr(tag, key) {
                        assert!(!url.starts_with('/'), "{rel}: root-relative {key}: {tag}");
                    }
                }
            }
        }
    }
}

#[test]
fn landing_page_links_to_every_page_and_owns_the_shared_anchors() {
    let html = page("index.html");
    for id in ["pillars", "use-cases", "example"] {
        assert!(html.contains(&format!("id=\"{id}\"")), "landing: #{id}");
    }
    for href in [
        "technical-guide/",
        "environmental-agency/",
        "lineage-migration/",
        "little-bunnies/",
    ] {
        assert!(
            html.contains(&format!("href=\"{href}\"")),
            "landing: {href}"
        );
    }
}

#[test]
fn technical_guide_has_pillar_anchors() {
    let html = page("technical-guide/index.html");
    for id in [
        "connectivity",
        "relational-context",
        "logic",
        "reflexes",
        "visibility-and-cost",
    ] {
        assert!(
            html.contains(&format!("id=\"{id}\"")),
            "technical guide: #{id}"
        );
    }
}

#[test]
fn demo_pages_announce_demo_status() {
    for rel in DEMOS {
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
        for img in tags(&page(rel), "img") {
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

#[test]
fn only_site_api_calls_fetch_and_demos_use_it() {
    let api = root().join("js/api.js");
    for path in browser_assets() {
        if path == api {
            continue;
        }
        let text = fs::read_to_string(&path).unwrap();
        assert!(
            !text.contains("fetch("),
            "{}: fetch outside js/api.js",
            path.display()
        );
    }
    let catalogue = page("environmental-agency/index.html");
    assert!(
        catalogue.contains("siteApi.catalogue("),
        "Environmental Agency loads the catalogue"
    );
    assert!(
        catalogue.contains("siteApi.pipeline("),
        "Environmental Agency calls the pipeline"
    );
    assert!(
        page("lineage-migration/index.html").contains("siteApi.pipeline("),
        "lineage calls the pipeline"
    );
}

#[test]
fn live_demos_keep_accessible_state_contracts() {
    for rel in [
        "environmental-agency/index.html",
        "lineage-migration/index.html",
    ] {
        let html = page(rel);
        assert!(html.contains("setStatus(\"Working"), "{rel}: working state");
        assert!(html.contains("\"error\")"), "{rel}: error state");
        assert!(html.contains("aria-busy"), "{rel}: busy state");
        assert!(
            html.contains("disabled = true"),
            "{rel}: submit disabled while working"
        );
    }
}

#[test]
fn removed_topics_and_retired_files_are_gone() {
    for path in browser_assets() {
        let text = fs::read_to_string(&path).unwrap().to_lowercase();
        for forbidden in ["parkinsons", "parkinson's", "dcceew", "demo-data.js"] {
            assert!(!text.contains(forbidden), "{}: {forbidden}", path.display());
        }
    }
    for retired in ["dcceew", "parkinsons", "demo-data.js"] {
        assert!(
            !root().join(retired).exists(),
            "static/{retired} still exists"
        );
    }
}

#[test]
fn browser_assets_contain_no_runtime_secrets_analytics_or_raw_personal_data() {
    for path in browser_assets() {
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

#[test]
fn pages_load_no_remote_scripts_and_only_google_fonts_styles() {
    for rel in PAGES {
        let html = page(rel);
        for tag in tags(&html, "script") {
            assert!(!tag.contains("src=\"http"), "{rel}: remote script: {tag}");
        }
        for tag in tags(&html, "link") {
            if let Some(href) = attr(tag, "href").filter(|h| h.starts_with("http")) {
                assert!(
                    href.starts_with("https://fonts.googleapis.com")
                        || href.starts_with("https://fonts.gstatic.com"),
                    "{rel}: remote link outside Google Fonts: {tag}"
                );
            }
        }
        for img in tags(&html, "img") {
            assert!(!img.contains("src=\"http"), "{rel}: remote image: {img}");
        }
    }
}

#[test]
fn pages_drop_retired_credits_and_unsupported_savings_claims() {
    for rel in PAGES {
        let lower = page(rel).to_lowercase();
        for forbidden in [
            "the palindrome",
            "thepalindrome.org",
            "cost savings",
            "save money",
            "cheaper by",
            "sub-100",
        ] {
            assert!(!lower.contains(forbidden), "{rel}: {forbidden}");
        }
    }
}

#[test]
fn every_page_references_local_favicon() {
    for rel in PAGES {
        let up = up(rel);
        assert!(
            page(rel).contains(&format!(
                r#"<link rel="icon" href="{up}favicon.svg" type="image/svg+xml">"#
            )),
            "{rel}: local favicon"
        );
    }
}

#[test]
fn technical_guide_keeps_spanner_spotlight_without_regrading() {
    let html = page("technical-guide/index.html");
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
fn technical_guide_preserves_labs_snippets_runs_and_sources() {
    let html = page("technical-guide/index.html") + &page("js/labs.js");
    for contract in [
        "id=\"labA\"",
        "id=\"labB\"",
        "id=\"labC\"",
        "id=\"snipTabs\"",
        "id=\"runList\"",
        "Sources and caveats",
        "const E=[",
        "const SNIPS=[",
        "const RUNS=[",
    ] {
        assert!(html.contains(contract), "technical guide: {contract}");
    }
}

fn engine_rows(html: &str) -> Vec<&str> {
    let start = html
        .find("const E=[")
        .expect("technical guide: engine metadata");
    let block = &html[start..];
    let end = block.find("\n];").expect("end of engine metadata");
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
fn engine_marks_are_local_and_documented() {
    let html = page("js/labs.js");
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
        let file = path
            .strip_prefix("../assets/engine-logos/")
            .filter(|file| !file.contains(".."))
            .unwrap_or_else(|| panic!("logo below ../assets/engine-logos: {path}"));
        assert!(
            Path::new(&root().join("assets/engine-logos").join(file)).is_file(),
            "logo file exists: {path}"
        );
    }
}

#[test]
fn engine_marks_are_inert() {
    let dir = root().join("assets/engine-logos");
    for entry in fs::read_dir(&dir).unwrap().flatten() {
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
fn sticky_header_offsets_and_touch_targets_hold() {
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
    assert!(
        css.contains("min-width:44px") && css.contains("min-height:44px"),
        "44px touch targets"
    );
}
