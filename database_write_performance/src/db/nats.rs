use async_nats::jetstream::{self, kv};
use std::sync::Arc;

use super::{Database, Errors, Result};

#[derive(Debug)]
pub struct NatsJetStream {
    kv_store: kv::Store,
}

impl NatsJetStream {
    pub async fn new() -> Result<Arc<dyn Database + Send + Sync>> {
        // NATS 클라이언트 연결
        let client = async_nats::connect("nats://127.0.0.1:4222")
            .await
            .map_err(|error| Errors::ConnectionError(error.to_string()))?;

        // JetStream 컨텍스트 생성
        let jetstream = jetstream::new(client);

        // Key-Value 스토어 생성 또는 연결
        let kv_store = match jetstream
            .create_key_value(jetstream::kv::Config {
                bucket: "benchmark".to_string(),
                description: "Benchmark Key-Value Store".to_string(),
                max_value_size: 1024 * 1024, // 1MB max value size
                storage: jetstream::stream::StorageType::File,
                num_replicas: 1,
                ..Default::default()
            })
            .await
        {
            Ok(store) => store,
            Err(_) => {
                // 이미 존재하는 경우 기존 스토어에 연결
                jetstream
                    .get_key_value("benchmark")
                    .await
                    .map_err(|error| Errors::ConnectionError(error.to_string()))?
            }
        };

        Ok(Arc::new(NatsJetStream { kv_store }))
    }
}

#[async_trait::async_trait]
impl Database for NatsJetStream {
    async fn ping(&self) -> Result<()> {
        // ping 테스트를 위해 간단한 put/get 수행
        self.kv_store
            .put("ping_test", "ping".into())
            .await
            .map_err(|error| Errors::ConnectionError(error.to_string()))?;

        self.kv_store
            .get("ping_test")
            .await
            .map_err(|error| Errors::ConnectionError(error.to_string()))?;

        Ok(())
    }

    async fn setup(&self) -> Result<()> {
        // NATS JetStream KV는 별도의 테이블 설정이 필요하지 않음
        // 필요시 기존 키들을 정리할 수 있지만 일반적으로는 필요 없음
        Ok(())
    }

    async fn write(&self, key: &str, value: &str) -> Result<()> {
        let value_bytes = value.as_bytes().to_vec();
        self.kv_store
            .put(key, value_bytes.into())
            .await
            .map_err(|e| Errors::WriteError(e.to_string()))?;

        Ok(())
    }
}
