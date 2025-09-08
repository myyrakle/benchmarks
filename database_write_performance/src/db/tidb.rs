use sqlx::{MySqlPool, mysql::MySqlPoolOptions};
use std::sync::Arc;

use super::{Database, Errors, Result};

#[derive(Debug)]
pub struct TiDB {
    pool: MySqlPool,
}

impl TiDB {
    pub async fn new() -> Result<Arc<dyn Database + Send + Sync>> {
        let connection_string = "mysql://root@127.0.0.1:4000/test";

        let pool = MySqlPoolOptions::new()
            .max_connections(1000) // TiDB에 적합한 연결 수
            .min_connections(500)
            .connect(connection_string)
            .await
            .map_err(|error| Errors::ConnectionError(error.to_string()))?;

        Ok(Arc::new(TiDB { pool }))
    }
}

#[async_trait::async_trait]
impl Database for TiDB {
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

        // 새 테이블 생성 (TiDB에 최적화)
        sqlx::query(
            "CREATE TABLE `key_value` (
                `key` VARCHAR(255) PRIMARY KEY,
                `value` VARCHAR(1000) NOT NULL
            )",
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
