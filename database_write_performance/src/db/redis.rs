use redis::AsyncCommands;
use std::sync::Arc;

use super::{Database, Errors, Result};

pub struct Redis {
    client: redis::Client,
}

impl Redis {
    pub async fn new() -> Result<Arc<dyn Database + Send + Sync>> {
        let client = redis::Client::open("redis://127.0.0.1:6379/")
            .map_err(|error| Errors::ConnectionError(error.to_string()))?;

        // 연결 테스트
        let mut conn = client
            .get_multiplexed_async_connection()
            .await
            .map_err(|error| Errors::ConnectionError(error.to_string()))?;

        let _: String = redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .map_err(|error| Errors::ConnectionError(error.to_string()))?;

        Ok(Arc::new(Redis { client }))
    }
}

#[async_trait::async_trait]
impl Database for Redis {
    async fn ping(&self) -> Result<()> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|error| Errors::ConnectionError(error.to_string()))?;

        // Redis PING 명령어로 연결 확인
        let _: String = redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .map_err(|error| Errors::ConnectionError(error.to_string()))?;

        Ok(())
    }

    async fn setup(&self) -> Result<()> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|error| Errors::ConnectionError(error.to_string()))?;

        // 벤치마크 네임스페이스의 기존 키들을 정리
        let pattern = "*";

        // KEYS 명령어를 사용해서 키 목록을 가져오고 삭제
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(pattern)
            .query_async(&mut conn)
            .await
            .map_err(|error| Errors::ConnectionError(error.to_string()))?;

        if !keys.is_empty() {
            let _: () = conn
                .del(&keys)
                .await
                .map_err(|error| Errors::ConnectionError(error.to_string()))?;
        }

        Ok(())
    }

    async fn write(&self, key: &str, value: &str) -> Result<()> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|error| Errors::ConnectionError(error.to_string()))?;

        let _: () = conn
            .set(key, value)
            .await
            .map_err(|error| Errors::WriteError(error.to_string()))?;

        Ok(())
    }
}
