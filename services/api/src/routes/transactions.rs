use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};

use crate::{
    error::ApiError,
    state::AppState,
    transaction::{create_transaction, get_transaction, CreateTransactionRequest},
};

pub fn transaction_routes() -> Router<AppState> {
    Router::new()
        .route("/transactions", axum::routing::post(create_transaction))
        .route("/transactions/:id", axum::routing::get(get_transaction))
}
