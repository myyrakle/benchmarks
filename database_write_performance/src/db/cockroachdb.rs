use sqlx::{PgPool, postgres::PgPoolOptions};
use std::sync::Arc;

use super::{Database, Errors, Result};

#[derive(Debug)]
pub struct CockroachDB {
    pool: PgPool,
}

impl CockroachDB {
    pub async fn new() -> Result<Arc<dyn Database + Send + Sync>> {
        let connection_string = "postgres://root@127.0.0.1:26257/benchmark?sslmode=disable";

        let pool = PgPoolOptions::new()
            .max_connections(1000) // 최대 연결 수
            .min_connections(1000) // 최소 연결 수
            .connect(connection_string)
            .await
            .map_err(|error| Errors::ConnectionError(error.to_string()))?;

        Ok(Arc::new(CockroachDB { pool }))
    }
}

#[async_trait::async_trait]
impl Database for CockroachDB {
    async fn ping(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(|error| Errors::ConnectionError(error.to_string()))?;
        Ok(())
    }

    async fn setup(&self) -> Result<()> {
        // 데이터베이스 생성 (존재하지 않을 경우)
        sqlx::query("CREATE DATABASE IF NOT EXISTS benchmark")
            .execute(&self.pool)
            .await
            .map_err(|e| Errors::WriteError(e.to_string()))?;

        // 테이블이 존재하면 삭제
        sqlx::query("DROP TABLE IF EXISTS key_value")
            .execute(&self.pool)
            .await
            .map_err(|e| Errors::WriteError(e.to_string()))?;

        // 새 테이블 생성 (CockroachDB 최적화 포함)
        sqlx::query(
            "CREATE TABLE key_value (
                key STRING PRIMARY KEY,
                value STRING NOT NULL
            )",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Errors::WriteError(e.to_string()))?;

        Ok(())
    }

    async fn write(&self, key: &str, value: &str) -> Result<()> {
        // CockroachDB의 UPSERT 구문 사용
        sqlx::query("UPSERT INTO key_value (key, value) VALUES ($1, $2)")
            .bind(key)
            .bind(value)
            .execute(&self.pool)
            .await
            .map_err(|e| Errors::WriteError(e.to_string()))?;

        Ok(())
    }
}
