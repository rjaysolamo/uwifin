//! Provider boundaries. Production never registers synthetic adapters.
use axum::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::{config::Config, error::ApiError};
pub mod stripe;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Fee { pub name: String, pub currency: String, pub amount_minor: String, pub included: bool }
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Quote {
    pub source_currency: String,
    pub source_principal_minor: String,
    pub source_total_minor: String,
    pub crypto_amount_atomic: String,
    pub recipient_php_minor: String,
    pub fees: Vec<Fee>,
    pub php_per_usdc: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub estimated: bool,
    pub delivery_estimate: String,
    pub provider_quote_reference: String,
    pub chain_id: u64,
    pub asset_contract: String,
    pub destination_reference: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecipientDestination { pub reference: String, pub masked: String, pub method: String, pub valid_until: chrono::DateTime<chrono::Utc> }
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Command { pub remittance_id: String, pub user_id: String, pub funding_account: String, pub recipient_reference: String, pub quote: Quote }
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Outcome { Pending, Succeeded, Failed, Unknown }
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Evidence {
    pub remittance_id: String,
    pub reference: String,
    pub outcome: Outcome,
    pub amount: Option<String>,
    pub currency: Option<String>,
    pub destination_reference: Option<String>,
    pub final_credit: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FundingQuote { pub principal_minor: String, pub total_minor: String, pub crypto_atomic: String, pub fees: Vec<Fee>, pub reference: String }
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FundingAction { pub client_secret: String, pub publishable_key: String }
#[derive(Clone, Debug)]
pub struct VerifiedEvent { pub id: String, pub reference: String, pub kind: String }

#[async_trait]
pub trait OnrampProvider: Send + Sync {
    fn account_id(&self) -> &str;
    async fn quote(&self, currency: &str, principal_minor: i64) -> Result<FundingQuote, ApiError>;
    async fn create(&self, key: &str, command: &Command, customer_wallet: &str) -> Result<Evidence, ApiError>;
    async fn status(&self, reference: &str) -> Result<Evidence, ApiError>;
    async fn action(&self, reference: &str) -> Result<FundingAction, ApiError>;
}
#[async_trait]
pub trait OfframpProvider: Send + Sync {
    fn account_id(&self) -> &str;
    async fn quote(&self, funding: &FundingQuote, recipient: &str) -> Result<Quote, ApiError>;
    async fn convert(&self, key: &str, command: &Command) -> Result<Evidence, ApiError>;
    async fn status(&self, reference: &str) -> Result<Evidence, ApiError>;
}
#[async_trait]
pub trait PayoutProvider: Send + Sync {
    fn account_id(&self) -> &str;
    async fn recipient(&self, enrollment_reference: &str, user_id: &str) -> Result<RecipientDestination, ApiError>;
    async fn pay(&self, key: &str, command: &Command) -> Result<Evidence, ApiError>;
    async fn status(&self, reference: &str) -> Result<Evidence, ApiError>;
}
#[async_trait]
pub trait BlockchainProvider: Send + Sync {
    fn account_id(&self) -> &str;
    async fn transfer(&self, key: &str, command: &Command) -> Result<Evidence, ApiError>;
    async fn status(&self, reference: &str) -> Result<Evidence, ApiError>;
}
#[async_trait]
pub trait WalletProvider: Send + Sync {
    /// Must be an approved end-customer account; never an arbitrary browser address.
    async fn customer_account(&self, user_id: &str) -> Result<String, ApiError>;
    /// Provider consent and policy checks must cover this exact accepted intent.
    async fn authorized(&self, command: &Command) -> Result<bool, ApiError>;
}
#[async_trait]
pub trait ComplianceProvider: Send + Sync {
    async fn permitted(&self, command: &Command) -> Result<bool, ApiError>;
}
#[async_trait]
pub trait WebhookVerifier: Send + Sync {
    /// Verify raw bytes, account/mode, timestamp and key rotation before returning IDs.
    async fn verify(&self, headers: &axum::http::HeaderMap, raw: &[u8]) -> Result<VerifiedEvent, ApiError>;
}

pub struct Corridor {
    pub id: String, pub sender_country: String, pub source_currency: String,
    pub payout_methods: Vec<String>, pub bundle_id: String,
    /// Documented replay retention. Unknown outcomes never replay automatically.
    pub approved: bool,
}
pub struct Providers {
    pub onramp: Arc<dyn OnrampProvider>,
    pub offramp: Option<Arc<dyn OfframpProvider>>,
    pub payout: Option<Arc<dyn PayoutProvider>>,
    pub blockchain: Option<Arc<dyn BlockchainProvider>>,
    pub wallet: Option<Arc<dyn WalletProvider>>,
    pub compliance: Option<Arc<dyn ComplianceProvider>>,
    pub corridor: Option<Corridor>,
    pub verifiers: std::collections::HashMap<String, Arc<dyn WebhookVerifier>>,
}
impl Providers {
    pub fn new(config: &Config) -> Self {
        Self { onramp: Arc::new(stripe::StripeOnramp::new(config)), offramp: None, payout: None, blockchain: None, wallet: None, compliance: None, corridor: None, verifiers: Default::default() }
    }
    pub fn ready(&self) -> bool {
        self.corridor.as_ref().is_some_and(|c| c.approved) && self.offramp.is_some() && self.payout.is_some() && self.blockchain.is_some() && self.wallet.is_some() && self.compliance.is_some()
    }
    pub fn require(&self) -> Result<&Corridor, ApiError> {
        if !self.ready() { return Err(ApiError::new("CORRIDOR_UNAVAILABLE", "Transfers to the Philippines are not available yet. No payment has been taken.")); }
        self.corridor.as_ref().ok_or_else(ApiError::unavailable)
    }
    pub fn matches(&self, bundle: &str) -> bool { self.ready() && self.corridor.as_ref().is_some_and(|c| c.bundle_id == bundle) }
}
