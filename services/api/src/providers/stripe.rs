use super::*;
use serde_json::{json, Value};
use crate::{payments::parse_minor, transaction::parse_amount};
pub struct StripeOnramp { config: Config, http: reqwest::Client }
impl StripeOnramp {
    pub fn new(config: &Config) -> Self {
        Self { config: config.clone(), http: reqwest::Client::builder().timeout(std::time::Duration::from_secs(15)).redirect(reqwest::redirect::Policy::none()).build().expect("HTTP configuration") }
    }
    async fn call(&self, method: reqwest::Method, path: &str, fields: &[(String,String)], key: Option<&str>) -> Result<Value,ApiError> {
        if !self.config.onramp_enabled() { return Err(ApiError::new("PROVIDER_NOT_CONFIGURED", "Funding is not available for this route.")); }
        let secret = self.config.stripe_secret_key.as_ref().ok_or_else(ApiError::unavailable)?;
        let mut req = self.http.request(method.clone(), format!("{}{path}", self.config.stripe_api_url)).basic_auth(secret, Some(""));
        if method == reqwest::Method::GET { req = req.query(fields); } else { req = req.form(fields); }
        if let Some(key) = key { req = req.header("Idempotency-Key", key); }
        let response = req.send().await.map_err(|_| ApiError::unavailable())?;
        if !response.status().is_success() { return Err(ApiError::unavailable()); }
        response.json().await.map_err(|_| ApiError::unavailable())
    }
    async fn session(&self, reference: &str) -> Result<Value,ApiError> {
        if !reference.starts_with("cos_") || reference.len() > 255 || !reference.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_') { return Err(ApiError::unavailable()); }
        let value = self.call(reqwest::Method::GET, &format!("/v1/crypto/onramp_sessions/{reference}"), &[], None).await?;
        if value["id"] != reference || value["object"] != "crypto.onramp_session" || value["livemode"].as_bool() != Some(self.config.stripe_live()) { return Err(ApiError::unavailable()); }
        Ok(value)
    }
    fn evidence(&self, value: &Value) -> Result<Evidence,ApiError> {
        let details = &value["transaction_details"];
        if value["livemode"].as_bool() != Some(self.config.stripe_live()) || details["destination_currency"] != "usdc" || details["destination_network"] != "base" || details["lock_wallet_address"] != true { return Err(ApiError::unavailable()); }
        let outcome = match value["status"].as_str() {
            Some("fulfillment_complete") => Outcome::Succeeded,
            Some("rejected") => Outcome::Failed,
            Some("initialized" | "requires_payment" | "fulfillment_processing") => Outcome::Pending,
            _ => Outcome::Unknown,
        };
        let amount = if outcome == Outcome::Succeeded { Some(parse_amount(details["destination_amount"].as_str().ok_or_else(ApiError::unavailable)?)?.to_string()) } else { None };
        Ok(Evidence { remittance_id: value["metadata"]["uwifin_remittance_id"].as_str().ok_or_else(ApiError::unavailable)?.into(), reference: value["id"].as_str().ok_or_else(ApiError::unavailable)?.into(), outcome, amount, currency: Some("USDC".into()), destination_reference: details["wallet_address"].as_str().map(str::to_owned), final_credit: false })
    }
}
#[async_trait]
impl OnrampProvider for StripeOnramp {
    fn account_id(&self) -> &str { if self.config.stripe_live() { "stripe-live" } else { "stripe-test" } }
    async fn quote(&self, currency: &str, principal_minor: i64) -> Result<FundingQuote,ApiError> {
        if !["USD","EUR"].contains(&currency) || principal_minor <= 0 { return Err(ApiError::new("INVALID_REQUEST", "Unsupported funding amount or currency.")); }
        let result = self.call(reqwest::Method::GET,"/v1/crypto/onramp_quotes",&[("source_currency".into(),currency.to_ascii_lowercase()),("source_amount".into(),format!("{}.{:02}",principal_minor/100,principal_minor%100)),("destination_currencies[]".into(),"usdc".into()),("destination_networks[]".into(),"base".into())],None).await?;
        // Response map labels differ from the destination_network enum; match fields.
        let quote = result["destination_network_quotes"].as_object().and_then(|networks| networks.values().filter_map(Value::as_array).flatten().find(|q| q["destination_currency"] == "usdc" && q["destination_network"] == "base")).ok_or_else(ApiError::unavailable)?;
        let exact = |field: &Value| field.as_str().ok_or_else(ApiError::unavailable);
        let total = parse_minor(exact(&quote["source_total_amount"])?)?;
        let fee = |field: &Value| -> Result<i64,ApiError> { let value=exact(field)?; if value == "0.00" || value == "0" { Ok(0) } else { parse_minor(value) } };
        let transaction_fee=fee(&quote["fees"]["transaction_fee_monetary"])?;
        let network_fee=fee(&quote["fees"]["network_fee_monetary"])?;
        if principal_minor.checked_add(transaction_fee).and_then(|v|v.checked_add(network_fee)) != Some(total) { return Err(ApiError::unavailable()); }
        Ok(FundingQuote { principal_minor: principal_minor.to_string(), total_minor: total.to_string(), crypto_atomic: parse_amount(exact(&quote["destination_amount"])?)?.to_string(), reference: exact(&quote["id"])?.into(), fees: vec![Fee{name:"On-ramp fee".into(),currency:currency.into(),amount_minor:transaction_fee.to_string(),included:false},Fee{name:"On-ramp network fee".into(),currency:currency.into(),amount_minor:network_fee.to_string(),included:false}] })
    }
    async fn create(&self, key: &str, command: &Command, customer_wallet: &str) -> Result<Evidence,ApiError> {
        crate::wallet_validation::validate_wallet_address(customer_wallet)?;
        let minor=command.quote.source_principal_minor.parse::<i64>().map_err(|_|ApiError::unavailable())?;
        let fields=vec![("source_currency".into(),command.quote.source_currency.to_ascii_lowercase()),("source_amount".into(),format!("{}.{:02}",minor/100,minor%100)),("destination_currencies[]".into(),"usdc".into()),("destination_networks[]".into(),"base".into()),("destination_currency".into(),"usdc".into()),("destination_network".into(),"base".into()),("wallet_addresses[base]".into(),customer_wallet.into()),("lock_wallet_address".into(),"true".into()),("metadata[uwifin_remittance_id]".into(),command.remittance_id.clone())];
        let value=self.call(reqwest::Method::POST,"/v1/crypto/onramp_sessions",&fields,Some(key)).await?;
        let evidence=self.evidence(&value)?;
        if evidence.remittance_id != command.remittance_id || evidence.destination_reference.as_deref().is_none_or(|a| !a.eq_ignore_ascii_case(customer_wallet)) { return Err(ApiError::unavailable()); }
        Ok(evidence)
    }
    async fn status(&self, reference: &str) -> Result<Evidence,ApiError> { self.evidence(&self.session(reference).await?) }
    async fn action(&self, reference: &str) -> Result<FundingAction,ApiError> {
        let value=self.session(reference).await?;
        if !matches!(value["status"].as_str(),Some("initialized"|"requires_payment")) { return Err(ApiError::new("CONFLICT", "This checkout is no longer awaiting payment.")); }
        Ok(FundingAction{client_secret:value["client_secret"].as_str().ok_or_else(ApiError::unavailable)?.into(),publishable_key:self.config.stripe_publishable_key.clone().ok_or_else(ApiError::unavailable)?})
    }
}
/// Only authenticated Stripe event IDs are persisted; reconciliation retrieves current state.
pub struct StripeWebhook { pub secret: String, pub live: bool }
#[async_trait]
impl WebhookVerifier for StripeWebhook {
    async fn verify(&self, headers: &axum::http::HeaderMap, raw: &[u8]) -> Result<VerifiedEvent,ApiError> {
        let signature=headers.get("stripe-signature").and_then(|v|v.to_str().ok()).unwrap_or("");
        if !crate::payments::verify_signature(&self.secret,signature,raw,chrono::Utc::now().timestamp()) { return Err(ApiError::new("INVALID_REQUEST","Invalid webhook signature.")); }
        let body:Value=serde_json::from_slice(raw).map_err(|_|ApiError::new("INVALID_REQUEST","Invalid webhook payload."))?;
        if body["livemode"] != json!(self.live) || body["type"] != "crypto.onramp_session_updated" { return Err(ApiError::new("INVALID_REQUEST","Unexpected webhook mode or event.")); }
        Ok(VerifiedEvent{id:body["id"].as_str().ok_or_else(ApiError::unavailable)?.into(),reference:body["data"]["object"]["id"].as_str().ok_or_else(ApiError::unavailable)?.into(),kind:"crypto.onramp_session_updated".into()})
    }
}
