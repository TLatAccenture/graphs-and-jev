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
const PUBLICATION_TITLE: &str = "Accelerating stack migration and modernisation with Google Cloud";
const BRAND_NAME: &str = "Migration Acceleration with Google Cloud";
const BRAND_TAGLINE: &str = "Stack migration and modernisation";
const GCP_ICONS: [(&str, &str); 8] = [
    ("compute-engine", "compute-engine.svg"),
    ("alloydb", "alloydb.svg"),
    ("cloud-run", "cloud-run.svg"),
    ("cloud-storage", "cloud-storage.svg"),
    ("bigquery", "bigquery.svg"),
    ("looker", "looker.svg"),
    ("adk", "adk.png"),
    ("gemini", "gemini.png"),
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
fn publication_brand_defaults_match_the_google_cloud_acceleration_story() {
    let config = page("config.js");
    assert!(
        config.contains(&format!("name: \"{BRAND_NAME}\"")),
        "static config brand name"
    );
    assert!(
        config.contains(&format!("tagline: \"{BRAND_TAGLINE}\"")),
        "static config brand tagline"
    );
    assert!(
        page("index.html").contains(&format!("<title>{PUBLICATION_TITLE}</title>")),
        "landing document title"
    );
    let hero = page("index.html");
    assert!(
        hero.contains("<h1 id=\"hero-title\">Accelerate stack migration and modernisation <em>with a Gemini Migration Agent</em></h1>"),
        "landing heading leads with the outcome and names the Gemini Migration Agent"
    );
    let plane = between(&hero, "<figure class=\"hero-plane\"", "</figure>");
    for text in [
        "Reference design",
        "Gemini Migration Agent",
        "Built with ADK",
        "Assess",
        "Plan",
        "Person approves",
        "Deploy",
        "Optimise",
        "Jev gate",
        "From",
        "To",
    ] {
        assert!(plane.contains(text), "hero control-plane summary: {text}");
    }
    assert!(
        !hero.contains("hero-shed.svg") && !hero.contains("hero-migration.svg"),
        "the hero summary is HTML text, not an image"
    );
    for rel in PAGES {
        assert!(
            page(rel).contains(BRAND_NAME),
            "{rel}: no-JavaScript brand fallback"
        );
    }
}

#[test]
fn landing_distinguishes_live_today_from_the_reference_architecture() {
    let html = page("index.html");
    assert!(
        html.contains("id=\"implementation-status\""),
        "implementation status region"
    );
    for label in ["Live today", "Google Cloud reference architecture"] {
        assert!(html.contains(label), "status label: {label}");
    }
    for text in [
        "On Cloud Run now: the decision layer that gates each migration step, shown in three working demonstrations.",
        "The blueprint for a Gemini Migration Agent that runs Assess, Plan, Deploy and Optimise end to end, built on that same decision layer. The full agent is not part of this public demo.",
    ] {
        assert!(html.contains(text), "exact live/reference wording: {text}");
    }
}

#[test]
fn landing_keeps_exactly_five_pillars_and_names_the_planned_product_roles() {
    let html = page("index.html");
    assert_eq!(
        tags(&html, "button")
            .into_iter()
            .filter(|tag| tag.contains("class=\"stack-layer\""))
            .count(),
        5,
        "accessible pillar controls"
    );
    for product in [
        "cloud-run",
        "compute-engine",
        "cloud-storage",
        "alloydb-ai",
        "hosted-jev-api",
        "rag-engine",
        "adk",
        "gemini",
        "bigquery",
        "spanner-graph",
        "looker",
    ] {
        assert!(
            html.contains(&format!("data-product=\"{product}\"")),
            "planned role: {product}"
        );
    }
    for text in [
        "Agent Development Kit (ADK)",
        "Gemini",
        "Google Cloud RAG Engine",
        "AlloyDB AI",
        "TypeSafe's hosted Jev API",
        "Cloud Run",
        "Compute Engine",
        "Cloud Storage",
        "BigQuery",
        "Spanner Graph",
        "Looker",
    ] {
        assert!(html.contains(text), "product role text: {text}");
    }
}

#[test]
fn landing_and_guide_link_the_required_primary_sources() {
    let html = page("index.html") + &page("technical-guide/index.html");
    for url in [
        "https://docs.cloud.google.com/gemini-enterprise-agent-platform/build/adk",
        "https://google.github.io/adk-docs/mcp/",
        "https://docs.cloud.google.com/gemini-enterprise-agent-platform/build/rag-engine/layout-parser-integration",
        "https://docs.cloud.google.com/architecture/single-agent-ai-system-adk-cloud-run",
        "https://codelabs.developers.google.com/alloydb-ai-jev?hl=en#0",
        "https://codelabs.developers.google.com/codelabs/discriminative-models#0",
        "https://cloud.google.com/icons",
        "https://ai.google.dev/gemini-api/docs/long-context",
        "https://aclanthology.org/2024.tacl-1.9/",
        "https://www.anthropic.com/engineering/code-execution-with-mcp",
        "https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents",
        "https://modelcontextprotocol.io/specification/2026-07-28/server",
        "https://stripe.dev/blog/minions-stripes-one-shot-end-to-end-coding-agents",
        "https://visdom-maturity-matrix.virtuslab.com/guides/infrastructure/toolshed-model-400-tools-behind-one-mcp-stripe",
    ] {
        assert!(html.contains(&format!("href=\"{url}\"")), "source: {url}");
    }
    assert!(
        html.contains("secondary source"),
        "VirtusLab is visibly labelled as a secondary source"
    );
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
            "AIza",
            "ya29.",
            "GOCSPX-",
            "-----BEGIN PRIVATE KEY-----",
            "GOOGLE_CLOUD_PROJECT=",
            "projectId:",
            "project_id:",
            "measurementId:",
        ] {
            assert!(!text.contains(forbidden), "{}: {forbidden}", path.display());
        }
    }
}

