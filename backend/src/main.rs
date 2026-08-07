mod auth;
mod catalog;
mod chat;
mod config;
mod store;
mod system;
mod user;

use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use axum::{routing::get, Router};
use axum::routing::post;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use crate::auth::api::actions::auth_actions_controller::{login, register};
use crate::config::config::Config;
use crate::config::state::AppState;
use crate::system::api::actions::read_health_action::read_health_action;
use crate::store::api::actions::store_actions_controller::{
    create_store,
    delete_store,
    get_store,
    list_stores,
    update_store,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,tower_http=debug".to_string()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();
    let config = Config::from_env()?;

    info!("Connecting to database: {}", config.db_url);

    let db_options = SqliteConnectOptions::from_str(&config.db_url)?
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
        .foreign_keys(true)
        .create_if_missing(true);

    let db_pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(db_options)
        .await?;

    sqlx::query("PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;")
        .execute(&db_pool)
        .await?;

    info!("✅ Database connected.");

    sqlx::migrate!("./db/migrations")
        .run(&db_pool)
        .await?;

    info!("✅ Migrations applied successfully.");

    let cancel_token = CancellationToken::new();

    let state = Arc::new(AppState::new(db_pool, config.clone()));

    let app = Router::new()
        .route("/api/health", get(read_health_action))
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(login))
        .route(
            "/api/stores",
            post(create_store).get(list_stores),
        )
        .route(
            "/api/stores/{store_id}",
            get(get_store)
            .put(update_store)
            .patch(update_store)
            .delete(delete_store),
        )
        .with_state(state)
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let addr: SocketAddr = format!("{}:{}", config.http_server_host, config.http_server_port)
        .parse()
        .expect("Failed to parse socket address");

    let listener = TcpListener::bind(addr).await?;
    info!("🚀 Server listening on {}", listener.local_addr()?);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(cancel_token))
        .await?;

    info!("👋 Server stopped gracefully.");
    Ok(())
}

async fn shutdown_signal(token: CancellationToken) {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("🛑 Shutdown signal received, notifying background workers...");
    token.cancel();
}