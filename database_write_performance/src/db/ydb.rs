use std::sync::Arc;

use super::{Database, Errors, Result};

pub struct YDB {
    // YDB 클라이언트를 사용하는 대신 HTTP API 사용을 위한 placeholder
    endpoint: String,
}

impl std::fmt::Debug for YDB {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("YDB")
            .field("endpoint", &self.endpoint)
            .finish()
    }
}

impl YDB {
    pub async fn new() -> Result<Arc<dyn Database + Send + Sync>> {
        // YDB HTTP API를 사용하여 간단한 구현
        let endpoint = "http://127.0.0.1:8765".to_string();

        // 연결 테스트
        let client = reqwest::Client::new();
        let _response = client
            .get(&format!("{}/viewer/json/query", endpoint))
            .send()
            .await
            .map_err(|error| Errors::ConnectionError(error.to_string()))?;

        Ok(Arc::new(YDB { endpoint }))
    }
}

#[async_trait::async_trait]
impl Database for YDB {
    async fn ping(&self) -> Result<()> {
        // YDB monitoring endpoint를 통한 ping
        let client = reqwest::Client::new();
        let _response = client
            .get(&format!("{}/viewer/json/query", self.endpoint))
            .send()
            .await
            .map_err(|error| Errors::ConnectionError(error.to_string()))?;

        Ok(())
    }

    async fn setup(&self) -> Result<()> {
        // YDB HTTP API를 통한 테이블 생성
        let client = reqwest::Client::new();

        // 테이블 삭제
        let drop_query = "DROP TABLE IF EXISTS benchmark_kv";
        let _response = client
            .post(&format!("{}/viewer/json/query", self.endpoint))
            .json(&serde_json::json!({
                "query": drop_query
            }))
            .send()
            .await
            .map_err(|_| Errors::WriteError)?;

        // 테이블 생성
        let create_query = "CREATE TABLE benchmark_kv (
                key Utf8 NOT NULL,
                value Utf8,
                PRIMARY KEY (key)
            )";

        let _response = client
            .post(&format!("{}/viewer/json/query", self.endpoint))
            .json(&serde_json::json!({
                "query": create_query
            }))
            .send()
            .await
            .map_err(|_| Errors::WriteError)?;

        Ok(())
    }

    async fn write(&self, key: &str, value: &str) -> Result<()> {
        let client = reqwest::Client::new();

        let query = format!(
            "UPSERT INTO benchmark_kv (key, value) VALUES ('{}', '{}')",
            key.replace("'", "''"), // SQL injection 방지를 위한 간단한 이스케이프
            value.replace("'", "''")
        );

        let _response = client
            .post(&format!("{}/viewer/json/query", self.endpoint))
            .json(&serde_json::json!({
                "query": query
            }))
            .send()
            .await
            .map_err(|_| Errors::WriteError)?;

        Ok(())
    }
}
