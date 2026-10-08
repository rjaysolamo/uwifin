use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlPool;
use std::sync::Arc;
use tracing_subscriber;

mod config;
mod db;
mod error;
mod handlers;
mod state;

use config::Config;
use state::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadyResponse {
    status: String,
    database: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let config = Config::from_env()?;
    tracing::info!("Config loaded: {:?}", config);

    // Connect to database
    let pool = MySqlPool::connect(&config.database_url).await?;
    tracing::info!("Database connected");

    // Run migrations
    db::run_migrations(&pool).await?;
    tracing::info!("Migrations completed");

    let app_state = AppState {
        pool: Arc::new(pool),
        config: Arc::new(config),
    };

    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/ready", get(ready_handler))
        .route("/auth/register", axum::routing::post(handlers::auth::register))
        .route("/auth/login", axum::routing::post(handlers::auth::login))
        .route("/auth/logout", axum::routing::post(handlers::auth::logout))
        .route("/auth/me", axum::routing::get(handlers::auth::me))
        .with_state(app_state)
        .layer(
            tower_http::cors::CorsLayer::permissive()
                .allow_origin(tower_http::cors::Any),
        )
        .layer(
            tower_http::trace::TraceLayer::new_for_http()
                .make_span_with(tower_http::trace::DefaultMakeSpan::new()
                    .level(tracing::Level::INFO)),
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    tracing::info!("Server listening on http://0.0.0.0:8080");

    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, Json(HealthResponse {
        status: "ok".to_string(),
    }))
}

async fn ready_handler(State(state): State<AppState>) -> impl IntoResponse {
    // Check database connection
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
