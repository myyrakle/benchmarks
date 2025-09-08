use scylla::{Session, SessionBuilder};
use std::sync::Arc;

use super::{Database, Errors, Result};

#[derive(Debug)]
pub struct ScyllaDB {
    session: Session,
}

impl ScyllaDB {
    pub async fn new() -> Result<Arc<dyn Database + Send + Sync>> {
        let session = SessionBuilder::new()
            .known_node("127.0.0.1:19043")
            .build()
            .await
            .map_err(|error| Errors::ConnectionError(error.to_string()))?;

        Ok(Arc::new(ScyllaDB { session }))
    }
}

#[async_trait::async_trait]
impl Database for ScyllaDB {
    async fn ping(&self) -> Result<()> {
        // Cassandra의 system.local 테이블을 쿼리하여 연결 테스트
        let _result = self
            .session
            .query_unpaged("SELECT cluster_name FROM system.local", ())
            .await
            .map_err(|error| Errors::ConnectionError(error.to_string()))?;
        Ok(())
    }

    async fn setup(&self) -> Result<()> {
        // 키스페이스 생성
        self.session
            .query_unpaged(
                "CREATE KEYSPACE IF NOT EXISTS benchmark 
                 WITH REPLICATION = {
                     'class': 'SimpleStrategy',
                     'replication_factor': 1
                 }",
                (),
            )
            .await
            .map_err(|e| Errors::WriteError(e.to_string()))?;

        // 키스페이스 사용
        self.session
            .use_keyspace("benchmark", false)
            .await
            .map_err(|e| Errors::WriteError(e.to_string()))?;

        // 기존 테이블 삭제 (있다면)
        self.session
            .query_unpaged("DROP TABLE IF EXISTS key_value", ())
            .await
            .map_err(|e| Errors::WriteError(e.to_string()))?;

        // 새 테이블 생성
        self.session
            .query_unpaged(
                "CREATE TABLE key_value (
                    key TEXT PRIMARY KEY,
                    value TEXT
                )",
                (),
            )
            .await
            .map_err(|e| Errors::WriteError(e.to_string()))?;

        Ok(())
    }

    async fn write(&self, key: &str, value: &str) -> Result<()> {
        self.session
            .query_unpaged(
                "INSERT INTO key_value (key, value) VALUES (?, ?)",
                (key, value),
            )
            .await
            .map_err(|e| Errors::WriteError(e.to_string()))?;

        Ok(())
    }
}
