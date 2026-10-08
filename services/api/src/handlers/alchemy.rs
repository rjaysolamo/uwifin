use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::{
    alchemy::AlchemyClient,
    config::Config,
    error::ApiError,
    state::AppState,
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SubmitTransferPayload {
    pub sender: String,
    pub target: String,
    pub value: String,
    pub data: String,
    pub gas_limit: Option<String>,
}

pub async fn submit_transfer(
    State(state): State<AppState>,
    Json(payload): Json<SubmitTransferPayload>,
) -> Result<impl IntoResponse, ApiError> {
    let client = AlchemyClient::new(
        &state.config.alchemy_api_key,
        &state.config.alchemy_network,
        &state.config.alchemy_policy_id,
    );

    let response = client
        .send_user_operation(&crate::alchemy::SendUserOperationRequest {
            sender: payload.sender,
            target: payload.target,
            value: payload.value,
            data: payload.data,
            gas_limit: payload.gas_limit,
        })
        .await
        .map_err(|_| ApiError::new("BLOCKCHAIN_ERROR", "Failed to submit user operation"))?;

    Ok((StatusCode::OK, Json(response)))
}

pub async fn get_receipt(
    State(state): State<AppState>,
    hash: String,
) -> Result<impl IntoResponse, ApiError> {
    let client = AlchemyClient::new(
        &state.config.alchemy_api_key,
        &state.config.alchemy_network,
        &state.config.alchemy_policy_id,
    );

    let response = client
        .get_transaction_receipt(&hash)
        .await
        .map_err(|_| ApiError::new("BLOCKCHAIN_ERROR", "Failed to fetch transaction receipt"))?;

    Ok((StatusCode::OK, Json(response)))
}
