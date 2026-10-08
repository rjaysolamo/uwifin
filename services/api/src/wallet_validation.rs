use crate::error::ApiError;
pub const NETWORK: &str = "base-sepolia";
pub const CHAIN_ID: &str = "0x14a34";
pub const USDC: &str = "0x036CbD53842c5426634e7929541eC2318f3dCF7e";
pub fn validate_wallet_address(address: &str) -> Result<String, ApiError> {
    if address.len() != 42 || !address.starts_with("0x") || !address[2..].bytes().all(|byte| byte.is_ascii_hexdigit()) || address[2..].bytes().all(|byte| byte == b'0') {
        return Err(ApiError::new("INVALID_ADDRESS", "Enter a valid nonzero EVM wallet address."));
    }
    Ok(address.to_ascii_lowercase())
}
pub fn validate_network(network: &str, configured: &str) -> Result<(), ApiError> {
    if network != configured { return Err(ApiError::new("UNSUPPORTED_NETWORK", "Use the network configured for this wallet.")); }
    Ok(())
}
