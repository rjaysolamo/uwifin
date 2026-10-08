use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::{
    config::Config,
    error::ApiError,
    state::AppState,
};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub session_hash: String,
    pub expires_at: chrono::DateTime<Utc>,
    pub created_at: chrono::DateTime<Utc>,
    pub revoked_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub id: String,
    pub email: String,
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_string());

    let token = match auth_header {
        Some(h) if h.starts_with("Bearer ") => Some(h.trim_start_matches("Bearer ").to_string()),
        _ => None,
    };

    let session_id = match token {
        Some(token) => token,
        None => {
            return Err(ApiError::new("UNAUTHORIZED", "Missing bearer token"));
        }
    };

    let session_hash = hash_session(&session_id).map_err(|_| ApiError::new("INTERNAL_ERROR", "Failed to hash session"))?;

    let session = sqlx::query_as::<_, Session>(
        "SELECT * FROM sessions WHERE session_hash = ? AND revoked_at IS NULL AND expires_at > NOW()"
    )
    .bind(&session_hash)
    .fetch_optional(state.pool.as_ref())
    .await
    .map_err(|_| ApiError::new("INTERNAL_ERROR", "Failed to validate session"))?
    .ok_or_else(|| ApiError::new("UNAUTHORIZED", "Invalid or expired session"))?;

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(&session.user_id)
        .fetch_optional(state.pool.as_ref())
        .await
        .map_err(|_| ApiError::new("INTERNAL_ERROR", "Failed to load user"))?
        .ok_or_else(|| ApiError::new("UNAUTHORIZED", "User not found"))?;

    request.extensions_mut().insert(AuthUser {
        id: user.id,
        email: user.email,
    });

    Ok(next.run(request).await)
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub email: String,
    pub password_hash: String,
    pub status: String,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    pub last_login_at: Option<chrono::DateTime<Utc>>,
}

pub fn hash_session(session_id: &str) -> anyhow::Result<String> {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(session_id.as_bytes());
    Ok(hex::encode(hasher.finalize()))
}

pub async fn issue_session(
    pool: &sqlx::MySqlPool,
    user_id: &str,
) -> Result<String, ApiError> {
    let session_id = uuid::Uuid::new_v4().to_string();
    let session_hash = hash_session(&session_id)
        .map_err(|_| ApiError::new("INTERNAL_ERROR", "Failed to hash session"))?;
    let expires_at = Utc::now() + Duration::days(7);
    let created_at = Utc::now();

    sqlx::query!(
        "INSERT INTO sessions (id, user_id, session_hash, expires_at, created_at) VALUES (?, ?, ?, ?, ?)",
        uuid::Uuid::new_v4().to_string(),
        user_id,
        session_hash,
        expires_at,
        created_at,
    )
    .execute(pool)
    .await
    .map_err(|_| ApiError::new("INTERNAL_ERROR", "Failed to create session"))?;

    Ok(session_id)
}

pub async fn revoke_session(
    pool: &sqlx::MySqlPool,
    session_id: &str,
) -> Result<(), ApiError> {
    let session_hash = hash_session(session_id)
        .map_err(|_| ApiError::new("INTERNAL_ERROR", "Failed to hash session"))?;

    sqlx::query!(
        "UPDATE sessions SET revoked_at = NOW() WHERE session_hash = ?",
        session_hash,
    )
    .execute(pool)
    .await
    .map_err(|_| ApiError::new("INTERNAL_ERROR", "Failed to revoke session"))?;

    Ok(())
}

pub fn secure_cookie_header(session_value: &str) -> String {
    format!(
        "session={}; HttpOnly; SameSite=Lax; Secure; Path=/; Max-Age=604800",
        session_value
    )
}