#[test]
fn browser_assets_reject_unqualified_live_google_cloud_claims() {
    for path in browser_assets() {
        let text = fs::read_to_string(&path).unwrap();
        for forbidden in [
            "MCP runs on GCE.",
            "Jev runs on AlloyDB.",
            "RAG is live.",
            "ADK powers this demo.",
            "Gemini decides the current demonstrations.",
            "DiffusionGemma is deployed here.",
        ] {
            assert!(!text.contains(forbidden), "{}: {forbidden}", path.display());
        }
    }
}

#[test]
fn pages_load_no_remote_scripts_styles_fonts_or_images() {
    for rel in PAGES {
        let html = page(rel);
        for tag in tags(&html, "script") {
            assert!(!tag.contains("src=\"http"), "{rel}: remote script: {tag}");
        }
        for tag in tags(&html, "link") {
            assert!(
                !attr(tag, "href").is_some_and(|h| h.starts_with("http") || h.starts_with("//")),
                "{rel}: remote link: {tag}"
            );
        }
        for img in tags(&html, "img") {
            assert!(!img.contains("src=\"http"), "{rel}: remote image: {img}");
        }
    }
}

#[test]
fn fonts_are_served_by_the_site() {
    let css = page("guide.css");
    let faces: Vec<&str> = css.split("@font-face{").skip(1).collect();
    assert_eq!(
        faces.len(),
        6,
        "three weights each of Instrument Sans and JetBrains Mono"
    );
    for face in faces {
        let url = face
            .split("url(\"")
            .nth(1)
            .and_then(|rest| rest.split('"').next())
            .expect("font url");
        assert!(
            url.starts_with("assets/fonts/") && url.ends_with(".woff2"),
            "{url}"
        );
        assert!(root().join(url).is_file(), "{url} is missing");
    }
    for licence in ["OFL-InstrumentSans.txt", "OFL-JetBrainsMono.txt"] {
        assert!(
            root().join("assets/fonts").join(licence).is_file(),
            "{licence} must ship with the fonts"
        );
    }
    assert!(!css.contains("fonts.googleapis") && !css.contains("fonts.gstatic"));
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
    for dir in ["assets/engine-logos", "assets/gcp-icons"] {
        let dir = root().join(dir);
        for entry in fs::read_dir(&dir).unwrap().flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("svg") {
                continue;
            }
            let svg = fs::read_to_string(&path).unwrap();
            assert!(
                svg_inert_violations(&svg).is_empty(),
                "{}: {:?}",
                path.display(),
                svg_inert_violations(&svg)
            );
        }
    }
}

