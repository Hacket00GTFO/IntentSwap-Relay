use std::env;

use anyhow::{Context, Result};
use sqlx::PgPool;

pub async fn init_pool() -> Result<PgPool> {
    let database_url = env::var("DATABASE_URL").context("DATABASE_URL is required")?;

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(15)
        .connect(&database_url)
        .await
        .context("failed to connect to PostgreSQL")?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("failed to run SQL migrations")?;

    seed_relayers(&pool).await?;

    Ok(pool)
}

pub async fn seed_relayers(pool: &PgPool) -> Result<()> {
    let whitelist = env::var("RELAY_ALLOWED_RELAYERS").unwrap_or_else(|_| {
        "0x1111111111111111111111111111111111111111:RelayOne,0x2222222222222222222222222222222222222222:RelayTwo"
            .to_string()
    });

    for item in whitelist.split(',') {
        let trimmed = item.trim();
        if trimmed.is_empty() {
            continue;
        }

        let mut parts = trimmed.split(':');
        let address = parts.next().unwrap_or_default().trim().to_lowercase();
        let name = parts.next().unwrap_or("UnnamedRelayer").trim();

        if address.is_empty() {
            continue;
        }

        sqlx::query(
            "
            INSERT INTO relayers (address, name, reputation_score, total_executed, total_volume, is_active)
            VALUES ($1, $2, 0, 0, '0', TRUE)
            ON CONFLICT (address) DO UPDATE SET
              name = EXCLUDED.name
            ",
        )
        .bind(address)
        .bind(name)
        .execute(pool)
        .await
        .context("failed to seed relayer")?;
    }

    Ok(())
}
