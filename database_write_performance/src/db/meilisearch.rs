use reqwest::Client;
use serde_json::json;
use std::sync::Arc;
use tokio::time::{Duration, sleep};

use super::{Database, Errors, Result};

#[derive(Debug)]
pub struct MeiliSearchDB {
    client: Client,
    base_url: String,
    index_name: String,
    master_key: String,
}

impl MeiliSearchDB {
    pub async fn new() -> Result<Arc<dyn Database + Send + Sync>> {
        let db = MeiliSearchDB {
            client: Client::new(),
            base_url: "http://localhost:17700".to_string(),
            index_name: "benchmark_kv".to_string(),
            master_key: "benchmark-master-key".to_string(),
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
            "Failed to connect to MeiliSearch".into(),
        ))
    }

    async fn check_health(&self) -> Result<()> {
        let url = format!("{}/health", self.base_url);
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.master_key))
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
impl Database for MeiliSearchDB {
    async fn ping(&self) -> Result<()> {
        self.wait_for_connection().await
    }

    async fn setup(&self) -> Result<()> {
        // Delete index if exists
        let delete_url = format!("{}/indexes/{}", self.base_url, self.index_name);
        let _ = self
            .client
            .delete(&delete_url)
            .header("Authorization", format!("Bearer {}", self.master_key))
            .send()
            .await;

        // Create new index
        let create_url = format!("{}/indexes", self.base_url);
        let index_config = json!({
            "uid": self.index_name,
            "primaryKey": "key"
        });

        let response = self
            .client
            .post(&create_url)
            .header("Authorization", format!("Bearer {}", self.master_key))
            .json(&index_config)
            .send()
            .await
            .map_err(|e| Errors::WriteError(e.to_string()))?;

        if !response.status().is_success() && response.status().as_u16() != 202 {
            return Err(Errors::WriteError("Failed to create index".into()));
        }

        // Wait a bit for index creation
        sleep(Duration::from_millis(500)).await;

        // Configure searchable attributes
        let settings_url = format!("{}/indexes/{}/settings", self.base_url, self.index_name);
        let settings = json!({
            "searchableAttributes": ["key", "value"],
            "displayedAttributes": ["*"],
            "stopWords": [],
            "synonyms": {}
        });

        let _ = self
            .client
            .patch(&settings_url)
            .header("Authorization", format!("Bearer {}", self.master_key))
            .json(&settings)
            .send()
            .await;

        Ok(())
    }

    async fn write(&self, key: &str, value: &str) -> Result<()> {
        let doc = json!({
            "key": key,
            "value": value
        });

        let url = format!("{}/indexes/{}/documents", self.base_url, self.index_name);
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.master_key))
            .json(&vec![doc])
            .send()
            .await
            .map_err(|e| Errors::WriteError(e.to_string()))?;

        if !response.status().is_success() && response.status().as_u16() != 202 {
            return Err(Errors::WriteError("Failed to insert document".into()));
        }

        Ok(())
    }

    fn worker_count(&self) -> usize {
        1000
    }
}
