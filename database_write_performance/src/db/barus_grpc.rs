use std::sync::Arc;
use tokio::time::{Duration, sleep};
use tonic::transport::Channel;

use super::{Database, Errors, Result};

// Include the generated proto code
pub mod barus {
    tonic::include_proto!("barus");
}

use barus::barus_service_client::BarusServiceClient;
use barus::{HealthRequest, PutRequest};

#[derive(Debug, Clone)]
pub struct BarusDBGrpc {
    channel: Channel,
    table_name: Arc<String>,
}

impl BarusDBGrpc {
    pub async fn new() -> Result<Arc<dyn Database + Send + Sync>> {
        let channel = Channel::from_static("http://localhost:53001")
            .http2_keep_alive_interval(Duration::from_secs(30))
            .keep_alive_timeout(Duration::from_secs(10))
            .keep_alive_while_idle(true)
            .tcp_keepalive(Some(Duration::from_secs(30)))
            .concurrency_limit(1000)
            .connect_timeout(Duration::from_secs(5))
            .connect()
            .await
            .map_err(|e| Errors::ConnectionError(format!("Failed to connect: {}", e)))?;

        let db = BarusDBGrpc {
            channel,
            table_name: Arc::new("benchmark_kv".to_string()),
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
            "Failed to connect to Barus gRPC".into(),
        ))
    }

    async fn check_health(&self) -> Result<()> {
        let mut client = BarusServiceClient::new(self.channel.clone());
        let request = tonic::Request::new(HealthRequest {});

        client
            .health(request)
            .await
            .map_err(|e| Errors::ConnectionError(format!("Health check failed: {}", e)))?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Database for BarusDBGrpc {
    async fn ping(&self) -> Result<()> {
        self.wait_for_connection().await
    }

    async fn setup(&self) -> Result<()> {
        Ok(())
    }

    async fn write(&self, key: &str, value: &str) -> Result<()> {
        let mut client = BarusServiceClient::new(self.channel.clone());

        let request = tonic::Request::new(PutRequest {
            table: self.table_name.as_ref().clone(),
            key: key.to_string(),
            value: value.to_string(),
        });

        client
            .put(request)
            .await
            .map_err(|e| Errors::WriteError(format!("gRPC put failed: {}", e)))?;

        Ok(())
    }

    fn worker_count(&self) -> usize {
        1000
    }
}
