use sqlx::{PgPool, postgres::PgPoolOptions};
use std::sync::Arc;

use super::{Database, Errors, Result};

#[derive(Debug)]
pub struct YugabyteDB {
    pool: PgPool,
}

impl YugabyteDB {
    pub async fn new() -> Result<Arc<dyn Database + Send + Sync>> {
        let connection_string = "postgres://yugabyte@127.0.0.1:15433/yugabyte";

        let pool = PgPoolOptions::new()
            .max_connections(1000) // YugabyteDB에 적합한 연결 수
            .min_connections(1000)
            .connect(connection_string)
            .await
            .map_err(|error| Errors::ConnectionError(error.to_string()))?;

        Ok(Arc::new(YugabyteDB { pool }))
    }
}

#[async_trait::async_trait]
impl Database for YugabyteDB {
    async fn ping(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(|error| Errors::ConnectionError(error.to_string()))?;
        Ok(())
    }

    async fn setup(&self) -> Result<()> {
        // 테이블이 존재하면 삭제
        sqlx::query("DROP TABLE IF EXISTS key_value")
            .execute(&self.pool)
            .await
            .map_err(|_| Errors::WriteError)?;

        // 새 테이블 생성 (YugabyteDB에 최적화)
        sqlx::query(
            "CREATE TABLE key_value (
                key VARCHAR(255) PRIMARY KEY,
                value TEXT NOT NULL
            )",
        )
        .execute(&self.pool)
        .await
        .map_err(|_| Errors::WriteError)?;

        Ok(())
    }

    async fn write(&self, key: &str, value: &str) -> Result<()> {
        sqlx::query(
            "INSERT INTO key_value (key, value) VALUES ($1, $2) 
             ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value",
        )
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await
        .map_err(|_| Errors::WriteError)?;

        Ok(())
    }
}
