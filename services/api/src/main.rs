use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlPool;
use std::sync::Arc;
use tower_http::trace::TraceLayer;

pub mod auth;
pub mod config;
pub mod db;
pub mod error;
pub mod state;
pub mod wallet;

use crate::{
    auth::{login, logout, me, register},
    config::Config,
    state::AppState,
    wallet::{create_wallet, get_wallet, get_wallet_balances, list_wallets, WalletCreateRequest},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadyResponse {
    pub status: String,
    pub database: String,
}

pub async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, Json(HealthResponse { status: "ok".to_string() }))
}

pub async fn ready_handler(State(state): State<AppState>) -> impl IntoResponse {
    match sqlx::query("SELECT 1").fetch_one(state.pool.as_ref()).await {
        Ok(_) => (
            StatusCode::OK,
            Json(ReadyResponse {
                status: "ok".to_string(),
                database: "connected".to_string(),
            }),
        ),
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ReadyResponse {
                status: "not_ready".to_string(),
                database: "disconnected".to_string(),
            }),
        ),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let config = Config::from_env()?;
    let pool = MySqlPool::connect(&config.database_url).await?;
    db::run_migrations(&pool).await?;

    let app_state = AppState {
        pool: Arc::new(pool),
        config: Arc::new(config),
    };

    let app = Router::new()
        .route("/health", axum::routing::get(health_handler))
        .route("/ready", axum::routing::get(ready_handler))
        .route("/auth/register", axum::routing::post(register))
        .route("/auth/login", axum::routing::post(login))
        .route("/auth/logout", axum::routing::post(logout))
        .route("/auth/me", axum::routing::get(me))
        .route("/wallets", axum::routing::get(list_wallets).post(create_wallet))
        .route("/wallets/:id", axum::routing::get(get_wallet))
        .route("/wallets/:id/balances", axum::routing::get(get_wallet_balances))
        .with_state(app_state)
        .layer(
            tower_http::cors::CorsLayer::permissive()
                .allow_origin(tower_http::cors::Any)
                .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
                .allow_headers([axum::http::header::AUTHORIZATION, axum::http::header::CONTENT_TYPE])
        )
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    tracing::info!("UwiFin API listening on port 8080");
    axum::serve(listener, app).await?;

    Ok(())
}
