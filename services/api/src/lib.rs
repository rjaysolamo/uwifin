pub mod alchemy;
pub mod auth;
pub mod config;
pub mod db;
pub mod error;
pub mod payments;
pub mod security;
pub mod state;
pub mod transaction;
pub mod wallet;
pub mod wallet_validation;
use axum::{extract::{DefaultBodyLimit, State}, http::StatusCode, middleware, routing::{get, post}, Json, Router};
use serde_json::json;
use state::AppState;
pub fn app(state: AppState) -> Router {
    let api = Router::new()
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/auth/logout", post(auth::logout))
        .route("/auth/me", get(auth::me))
        .route("/users/me", get(auth::me).patch(auth::update_profile))
        .route("/wallets", get(wallet::list_wallets).post(wallet::create_wallet))
        .route("/wallets/:id", get(wallet::get_wallet))
        .route("/wallets/:id/balances", get(wallet::get_wallet_balances))
        .route("/transactions", get(transaction::list_transactions).post(transaction::create_transaction))
        .route("/transactions/:id", get(transaction::get_transaction))
        .route("/payments", get(payments::list_payments).post(payments::create_payment))
        .route("/payments/stripe/webhook", post(payments::webhook));
    Router::new()
        .route("/health", get(|| async { Json(json!({ "status": "ok" })) }))
        .route("/ready", get(ready))
        .nest("/api/v1", api)
        .fallback(|| async { error::ApiError::new("NOT_FOUND", "Endpoint not found.") })
        .layer(DefaultBodyLimit::max(65_536))
        .layer(middleware::from_fn_with_state(state.clone(), security::request_controls))
        .with_state(state)
}
async fn ready(State(state): State<AppState>) -> (StatusCode, Json<serde_json::Value>) {
    let database = sqlx::query("SELECT 1").execute(&state.pool).await.is_ok();
    let providers = state.config.alchemy_rpc_url.is_some() && state.config.stripe_webhook_secret.is_some();
    // Full financial readiness requires wallet signing, sponsorship, and payment creation adapters.
    (StatusCode::SERVICE_UNAVAILABLE, Json(json!({ "status": "not_ready", "database": database, "provider_configuration": providers, "financial_operations": false })))
}
