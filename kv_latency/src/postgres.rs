use futures::executor::block_on;
use sqlx::postgres::PgPoolOptions;

pub fn create_postgres_client() -> anyhow::Result<sqlx::Pool<sqlx::Postgres>> {
    block_on(async {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect("postgres://testuser:testpassword@localhost:15432/testdb")
            .await?;

        Ok(pool)
    })
}

pub fn init_schema(pool: &sqlx::Pool<sqlx::Postgres>) -> anyhow::Result<()> {
    block_on(async {
        sqlx::query(
            r#"
        CREATE TABLE IF NOT EXISTS kv_store (
            id SERIAL PRIMARY KEY,
            key TEXT NOT NULL,
            value TEXT NOT NULL
        );
        "#,
        )
        .execute(pool)
        .await?;

        sqlx::query(
            r#"
        CREATE UNIQUE INDEX IF NOT EXISTS idx_key ON kv_store (key);
        "#,
        )
        .execute(pool)
        .await?;

        Ok(())
    })
}

pub fn set_key_value(
    pool: &sqlx::Pool<sqlx::Postgres>,
    key: &str,
    value: &str,
) -> anyhow::Result<()> {
    block_on(async {
        sqlx::query(
            r#"
        INSERT INTO kv_store (key, value)
        VALUES ($1, $2)
        ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value;
        "#,
        )
        .bind(key)
        .bind(value)
        .execute(pool)
        .await?;

        Ok(())
    })
}

pub fn get_key_value(pool: &sqlx::Pool<sqlx::Postgres>, key: &str) -> anyhow::Result<String> {
    block_on(async {
        let value: String = sqlx::query_scalar(
            r#"
        SELECT value FROM kv_store WHERE key = $1;
        "#,
        )
        .bind(key)
        .fetch_one(pool)
        .await?;

        Ok(value)
    })
}
