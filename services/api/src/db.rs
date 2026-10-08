use sqlx::MySqlPool;
pub async fn run_migrations(pool: &MySqlPool) -> anyhow::Result<()> {
    sqlx::migrate!("./migrations").run(pool).await?;
    Ok(())
}
