use axum::{extract::{Path, State}, http::HeaderMap, Json};
use serde::Serialize;
use crate::{auth::authorize, error::ApiError, state::AppState, wallet_validation::{NETWORK, USDC}, transaction::format_amount};
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct WalletResponse {
    pub id: String, pub address: String, pub network: String, pub wallet_type: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
pub async fn list_wallets(State(state): State<AppState>, headers: HeaderMap) -> Result<Json<Vec<WalletResponse>>, ApiError> {
    let user = authorize(&state, &headers).await?;
    Ok(Json(sqlx::query_as("SELECT id, address, network, wallet_type, created_at FROM wallets WHERE user_id = ? AND verified_at IS NOT NULL AND network = ? ORDER BY created_at DESC")
        .bind(user.id).bind(NETWORK).fetch_all(&state.pool).await?))
}
pub async fn owned_wallet(state: &AppState, user_id: &str, wallet_id: &str) -> Result<WalletResponse, ApiError> {
    sqlx::query_as("SELECT id, address, network, wallet_type, created_at FROM wallets WHERE id = ? AND user_id = ? AND verified_at IS NOT NULL AND network = ?")
        .bind(wallet_id).bind(user_id).bind(NETWORK).fetch_optional(&state.pool).await?
        .ok_or_else(|| ApiError::new("NOT_FOUND", "Wallet not found."))
}
pub async fn get_wallet(State(state): State<AppState>, headers: HeaderMap, Path(id): Path<String>) -> Result<Json<WalletResponse>, ApiError> {
    let user = authorize(&state, &headers).await?;
    Ok(Json(owned_wallet(&state, &user.id, &id).await?))
}
pub async fn get_wallet_balances(State(state): State<AppState>, headers: HeaderMap, Path(id): Path<String>) -> Result<Json<serde_json::Value>, ApiError> {
    let user = authorize(&state, &headers).await?;
    let wallet = owned_wallet(&state, &user.id, &id).await?;
    let balance = crate::alchemy::usdc_balance(&state, &wallet.address).await?;
    Ok(Json(serde_json::json!([{ "asset": "USDC", "network": NETWORK, "balance": format_amount(balance), "contract_address": USDC }])))
}
pub async fn create_wallet(State(state): State<AppState>, headers: HeaderMap) -> Result<Json<serde_json::Value>, ApiError> {
    let user = authorize(&state, &headers).await?;
    state.limits.check(format!("wallet:{}", user.id), 5)?;
    Err(ApiError::new("PROVIDER_NOT_CONFIGURED", "Wallet creation requires the approved signing provider and proof of wallet ownership."))
}
