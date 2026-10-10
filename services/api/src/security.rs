use std::{collections::HashMap, sync::Mutex, time::{Duration, Instant}};
use axum::{extract::{ConnectInfo, Request, State}, middleware::Next, response::{Response, IntoResponse}};
use sha2::{Digest, Sha256};
use crate::{error::ApiError, state::AppState};

tokio::task_local! { static REQUEST_ID: String; }
pub fn request_id() -> String { REQUEST_ID.try_with(Clone::clone).unwrap_or_else(|_| uuid::Uuid::new_v4().to_string()) }

#[derive(Default)]
pub struct RateLimits { buckets: Mutex<HashMap<String, (Instant, u32)>> }
impl RateLimits {
    pub fn check(&self, key: String, maximum: u32) -> Result<(), ApiError> {
        let now = Instant::now();
        let mut buckets = self.buckets.lock().map_err(|_| ApiError::unavailable())?;
        buckets.retain(|_, (start, _)| now.duration_since(*start) < Duration::from_secs(60));
        if buckets.len() >= 10_000 && !buckets.contains_key(&key) { return Err(ApiError::new("RATE_LIMITED", "Please wait a minute before trying again.")); }
        let entry = buckets.entry(key).or_insert((now, 0));
        if entry.1 >= maximum { return Err(ApiError::new("RATE_LIMITED", "Please wait a minute before trying again.")); }
        entry.1 += 1;
        Ok(())
    }
}
pub fn hash(value: &str) -> String { hex::encode(Sha256::digest(value.as_bytes())) }

pub async fn request_controls(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let request_id = uuid::Uuid::new_v4().to_string();
    REQUEST_ID.scope(request_id.clone(), async move {
        let path = request.uri().path().to_string();
        let started = Instant::now();
        let limit = if path.starts_with("/api/") {
            let peer = request.extensions().get::<ConnectInfo<std::net::SocketAddr>>().map(|peer| peer.0.ip().to_string()).unwrap_or_else(|| "local".into());
            let auth = path.ends_with("/auth/login") || path.ends_with("/auth/register");
            state.limits.check(format!("{}:{peer}", if auth { "auth-ip" } else { "api-ip" }), if auth { 20 } else { 240 })
        } else { Ok(()) };
        let mut response = match limit { Ok(()) => next.run(request).await, Err(error) => error.into_response() };
        if response.status().is_client_error() && !response.headers().get("content-type").is_some_and(|value| value == "application/json") {
            response = ApiError::new("INVALID_REQUEST", "Check the request format and required fields.").into_response();
        }
        response.headers_mut().insert("x-request-id", request_id.parse().expect("UUID header"));
        response.headers_mut().insert("cache-control", "no-store".parse().expect("static header"));
        tracing::info!(request_id, operation = %path, duration_ms = started.elapsed().as_millis(), status = response.status().as_u16(), "request completed");
        response
    }).await
}

pub async fn audit(connection: &mut sqlx::MySqlConnection, user: &str, event: &str) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO audit_events (id, user_id, event_type, request_id, metadata, created_at) VALUES (?, ?, ?, ?, '{}', NOW())")
        .bind(uuid::Uuid::new_v4().to_string()).bind(user).bind(event).bind(request_id()).execute(connection).await?;
    Ok(())
}
