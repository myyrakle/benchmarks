use reqwest::Client;
use serde_json::json;
use std::sync::Arc;
use tokio::time::{Duration, sleep};

use super::{Database, Errors, Result};

#[derive(Debug)]
pub struct BarusDB {
    client: Client,
    base_url: String,
    table_name: String,
}

impl BarusDB {
    pub async fn new() -> Result<Arc<dyn Database + Send + Sync>> {
        let db = BarusDB {
            client: Client::builder()
                .tcp_keepalive(Some(Duration::from_secs(30)))
                .pool_max_idle_per_host(1000)
                .build()
                .unwrap(),
            base_url: "http://localhost:53000".to_string(),
            table_name: "benchmark_kv".to_string(),
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
        Err(Errors::ConnectionError("Failed to connect to Barus".into()))
    }

    async fn check_health(&self) -> Result<()> {
        let url = format!("{}/", self.base_url);
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
impl Database for BarusDB {
    async fn ping(&self) -> Result<()> {
        self.wait_for_connection().await
    }

    async fn setup(&self) -> Result<()> {
        Ok(())
    }

    async fn write(&self, key: &str, value: &str) -> Result<()> {
        let doc = json!({
            "key": key,
            "value": value
        });

        let url = format!("{}/{}/value", self.base_url, self.table_name);
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
