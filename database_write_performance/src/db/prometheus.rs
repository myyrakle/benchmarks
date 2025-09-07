use prometheus_remote_write::{Label, Sample, TimeSeries};
use reqwest::Client;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use super::{Database, Errors, Result};

pub struct Prometheus {
    client: Client,
    url: String,
}

impl Prometheus {
    pub async fn new() -> Result<Arc<dyn Database + Send + Sync>> {
        let client = Client::new();
        let url = "http://localhost:19090/api/v1/write".to_string();

        Ok(Arc::new(Prometheus { client, url }))
    }
}

#[async_trait::async_trait]
impl Database for Prometheus {
    async fn ping(&self) -> Result<()> {
        let health_url = "http://localhost:19090/-/healthy";
        let response = self
            .client
            .get(health_url)
            .send()
            .await
            .map_err(|e| Errors::ConnectionError(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(Errors::ConnectionError("Health check failed".to_string()))
        }
    }

    async fn setup(&self) -> Result<()> {
        // Prometheus는 스키마가 필요 없음

        // 기존 데이터 clear

        Ok(())
    }

    async fn write(&self, key: &str, value: &str) -> Result<()> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        // 높은 cardinality를 위해 key와 value를 레이블로 추가
        // 각 고유한 key-value 쌍은 별도의 시계열이 됨
        let time_series = TimeSeries {
            labels: vec![
                Label {
                    name: "__name__".to_string(),
                    value: "benchmark_high_cardinality_data".to_string(),
                },
                Label {
                    name: "data_key".to_string(),
                    value: key.to_string(),
                },
                Label {
                    name: "data_value".to_string(),
                    value: value.to_string(),
                },
            ],
            samples: vec![Sample {
                value: 1.0, // 카운터 형태로 사용 (존재함을 표시)
                timestamp,
            }],
        };

        // Prometheus remote write 포맷으로 직렬화
        let write_request = prometheus_remote_write::WriteRequest {
            timeseries: vec![time_series],
        };

        let compressed = write_request
            .encode_compressed()
            .map_err(|_| Errors::WriteError)?;

        let response = self
            .client
            .post(&self.url)
            .header("Content-Type", "application/x-protobuf")
            .header("Content-Encoding", "snappy")
            .header("X-Prometheus-Remote-Write-Version", "0.1.0")
            .body(compressed)
            .send()
            .await
            .map_err(|e| Errors::ConnectionError(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            let _error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(Errors::WriteError)
        }
    }
}
