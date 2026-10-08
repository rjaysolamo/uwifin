use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::{
    error::ApiError,
    state::AppState,
};

#[derive(Debug, Clone)]
pub struct WalletValidationResult {
    pub is_valid: bool,
    pub normalized_address: String,
}

pub fn validate_wallet_address(address: &str) -> Result<WalletValidationResult, ApiError> {
    let normalized = address.trim();
    if normalized.is_empty() {
        return Err(ApiError::new("INVALID_ADDRESS", "Wallet address is required"));
    }

    if normalized.len() < 20 || normalized.len() > 100 {
        return Err(ApiError::new("INVALID_ADDRESS", "Address length is invalid"));
    }

    if !normalized.starts_with("0x") {
        return Err(ApiError::new("INVALID_ADDRESS", "Address must start with 0x"));
    }

    let hex_part = &normalized[2..];
    if hex_part.len() % 2 != 0 || !hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ApiError::new("INVALID_ADDRESS", "Address must be a valid hex string"));
    }

    Ok(WalletValidationResult {
        is_valid: true,
        normalized_address: normalized.to_string(),
    })
}

pub fn validate_network(network: &str) -> Result<String, ApiError> {
    let normalized = network.trim().to_ascii_lowercase();
    let supported = ["base", "ethereum", "polygon"];

    if !supported.contains(&normalized.as_str()) {
        return Err(ApiError::new("UNSUPPORTED_NETWORK", "Network is not supported in MVP"));
    }

    Ok(normalized)
}

pub fn validate_wallet_type(wallet_type: &str) -> Result<String, ApiError> {
    let normalized = wallet_type.trim().to_ascii_lowercase();
    let allowed = ["smart_account", "external", "embedded"];

    if !allowed.contains(&normalized.as_str()) {
        return Err(ApiError::new("INVALID_REQUEST", "Wallet type is invalid"));
    }

    Ok(normalized)
}

pub async fn wallet_validation_tests() {
    let ok = validate_wallet_address("0x1234567890abcdef1234567890abcdef12345678").unwrap();
    assert!(ok.is_valid);

    let invalid = validate_wallet_address("not-a-wallet");
    assert!(invalid.is_err());

    let network = validate_network("base").unwrap();
    assert_eq!(network, "base");

    let wallet_type = validate_wallet_type("smart_account").unwrap();
    assert_eq!(wallet_type, "smart_account");
}
