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

#[test]
fn homepage_leads_with_migration_as_an_ai_accelerator() {
    let html = page("index.html");
    assert!(html.contains("Turn data migration into an <em>AI accelerator.</em>"));
    assert!(html.contains("MCP reaches fragmented source systems"));
    assert!(html.contains("graphs connect the facts"));
    assert!(html.contains("decision models bound routine decisions"));
    assert!(html.contains("reasoning on ambiguity, synthesis, planning and novel work"));
    for stage in ["Migration", "Modernisation", "Activation", "Acceleration"] {
        assert!(
            html.contains(&format!(">{stage}<")),
            "missing stage: {stage}"
        );
    }
}

#[test]
fn homepage_keeps_the_approved_value_story_contracts() {
    let html = page("index.html");
    assert!(html.contains("Spend reasoning where it creates value."));
    assert!(html.contains("Illustrative operating model—not a benchmark."));
    for outcome in ["Answer", "Clarify", "Reject", "Review", "No data"] {
        assert!(
            html.contains(&format!(">{outcome}<")),
            "missing outcome: {outcome}"
        );
    }
    for stage in [
        "MCP entry",
        "Evidence",
        "Relationship",
        "Decision",
        "Reasoning",
        "Assurance",
    ] {
        assert!(html.contains(stage), "missing agent-spine stage: {stage}");
    }
    for measure in [
        "useful work per reasoning token",
        "decision accuracy and calibration",
        "unsupported-action rate",
        "escalation precision",
        "human-review time",
        "route latency",
        "cost of incorrect automation",
    ] {
        assert!(html.contains(measure), "missing future measure: {measure}");
    }
}

#[test]
fn homepage_exposes_one_accessible_eight_beat_rail_and_exact_model() {
    let html = page("index.html");
    assert_eq!(
        html.matches("data-beat-target=").count(),
        8,
        "single 1–8 rail"
    );
    assert_eq!(
        html.matches("class=\"vs-progress\"").count(),
        1,
        "one progress rail"
    );
    assert!(html.contains("id=\"previousBeat\"") && html.contains("id=\"nextBeat\""));
    assert!(
        html.contains("id=\"expandBeats\"")
            && html.contains("id=\"beatStatus\" aria-live=\"polite\"")
    );
    assert!(
        html.contains("event.key === \"ArrowRight\"")
            && html.contains("event.key === \"ArrowLeft\"")
    );
    assert!(
        html.contains("id=\"motionToggle\"")
            && html.contains("Preview motion")
            && html.contains("Respect reduced motion")
    );
    assert!(html.contains("residual = supported - allocated.reduce"));
}

#[test]
fn homepage_removes_retired_copy_footer_and_remote_runtime_assets() {
    let html = page("index.html");
    let lower = html.to_lowercase();
    for forbidden in [
        "the palindrome",
        "thepalindrome.org",
        "<footer",
        "cost savings",
        "save money",
        "cheaper by",
        ">escalate<",
        "outcome:\"escalate\"",
    ] {
        assert!(
            !lower.contains(forbidden),
            "retired or unsupported copy: {forbidden}"
        );
    }
    for tag in tags(&html, "script").into_iter().chain(tags(&html, "link")) {
        assert!(
            !tag.contains("src=\"http") && !tag.contains("href=\"http"),
            "remote runtime asset: {tag}"
        );
    }
}

#[test]
fn homepage_preserves_demo_use_cases_and_complete_technical_guide() {
    let html = page("index.html");
    for contract in [
        "id=\"example\"",
        "href=\"/dcceew/\"",
        "href=\"/parkinsons/\"",
        "href=\"/little-bunnies/\"",
        "id=\"spanner-graph-spotlight\"",
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
        assert!(
            html.contains(contract),
            "missing preserved contract: {contract}"
        );
    }
}

