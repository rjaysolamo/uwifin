use axum::{extract::{Path, Query, State}, http::{HeaderMap, StatusCode}, Json};
use serde::{Deserialize, Serialize};
use crate::{auth::authorize, error::ApiError, security::{audit, hash}, state::AppState, wallet::owned_wallet, wallet_validation::{validate_network, validate_wallet_address, NETWORK}};
pub fn parse_amount(value: &str) -> Result<i64, ApiError> {
    let invalid = || ApiError::new("INVALID_REQUEST", "Enter a positive amount with at most six decimal places.");
    let (whole, fraction) = value.split_once('.').unwrap_or((value, ""));
    if whole.is_empty() || whole.len() > 18 || !whole.bytes().all(|c| c.is_ascii_digit()) || fraction.len() > 6 || !fraction.bytes().all(|c| c.is_ascii_digit()) || (value.contains('.') && fraction.is_empty()) { return Err(invalid()); }
    let whole = whole.parse::<i64>().map_err(|_| invalid())?;
    let fraction = format!("{fraction:0<6}").parse::<i64>().map_err(|_| invalid())?;
    whole.checked_mul(1_000_000).and_then(|amount| amount.checked_add(fraction)).filter(|amount| *amount > 0).ok_or_else(invalid)
}
pub fn format_amount(value: u128) -> String { format!("{}.{:06}", value / 1_000_000, value % 1_000_000) }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionStatus { Created, Validating, Ready, Submitted, Pending, Confirmed, Failed }
impl TransactionStatus {
    pub fn can_transition_to(self, next: Self) -> bool {
        use TransactionStatus::*;
        matches!((self, next), (Created, Validating | Failed) | (Validating, Ready | Failed) | (Ready, Submitted | Failed) | (Submitted, Pending | Failed) | (Pending, Confirmed | Failed))
    }
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CreateTransactionRequest { pub wallet_id: String, pub network: String, pub asset: String, pub recipient: String, pub amount: String }
#[derive(Serialize, sqlx::FromRow)]
struct TransactionRecord {
    id: String, status: String, tx_hash: Option<String>, sender: String, recipient: String,
    amount_atomic: i64, network: String, created_at: chrono::DateTime<chrono::Utc>,
    confirmed_at: Option<chrono::DateTime<chrono::Utc>>, request_fingerprint: Option<String>,
}
fn record_json(record: &TransactionRecord) -> serde_json::Value {
    serde_json::json!({
        "id": record.id, "status": record.status.to_ascii_lowercase(), "tx_hash": record.tx_hash,
        "sender": record.sender, "recipient": record.recipient, "address": record.recipient,
        "asset": "USDC", "amount": format_amount(record.amount_atomic as u128), "amount_atomic": record.amount_atomic.to_string(),
        "network": record.network, "created_at": record.created_at, "confirmed_at": record.confirmed_at,
        "kind": "sent", "name": "Wallet transfer"
    })
}
pub async fn create_transaction(State(state): State<AppState>, headers: HeaderMap, Json(mut req): Json<CreateTransactionRequest>) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let user = authorize(&state, &headers).await?;
    state.limits.check(format!("transfer:{}", user.id), 10)?;
    let key = headers.get("idempotency-key").and_then(|v| v.to_str().ok()).filter(|key| (8..=128).contains(&key.len()) && key.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_'))
        .ok_or_else(|| ApiError::new("INVALID_REQUEST", "A valid Idempotency-Key header is required."))?;
    validate_network(&req.network)?;
    if req.asset != "USDC" { return Err(ApiError::new("UNSUPPORTED_ASSET", "Only USDC is supported.")); }
    req.recipient = validate_wallet_address(&req.recipient)?;
    let amount = parse_amount(&req.amount)?;
    req.amount = format_amount(amount as u128);
    let fingerprint = hash(&serde_json::to_string(&req).map_err(|_| ApiError::unavailable())?);
    let existing = sqlx::query_as::<_, TransactionRecord>("SELECT * FROM transactions WHERE user_id = ? AND idempotency_key = ?")
        .bind(&user.id).bind(key).fetch_optional(&state.pool).await?;
    if let Some(record) = existing {
        if record.request_fingerprint.as_deref() != Some(&fingerprint) { return Err(ApiError::new("CONFLICT", "This idempotency key was already used for different transfer details.")); }
        return Ok((StatusCode::OK, Json(record_json(&record))));
    }
    let wallet = owned_wallet(&state, &user.id, &req.wallet_id).await?;
    if wallet.network != req.network { return Err(ApiError::new("UNSUPPORTED_NETWORK", "The wallet and transaction networks must match.")); }
    if wallet.address.eq_ignore_ascii_case(&req.recipient) { return Err(ApiError::new("INVALID_REQUEST", "Choose a recipient other than your own wallet.")); }
    let asset_id: String = sqlx::query_scalar("SELECT id FROM assets WHERE symbol = 'USDC' AND network = ? AND contract_address = ? AND decimals = 6 AND is_active = TRUE")
        .bind(NETWORK).bind(crate::wallet_validation::USDC).fetch_optional(&state.pool).await?.ok_or_else(|| ApiError::new("UNSUPPORTED_ASSET", "This asset is currently unavailable."))?;
    if crate::alchemy::usdc_balance(&state, &wallet.address).await? < amount as u128 { return Err(ApiError::new("INSUFFICIENT_BALANCE", "Insufficient USDC balance.")); }
    let mut tx = state.pool.begin().await?;
    let id = uuid::Uuid::new_v4().to_string();
    // UNIQUE(user_id, idempotency_key) makes concurrent retries safe.
    let inserted = sqlx::query("INSERT INTO transactions (id, user_id, wallet_id, network, sender, recipient, asset_id, amount_atomic, status, idempotency_key, request_fingerprint, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'CREATED', ?, ?, NOW(), NOW()) ON DUPLICATE KEY UPDATE id = id")
        .bind(&id).bind(&user.id).bind(&wallet.id).bind(NETWORK).bind(&wallet.address).bind(&req.recipient).bind(asset_id).bind(amount).bind(key).bind(&fingerprint).execute(&mut *tx).await?;
    let record = sqlx::query_as::<_, TransactionRecord>("SELECT * FROM transactions WHERE user_id = ? AND idempotency_key = ?")
        .bind(&user.id).bind(key).fetch_one(&mut *tx).await?;
    if record.request_fingerprint.as_deref() != Some(&fingerprint) { return Err(ApiError::new("CONFLICT", "This idempotency key was already used for different transfer details.")); }
    if inserted.rows_affected() == 1 && record.id == id { audit(&mut tx, &user.id, "transaction.intent_created").await?; }
    tx.commit().await?;
    // An intent is not submission or settlement. Only a provider signing adapter may advance it.
    Ok((if record.id == id { StatusCode::CREATED } else { StatusCode::OK }, Json(record_json(&record))))
}
pub async fn get_transaction(State(state): State<AppState>, headers: HeaderMap, Path(id): Path<String>) -> Result<Json<serde_json::Value>, ApiError> {
    let user = authorize(&state, &headers).await?;
    let record = sqlx::query_as::<_, TransactionRecord>("SELECT * FROM transactions WHERE id = ? AND user_id = ?")
        .bind(id).bind(user.id).fetch_optional(&state.pool).await?.ok_or_else(|| ApiError::new("NOT_FOUND", "Transaction not found."))?;
    Ok(Json(record_json(&record)))
}
#[derive(Deserialize)]
pub struct TransactionQuery { page: Option<u32>, limit: Option<u32>, status: Option<String> }
pub async fn list_transactions(State(state): State<AppState>, headers: HeaderMap, Query(query): Query<TransactionQuery>) -> Result<Json<serde_json::Value>, ApiError> {
    let user = authorize(&state, &headers).await?;
    let page = query.page.unwrap_or(1); let limit = query.limit.unwrap_or(20);
    if page == 0 || page > 100_000 || limit == 0 || limit > 100 { return Err(ApiError::new("INVALID_REQUEST", "Use page >= 1 and limit between 1 and 100.")); }
    let status = query.status.map(|status| status.to_ascii_uppercase());
    if status.as_ref().is_some_and(|status| !["CREATED", "VALIDATING", "READY", "SUBMITTED", "PENDING", "CONFIRMED", "FAILED"].contains(&status.as_str())) { return Err(ApiError::new("INVALID_REQUEST", "Unsupported transaction status.")); }
    let records = sqlx::query_as::<_, TransactionRecord>("SELECT * FROM transactions WHERE user_id = ? AND (? IS NULL OR status = ?) ORDER BY created_at DESC, id DESC LIMIT ? OFFSET ?")
        .bind(user.id).bind(&status).bind(&status).bind(limit).bind((page - 1) * limit).fetch_all(&state.pool).await?;
    Ok(Json(serde_json::json!({ "transactions": records.iter().map(record_json).collect::<Vec<_>>(), "page": page, "limit": limit })))
}
