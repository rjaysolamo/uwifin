use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::{
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
    pub user: UserResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub user: UserResponse,
    pub session_id: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub struct User {
    pub id: String,
    pub email: String,
    pub password_hash: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Validate email and password
    if req.email.is_empty() || req.password.is_empty() {
        return Err(ApiError::new(
            "INVALID_REQUEST",
            "Email and password are required",
        ));
    }

    if req.password.len() < 8 {
        return Err(ApiError::new(
            "INVALID_REQUEST",
            "Password must be at least 8 characters",
        ));
    }

    // Check if user exists
    let existing = sqlx::query!("SELECT id FROM users WHERE email = ?", req.email)
        .fetch_optional(state.pool.as_ref())
        .await
        .map_err(|_| ApiError::new("INTERNAL_ERROR", "Database error"))?
        .is_some();

    if existing {
        return Err(ApiError::new(
            "INVALID_REQUEST",
            "User already exists",
        ));
    }

    // Hash password
    let password_hash = hash_password(&req.password)
        .map_err(|_| ApiError::new("INTERNAL_ERROR", "Password hashing failed"))?;

    // Create user
    let user_id = Uuid::new_v4().to_string();
    let now = Utc::now();

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

    tracing::info!("User registered: {}", req.email);

    Ok((StatusCode::CREATED, Json(RegisterResponse {
        user: UserResponse {
            id: user_id,
            email: req.email,
            created_at: now,
        },
    })))
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<impl IntoResponse, ApiError> {
    if req.email.is_empty() || req.password.is_empty() {
        return Err(ApiError::new(
            "INVALID_REQUEST",
            "Email and password are required",
        ));
    }

    // Find user
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = ?")
        .bind(&req.email)
        .fetch_optional(state.pool.as_ref())
        .await
        .map_err(|_| ApiError::new("INTERNAL_ERROR", "Database error"))?
        .ok_or_else(|| ApiError::new("UNAUTHORIZED", "Invalid credentials"))?;

    // Verify password
    if !verify_password(&req.password, &user.password_hash)
        .map_err(|_| ApiError::new("INTERNAL_ERROR", "Password verification failed"))?
    {
        return Err(ApiError::new("UNAUTHORIZED", "Invalid credentials"));
    }

    // Create session
    let session_id = Uuid::new_v4().to_string();
    let session_hash = hash_session(&session_id)
        .map_err(|_| ApiError::new("INTERNAL_ERROR", "Session creation failed"))?;
    let expires_at = Utc::now() + chrono::Duration::days(7);
    let now = Utc::now();

    sqlx::query!(
        "INSERT INTO sessions (id, user_id, session_hash, expires_at, created_at) VALUES (?, ?, ?, ?, ?)",
        Uuid::new_v4().to_string(),
        user.id,
        session_hash,
        expires_at,
        now,
    )
    .execute(state.pool.as_ref())
    .await
    .map_err(|_| ApiError::new("INTERNAL_ERROR", "Failed to create session"))?;

    // Update last login
    sqlx::query!("UPDATE users SET last_login_at = ? WHERE id = ?", now, user.id)
        .execute(state.pool.as_ref())
        .await
        .map_err(|_| ApiError::new("INTERNAL_ERROR", "Failed to update user"))?;

    tracing::info!("User logged in: {}", user.email);

    Ok((StatusCode::OK, Json(LoginResponse {
        user: UserResponse {
            id: user.id,
            email: user.email,
            created_at: user.created_at,
        },
        session_id,
    })))
}

pub async fn logout(
    State(_state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    // TODO: Implement logout with session validation
    Ok((StatusCode::OK, Json(serde_json::json!({ "status": "logged_out" }))))
}

pub async fn me(
    State(_state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    // TODO: Implement me endpoint with session validation
    Err(ApiError::new("UNAUTHORIZED", "Session validation not yet implemented"))
}

fn hash_password(password: &str) -> anyhow::Result<String> {
    use argon2::{Argon2, PasswordHasher};
    use argon2::password_hash::SaltString;
    use rand::rngs::OsRng;

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();

    Ok(password_hash)
}

fn verify_password(password: &str, hash: &str) -> anyhow::Result<bool> {
    use argon2::{Argon2, PasswordHash, PasswordVerifier};

    let parsed_hash = PasswordHash::new(hash)?;
    let argon2 = Argon2::default();

    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

fn hash_session(session_id: &str) -> anyhow::Result<String> {
    use sha2::{Sha256, Digest};

    let mut hasher = Sha256::new();
    hasher.update(session_id.as_bytes());
    let result = hasher.finalize();

    Ok(hex::encode(result))
}
