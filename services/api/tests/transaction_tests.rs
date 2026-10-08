use axum::{
    body::Body,
    http::{Method, StatusCode},
    Router,
};
use serde_json::json;
use tower::ServiceExt;

use crate::{
    auth::{login, me, register},
    config::Config,
    db,
    state::AppState,
};

pub async fn setup_tx_test_app() -> Router {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "mysql://uwifin:uwifin@localhost:3306/uwifin".to_string());

    let pool = sqlx::MySqlPool::connect(&database_url)
        .await
        .expect("Database unavailable for transaction tests");

    db::run_migrations(&pool)
        .await
        .expect("Migration failed for transaction tests");

    let config = Config::from_env().expect("Config failed");
    let state = AppState {
        pool: std::sync::Arc::new(pool),
        config: std::sync::Arc::new(config),
    };

    Router::new()
        .route("/auth/register", axum::routing::post(register))
        .route("/auth/login", axum::routing::post(login))
        .route("/auth/me", axum::routing::get(me))
        .route("/transactions", axum::routing::post(crate::transaction::create_transaction))
        .route("/transactions/:id", axum::routing::get(crate::transaction::get_transaction))
        .with_state(state)
}

#[tokio::test]
async fn test_create_transaction_requires_idempotency_key() {
    let app = setup_tx_test_app().await;

    let req = axum::http::Request::builder()
        .uri("/transactions")
        .method(Method::POST)
        .header("content-type", "application/json")
        .header("authorization", "Bearer invalid-token")
        .body(Body::from(json!({
            "wallet_id": "abc",
            "network": "base",
            "asset": "USDC",
            "recipient": "0x1234567890abcdef1234567890abcdef12345678",
            "amount": "10.00",
            "idempotency_key": ""
        }).to_string()))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_transaction_request_validates_amount() {
    let result = crate::transaction::CreateTransactionRequest {
        wallet_id: "wallet-1".to_string(),
        network: "base".to_string(),
        asset: "USDC".to_string(),
        recipient: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
        amount: "0.00".to_string(),
        idempotency_key: "key-1".to_string(),
    }.validate();

    assert!(result.is_err());
}
