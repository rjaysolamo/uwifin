/// Secrets are intentionally neither Debug nor Serialize.
#[derive(Clone)]
pub struct Config {
    pub app_env: String,
    pub app_base_url: String,
    pub database_url: String,
    pub network: String,
    pub chain_id: u64,
    pub usdc_address: String,
    pub alchemy_rpc_url: Option<String>,
    pub alchemy_wallet_url: Option<String>,
    pub alchemy_policy_id: Option<String>,
    pub stripe_api_url: String,
    pub stripe_secret_key: Option<String>,
    pub stripe_publishable_key: Option<String>,
    pub stripe_webhook_secret: Option<String>,
    pub stripe_onramp_enabled: bool,
    pub confirmations: u64,
    pub transfer_limit_atomic: i64,
}
impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let value = |key: &str| std::env::var(key).ok().filter(|v| !v.is_empty() && !v.starts_with("your-") && !v.starts_with("change-me"));
        let app_env = value("APP_ENV").unwrap_or_else(|| "development".into());
        let app_base_url = value("APP_BASE_URL").unwrap_or_else(|| "http://localhost:3000".into());
        let network = value("ALCHEMY_NETWORK").unwrap_or_else(|| "base-sepolia".into());
        let (chain_id, usdc_address) = match network.as_str() {
            "base" => (8453, "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913"),
            "base-sepolia" => (84532, "0x036CbD53842c5426634e7929541eC2318f3dCF7e"),
            _ => anyhow::bail!("Only Base or Base Sepolia may be configured"),
        };
        if app_env == "production" {
            anyhow::ensure!(app_base_url.starts_with("https://"), "Production requires an HTTPS APP_BASE_URL");
            anyhow::ensure!(value("DATABASE_URL").is_some(), "Production requires DATABASE_URL");
        }
        let alchemy_key = value("ALCHEMY_API_KEY");
        let alchemy_rpc_url = value("ALCHEMY_RPC_URL").or_else(|| alchemy_key.as_ref().map(|key| format!("https://{network}.g.alchemy.com/v2/{key}")));
        let alchemy_wallet_url = value("ALCHEMY_WALLET_URL").or_else(|| alchemy_key.map(|key| format!("https://api.g.alchemy.com/v2/{key}")));
        let stripe_api_url = value("STRIPE_API_URL").unwrap_or_else(|| "https://api.stripe.com".into());
        for endpoint in alchemy_rpc_url.iter().chain(alchemy_wallet_url.iter()).chain(std::iter::once(&stripe_api_url)) {
            let parsed = reqwest::Url::parse(endpoint)?;
            anyhow::ensure!(parsed.scheme() == "https" || (app_env != "production" && parsed.scheme() == "http" && matches!(parsed.host_str(), Some("localhost" | "127.0.0.1"))), "Providers require HTTPS, except localhost development fixtures");
        }
        if app_env == "production" {
            anyhow::ensure!(stripe_api_url == "https://api.stripe.com", "Production Stripe traffic must use api.stripe.com");
        }
        let stripe_secret_key = value("STRIPE_SECRET_KEY");
        let stripe_publishable_key = value("STRIPE_PUBLISHABLE_KEY");
        if let Some(secret) = &stripe_secret_key {
            anyhow::ensure!(secret.starts_with("sk_test_") || secret.starts_with("sk_live_"), "Invalid Stripe key type");
            if let Some(public) = &stripe_publishable_key {
                anyhow::ensure!((secret.starts_with("sk_live_") && public.starts_with("pk_live_")) || (secret.starts_with("sk_test_") && public.starts_with("pk_test_")), "Stripe key modes must agree");
            }
        }
        let confirmations = value("BLOCK_CONFIRMATIONS").unwrap_or_else(|| "3".into()).parse::<u64>()?;
        anyhow::ensure!((1..=100).contains(&confirmations), "BLOCK_CONFIRMATIONS must be 1 to 100");
        let transfer_limit_atomic = value("MAX_TRANSFER_ATOMIC").unwrap_or_else(|| "1000000000".into()).parse::<i64>()?;
        anyhow::ensure!(transfer_limit_atomic > 0, "MAX_TRANSFER_ATOMIC must be positive");
        Ok(Self {
            database_url: value("DATABASE_URL").unwrap_or_else(|| "mysql://uwifin:uwifin@localhost:3306/uwifin".into()),
            app_env, app_base_url, network, chain_id, usdc_address: usdc_address.into(), alchemy_rpc_url, alchemy_wallet_url,
            alchemy_policy_id: value("ALCHEMY_POLICY_ID"), stripe_api_url, stripe_secret_key, stripe_publishable_key,
            stripe_webhook_secret: value("STRIPE_WEBHOOK_SECRET").filter(|secret| secret.starts_with("whsec_") && secret.len() > 16),
            stripe_onramp_enabled: value("STRIPE_ONRAMP_ENABLED").as_deref() == Some("true"), confirmations, transfer_limit_atomic,
        })
    }
    pub fn wallet_enabled(&self) -> bool { self.alchemy_rpc_url.is_some() && self.alchemy_wallet_url.is_some() }
    pub fn onramp_enabled(&self) -> bool {
        self.stripe_onramp_enabled && self.network == "base" && self.stripe_secret_key.is_some() && self.stripe_publishable_key.is_some() && self.stripe_webhook_secret.is_some()
    }
    pub fn stripe_live(&self) -> bool { self.stripe_secret_key.as_ref().is_some_and(|key| key.starts_with("sk_live_")) }
}