fn svg_inert_violations(svg: &str) -> Vec<&'static str> {
    let lower = svg.to_ascii_lowercase();
    let mut violations = Vec::new();

    if lower.contains("<!doctype") {
        push_violation(&mut violations, "document type declaration");
    }
    if lower.contains("<!entity") {
        push_violation(&mut violations, "entity declaration");
    }
    if lower.contains("<?xml-stylesheet") {
        push_violation(&mut violations, "external stylesheet");
    }
    scan_css_references(&lower, &mut violations);

    let bytes = lower.as_bytes();
    let mut cursor = 0;
    while cursor < bytes.len() {
        let Some(relative_start) = lower[cursor..].find('<') else {
            break;
        };
        let start = cursor + relative_start;
        if lower[start..].starts_with("<!--") {
            cursor = lower[start + 4..]
                .find("-->")
                .map_or(bytes.len(), |end| start + 4 + end + 3);
            continue;
        }
        let end = xml_tag_end(bytes, start + 1);
        let mut name_start = start + 1;
        while name_start < end && bytes[name_start].is_ascii_whitespace() {
            name_start += 1;
        }
        if name_start >= end
            || matches!(bytes[name_start], b'/' | b'!' | b'?')
            || bytes[name_start] == b'>'
        {
            cursor = end.saturating_add(1);
            continue;
        }
        let mut name_end = name_start;
        while name_end < end
            && !bytes[name_end].is_ascii_whitespace()
            && !matches!(bytes[name_end], b'/' | b'>')
        {
            name_end += 1;
        }
        let local_name = lower[name_start..name_end]
            .rsplit(':')
            .next()
            .unwrap_or_default();
        let qualified_name = &lower[name_start..name_end];
        if matches!(
            local_name,
            "script"
                | "foreignobject"
                | "iframe"
                | "object"
                | "embed"
                | "animate"
                | "animatemotion"
                | "animatetransform"
                | "animatecolor"
                | "set"
        ) {
            push_violation(&mut violations, "active element");
        }
        if qualified_name.contains(':') || !allowed_svg_element(local_name) {
            push_violation(&mut violations, "unsupported SVG element");
        }
        scan_xml_attributes(&lower[name_end..end], &mut violations);
        if local_name == "style" {
            let content_start = end.saturating_add(1);
            if let Some(relative_close) = lower[content_start..].find("</style") {
                let content = &lower[content_start..content_start + relative_close];
                if content.contains('<') {
                    push_violation(&mut violations, "markup inside style element");
                } else {
                    scan_css_references(content, &mut violations);
                }
            } else {
                push_violation(&mut violations, "unclosed style element");
            }
        }
        cursor = end.saturating_add(1);
    }

    violations
}

fn allowed_svg_element(name: &str) -> bool {
    matches!(
        name,
        "circle"
            | "clippath"
            | "defs"
            | "g"
            | "lineargradient"
            | "path"
            | "polygon"
            | "radialgradient"
            | "rect"
            | "stop"
            | "style"
            | "svg"
            | "title"
    )
}

fn push_violation(violations: &mut Vec<&'static str>, label: &'static str) {
    if !violations.contains(&label) {
        violations.push(label);
    }
}

fn xml_tag_end(bytes: &[u8], start: usize) -> usize {
    let mut quote = None;
    for (index, byte) in bytes.iter().enumerate().skip(start) {
        match (quote, *byte) {
            (Some(active), current) if current == active => quote = None,
            (None, b'\'' | b'"') => quote = Some(*byte),
            (None, b'>') => return index,
            _ => {}
        }
    }
    bytes.len()
}

