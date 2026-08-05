mod config;
mod system;

use std::sync::Arc;
use std::net::SocketAddr;
use axum::{routing::get, Router};
use sqlx::sqlite::SqlitePoolOptions;
use tokio_util::sync::CancellationToken;
use tokio::signal;
use crate::config::state::AppState;
use crate::config::config::Config;
use crate::system::api::actions::read_health_action::read_health_action;


#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let config = Config::from_env().expect("Failed to load configuration from .env");

    println!("Connecting to database: {}", config.db_url);

    let db_pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&config.db_url)
        .await
        .expect("Failed to connect to SQLite");

    println!("✅ Database connected.");

    sqlx::migrate!("./db/migrations")
        .run(&db_pool)
        .await
        .expect("Failed to run migrations");

    println!("✅ Migrations applied successfully.");

    let cancel_token = CancellationToken::new();

    let state = Arc::new(AppState::new(db_pool, config.clone()));

    let app = Router::new()
        .route("/api/health", get(read_health_action))
        .with_state(state);

    let host = config.http_server_host.clone();
    let port = config.http_server_port;
    let addr: SocketAddr = format!("{}:{}", host, port).parse().unwrap();

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("🚀 Server listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(cancel_token))
        .await
        .unwrap();
}

async fn shutdown_signal(token: CancellationToken) {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
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

    println!("🛑 Shutdown signal received, notifying worker...");
    token.cancel();

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
}
