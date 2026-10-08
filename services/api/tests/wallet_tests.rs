use std::sync::Arc;

use axum::{
    body::Body,
    http::{Method, StatusCode},
    Router,
};
use tower::ServiceExt;

use crate::{
    auth::{login, logout, me, register},
    config::Config,
    db,
    state::AppState,
    wallet::WalletCreateRequest,
};

use serde_json::json;

pub async fn setup_app_for_wallet_tests() -> Router {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "mysql://uwifin:uwifin@localhost:3306/uwifin".to_string());

    let pool = sqlx::MySqlPool::connect(&database_url)
        .await
        .expect("Database not available for wallet tests");

    db::run_migrations(&pool)
        .await
        .expect("Failed to run database migrations");

    let config = Config::from_env().expect("Failed to load config");
    let state = AppState {
        pool: Arc::new(pool),
        config: Arc::new(config),
    };

    Router::new()
        .route("/health", axum::routing::get(crate::health_handler))
        .route("/auth/register", axum::routing::post(register))
        .route("/auth/login", axum::routing::post(login))
        .route("/auth/me", axum::routing::get(me))
        .route("/wallets", axum::routing::get(crate::wallet::list_wallets).post(crate::wallet::create_wallet))
        .route("/wallets/:id", axum::routing::get(crate::wallet::get_wallet))
        .route("/wallets/:id/balances", axum::routing::get(crate::wallet::get_wallet_balances))
        .with_state(state)
}

#[tokio::test]
async fn test_wallet_address_validation_accepts_valid_address() {
    let result = crate::wallet_validation::validate_wallet_address("0x1234567890abcdef1234567890abcdef12345678");
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_wallet_address_validation_rejects_invalid_address() {
    let result = crate::wallet_validation::validate_wallet_address("not-a-valid-address");
    assert!(result.is_err());
}

#[tokio::test]
async fn test_wallet_network_validation_accepts_supported_network() {
    let result = crate::wallet_validation::validate_network("base");
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_wallet_type_validation_accepts_supported_type() {
    let result = crate::wallet_validation::validate_wallet_type("smart_account");
    assert!(result.is_ok());
}
