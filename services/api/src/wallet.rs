use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::hash_session,
    error::ApiError,
    state::AppState,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletCreateRequest {
    pub address: String,
    pub network: String,
    pub wallet_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletResponse {
    pub id: String,
    pub user_id: String,
    pub address: String,
    pub network: String,
    pub wallet_type: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BalanceResponse {
    pub asset: String,
    pub balance: String,
    pub network: String,
    pub contract_address: Option<String>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct WalletRecord {
    pub id: String,
    pub user_id: String,
    pub address: String,
    pub network: String,
    pub wallet_type: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub async fn list_wallets(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    let user = authorize_user(&state, &headers).await?;

    let wallets = sqlx::query_as::<_, WalletRecord>(
        "SELECT * FROM wallets WHERE user_id = ? ORDER BY created_at DESC"
    )
    .bind(&user.id)
    .fetch_all(state.pool.as_ref())
    .await
    .map_err(|_| ApiError::new("INTERNAL_ERROR", "Failed to load wallets"))?;

    let response: Vec<WalletResponse> = wallets
        .into_iter()
        .map(|wallet| WalletResponse {
            id: wallet.id,
            user_id: wallet.user_id,
            address: wallet.address,
            network: wallet.network,
            wallet_type: wallet.wallet_type,
            created_at: wallet.created_at.to_rfc3339(),
            updated_at: wallet.updated_at.to_rfc3339(),
        })
        .collect();

    Ok((StatusCode::OK, Json(response)))
}

pub async fn create_wallet(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<WalletCreateRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let user = authorize_user(&state, &headers).await?;

    if req.address.trim().is_empty() {
        return Err(ApiError::new("INVALID_REQUEST", "Wallet address is required"));
    }

    if req.network.trim().is_empty() {
        return Err(ApiError::new("INVALID_REQUEST", "Network is required"));
    }

    let wallet_type = req.wallet_type.unwrap_or_else(|| "smart_account".to_string());
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now();

    sqlx::query!(
        "INSERT INTO wallets (id, user_id, address, network, wallet_type, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
        id,
        user.id,
        req.address,
        req.network,
        wallet_type,
        now,
        now,
    )
    .execute(state.pool.as_ref())
    .await
    .map_err(|_| ApiError::new("INTERNAL_ERROR", "Failed to create wallet"))?;

    Ok((StatusCode::CREATED, Json(WalletResponse {
        id,
        user_id: user.id,
        address: req.address,
        network: req.network,
        wallet_type,
        created_at: now.to_rfc3339(),
        updated_at: now.to_rfc3339(),
    })))
}

pub async fn get_wallet(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Path(wallet_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let user = authorize_user(&state, &headers).await?;

    let wallet = sqlx::query_as::<_, WalletRecord>(
        "SELECT * FROM wallets WHERE id = ? AND user_id = ?"
    )
    .bind(&wallet_id)
    .bind(&user.id)
    .fetch_optional(state.pool.as_ref())
    .await
    .map_err(|_| ApiError::new("INTERNAL_ERROR", "Failed to load wallet"))?
    .ok_or_else(|| ApiError::new("NOT_FOUND", "Wallet not found"))?;

    Ok((StatusCode::OK, Json(WalletResponse {
        id: wallet.id,
        user_id: wallet.user_id,
        address: wallet.address,
        network: wallet.network,
        wallet_type: wallet.wallet_type,
        created_at: wallet.created_at.to_rfc3339(),
        updated_at: wallet.updated_at.to_rfc3339(),
    })))
}

pub async fn get_wallet_balances(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Path(wallet_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let user = authorize_user(&state, &headers).await?;

    let wallet_exists = sqlx::query_scalar!(
        "SELECT COUNT(*) as count FROM wallets WHERE id = ? AND user_id = ?",
        wallet_id,
        user.id,
    )
    .fetch_one(state.pool.as_ref())
    .await
    .map_err(|_| ApiError::new("INTERNAL_ERROR", "Failed to check wallet ownership"))?
    .count
    .unwrap_or(0);

    if wallet_exists == 0 {
        return Err(ApiError::new("NOT_FOUND", "Wallet not found"));
    }

    let supported_assets = sqlx::query!(
        "SELECT symbol, contract_address, network FROM assets WHERE is_active = TRUE ORDER BY symbol ASC"
    )
    .fetch_all(state.pool.as_ref())
    .await
    .map_err(|_| ApiError::new("INTERNAL_ERROR", "Failed to load asset metadata"))?;

    let balances: Vec<BalanceResponse> = supported_assets
        .into_iter()
        .map(|asset| BalanceResponse {
            asset: asset.symbol,
            balance: "0.000000".to_string(),
            network: asset.network,
            contract_address: Some(asset.contract_address),
        })
        .collect();

    Ok((StatusCode::OK, Json(balances)))
}

async fn authorize_user(state: &AppState, headers: &axum::http::HeaderMap) -> Result<UserSummary, ApiError> {
    let auth_header = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .filter(|v| v.starts_with("Bearer "))
        .map(|v| v.trim_start_matches("Bearer ").to_string())
        .ok_or_else(|| ApiError::new("UNAUTHORIZED", "Missing bearer token"))?;

    let session_hash = hash_session(&auth_header)
        .map_err(|_| ApiError::new("INTERNAL_ERROR", "Failed to hash session"))?;

    let session = sqlx::query!(
        "SELECT * FROM sessions WHERE session_hash = ? AND revoked_at IS NULL AND expires_at > NOW()",
        session_hash,
    )
    .fetch_optional(state.pool.as_ref())
    .await
    .map_err(|_| ApiError::new("INTERNAL_ERROR", "Failed to validate session"))?
    .ok_or_else(|| ApiError::new("UNAUTHORIZED", "Invalid or expired session"))?;

    let user = sqlx::query_as::<_, UserSummary>(
        "SELECT id, email FROM users WHERE id = ?"
    )
    .bind(&session.user_id)
    .fetch_optional(state.pool.as_ref())
    .await
    .map_err(|_| ApiError::new("INTERNAL_ERROR", "Failed to load user"))?
    .ok_or_else(|| ApiError::new("UNAUTHORIZED", "User not found"))?;

    Ok(user)
}

#[derive(Debug, sqlx::FromRow)]
pub struct UserSummary {
    pub id: String,
    pub email: String,
}
