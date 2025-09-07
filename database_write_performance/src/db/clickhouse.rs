use clickhouse::Client;
use std::sync::Arc;

use super::{Database, Errors, Result};

pub struct ClickHouse {
    client: Client,
}

impl ClickHouse {
    pub async fn new() -> Result<Arc<dyn Database + Send + Sync>> {
        let client = Client::default()
            .with_url("http://localhost:8123")
            .with_user("default")
            .with_database("benchmark");

        Ok(Arc::new(ClickHouse { client }))
    }
}

#[async_trait::async_trait]
impl Database for ClickHouse {
    async fn ping(&self) -> Result<()> {
        let result: u32 = self
            .client
            .query("SELECT 1")
            .fetch_one()
            .await
            .map_err(|e| Errors::ConnectionError(e.to_string()))?;

        if result == 1 {
            Ok(())
        } else {
            Err(Errors::ConnectionError("Ping failed".to_string()))
        }
    }

    async fn setup(&self) -> Result<()> {
        // 데이터베이스 생성 (존재하지 않을 경우)
        self.client
            .query("CREATE DATABASE IF NOT EXISTS benchmark")
            .execute()
            .await
            .map_err(|e| Errors::ConnectionError(e.to_string()))?;

        // 테이블이 존재하면 삭제
        self.client
            .query("DROP TABLE IF EXISTS benchmark.key_value")
            .execute()
            .await
            .map_err(|e| Errors::ConnectionError(e.to_string()))?;

        // 새 테이블 생성 (ClickHouse 최적화 포함)
        self.client
            .query(
                "CREATE TABLE benchmark.key_value (
                    key String,
                    value String,
                    created_at DateTime DEFAULT now()
                ) ENGINE = MergeTree()
                ORDER BY key
                SETTINGS index_granularity = 8192",
            )
            .execute()
            .await
            .map_err(|e| Errors::ConnectionError(e.to_string()))?;

        Ok(())
    }

    async fn write(&self, key: &str, value: &str) -> Result<()> {
        self.client
            .query("INSERT INTO benchmark.key_value (key, value) VALUES (?, ?)")
            .bind(key)
            .bind(value)
            .execute()
            .await
            .map_err(|_| Errors::WriteError)?;

        Ok(())
    }
}
