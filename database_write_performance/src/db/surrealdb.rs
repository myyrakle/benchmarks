use serde::{Deserialize, Serialize};
use std::sync::Arc;
use surrealdb::Surreal;
use surrealdb::engine::remote::http::Client;
use surrealdb::engine::remote::http::Http;
use surrealdb::opt::auth::Root;

use super::{Database, Errors, Result};

#[derive(Serialize, Deserialize, Debug)]
struct BenchmarkRecord {
    value: String,
}

pub struct SurrealDB {
    db: Surreal<Client>,
}

impl SurrealDB {
    pub async fn new() -> Result<Arc<dyn Database + Send + Sync>> {
        // SurrealDB HTTP 연결
        let db = Surreal::new::<Http>("127.0.0.1:8000")
            .with_capacity(500)
            .await
            .map_err(|error| Errors::ConnectionError(error.to_string()))?;

        // 인증
        db.signin(Root {
            username: "user",
            password: "q1w2e3r4",
        })
        .await
        .map_err(|error| Errors::ConnectionError(error.to_string()))?;

        // 네임스페이스와 데이터베이스 선택
        db.use_ns("benchmark")
            .use_db("benchmark")
            .await
            .map_err(|error| Errors::ConnectionError(error.to_string()))?;

        Ok(Arc::new(SurrealDB { db }))
    }
}

#[async_trait::async_trait]
impl Database for SurrealDB {
    async fn ping(&self) -> Result<()> {
        // SurrealDB health check - 간단한 버전 확인 쿼리
        let result = self
            .db
            .version()
            .await
            .map_err(|error| Errors::ConnectionError(error.to_string()))?;

        println!("SurrealDB version: {}", result);

        Ok(())
    }

    async fn setup(&self) -> Result<()> {
        // 기존 benchmark 테이블의 모든 레코드 삭제
        // SurrealDB에서는 쿼리를 사용해서 테이블을 비웁니다
        let _result = self
            .db
            .query("DELETE benchmark")
            .await
            .map_err(|error| Errors::ConnectionError(error.to_string()))?;

        Ok(())
    }

    async fn write(&self, key: &str, value: &str) -> Result<()> {
        // SurrealDB에서는 쿼리를 사용해서 레코드를 생성하거나 업데이트
        let _result: Option<BenchmarkRecord> = self
            .db
            .create(("benchmark", key))
            .content(BenchmarkRecord {
                value: value.to_string(),
            })
            .await
            .map_err(|error| {
                println!("error {}", error.to_string());
                Errors::WriteError(error.to_string())
            })?;

        Ok(())
    }
}
