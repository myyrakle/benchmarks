use std::sync::Arc;

use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::db::{Database, Errors, Result};

#[derive(Clone)]
pub struct TimescaleDB {
    pool: PgPool,
}

impl TimescaleDB {
    pub async fn new() -> Result<Arc<dyn Database + Send + Sync>> {
        let connection_string = "postgresql://user:q1w2e3r4@localhost:25432/benchmark";

        let pool = PgPoolOptions::new()
            .max_connections(1000) // 최대 연결 수
            .min_connections(1000) // 최소 연결 수 (즉시 생성)
            .connect(connection_string)
            .await
            .map_err(|error| Errors::ConnectionError(error.to_string()))?;

        let db = TimescaleDB { pool };

        Ok(Arc::new(db))
    }
}

#[async_trait::async_trait]
impl Database for TimescaleDB {
    async fn ping(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                eprintln!("TimescaleDB ping error: {:?}", e);
                Errors::ConnectionError(format!("TimescaleDB ping failed: {}", e))
            })?;

        Ok(())
    }

    async fn setup(&self) -> Result<()> {
        // TimescaleDB 확장 활성화
        sqlx::query("CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE")
            .execute(&self.pool)
            .await
            .map_err(|e| {
                eprintln!("TimescaleDB extension creation error: {:?}", e);
                Errors::ConnectionError(format!("Failed to create TimescaleDB extension: {}", e))
            })?;

        // 기존 테이블 삭제
        sqlx::query("DROP TABLE IF EXISTS benchmark")
            .execute(&self.pool)
            .await
            .map_err(|e| {
                eprintln!("TimescaleDB table drop error: {:?}", e);
                Errors::WriteError
            })?;

        // 하이퍼테이블 생성
        sqlx::query(
            r#"
            CREATE TABLE benchmark (
                time TIMESTAMPTZ NOT NULL,
                key TEXT NOT NULL,
                value TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            eprintln!("TimescaleDB table creation error: {:?}", e);
            Errors::WriteError
        })?;

        // 하이퍼테이블로 변환
        sqlx::query("SELECT create_hypertable('benchmark', 'time')")
            .execute(&self.pool)
            .await
            .map_err(|e| {
                eprintln!("TimescaleDB hypertable creation error: {:?}", e);
                Errors::WriteError
            })?;

        Ok(())
    }

    async fn write(&self, key: &str, value: &str) -> Result<()> {
        sqlx::query("INSERT INTO benchmark (time, key, value) VALUES (NOW(), $1, $2)")
            .bind(key)
            .bind(value)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                eprintln!("TimescaleDB write error: {:?}", e);
                Errors::WriteError
            })?;

        Ok(())
    }
}
