use reqwest::Client;
use serde_json::json;
use std::sync::Arc;
use tokio::time::{Duration, sleep};

use super::{Database, Errors, Result};

#[derive(Debug)]
pub struct OpenSearchDB {
    client: Client,
    base_url: String,
    index_name: String,
}

impl OpenSearchDB {
    pub async fn new() -> Result<Arc<dyn Database + Send + Sync>> {
        let db = OpenSearchDB {
            client: Client::new(),
            base_url: "http://localhost:19201".to_string(),
            index_name: "benchmark_kv".to_string(),
        };

        Ok(Arc::new(db))
    }

    async fn wait_for_connection(&self) -> Result<()> {
        for _ in 0..30 {
            match self.check_health().await {
                Ok(_) => return Ok(()),
                Err(_) => {
                    sleep(Duration::from_secs(2)).await;
                }
            }
        }
        Err(Errors::ConnectionError(
            "Failed to connect to OpenSearch".into(),
        ))
    }

    async fn check_health(&self) -> Result<()> {
        let url = format!("{}/_cluster/health", self.base_url);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| Errors::ConnectionError(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(Errors::ConnectionError("Health check failed".into()))
        }
    }
}

#[async_trait::async_trait]
impl Database for OpenSearchDB {
    async fn ping(&self) -> Result<()> {
        self.wait_for_connection().await
    }

    async fn setup(&self) -> Result<()> {
        let mapping = json!({
            "mappings": {
                "properties": {
                    "key": {"type": "keyword"},
                    "value": {"type": "text"}
                }
            },
            "settings": {
                "number_of_shards": 1,
                "number_of_replicas": 0,
                "refresh_interval": "30s"
            }
        });

        let url = format!("{}/{}", self.base_url, self.index_name);

        // Delete index if exists
        let _ = self.client.delete(&url).send().await;

        // Create new index
        let response = self
            .client
            .put(&url)
            .json(&mapping)
            .send()
            .await
            .map_err(|e| Errors::WriteError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(Errors::WriteError("Failed to create index".into()));
        }

        Ok(())
    }

    async fn write(&self, key: &str, value: &str) -> Result<()> {
        let doc = json!({
            "key": key,
            "value": value
        });

        let url = format!("{}/{}/_doc/{}", self.base_url, self.index_name, key);
        let response = self
            .client
            .put(&url)
            .json(&doc)
            .send()
            .await
            .map_err(|e| Errors::WriteError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(Errors::WriteError("Failed to insert document".into()));
        }

        Ok(())
    }

    fn worker_count(&self) -> usize {
        1000
    }
}
