use clickhouse::Client;
use std::sync::Arc;

use super::{Database, Errors, Result};

pub struct ClickHouse {
    client: Client,
}

impl ClickHouse {
    pub async fn new() -> Result<Arc<dyn Database + Send + Sync>> {
        let client = Client::default()
            .with_url("http://127.0.0.1:18123")
            .with_user("user")
            .with_password("q1w2e3r4")
            .with_database("benchmark");

        Ok(Arc::new(ClickHouse { client }))
    }
}

#[async_trait::async_trait]
impl Database for ClickHouse {
    async fn ping(&self) -> Result<()> {
        // 단순히 쿼리 실행이 성공하는지 확인
        self.client
            .query("SELECT 1")
            .execute()
            .await
            .map_err(|e| Errors::ConnectionError(e.to_string()))?;

        Ok(())
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
            .query("INSERT INTO benchmark.key_value (key, value) SETTINGS async_insert=1, wait_for_async_insert=1 VALUES (?, ?)")
            .bind(key)
            .bind(value)
            .execute()
            .await
            .map_err(|e| Errors::WriteError(e.to_string()))?;

        Ok(())
    }
}
