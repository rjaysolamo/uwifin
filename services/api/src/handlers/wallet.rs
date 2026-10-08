use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::{
    error::ApiError,
    state::AppState,
    wallet::{create_wallet, get_wallet, get_wallet_balances, list_wallets},
};

pub async fn handle_list_wallets(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    list_wallets(State(state), headers).await
}

pub async fn handle_create_wallet(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<crate::wallet::WalletCreateRequest>,
) -> Result<impl IntoResponse, ApiError> {
    create_wallet(State(state), headers, Json(req)).await
}

pub async fn handle_get_wallet(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    wallet_id: String,
) -> Result<impl IntoResponse, ApiError> {
    get_wallet(State(state), headers, axum::extract::Path(wallet_id)).await
}

pub async fn handle_get_wallet_balances(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    wallet_id: String,
) -> Result<impl IntoResponse, ApiError> {
    get_wallet_balances(State(state), headers, axum::extract::Path(wallet_id)).await
}
