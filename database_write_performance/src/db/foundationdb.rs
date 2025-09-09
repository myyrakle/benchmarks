use foundationdb::*;
use std::sync::Arc;

use super::{Database, Errors, Result};

pub struct FoundationDB {
    db: foundationdb::Database,
}

impl FoundationDB {
    pub async fn new() -> Result<Arc<dyn Database + Send + Sync>> {
        // FoundationDB 네트워크 초기화
        let network = FdbClusterFile::default()
            .connect()
            .await
            .map_err(|error| Errors::ConnectionError(error.to_string()))?;

        // 기본 데이터베이스 열기
        let db = network
            .open_database(b"")
            .await
            .map_err(|error| Errors::ConnectionError(error.to_string()))?;

        Ok(Arc::new(FoundationDB { db }))
    }
}

#[async_trait::async_trait]
impl Database for FoundationDB {
    async fn ping(&self) -> Result<()> {
        // FoundationDB에서는 간단한 트랜잭션으로 연결 확인
        let transaction = self.db.create_trx().expect("Failed to create transaction");

        // 테스트용 키로 get 연산 수행
        let test_key = b"__ping_test__";

        match transaction.get(test_key, false).await {
            Ok(_) => {
                transaction.commit().await.map_err(|error| {
                    Errors::ConnectionError(format!("Failed to commit ping transaction: {}", error))
                })?;
                Ok(())
            }
            Err(error) => Err(Errors::ConnectionError(error.to_string())),
        }
    }

    async fn setup(&self) -> Result<()> {
        // FoundationDB는 schema-less이므로 별도 테이블 생성이 필요 없음
        // 기존 벤치마크 키들을 정리 (선택적)

        let transaction = self.db.create_trx().expect("Failed to create transaction");

        // 벤치마크 네임스페이스 접두사
        let prefix = b"benchmark:";
        let end_key = b"benchmark;"; // ':' 다음 문자로 range 끝 설정

        // 기존 키들을 스캔하고 삭제 (선택적)
        match transaction
            .get_range(
                RangeOption {
                    begin: KeySelector::first_greater_or_equal(prefix),
                    end: KeySelector::first_greater_or_equal(end_key),
                    limit: Some(1000),
                    reverse: false,
                    mode: StreamingMode::WantAll,
                },
                false,
            )
            .await
        {
            Ok(range_result) => {
                let kvs = range_result.key_values();
                if !kvs.is_empty() {
                    for kv in kvs {
                        transaction.clear(&kv.key());
                    }
                }

                transaction.commit().await.map_err(|error| {
                    Errors::ConnectionError(format!(
                        "Failed to commit setup transaction: {}",
                        error
                    ))
                })?;
            }
            Err(_) => {
                // 스캔 실패는 무시 (키가 없을 수 있음)
                transaction.reset();
            }
        }

        Ok(())
    }

    async fn write(&self, key: &str, value: &str) -> Result<()> {
        let transaction = self.db.create_trx().expect("Failed to create transaction");

        // 벤치마크 네임스페이스 추가
        let namespaced_key = format!("benchmark:{}", key);

        transaction.set(namespaced_key.as_bytes(), value.as_bytes());

        transaction.commit().await.map_err(|error| {
            Errors::WriteError(format!("Failed to write to FoundationDB: {}", error))
        })?;

        Ok(())
    }
}
