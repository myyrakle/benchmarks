use etcd_rs::{Client, ClientConfig, Endpoint, KeyRange, KeyValueOp, PutRequest, RangeRequest};
use std::sync::Arc;

use super::{Database, Errors, Result};

pub struct Etcd {
    client: Client,
}

impl std::fmt::Debug for Etcd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Etcd").field("client", &"Client").finish()
    }
}

impl Etcd {
    pub async fn new() -> Result<Arc<dyn Database + Send + Sync>> {
        let endpoint = Endpoint::new("http://127.0.0.1:2379");
        let config = ClientConfig::new(vec![endpoint]);
        let client = Client::connect(config)
            .await
            .map_err(|error| Errors::ConnectionError(error.to_string()))?;

        Ok(Arc::new(Etcd { client }))
    }
}

#[async_trait::async_trait]
impl Database for Etcd {
    async fn ping(&self) -> Result<()> {
        // etcd에서 ping 테스트를 위해 간단한 get 요청 수행
        let key_range = KeyRange::key("ping_test");
        let req = RangeRequest::new(key_range);
        self.client
            .get(req)
            .await
            .map_err(|error| Errors::ConnectionError(error.to_string()))?;
        Ok(())
    }

    async fn setup(&self) -> Result<()> {
        // etcd는 별도의 테이블 설정이 필요하지 않음
        // 필요시 기존 key들을 정리할 수 있지만 일반적으로는 필요 없음
        Ok(())
    }

    async fn write(&self, key: &str, value: &str) -> Result<()> {
        let put_request = PutRequest::new(key, value);

        self.client
            .put(put_request)
            .await
            .map_err(|e| Errors::WriteError(e.to_string()))?;

        Ok(())
    }
}
