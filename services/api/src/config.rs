/// Secrets are intentionally neither Debug nor Serialize.
#[derive(Clone)]
pub struct Config {
    pub app_env: String,
    pub app_base_url: String,
    pub database_url: String,
    pub alchemy_rpc_url: Option<String>,
    pub stripe_webhook_secret: Option<String>,
}
impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let app_env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".into());
        let app_base_url = std::env::var("APP_BASE_URL").unwrap_or_else(|_| "http://localhost:3000".into());
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "mysql://uwifin:uwifin@localhost:3306/uwifin".into());
        if app_env == "production" {
            anyhow::ensure!(app_base_url.starts_with("https://"), "Production requires an HTTPS APP_BASE_URL");
            anyhow::ensure!(std::env::var("DATABASE_URL").is_ok(), "Production requires DATABASE_URL");
        }
        let alchemy_rpc_url = std::env::var("ALCHEMY_RPC_URL").ok().or_else(|| {
            std::env::var("ALCHEMY_API_KEY").ok().filter(|key| !key.starts_with("your-") && !key.is_empty())
                .map(|key| format!("https://base-sepolia.g.alchemy.com/v2/{key}"))
        });
        if let Some(url) = &alchemy_rpc_url {
            let parsed = reqwest::Url::parse(url)?;
            anyhow::ensure!(parsed.scheme() == "https" || (app_env != "production" && parsed.scheme() == "http"), "Invalid RPC transport");
        }
        Ok(Self {
            app_env, app_base_url, database_url, alchemy_rpc_url,
            stripe_webhook_secret: std::env::var("STRIPE_WEBHOOK_SECRET").ok().filter(|secret| secret.starts_with("whsec_") && secret.len() > 16),
        })
    }
}
