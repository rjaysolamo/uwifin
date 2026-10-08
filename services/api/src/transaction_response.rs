use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum TransactionStatus {
    #[default]
    CREATED,
    VALIDATING,
    READY,
    SUBMITTED,
    PENDING,
    CONFIRMED,
    FAILED,
    REJECTED,
    CANCELLED,
}

impl TransactionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TransactionStatus::CREATED => "CREATED",
            TransactionStatus::VALIDATING => "VALIDATING",
            TransactionStatus::READY => "READY",
            TransactionStatus::SUBMITTED => "SUBMITTED",
            TransactionStatus::PENDING => "PENDING",
            TransactionStatus::CONFIRMED => "CONFIRMED",
            TransactionStatus::FAILED => "FAILED",
            TransactionStatus::REJECTED => "REJECTED",
            TransactionStatus::CANCELLED => "CANCELLED",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTransactionRequest {
    pub wallet_id: String,
    pub network: String,
    pub asset: String,
    pub recipient: String,
    pub amount: String,
    pub idempotency_key: String,
}

impl CreateTransactionRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.wallet_id.trim().is_empty() {
            return Err("wallet_id is required".to_string());
        }
        if self.network.trim().is_empty() {
            return Err("network is required".to_string());
        }
        if self.asset.trim().is_empty() {
            return Err("asset is required".to_string());
        }
        if self.recipient.trim().is_empty() {
            return Err("recipient is required".to_string());
        }
        if self.amount.trim().is_empty() || self.amount.parse::<f64>().unwrap_or(-1.0) <= 0.0 {
            return Err("amount must be greater than zero".to_string());
        }
        if self.idempotency_key.trim().is_empty() {
            return Err("idempotency_key is required".to_string());
        }
        Ok(())
    }
}

pub fn new_transaction_id() -> String {
    Uuid::new_v4().to_string()
}
