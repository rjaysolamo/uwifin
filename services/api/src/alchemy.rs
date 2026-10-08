use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlchemyConfig {
    pub api_key: String,
    pub network: String,
    pub policy_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendUserOperationRequest {
    pub sender: String,
    pub target: String,
    pub value: String,
    pub data: String,
    pub gas_limit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserOperationResponse {
    pub user_operation_hash: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptResponse {
    pub status: String,
    pub transaction_hash: Option<String>,
    pub block_number: Option<String>,
}

pub struct AlchemyClient {
    client: Client,
    config: AlchemyConfig,
}

impl AlchemyClient {
    pub fn new(api_key: &str, network: &str, policy_id: &str) -> Self {
        Self {
            client: Client::new(),
            config: AlchemyConfig {
                api_key: api_key.to_string(),
                network: network.to_string(),
                policy_id: policy_id.to_string(),
            },
        }
    }

    pub fn base_url(&self) -> String {
        format!(
            "https://{}.g.alchemy.com/v2/{}",
            self.config.network, self.config.api_key
        )
    }

    pub async fn send_user_operation(&self, payload: &SendUserOperationRequest) -> Result<UserOperationResponse> {
        let url = format!("{}/v2/{}", self.base_url(), self.config.api_key);

        let resp = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "method": "eth_sendUserOperation",
                "params": [
                    {
                        "sender": payload.sender,
                        "target": payload.target,
                        "value": payload.value,
                        "data": payload.data,
                        "gasLimit": payload.gas_limit.clone().unwrap_or_else(|| "0x0".to_string())
                    },
                    self.config.policy_id
                ],
                "id": 1
            }))
            .send()
            .await?;

        let body = resp.json::<serde_json::Value>().await?;

        Ok(UserOperationResponse {
            user_operation_hash: body["result"]
                .as_str()
                .unwrap_or("unknown")
                .to_string(),
            status: "submitted".to_string(),
        })
    }

    pub async fn get_transaction_receipt(&self, hash: &str) -> Result<ReceiptResponse> {
        let url = format!("{}/v2/{}", self.base_url(), self.config.api_key);

        let resp = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "method": "eth_getTransactionReceipt",
                "params": [hash],
                "id": 1
            }))
            .send()
            .await?;

        let body = resp.json::<serde_json::Value>().await?;
        let result = body.get("result").cloned().unwrap_or(serde_json::Value::Null);

        Ok(ReceiptResponse {
            status: if result.is_null() { "pending".to_string() } else { "confirmed".to_string() },
            transaction_hash: result.get("transactionHash").and_then(|v| v.as_str()).map(|s| s.to_string()),
            block_number: result.get("blockNumber").and_then(|v| v.as_str()).map(|s| s.to_string()),
        })
    }
}
