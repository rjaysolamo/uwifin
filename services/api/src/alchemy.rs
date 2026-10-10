use crate::{error::ApiError, state::AppState, wallet_validation::validate_wallet_address};
use serde_json::{json, Value};
async fn rpc(
    state: &AppState,
    url: Option<&str>,
    method: &str,
    params: Value,
) -> Result<Value, ApiError> {
    let url = url.ok_or_else(|| {
        ApiError::new(
            "PROVIDER_NOT_CONFIGURED",
            "Wallet services are not configured yet.",
        )
    })?;
    let response = state
        .http
        .post(url)
        .json(&json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params }))
        .send()
        .await
        .map_err(|_| ApiError::unavailable())?;
    if !response.status().is_success() {
        return Err(ApiError::unavailable());
    }
    let body: Value = response.json().await.map_err(|_| ApiError::unavailable())?;
    if body.get("error").is_some() {
        return Err(ApiError::new("PROVIDER_REJECTED", "The wallet provider could not complete this request. Check your wallet, network and sponsorship eligibility."));
    }
    body.get("result")
        .cloned()
        .ok_or_else(ApiError::unavailable)
}
pub async fn chain_rpc(state: &AppState, method: &str, params: Value) -> Result<Value, ApiError> {
    rpc(
        state,
        state.config.alchemy_rpc_url.as_deref(),
        method,
        params,
    )
    .await
}
pub async fn wallet_rpc(state: &AppState, method: &str, params: Value) -> Result<Value, ApiError> {
    rpc(
        state,
        state.config.alchemy_wallet_url.as_deref(),
        method,
        params,
    )
    .await
}
pub async fn verify_chain(state: &AppState) -> Result<(), ApiError> {
    let chain = chain_rpc(state, "eth_chainId", json!([])).await?;
    if hex_u64(&chain) != Some(state.config.chain_id) {
        return Err(ApiError::new(
            "UNSUPPORTED_NETWORK",
            "The blockchain provider is connected to a different network.",
        ));
    }
    Ok(())
}
pub async fn usdc_balance(state: &AppState, address: &str) -> Result<u128, ApiError> {
    let address = validate_wallet_address(address)?;
    verify_chain(state).await?;
    let data = format!("0x70a08231{:0>64}", &address[2..]);
    let result = chain_rpc(
        state,
        "eth_call",
        json!([{ "to": state.config.usdc_address, "data": data }, "latest"]),
    )
    .await?;
    let encoded = result
        .as_str()
        .and_then(|s| s.strip_prefix("0x"))
        .ok_or_else(ApiError::unavailable)?;
    u128::from_str_radix(encoded, 16).map_err(|_| ApiError::unavailable())
}
pub fn hex_u64(value: &Value) -> Option<u64> {
    value
        .as_str()
        .and_then(|s| s.strip_prefix("0x"))
        .and_then(|s| u64::from_str_radix(s, 16).ok())
}
#[derive(Debug, PartialEq)]
pub enum ReceiptState {
    Pending,
    Confirmed,
    Failed,
}
pub fn receipt_state(value: &Value) -> Result<ReceiptState, ApiError> {
    if value.is_null() {
        return Ok(ReceiptState::Pending);
    }
    match value.get("status").and_then(Value::as_str) {
        Some("0x1")
            if value
                .get("transactionHash")
                .and_then(Value::as_str)
                .is_some_and(valid_hash) =>
        {
            Ok(ReceiptState::Confirmed)
        }
        Some("0x0") => Ok(ReceiptState::Failed),
        _ => Err(ApiError::unavailable()),
    }
}
pub fn valid_hash(hash: &str) -> bool {
    hash.len() == 66 && hash.starts_with("0x") && hash[2..].bytes().all(|c| c.is_ascii_hexdigit())
}
pub fn contains_transfer(
    receipt: &Value,
    token: &str,
    sender: &str,
    recipient: &str,
    amount: i64,
) -> bool {
    if amount <= 0
        || validate_wallet_address(sender).is_err()
        || validate_wallet_address(recipient).is_err()
    {
        return false;
    }
    const TRANSFER: &str = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";
    let topic = |address: &str| format!("0x{:0>64}", &address[2..].to_ascii_lowercase());
    receipt
        .get("logs")
        .and_then(Value::as_array)
        .is_some_and(|logs| {
            logs.iter().any(|log| {
                log["address"]
                    .as_str()
                    .is_some_and(|address| address.eq_ignore_ascii_case(token))
                    && log["topics"][0].as_str() == Some(TRANSFER)
                    && log["topics"][1]
                        .as_str()
                        .is_some_and(|s| s.eq_ignore_ascii_case(&topic(sender)))
                    && log["topics"][2]
                        .as_str()
                        .is_some_and(|s| s.eq_ignore_ascii_case(&topic(recipient)))
                    && log["data"]
                        .as_str()
                        .and_then(|s| s.strip_prefix("0x"))
                        .and_then(|s| u128::from_str_radix(s, 16).ok())
                        == Some(amount as u128)
            })
        })
}
