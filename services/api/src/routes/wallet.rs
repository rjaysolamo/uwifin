use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

use crate::{
    error::ApiError,
    state::AppState,
    wallet::{create_wallet, get_wallet, get_wallet_balances, list_wallets, WalletCreateRequest},
};

pub fn wallet_routes() -> Router<AppState> {
    Router::new()
        .route("/wallets", get(list_wallets).post(create_wallet))
        .route("/wallets/:id", get(get_wallet))
        .route("/wallets/:id/balances", get(get_wallet_balances))
}
