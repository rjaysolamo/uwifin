use axum::{body::Bytes, extract::State, http::HeaderMap, Json};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use serde::Deserialize;
use crate::{auth::authorize, error::ApiError, security::audit, state::AppState};
pub fn verify_signature(secret: &str, signature: &str, payload: &[u8], now: i64) -> bool {
    let timestamp = signature.split(',').filter_map(|part| part.trim().strip_prefix("t=")).collect::<Vec<_>>();
    if timestamp.len() != 1 { return false; }
    let Some(timestamp) = timestamp[0].parse::<i64>().ok() else { return false; };
    if now.abs_diff(timestamp) > 300 { return false; }
    for candidate in signature.split(',').filter_map(|part| part.trim().strip_prefix("v1=")) {
        let Ok(bytes) = hex::decode(candidate) else { continue; };
        let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(secret.as_bytes()) else { return false; };
        mac.update(format!("{timestamp}.").as_bytes()); mac.update(payload);
        if mac.verify_slice(&bytes).is_ok() { return true; }
    }
    false
}
#[derive(Deserialize)]
struct Event { id: String, #[serde(rename = "type")] kind: String, created: i64, livemode: bool, data: EventData }
#[derive(Deserialize)]
struct EventData { object: serde_json::Value }
#[derive(sqlx::FromRow)]
struct Payment { id: String, user_id: String, status: String, last_event_created: i64 }
pub async fn webhook(State(state): State<AppState>, headers: HeaderMap, body: Bytes) -> Result<Json<serde_json::Value>, ApiError> {
    let secret = state.config.stripe_webhook_secret.as_ref().ok_or_else(ApiError::unavailable)?;
    let signature = headers.get("stripe-signature").and_then(|v| v.to_str().ok()).unwrap_or("");
    if !verify_signature(secret, signature, &body, chrono::Utc::now().timestamp()) { return Err(ApiError::new("INVALID_REQUEST", "Invalid webhook signature.")); }
    let event: Event = serde_json::from_slice(&body).map_err(|_| ApiError::new("INVALID_REQUEST", "Invalid webhook payload."))?;
    if event.id.is_empty() || event.id.len() > 255 || event.kind.len() > 100 || event.created < 0 || event.livemode {
        return Err(ApiError::new("INVALID_REQUEST", "Only valid Stripe test events are supported by this development API."));
    }
    let mut tx = state.pool.begin().await?;
    let inserted = sqlx::query("INSERT IGNORE INTO webhook_events (id, event_type) VALUES (?, ?)").bind(&event.id).bind(&event.kind).execute(&mut *tx).await?;
    if inserted.rows_affected() == 0 { tx.commit().await?; return Ok(Json(serde_json::json!({ "received": true }))); }
    if event.kind == "crypto.onramp_session_updated" {
        let reference = event.data.object.get("id").and_then(|v| v.as_str()).ok_or_else(|| ApiError::new("INVALID_REQUEST", "Missing provider reference."))?;
        let payment = sqlx::query_as::<_, Payment>("SELECT id, user_id, status, last_event_created FROM payments WHERE provider = 'stripe' AND provider_reference = ? FOR UPDATE")
            .bind(reference).fetch_optional(&mut *tx).await?;
        if let Some(payment) = payment {
            let status = match event.data.object.get("status").and_then(|v| v.as_str()) {
                Some("fulfillment_complete") => Some("confirmed"),
                Some("rejected") => Some("failed"),
                Some("initialized" | "requires_payment" | "fulfillment_processing") => Some("pending"),
                _ => None,
            };
            // Terminal states cannot regress; older events cannot overwrite newer state.
            if let Some(status) = status.filter(|_| event.created >= payment.last_event_created && !["confirmed", "failed"].contains(&payment.status.as_str())) {
                sqlx::query("UPDATE payments SET status = ?, last_event_created = ?, updated_at = NOW() WHERE id = ?")
                    .bind(status).bind(event.created).bind(payment.id).execute(&mut *tx).await?;
                audit(&mut tx, &payment.user_id, "payment.status_updated").await?;
            }
        }
    }
    tx.commit().await?;
    Ok(Json(serde_json::json!({ "received": true })))
}
pub async fn create_payment(State(state): State<AppState>, headers: HeaderMap) -> Result<Json<serde_json::Value>, ApiError> {
    let user = authorize(&state, &headers).await?;
    state.limits.check(format!("payment:{}", user.id), 5)?;
    Err(ApiError::new("PROVIDER_NOT_CONFIGURED", "Stripe on-ramp eligibility and a compatible destination network must be verified before purchases are enabled."))
}
#[derive(serde::Serialize, sqlx::FromRow)]
struct PaymentResponse { id: String, provider: String, payment_type: String, fiat_currency: String, fiat_amount_minor: i64, status: String, created_at: chrono::DateTime<chrono::Utc> }
pub async fn list_payments(State(state): State<AppState>, headers: HeaderMap) -> Result<Json<serde_json::Value>, ApiError> {
    let user = authorize(&state, &headers).await?;
    let records = sqlx::query_as::<_, PaymentResponse>("SELECT id, provider, payment_type, fiat_currency, fiat_amount_minor, status, created_at FROM payments WHERE user_id = ? ORDER BY created_at DESC LIMIT 100")
        .bind(user.id).fetch_all(&state.pool).await?;
    Ok(Json(serde_json::json!({ "payments": records })))
}
