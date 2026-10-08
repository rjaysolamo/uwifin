use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub app_env: String,
    pub app_base_url: String,
    pub database_url: String,
    pub session_secret: String,
    pub alchemy_api_key: String,
    pub alchemy_policy_id: String,
    pub alchemy_network: String,
    pub stripe_secret_key: String,
    pub stripe_webhook_secret: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            app_env: std::env::var("APP_ENV").unwrap_or("development".to_string()),
            app_base_url: std::env::var("APP_BASE_URL").unwrap_or("http://localhost:3000".to_string()),
            database_url: std::env::var("DATABASE_URL")
                .or_else(|_| std::env::var("DATABASE_URL"))
                .unwrap_or("mysql://uwifin:uwifin@localhost:3306/uwifin".to_string()),
            session_secret: std::env::var("SESSION_SECRET")
                .or_else(|_| Err(anyhow::anyhow!("SESSION_SECRET not set")))
                .or_else(|_| Ok("dev-secret-change-me-in-production".to_string()))?,
            alchemy_api_key: std::env::var("ALCHEMY_API_KEY")
                .unwrap_or("your-alchemy-api-key".to_string()),
            alchemy_policy_id: std::env::var("ALCHEMY_POLICY_ID")
                .unwrap_or("your-alchemy-policy-id".to_string()),
            alchemy_network: std::env::var("ALCHEMY_NETWORK")
                .unwrap_or("base-sepolia".to_string()),
            stripe_secret_key: std::env::var("STRIPE_SECRET_KEY")
                .unwrap_or("your-stripe-secret-key".to_string()),
            stripe_webhook_secret: std::env::var("STRIPE_WEBHOOK_SECRET")
                .unwrap_or("your-stripe-webhook-secret".to_string()),
        })
    }
}
