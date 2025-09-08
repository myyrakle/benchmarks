use std::sync::Arc;
use tikv_client::RawClient;

use super::{Database, Errors, Result};

pub struct TiKV {
    client: RawClient,
}

impl TiKV {
    pub async fn new() -> Result<Arc<dyn Database + Send + Sync>> {
        // PD (Placement Driver) 엔드포인트
        let pd_endpoints = vec!["127.0.0.1:2379"];

        let client = RawClient::new(pd_endpoints)
            .await
            .map_err(|error| Errors::ConnectionError(error.to_string()))?;

        Ok(Arc::new(TiKV { client }))
    }
}

#[async_trait::async_trait]
impl Database for TiKV {
    async fn ping(&self) -> Result<()> {
        // TiKV에서는 간단한 get 연산으로 연결 확인
        let test_key = b"__ping_test__";

        match self.client.get(test_key.to_vec()).await {
            Ok(_) => Ok(()), // key가 존재하지 않아도 연결이 정상이면 Ok
            Err(error) => Err(Errors::ConnectionError(error.to_string())),
        }
    }

    async fn setup(&self) -> Result<()> {
        // TiKV는 schema-less이므로 별도 테이블 생성이 필요 없음
        // 기존 벤치마크 키들을 정리 (선택적)

        // 벤치마크 네임스페이스 접두사로 range 생성
        let prefix = b"benchmark:";
        let end_prefix = b"benchmark;"; // ':' 다음 문자로 range 끝 설정

        // 기존 키들을 스캔하고 삭제 (선택적 - 대량 데이터가 있을 수 있으니 주의)
        match self
            .client
            .scan(prefix.to_vec()..end_prefix.to_vec(), 1000)
            .await
        {
            Ok(kvs) => {
                if !kvs.is_empty() {
                    let keys: Vec<_> = kvs.into_iter().map(|kv| kv.0).collect();
                    if let Err(error) = self.client.batch_delete(keys).await {
                        eprintln!("Warning: Failed to clean existing keys: {}", error);
                    }
                }
            }
            Err(_) => {
                // 스캔 실패는 무시 (키가 없을 수 있음)
            }
        }

        Ok(())
    }

    async fn write(&self, key: &str, value: &str) -> Result<()> {
        // 벤치마크 네임스페이스 추가
        let namespaced_key = format!("benchmark:{}", key);

        self.client
            .put(namespaced_key.into_bytes(), value.as_bytes().to_vec())
            .await
            .map_err(|_| Errors::WriteError("Failed to write to TiKV".to_string()))?;

        Ok(())
    }
}
