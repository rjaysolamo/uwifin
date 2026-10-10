use axum::{extract::State, http::{HeaderMap, StatusCode}, Json};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use crate::{error::ApiError, security::{audit, hash}, state::AppState};

#[derive(Clone, Debug, Serialize, sqlx::FromRow)]
pub struct UserResponse {
    pub id: String, pub email: String, pub name: String, pub role: String, pub created_at: chrono::DateTime<chrono::Utc>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Credentials { pub email: String, pub password: String, pub name: Option<String> }
#[derive(Serialize)]
pub struct SessionResponse { pub user: UserResponse, pub session_id: String }

fn email(value: &str) -> Result<String, ApiError> {
    let value = value.trim().to_ascii_lowercase();
    if value.len() > 254 || !value.is_ascii() || value.bytes().any(|c| c.is_ascii_whitespace()) || value.matches('@').count() != 1 {
        return Err(ApiError::new("INVALID_REQUEST", "Enter a valid email address."));
    }
    let (local, domain) = value.split_once('@').ok_or_else(|| ApiError::new("INVALID_REQUEST", "Enter a valid email address."))?;
    if local.is_empty() || !domain.contains('.') || domain.starts_with('.') || domain.ends_with('.') {
        return Err(ApiError::new("INVALID_REQUEST", "Enter a valid email address."));
    }
    Ok(value)
}
pub fn validate_name(value: &str) -> Result<String, ApiError> {
    let name = value.trim();
    if !(2..=60).contains(&name.chars().count()) || name.chars().any(char::is_control) { return Err(ApiError::new("INVALID_REQUEST", "Name must contain 2 to 60 characters.")); }
    Ok(name.into())
}
pub fn session_token(headers: &HeaderMap) -> Result<&str, ApiError> {
    headers.get("authorization").and_then(|v| v.to_str().ok()).and_then(|v| v.strip_prefix("Bearer "))
        .filter(|token| token.len() >= 32 && token.len() <= 128).ok_or_else(|| ApiError::new("UNAUTHORIZED", "Please sign in to continue."))
}
pub async fn authorize(state: &AppState, headers: &HeaderMap) -> Result<UserResponse, ApiError> {
    let token = session_token(headers)?;
    let user = sqlx::query_as::<_, UserResponse>("SELECT u.id, u.email, u.name, u.role, u.created_at FROM users u JOIN sessions s ON s.user_id = u.id WHERE s.session_hash = ? AND s.revoked_at IS NULL AND s.expires_at > NOW() AND u.status = 'active'")
        .bind(hash(token)).fetch_optional(&state.pool).await?.ok_or_else(|| ApiError::new("UNAUTHORIZED", "Your session has expired. Please sign in again."))?;
    state.limits.check(format!("user:{}", user.id), 120)?;
    tracing::info!(request_id = crate::security::request_id(), user_id = user.id, operation = "authorize", "request authenticated");
    Ok(user)
}
async fn issue(connection: &mut sqlx::MySqlConnection, user_id: &str) -> Result<String, ApiError> {
    let mut bytes = [0u8; 32]; OsRng.fill_bytes(&mut bytes);
    let token = URL_SAFE_NO_PAD.encode(bytes);
    sqlx::query("INSERT INTO sessions (id, user_id, session_hash, expires_at, created_at) VALUES (?, ?, ?, DATE_ADD(NOW(), INTERVAL 1 DAY), NOW())")
        .bind(uuid::Uuid::new_v4().to_string()).bind(user_id).bind(hash(&token)).execute(connection).await?;
    Ok(token)
}
pub async fn register(State(state): State<AppState>, Json(req): Json<Credentials>) -> Result<(StatusCode, Json<SessionResponse>), ApiError> {
    let email = email(&req.email)?;
    let name = validate_name(req.name.as_deref().unwrap_or("UwiFin user"))?;
    if !(12..=128).contains(&req.password.len()) { return Err(ApiError::new("INVALID_REQUEST", "Use a password between 12 and 128 characters.")); }
    state.limits.check(format!("auth-email:{}", hash(&email)), 5)?;
    let permit = state.password_slots.clone().try_acquire_owned().map_err(|_| ApiError::new("RATE_LIMITED", "Please try again shortly."))?;
    let password_hash = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        Argon2::default().hash_password(req.password.as_bytes(), &SaltString::generate(&mut OsRng)).map(|hash| hash.to_string()).map_err(|_| ApiError::unavailable())
    }).await.map_err(|_| ApiError::unavailable())??;
    let user_id = uuid::Uuid::new_v4().to_string();
    let mut tx = state.pool.begin().await?;
    let inserted = sqlx::query("INSERT INTO users (id, email, name, password_hash, status, created_at, updated_at, last_login_at) VALUES (?, ?, ?, ?, 'active', NOW(), NOW(), NOW())")
        .bind(&user_id).bind(&email).bind(&name).bind(password_hash).execute(&mut *tx).await;
    if let Err(err) = inserted {
        return Err(if err.as_database_error().is_some_and(|err| err.is_unique_violation()) { ApiError::new("INVALID_REQUEST", "An account could not be created with these details. Try signing in.") } else { err.into() });
    }
    let session_id = issue(&mut tx, &user_id).await?;
    audit(&mut tx, &user_id, "account.registered").await?;
    let user = sqlx::query_as::<_, UserResponse>("SELECT id, email, name, role, created_at FROM users WHERE id = ?").bind(&user_id).fetch_one(&mut *tx).await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(SessionResponse { user, session_id })))
}
#[derive(sqlx::FromRow)]
struct LoginRecord { id: String, password_hash: String, status: String }
pub async fn login(State(state): State<AppState>, Json(req): Json<Credentials>) -> Result<Json<SessionResponse>, ApiError> {
    let email = email(&req.email)?;
    if req.password.is_empty() || req.password.len() > 128 { return Err(ApiError::new("UNAUTHORIZED", "Invalid email or password.")); }
    state.limits.check(format!("auth-email:{}", hash(&email)), 5)?;
    let record = sqlx::query_as::<_, LoginRecord>("SELECT id, password_hash, status FROM users WHERE email = ?").bind(&email).fetch_optional(&state.pool).await?;
    let permit = state.password_slots.clone().try_acquire_owned().map_err(|_| ApiError::new("RATE_LIMITED", "Please try again shortly."))?;
    let stored_hash = record.as_ref().map(|user| user.password_hash.clone());
    let valid = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        if let Some(stored_hash) = stored_hash {
            PasswordHash::new(&stored_hash).map(|parsed| Argon2::default().verify_password(req.password.as_bytes(), &parsed).is_ok()).unwrap_or(false)
        } else {
            // Spend the same password-hashing work for unknown accounts.
            let _ = Argon2::default().hash_password(req.password.as_bytes(), &SaltString::generate(&mut OsRng));
            false
        }
    }).await.map_err(|_| ApiError::unavailable())?;
    let record = record.filter(|user| valid && user.status == "active").ok_or_else(|| ApiError::new("UNAUTHORIZED", "Invalid email or password."))?;
    let mut tx = state.pool.begin().await?;
    // Serialize session issuance with password changes, which revoke sessions under this row lock.
    let current: (String, String) = sqlx::query_as("SELECT password_hash, status FROM users WHERE id = ? FOR UPDATE")
        .bind(&record.id).fetch_one(&mut *tx).await?;
    if current.0 != record.password_hash || current.1 != "active" { return Err(ApiError::new("UNAUTHORIZED", "Invalid email or password.")); }
    let session_id = issue(&mut tx, &record.id).await?;
    sqlx::query("UPDATE users SET last_login_at = NOW() WHERE id = ?").bind(&record.id).execute(&mut *tx).await?;
    audit(&mut tx, &record.id, "account.login").await?;
    let user = sqlx::query_as::<_, UserResponse>("SELECT id, email, name, role, created_at FROM users WHERE id = ?").bind(&record.id).fetch_one(&mut *tx).await?;
    tx.commit().await?;
    Ok(Json(SessionResponse { user, session_id }))
}
pub async fn me(State(state): State<AppState>, headers: HeaderMap) -> Result<Json<UserResponse>, ApiError> { Ok(Json(authorize(&state, &headers).await?)) }
pub async fn logout(State(state): State<AppState>, headers: HeaderMap) -> Result<Json<serde_json::Value>, ApiError> {
    let user = authorize(&state, &headers).await?;
    let mut tx = state.pool.begin().await?;
    sqlx::query("UPDATE sessions SET revoked_at = NOW() WHERE session_hash = ? AND user_id = ?").bind(hash(session_token(&headers)?)).bind(&user.id).execute(&mut *tx).await?;
    audit(&mut tx, &user.id, "account.logout").await?;
    tx.commit().await?;
    Ok(Json(serde_json::json!({ "status": "logged_out" })))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileUpdate { name: String }
pub async fn update_profile(State(state): State<AppState>, headers: HeaderMap, Json(req): Json<ProfileUpdate>) -> Result<Json<UserResponse>, ApiError> {
    let mut user = authorize(&state, &headers).await?;
    let name = validate_name(&req.name)?;
    let mut tx = state.pool.begin().await?;
    sqlx::query("UPDATE users SET name = ?, updated_at = NOW() WHERE id = ?").bind(&name).bind(&user.id).execute(&mut *tx).await?;
    audit(&mut tx, &user.id, "account.profile_updated").await?;
    tx.commit().await?;
    user.name = name;
    Ok(Json(user))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PasswordChange { current_password: String, new_password: String }
pub async fn change_password(State(state): State<AppState>, headers: HeaderMap, Json(req): Json<PasswordChange>) -> Result<Json<serde_json::Value>, ApiError> {
    let user = authorize(&state,&headers).await?;
    state.limits.check(format!("password:{}",user.id),5)?;
    if req.current_password.len()>128 || !(12..=128).contains(&req.new_password.len()) { return Err(ApiError::new("INVALID_REQUEST","Use a new password between 12 and 128 characters.")); }
    let old: String = sqlx::query_scalar("SELECT password_hash FROM users WHERE id = ?").bind(&user.id).fetch_one(&state.pool).await?;
    let previous = old.clone();
    let permit = state.password_slots.clone().try_acquire_owned().map_err(|_|ApiError::new("RATE_LIMITED","Try again shortly."))?;
    let new = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        let parsed = PasswordHash::new(&old).map_err(|_|ApiError::unavailable())?;
        if Argon2::default().verify_password(req.current_password.as_bytes(),&parsed).is_err() { return Err(ApiError::new("FORBIDDEN","The current password is incorrect.")); }
        Argon2::default().hash_password(req.new_password.as_bytes(),&SaltString::generate(&mut OsRng)).map(|v|v.to_string()).map_err(|_|ApiError::unavailable())
    }).await.map_err(|_|ApiError::unavailable())??;
    let mut tx = state.pool.begin().await?;
    let changed = sqlx::query("UPDATE users SET password_hash = ?,updated_at = NOW() WHERE id = ? AND password_hash = ?").bind(new).bind(&user.id).bind(previous).execute(&mut *tx).await?;
    if changed.rows_affected()!=1 { return Err(ApiError::new("CONFLICT","Your password changed. Sign in again.")); }
    sqlx::query("UPDATE sessions SET revoked_at = NOW() WHERE user_id = ? AND session_hash <> ? AND revoked_at IS NULL").bind(&user.id).bind(hash(session_token(&headers)?)).execute(&mut *tx).await?;
    audit(&mut tx,&user.id,"account.password_changed").await?; tx.commit().await?;
    Ok(Json(serde_json::json!({"status":"updated"})))
}
pub async fn revoke_other_sessions(State(state): State<AppState>, headers: HeaderMap) -> Result<Json<serde_json::Value>,ApiError> {
    let user = authorize(&state,&headers).await?;
    let mut tx = state.pool.begin().await?;
    sqlx::query("UPDATE sessions SET revoked_at = NOW() WHERE user_id = ? AND session_hash <> ? AND revoked_at IS NULL").bind(&user.id).bind(hash(session_token(&headers)?)).execute(&mut *tx).await?;
    audit(&mut tx,&user.id,"account.other_sessions_revoked").await?; tx.commit().await?;
    Ok(Json(serde_json::json!({"status":"revoked"})))
}
