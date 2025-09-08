use std::sync::Arc;

use super::{Database, Errors, Result};

pub struct YDB {
    client: ydb::Client,
}

impl std::fmt::Debug for YDB {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("YDB").field("client", &"YDBClient").finish()
    }
}

impl YDB {
    pub async fn new() -> Result<Arc<dyn Database + Send + Sync>> {
        // YDB 클라이언트 생성
        let client =
            ydb::ClientBuilder::new_from_connection_string("grpc://127.0.0.1:2136?database=local")
                .map_err(|error| Errors::ConnectionError(error.to_string()))?
                .client()
                .map_err(|error| Errors::ConnectionError(error.to_string()))?;

        client
            .wait()
            .await
            .map_err(|error| Errors::ConnectionError(error.to_string()))?;

        Ok(Arc::new(YDB { client }))
    }
}

#[async_trait::async_trait]
impl Database for YDB {
    async fn ping(&self) -> Result<()> {
        // YDB ping 테스트
        let _result = self
            .client
            .table_client()
            .retry_transaction(|mut transaction| async move {
                let query = ydb::Query::new("SELECT 1 + 1 as sum");
                transaction.query(query).await.map_err(|e| {
                    ydb::YdbOrCustomerError::Customer(Arc::new(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Query error: {}", e),
                    ))))
                })
            })
            .await
            .map_err(|error| Errors::ConnectionError(error.to_string()))?;

        Ok(())
    }

    async fn setup(&self) -> Result<()> {
        // 테이블 삭제 (에러 무시)
        let _result = self
            .client
            .table_client()
            .retry_execute_scheme_query("DROP TABLE IF EXISTS benchmark_kv")
            .await
            .map_err(|e| Errors::WriteError(e.to_string()))?;

        // 테이블 생성
        let _result = self
            .client
            .table_client()
            .retry_execute_scheme_query(
                "CREATE TABLE benchmark_kv (key String, value String, PRIMARY KEY (key))",
            )
            .await
            .map_err(|e| Errors::WriteError(e.to_string()))?;

        Ok(())
    }

    async fn write(&self, key: &str, value: &str) -> Result<()> {
        let key_owned = key.to_string();
        let value_owned = value.to_string();

        let _result = self
            .client
            .table_client()
            .retry_transaction(move |mut transaction| {
                let key = key_owned.clone();
                let value = value_owned.clone();
                async move {
                    let query = ydb::Query::new(&format!(
                        "UPSERT INTO benchmark_kv (key, value) VALUES ('{}', '{}')",
                        key.replace("'", "''"),
                        value.replace("'", "''")
                    ));
                    transaction.query(query).await.map_err(|e| {
                        ydb::YdbOrCustomerError::Customer(Arc::new(Box::new(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("Query error: {}", e),
                        ))))
                    })
                }
            })
            .await
            .map_err(|e| Errors::WriteError(e.to_string()))?;

        Ok(())
    }
}