fn scan_xml_attributes(attributes: &str, violations: &mut Vec<&'static str>) {
    let bytes = attributes.as_bytes();
    let mut cursor = 0;
    while cursor < bytes.len() {
        while cursor < bytes.len() && (bytes[cursor].is_ascii_whitespace() || bytes[cursor] == b'/')
        {
            cursor += 1;
        }
        let name_start = cursor;
        while cursor < bytes.len()
            && !bytes[cursor].is_ascii_whitespace()
            && !matches!(bytes[cursor], b'=' | b'/' | b'>')
        {
            cursor += 1;
        }
        if name_start == cursor {
            cursor += usize::from(cursor < bytes.len());
            continue;
        }
        let name = &attributes[name_start..cursor];
        if !allowed_svg_attribute(name) {
            push_violation(violations, "unsupported SVG attribute");
        }
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= bytes.len() || bytes[cursor] != b'=' {
            continue;
        }
        cursor += 1;
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        let value = if cursor < bytes.len() && matches!(bytes[cursor], b'\'' | b'"') {
            let quote = bytes[cursor];
            cursor += 1;
            let value_start = cursor;
            while cursor < bytes.len() && bytes[cursor] != quote {
                cursor += 1;
            }
            let value = &attributes[value_start..cursor];
            cursor += usize::from(cursor < bytes.len());
            value
        } else {
            let value_start = cursor;
            while cursor < bytes.len()
                && !bytes[cursor].is_ascii_whitespace()
                && !matches!(bytes[cursor], b'/' | b'>')
            {
                cursor += 1;
            }
            &attributes[value_start..cursor]
        };

        let local_name = name.rsplit(':').next().unwrap_or_default();
        if local_name.starts_with("on") && local_name.len() > 2 {
            push_violation(violations, "event-handler attribute");
        }
        if matches!(local_name, "href" | "src") && !value.trim_start().starts_with('#') {
            push_violation(violations, "external resource reference");
        }
        if matches!(local_name, "clip-path" | "fill" | "style") {
            scan_css_references(value, violations);
        }
    }
}

fn allowed_svg_attribute(name: &str) -> bool {
    matches!(
        name,
        "class"
            | "clip-path"
            | "clip-rule"
            | "cx"
            | "cy"
            | "d"
            | "enable-background"
            | "fill"
            | "fill-opacity"
            | "fill-rule"
            | "gradienttransform"
            | "gradientunits"
            | "height"
            | "id"
            | "offset"
            | "points"
            | "r"
            | "stop-color"
            | "stop-opacity"
            | "style"
            | "type"
            | "version"
            | "viewbox"
            | "width"
            | "x"
            | "x1"
            | "x2"
            | "xml:space"
            | "xmlns"
            | "xmlns:xlink"
            | "y"
            | "y1"
            | "y2"
    )
}

fn scan_css_references(css: &str, violations: &mut Vec<&'static str>) {
    if css.contains('\\') {
        push_violation(violations, "CSS escape");
    }
    if css.contains("&#") {
        push_violation(violations, "numeric character reference");
    }
    if css.contains("/*") {
        push_violation(violations, "CSS comment");
    }
    let compact: String = css
        .chars()
        .filter(|character| !character.is_ascii_whitespace())
        .collect();
    if compact.contains("@import") {
        push_violation(violations, "CSS import");
    }
    if compact.contains("image-set(") {
        push_violation(violations, "external CSS URL");
    }

    let bytes = css.as_bytes();
    let mut cursor = 0;
    while cursor + 3 <= bytes.len() {
        if &bytes[cursor..cursor + 3] != b"url" {
            cursor += 1;
            continue;
        }
        let mut value_start = cursor + 3;
        while value_start < bytes.len() && bytes[value_start].is_ascii_whitespace() {
            value_start += 1;
        }
        if value_start >= bytes.len() || bytes[value_start] != b'(' {
            cursor += 3;
            continue;
        }
        value_start += 1;
        while value_start < bytes.len() && bytes[value_start].is_ascii_whitespace() {
            value_start += 1;
        }
        let quote = bytes
            .get(value_start)
            .copied()
            .filter(|byte| matches!(byte, b'\'' | b'"'));
        value_start += usize::from(quote.is_some());
        while value_start < bytes.len() && bytes[value_start].is_ascii_whitespace() {
            value_start += 1;
        }
        if bytes.get(value_start) != Some(&b'#') {
            push_violation(violations, "external CSS URL");
        }
        cursor = value_start.saturating_add(1);
    }
}

