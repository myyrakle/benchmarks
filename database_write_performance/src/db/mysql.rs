use sqlx::MySqlPool;
use std::sync::Arc;

use super::{Database, Errors, Result};

#[derive(Debug)]
pub struct MySqlDB {
    pool: MySqlPool,
}

impl MySqlDB {
    pub async fn new() -> Result<Arc<dyn Database + Send + Sync>> {
        let connection_string = "mysql://user:q1w2e3r4@127.0.0.1:13306/benchmark";

        let pool = MySqlPool::connect(connection_string)
            .await
            .map_err(|_| Errors::ConnectionError)?;

        Ok(Arc::new(MySqlDB { pool }))
    }
}

#[async_trait::async_trait]
impl Database for MySqlDB {
    async fn ping(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(|_| Errors::ConnectionError)?;
        Ok(())
    }

    async fn setup(&self) -> Result<()> {
        // 테이블이 존재하면 삭제
        sqlx::query("DROP TABLE IF EXISTS key_value")
            .execute(&self.pool)
            .await
            .map_err(|_| Errors::WriteError)?;

        // 새 테이블 생성
        sqlx::query(
            "CREATE TABLE key_value (
                `key` VARCHAR(255) PRIMARY KEY,
                `value` TEXT NOT NULL
            )",
        )
        .execute(&self.pool)
        .await
        .map_err(|_| Errors::WriteError)?;

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
        .map_err(|_| Errors::WriteError)?;

        Ok(())
    }
}
