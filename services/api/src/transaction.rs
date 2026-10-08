use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{
    auth::hash_session,
    error::ApiError,
    state::AppState,
    transaction::{CreateTransactionRequest, TransactionStatus},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionCreateResponse {
    pub id: String,
    pub status: String,
}

pub async fn create_transaction(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<CreateTransactionRequest>,
) -> Result<impl IntoResponse, ApiError> {
    req.validate().map_err(|err| ApiError::new("INVALID_REQUEST", err))?;

    let user = authorize_user(&state, &headers).await?;

    if req.network.trim().is_empty() {
        return Err(ApiError::new("INVALID_REQUEST", "Network is required"));
    }

    let supported_networks = ["base", "ethereum", "polygon"];
    if !supported_networks.contains(&req.network.trim().to_ascii_lowercase().as_str()) {
        return Err(ApiError::new("UNSUPPORTED_NETWORK", "Network is not supported in MVP"));
    }

    let recipient_validation = crate::wallet_validation::validate_wallet_address(&req.recipient)?;
    if !recipient_validation.is_valid {
        return Err(ApiError::new("INVALID_ADDRESS", "Recipient address is invalid"));
    }

    let wallet = sqlx::query!(
        "SELECT * FROM wallets WHERE id = ? AND user_id = ?",
        req.wallet_id,
        user.id,
    )
    .fetch_optional(state.pool.as_ref())
    .await
    .map_err(|_| ApiError::new("INTERNAL_ERROR", "Failed to fetch wallet"))?
    .ok_or_else(|| ApiError::new("FORBIDDEN", "Wallet does not belong to user"))?;

    let asset = sqlx::query!(
        "SELECT * FROM assets WHERE symbol = ? AND network = ? AND is_active = TRUE",
        req.asset,
        req.network,
    )
    .fetch_optional(state.pool.as_ref())
    .await
    .map_err(|_| ApiError::new("INTERNAL_ERROR", "Failed to load asset"))?
    .ok_or_else(|| ApiError::new("UNSUPPORTED_ASSET", "Asset is unsupported on this network"))?;

    let amount_atomic = convert_decimal_to_atomic(&req.amount, asset.decimals as u32)
        .map_err(|_| ApiError::new("INVALID_REQUEST", "Amount must be a valid decimal value"))?;

    let idempotency_check = sqlx::query!(
        "SELECT id FROM transactions WHERE user_id = ? AND wallet_id = ? AND recipient = ? AND amount_atomic = ? AND status IN ('CREATED','VALIDATING','READY','SUBMITTED','PENDING') LIMIT 1",
        user.id,
        wallet.id,
        req.recipient,
        amount_atomic,
    )
    .fetch_optional(state.pool.as_ref())
    .await
    .map_err(|_| ApiError::new("INTERNAL_ERROR", "Failed to check idempotency"))?;

    if let Some(existing) = idempotency_check {
        return Ok((StatusCode::OK, Json(TransactionCreateResponse {
            id: existing.id,
            status: "created".to_string(),
        })));
    }

    let tx_id = crate::transaction::new_transaction_id();
    let now = chrono::Utc::now();

    sqlx::query!(
        "INSERT INTO transactions (id, user_id, wallet_id, network, sender, recipient, asset_id, amount_atomic, status, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        tx_id,
        user.id,
        wallet.id,
        req.network,
        wallet.address,
        req.recipient,
        asset.id,
        amount_atomic,
        TransactionStatus::CREATED.as_str(),
        now,
        now,
    )
    .execute(state.pool.as_ref())
    .await
    .map_err(|_| ApiError::new("INTERNAL_ERROR", "Failed to create transaction"))?;

    Ok((StatusCode::CREATED, Json(TransactionCreateResponse {
        id: tx_id,
        status: "created".to_string(),
    })))
}

pub async fn get_transaction(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Path(tx_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let user = authorize_user(&state, &headers).await?;

    let record = sqlx::query!(
        "SELECT * FROM transactions WHERE id = ? AND user_id = ?",
        tx_id,
        user.id,
    )
    .fetch_optional(state.pool.as_ref())
    .await
    .map_err(|_| ApiError::new("INTERNAL_ERROR", "Failed to fetch transaction"))?
    .ok_or_else(|| ApiError::new("NOT_FOUND", "Transaction not found"))?;

    Ok((StatusCode::OK, Json(serde_json::json!({
        "id": record.id,
        "status": record.status,
        "tx_hash": record.tx_hash,
        "sender": record.sender,
        "recipient": record.recipient,
        "asset": "USDC",
        "amount": format_amount(record.amount_atomic, 6),
        "created_at": record.created_at,
        "confirmed_at": record.confirmed_at,
    }))))
}

fn convert_decimal_to_atomic(amount: &str, decimals: u32) -> Result<i64, String> {
    let parsed = amount.parse::<f64>().map_err(|_| "invalid amount".to_string())?;
    let factor = 10_f64.powi(decimals as i32);
    let atomic = parsed * factor;

    if atomic.fract() != 0.0 {
        return Err("precision loss detected".to_string());
    }

    Ok(atomic as i64)
}

fn format_amount(amount_atomic: i64, decimals: i32) -> String {
    let divisor = 10_i64.pow(decimals as u32);
    let whole = amount_atomic / divisor;
    let remainder = (amount_atomic % divisor).abs();
    format!("{}.{}", whole, format!("{:0>width$}", remainder, width = decimals as usize))
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

    let user = sqlx::query_as::<_, UserSummary>("SELECT id, email FROM users WHERE id = ?")
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