#[test]
fn svg_inert_gate_rejects_active_content_classes() {
    let active_fixtures = [
        "<svg><script>alert(1)</script></svg>",
        "<svg onpointerenter = \"alert(1)\"></svg>",
        "<svg><foreignObject><iframe /></foreignObject></svg>",
        "<svg><style>@import url(https://example.test/x.css)</style></svg>",
        "<svg><a href=\"javascript:alert(1)\">x</a></svg>",
        "<svg><image href='https://example.test/x.png' /></svg>",
        "<svg xmlns:s=\"urn:test\"><s:script>alert(1)</s:script></svg>",
        "<svg><image href = \"https://example.test/x.png\" /></svg>",
        "<svg><image href=\"//example.test/x.png\" /></svg>",
        "<!DOCTYPE svg SYSTEM \"https://example.test/svg.dtd\"><svg />",
        "<svg><style>.x { fill: url( https://example.test/x.svg ) }</style></svg>",
        r#"<svg><style>@\69mport "https://example.test/x.css"</style></svg>"#,
        r#"<svg><style>@&#x69;mport "https://example.test/x.css"</style></svg>"#,
        r#"<svg><style>.x { fill: u\72l(https://example.test/x.svg) }</style></svg>"#,
        r#"<svg><rect style="fill:u\72l(https://example.test/x.svg)" /></svg>"#,
        r#"<svg><style>@im<![CDATA[port "https://example.test/x.css";]]></style></svg>"#,
        r#"<svg><style>@im<!--split-->port "https://example.test/x.css";</style></svg>"#,
        r#"<svg><style>svg{background-image:image-set("https://example.test/x.svg" 1x)}</style></svg>"#,
        r##"<svg><a href="#x" ping="https://example.test/ping">x</a></svg>"##,
    ];
    let bypasses: Vec<&str> = active_fixtures
        .into_iter()
        .filter(|active| svg_inert_violations(active).is_empty())
        .collect();
    assert!(
        bypasses.is_empty(),
        "active SVG fixtures passed:\n{}",
        bypasses.join("\n")
    );
}

#[test]
fn google_cloud_marks_are_local_present_and_documented() {
    let html = page("index.html");
    let provenance = fs::read_to_string(root().parent().unwrap().join("docs/asset-provenance.md"))
        .expect("docs/asset-provenance.md");
    for (product, file) in GCP_ICONS {
        let path = format!("assets/gcp-icons/{file}");
        assert!(
            html.contains(&format!("data-product-mark=\"{product}\" src=\"{path}\"")),
            "landing mark: {product}"
        );
        assert!(root().join(&path).is_file(), "local icon: {path}");
        assert!(provenance.contains(&path), "provenance: {path}");
    }
    let spanner = "assets/engine-logos/google-cloud-spanner.svg";
    assert!(
        html.contains(spanner),
        "Spanner Graph reuses its existing mark"
    );
    assert!(provenance.contains(spanner), "Spanner Graph provenance");
    assert!(
        !html.contains("data-product-mark=\"rag-engine\""),
        "rag-engine must remain text only"
    );
}

/// The text between the first `start` marker and the next `end` marker after it.
fn between<'a>(html: &'a str, start: &str, end: &str) -> &'a str {
    let from = html
        .find(start)
        .unwrap_or_else(|| panic!("missing {start}"));
    let to = html[from..]
        .find(end)
        .unwrap_or_else(|| panic!("missing {end} after {start}"));
    &html[from..from + to]
}

