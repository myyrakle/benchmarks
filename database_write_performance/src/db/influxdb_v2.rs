use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use futures::stream;
use influxdb2::{Client, models::DataPoint};

use crate::db::{Database, Errors, Result};

#[derive(Clone)]
pub struct InfluxDB {
    client: Client,
}

impl InfluxDB {
    pub async fn new() -> Result<Arc<dyn Database + Send + Sync>> {
        let host = "http://localhost:18086";
        let org = "benchmark";
        let token = "benchmark-token-1234567890abcdef";

        let client = Client::new(host, org, token);

        let db = InfluxDB { client };

        Ok(Arc::new(db))
    }
}

#[async_trait::async_trait]
impl Database for InfluxDB {
    async fn ping(&self) -> Result<()> {
        self.client.health().await.map_err(|e| {
            eprintln!("InfluxDB health check error: {:?}", e);
            Errors::ConnectionError(format!("InfluxDB health check failed: {}", e))
        })?;

        Ok(())
    }

    async fn setup(&self) -> Result<()> {
        // InfluxDB는 스키마가 없으므로 특별한 setup이 필요 없음
        // bucket은 이미 docker-compose에서 초기화됨
        Ok(())
    }

    async fn write(&self, key: &str, value: &str) -> Result<()> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as i64;

        let data_point = DataPoint::builder("benchmark")
            .field("key", key)
            .field("value", value)
            .timestamp(timestamp)
            .build()
            .map_err(|e| Errors::WriteError(e.to_string()))?;

        match self
            .client
            .write("benchmark", stream::iter(vec![data_point]))
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!("InfluxDB write error: {:?}", e);
                Err(Errors::WriteError(e.to_string()))
            }
        }
    }
}
