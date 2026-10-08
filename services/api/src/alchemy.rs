use serde_json::{json, Value};
use crate::{error::ApiError, state::AppState, wallet_validation::{validate_wallet_address, CHAIN_ID, USDC}};
async fn rpc(state: &AppState, method: &str, params: Value) -> Result<Value, ApiError> {
    let url = state.config.alchemy_rpc_url.as_ref().ok_or_else(|| ApiError::new("PROVIDER_NOT_CONFIGURED", "Blockchain balance service is not configured."))?;
    let response = state.http.post(url).json(&json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params }))
        .send().await.map_err(|_| ApiError::unavailable())?;
    if !response.status().is_success() { return Err(ApiError::unavailable()); }
    let body: Value = response.json().await.map_err(|_| ApiError::unavailable())?;
    if body.get("error").is_some() { return Err(ApiError::unavailable()); }
    body.get("result").cloned().ok_or_else(ApiError::unavailable)
}
pub async fn usdc_balance(state: &AppState, address: &str) -> Result<u128, ApiError> {
    let address = validate_wallet_address(address)?;
    if rpc(state, "eth_chainId", json!([])).await?.as_str() != Some(CHAIN_ID) {
        return Err(ApiError::new("UNSUPPORTED_NETWORK", "The configured blockchain provider is on an unsupported network."));
    }
    // balanceOf(address), fixed token and selector. Callers cannot supply calldata.
    let data = format!("0x70a08231{:0>64}", &address[2..]);
    let result = rpc(state, "eth_call", json!([{ "to": USDC, "data": data }, "latest"])).await?;
    let encoded = result.as_str().and_then(|s| s.strip_prefix("0x")).ok_or_else(ApiError::unavailable)?;
    u128::from_str_radix(encoded, 16).map_err(|_| ApiError::unavailable())
}
#[derive(Debug, PartialEq)]
pub enum ReceiptState { Pending, Confirmed, Failed }
/// A receipt's execution status, not HTTP status, determines settlement.
pub fn receipt_state(value: &Value) -> Result<ReceiptState, ApiError> {
    if value.is_null() { return Ok(ReceiptState::Pending); }
    match value.get("status").and_then(Value::as_str) {
        Some("0x1") if value.get("transactionHash").and_then(Value::as_str).is_some_and(|hash| hash.len() == 66 && hash.starts_with("0x") && hash[2..].bytes().all(|c| c.is_ascii_hexdigit())) => Ok(ReceiptState::Confirmed),
        Some("0x0") => Ok(ReceiptState::Failed),
        _ => Err(ApiError::unavailable()),
    }
}
