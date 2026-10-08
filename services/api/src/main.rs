use uwifin_api::{app, config::Config, db, state::AppState};
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt().json().init();
    let config = Config::from_env()?;
    let pool = sqlx::mysql::MySqlPoolOptions::new().max_connections(10).acquire_timeout(std::time::Duration::from_secs(5)).connect(&config.database_url).await?;
    db::run_migrations(&pool).await?;
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    tracing::info!("UwiFin API listening on port 8080; financial providers require configuration");
    axum::serve(listener, app(AppState::new(pool, config)).into_make_service_with_connect_info::<std::net::SocketAddr>()).with_graceful_shutdown(async { let _ = tokio::signal::ctrl_c().await; }).await?;
    Ok(())
}
