use sqlx::mysql::MySqlPool;
use std::fs;
use std::path::Path;

pub async fn run_migrations(pool: &MySqlPool) -> anyhow::Result<()> {
    let migrations_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");

    if !migrations_dir.exists() {
        tracing::warn!("No migrations directory found at {:?}", migrations_dir);
        return Ok(());
    }

    let mut migration_paths = fs::read_dir(&migrations_dir)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .collect::<Vec<_>>();

    migration_paths.sort();

    for migration_path in migration_paths {
        if migration_path.extension().and_then(|ext| ext.to_str()) != Some("sql") {
            continue;
        }

        let migration_name = migration_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown");

        let sql = fs::read_to_string(&migration_path)?;
        let statements = split_sql_statements(&sql);

        for statement in statements {
            if statement.trim().is_empty() {
                continue;
            }

            sqlx::query(statement)
                .execute(pool)
                .await
                .map_err(|err| anyhow::anyhow!("Failed to execute migration {}: {}", migration_name, err))?;
        }

        tracing::info!("Applied migration: {}", migration_name);
    }

    Ok(())
}

fn split_sql_statements(sql: &str) -> Vec<&str> {
    sql.split(';')
        .filter(|statement| !statement.trim().is_empty())
        .collect()
}
