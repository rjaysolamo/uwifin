use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{issue_session, secure_cookie_header},
    error::ApiError,
    state::AppState,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub user: UserSummary,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub user: UserSummary,
    pub session: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSummary {
    pub id: String,
    pub email: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub email: String,
    pub password_hash: String,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub last_login_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<impl IntoResponse, ApiError> {
    if req.email.trim().is_empty() || req.password.trim().is_empty() {
        return Err(ApiError::new("INVALID_REQUEST", "Email and password are required"));
    }

    if req.password.len() < 8 {
        return Err(ApiError::new("INVALID_REQUEST", "Password must be at least 8 characters"));
    }

    let exists = sqlx::query_scalar!("SELECT COUNT(*) as count FROM users WHERE email = ?", req.email)
        .fetch_one(state.pool.as_ref())
        .await
        .map_err(|_| ApiError::new("INTERNAL_ERROR", "Failed to query user existence"))?
        .count
        .unwrap_or(0)
        > 0;

    if exists {
        return Err(ApiError::new("INVALID_REQUEST", "User already exists"));
    }

    let password_hash = hash_password(&req.password)
        .map_err(|_| ApiError::new("INTERNAL_ERROR", "Password hashing failed"))?;

    let user_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now();

    sqlx::query!(
        "INSERT INTO users (id, email, password_hash, status, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
        user_id,
        req.email,
        password_hash,
        "active",
        now,
        now,
    )
    .execute(state.pool.as_ref())
    .await
    .map_err(|_| ApiError::new("INTERNAL_ERROR", "Failed to create user"))?;

    Ok((StatusCode::CREATED, Json(RegisterResponse {
        user: UserSummary {
            id: user_id,
            email: req.email,
        },
    })))
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<impl IntoResponse, ApiError> {
    if req.email.trim().is_empty() || req.password.trim().is_empty() {
        return Err(ApiError::new("INVALID_REQUEST", "Email and password are required"));
    }

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = ?")
        .bind(&req.email)
        .fetch_optional(state.pool.as_ref())
        .await
        .map_err(|_| ApiError::new("INTERNAL_ERROR", "Database error"))?
        .ok_or_else(|| ApiError::new("UNAUTHORIZED", "Invalid credentials"))?;

    let valid = verify_password(&req.password, &user.password_hash)
        .map_err(|_| ApiError::new("INTERNAL_ERROR", "Password verification failed"))?;

    if !valid {
        return Err(ApiError::new("UNAUTHORIZED", "Invalid credentials"));
    }

    let session_id = issue_session(state.pool.as_ref(), &user.id).await?;
    let last_login_at = chrono::Utc::now();

    sqlx::query!("UPDATE users SET last_login_at = ? WHERE id = ?", last_login_at, user.id)
        .execute(state.pool.as_ref())
        .await
        .map_err(|_| ApiError::new("INTERNAL_ERROR", "Failed to update login metadata"))?;

    let cookie = secure_cookie_header(&session_id);

    let mut response = (
        StatusCode::OK,
        Json(LoginResponse {
            user: UserSummary {
                id: user.id,
                email: user.email,
            },
            session: session_id.clone(),
        }),
    )
        .into_response();

    response.headers_mut().insert(
        header::SET_COOKIE,
        cookie.parse().unwrap(),
    );

    Ok(response)
}

pub async fn logout(
    State(state): State<AppState>,
    req: axum::extract::Request,
) -> Result<impl IntoResponse, ApiError> {
    let auth = req.headers().get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .filter(|h| h.starts_with("Bearer "))
        .map(|h| h.trim_start_matches("Bearer ").to_string());

    if let Some(session_id) = auth {
        crate::auth::revoke_session(state.pool.as_ref(), &session_id).await?;
    }

    let mut response = (StatusCode::OK, Json(serde_json::json!({ "status": "logged_out" }))).into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        "session=; HttpOnly; SameSite=Lax; Secure; Path=/; Max-Age=0".parse().unwrap(),
    );

    Ok(response)
}

pub async fn me(
    State(state): State<AppState>,
    req: axum::extract::Request,
) -> Result<impl IntoResponse, ApiError> {
    let auth = req.headers().get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .filter(|h| h.starts_with("Bearer "))
        .map(|h| h.trim_start_matches("Bearer ").to_string());

    let session_id = auth.ok_or_else(|| ApiError::new("UNAUTHORIZED", "Missing bearer token"))?;
    let session_hash = crate::auth::hash_session(&session_id)
        .map_err(|_| ApiError::new("INTERNAL_ERROR", "Failed to hash session"))?;

    let session = sqlx::query!(
        "SELECT * FROM sessions WHERE session_hash = ? AND revoked_at IS NULL AND expires_at > NOW()",
        session_hash,
    )
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

    Ok((StatusCode::OK, Json(UserSummary {
        id: user.id,
        email: user.email,
    })))
}

fn hash_password(password: &str) -> anyhow::Result<String> {
    use argon2::{Argon2, PasswordHasher};
    use argon2::password_hash::SaltString;
    use rand::rngs::OsRng;

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)?
        .to_string();

    Ok(password_hash)
}

fn verify_password(password: &str, hashed_password: &str) -> anyhow::Result<bool> {
    use argon2::{Argon2, PasswordHash, PasswordVerifier};

    let parsed_hash = PasswordHash::new(hashed_password)?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}
