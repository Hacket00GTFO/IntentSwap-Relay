mod api;
mod db;
mod models;
mod signature;

use std::{env, net::SocketAddr};

use anyhow::Result;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
use tracing::info;

use api::{router, AppState};

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(
            env::var("RUST_LOG")
                .unwrap_or_else(|_| "intent_relay_backend=debug,tower_http=info".to_string()),
        )
        .init();

    let db = db::init_pool().await?;
    let signature_config = signature::load_signature_config()?;

    let (tx, _rx) = broadcast::channel(1024);

    let state = AppState {
        db,
        signature_config,
        broadcaster: tx,
    };

    let app = router(state).layer(CorsLayer::permissive());

    let port = env::var("PORT")
        .ok()
        .and_then(|raw| raw.parse::<u16>().ok())
        .unwrap_or(8080);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind tcp listener");

    info!("intent relay backend listening on http://{}", addr);
    axum::serve(listener, app)
        .await
        .expect("server terminated with error");

    Ok(())
}
