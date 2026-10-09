use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use serve::{router, App};

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key)
        .ok()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| default.to_string())
}

#[tokio::main]
async fn main() {
    let app = Arc::new(
        App::load(
            &PathBuf::from(env_or("CATALOGUE_PATH", "catalogue.json")),
            env_or("MIN_CONFIDENCE", "0.8").parse().unwrap_or(0.8),
            env_or("MAX_STATE_CHARS", "4000").parse().unwrap_or(4000),
            env_or("RATE_LIMIT_PER_MINUTE", "120")
                .parse()
                .unwrap_or(120),
        )
        .unwrap_or_else(|error| {
            eprintln!("{error}");
            std::process::exit(1);
        }),
    );
    app.warm();
    let port: u16 = env_or("PORT", "8080").parse().unwrap_or(8080);
    let listener = tokio::net::TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], port)))
        .await
        .expect("bind");
    axum::serve(
        listener,
        router(app, &PathBuf::from(env_or("STATIC_DIR", "static"))),
    )
    .await
    .expect("serve");
}