#[test]
fn review_copy_places_products_and_qualifies_claims() {
    let html = page("index.html");
    assert!(html.contains(">The case in one minute<"));
    assert_eq!(html.matches("Traceable for supported flows").count(), 2);
    assert!(html.contains(
        "Every supported outcome returns the relevant path and available sources behind it."
    ));
    assert!(html.contains("Index candidate"));
    assert!(html.contains("Analytical data"));
    assert!(html.contains("Relationship data"));
    assert!(html.contains("Archive · redact · review"));
    let scene = html.split("class=\"migration-scene\"").nth(1).unwrap();
    let modernisation = scene.split("migration-stage--sift").nth(1).unwrap();
    let before_landing = modernisation
        .split("migration-stage--landing")
        .next()
        .unwrap();
    assert!(!before_landing.contains("BigQuery") && !before_landing.contains("Spanner Graph"));
    assert!(html.contains("Little Bunnies: two hops in this sample"));
    assert!(html.contains("In this sample small-world graph"));
}

#[test]
fn story_accessibility_contracts_do_not_hijack_controls() {
    let html = page("index.html");
    assert!(html.contains("if (event.target !== story) return;"));
    assert!(!html.contains("assertWorkloadModel"));
    assert!(!html.contains("console.assert"));
    for id in ["totalUnits", "boundedShare", "evidenceShare"] {
        assert!(
            html.contains(&format!("id=\"{id}\" type=\"range\""))
                && html.contains(&format!("aria-describedby=\"{id}Value")),
            "descriptive range contract: {id}"
        );
    }
    assert!(html.contains("class=\"vs-story-controls\" hidden"));
}

#[test]
fn workload_allocation_is_exact_for_all_6137_reachable_inputs() {
    fn allocate(total: i32, bounded_share: i32, evidence_share: i32) -> [i32; 5] {
        let bounded = (total * bounded_share + 50) / 100;
        let supported = (bounded * evidence_share + 50) / 100;
        let weights = [7, 1, 1, 1];
        let mut allocated = weights.map(|weight| supported * weight / 10);
        let mut residual = supported - allocated.iter().sum::<i32>();
        let mut order = [0, 1, 2, 3];
        order.sort_by_key(|&index| (-(supported * weights[index] % 10), index));
        for index in order {
            if residual == 0 {
                break;
            }
            allocated[index] += 1;
            residual -= 1;
        }
        [
            allocated[0],
            allocated[1],
            allocated[2],
            total - supported,
            allocated[3],
        ]
    }

    let mut combinations = 0;
    for total in (40..=200).step_by(10) {
        for bounded in (10..=100).step_by(5) {
            for evidence in (10..=100).step_by(5) {
                let values = allocate(total, bounded, evidence);
                assert!(values.iter().all(|value| *value >= 0));
                assert_eq!(values.iter().sum::<i32>(), total);
                combinations += 1;
            }
        }
    }
    assert_eq!(combinations, 6_137);
}

#[test]
fn migration_and_value_story_css_are_scoped_and_consolidated() {
    let css = page("guide.css");
    assert!(css.contains(".migration-story .migration-scene"));
    for selector in [
        ".migration-scene{",
        ".migration-stage{",
        ".source-pool{",
        ".sift-fan{",
        ".activate-rail{",
        ".google-plane{",
    ] {
        assert!(
            !css.contains(&format!("\n{selector}")),
            "unscoped selector: {selector}"
        );
    }
    assert_eq!(
        css.matches(".vs-progress button{").count(),
        1,
        "one progress rule"
    );
    assert_eq!(
        css.matches(".vs-stage[data-beat=\"5\"] .vs-queue").count(),
        2,
        "one base and one mobile operating-mark queue rule"
    );
    assert!(css.contains(".vs-story.vs-enhanced .vs-story-controls{display:flex"));
    assert!(css.contains("min-width:44px") && css.contains("min-height:44px"));
}

#[test]
fn value_story_css_has_type_floors_reduced_motion_and_320px_guards() {
    let css = page("guide.css");
    for contract in [
        ".value-story",
        ".migration-story",
        "min-block-size:clamp(90svh,96vh,100svh)",
        ".vs-story.reduce-motion",
        ".vs-story.force-motion",
        "@media (max-width:320px)",
        "overflow-wrap:anywhere",
        "font-size:max(16px",
    ] {
        assert!(
            css.contains(contract),
            "missing responsive contract: {contract}"
        );
    }
}
