use sqlx::{MySqlPool, mysql::MySqlPoolOptions};
use std::sync::Arc;

use super::{Database, Errors, Result};

#[derive(Debug)]
pub struct MariaDB {
    pool: MySqlPool,
}

impl MariaDB {
    pub async fn new() -> Result<Arc<dyn Database + Send + Sync>> {
        let connection_string = "mysql://user:q1w2e3r4@127.0.0.1:23306/benchmark";

        let pool = MySqlPoolOptions::new()
            .max_connections(1000) // 최대 연결 수
            .min_connections(1000) // 최소 연결 수 (즉시 생성)
            .connect(connection_string)
            .await
            .map_err(|error| Errors::ConnectionError(error.to_string()))?;

        Ok(Arc::new(MariaDB { pool }))
    }
}

#[async_trait::async_trait]
impl Database for MariaDB {
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
            .map_err(|e| Errors::WriteError(e.to_string()))?;

        // 새 테이블 생성
        sqlx::query(
            "CREATE TABLE key_value (
                `key` VARCHAR(255) PRIMARY KEY,
                `value` TEXT NOT NULL
            ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Errors::WriteError(e.to_string()))?;

        Ok(())
    }

    async fn write(&self, key: &str, value: &str) -> Result<()> {
        sqlx::query(
            "INSERT INTO key_value (`key`, `value`) VALUES (?, ?) 
             ON DUPLICATE KEY UPDATE `value` = VALUES(`value`)",
        )
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await
        .map_err(|e| Errors::WriteError(e.to_string()))?;

        Ok(())
    }
}