#[test]
fn landing_control_plane_has_one_orchestrator_four_gated_phases_and_an_approval() {
    let html = page("index.html");
    let section = between(&html, "id=\"google-cloud-architecture\"", "</section>");
    assert_eq!(
        section.matches("data-orchestrator=").count(),
        1,
        "one orchestrator region"
    );
    for text in [
        "Gemini Migration Agent",
        "Built with Agent Development Kit (ADK)",
    ] {
        assert!(section.contains(text), "orchestrator: {text}");
    }
    let order: Vec<usize> = ["assess", "plan", "deploy", "optimise"]
        .iter()
        .map(|phase| {
            section
                .find(&format!("data-phase=\"{phase}\""))
                .unwrap_or_else(|| panic!("phase {phase}"))
        })
        .collect();
    assert!(
        order.windows(2).all(|w| w[0] < w[1]),
        "official phase order"
    );
    assert_eq!(
        section.matches("data-phase=").count(),
        4,
        "exactly four phases"
    );
    let checkpoint = section
        .find("data-checkpoint=\"human-approval\"")
        .expect("human approval checkpoint");
    assert!(
        order[1] < checkpoint && checkpoint < order[2],
        "approval sits between Plan and Deploy"
    );
    for phase in ["assess", "plan", "deploy", "optimise"] {
        let card = between(section, &format!("data-phase=\"{phase}\""), "</li>");
        for field in [
            "<dt>Inputs</dt>",
            "<dt>Agent action</dt>",
            "<dt>Jev gate</dt>",
            "<dt>Output</dt>",
            "<dt>State</dt><dd>Reference design</dd>",
        ] {
            assert!(card.contains(field), "{phase}: {field}");
        }
    }
    for text in [
        "It does not move bytes.",
        "Code and SQL count rows and validate schemas.",
        "Validate before cutover.",
        "Roll back if validation fails.",
        "The full agent is a blueprint and is not part of this public demo.",
        "href=\"https://docs.cloud.google.com/architecture/migration-to-gcp-getting-started\"",
        "href=\"https://docs.cloud.google.com/docs/migration\"",
        "href=\"https://cloud.google.com/migration-center/docs/migration-center-overview\"",
    ] {
        assert!(section.contains(text), "control plane: {text}");
    }
}

#[test]
fn landing_lanes_name_source_payload_service_destination_and_evidence() {
    let html = page("index.html");
    let section = between(&html, "id=\"google-cloud-architecture\"", "</section>");
    assert!(
        section.contains("Destination is chosen by workload. Not all data belongs in BigQuery."),
        "workload-based destination warning"
    );
    let lanes = ["documents", "tabular", "relational", "applications"];
    assert_eq!(
        section.matches("data-lane=").count(),
        lanes.len(),
        "four lanes"
    );
    for lane in lanes {
        let row = between(section, &format!("data-lane=\"{lane}\""), "</li>");
        assert!(
            row.contains("Representative reference path"),
            "{lane}: label"
        );
        for field in [
            "<dt>Source</dt>",
            "<dt>What moves</dt>",
            "<dt>Movement or service</dt>",
            "<dt>Destination</dt>",
            "<dt>Validation or evidence</dt>",
        ] {
            assert!(row.contains(field), "{lane}: {field}");
        }
    }
    for url in [
        "https://cloud.google.com/storage-transfer/docs/overview",
        "https://cloud.google.com/database-migration/docs",
        "https://cloud.google.com/datastream/docs/overview",
        "https://cloud.google.com/migrate/virtual-machines/docs/5.0/discover/overview",
        "https://cloud.google.com/migrate/containers/docs",
    ] {
        assert!(
            section.contains(&format!("href=\"{url}\"")),
            "lane source: {url}"
        );
    }
    assert!(
        section.contains("Jev is not stored or hosted in AlloyDB."),
        "AlloyDB calls the hosted Jev API; Jev is not inside it"
    );
    for forbidden in [
        "All data goes to BigQuery",
        "all data goes to BigQuery",
        "universal SAP",
        "MCP copies",
        "Gemini moves",
    ] {
        assert!(
            !html.contains(forbidden),
            "landing must not claim: {forbidden}"
        );
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
