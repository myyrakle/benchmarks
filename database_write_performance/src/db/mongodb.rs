use mongodb::{Client, options::ClientOptions};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::{Database, Errors, Result};

#[derive(Debug, Serialize, Deserialize)]
struct KeyValue {
    #[serde(rename = "_id")]
    key: String,
    value: String,
}

#[derive(Debug)]
pub struct MongoDB {
    client: Client,
}

impl MongoDB {
    pub async fn new() -> Result<Arc<dyn Database + Send + Sync>> {
        let connection_string = "mongodb://user:q1w2e3r4@127.0.0.1:27017/benchmark";

        let mut client_options = ClientOptions::parse(connection_string)
            .await
            .map_err(|_| Errors::ConnectionError)?;

        // 커넥션 풀 설정
        client_options.max_pool_size = Some(1000);
        client_options.min_pool_size = Some(10);

        let client = Client::with_options(client_options).map_err(|_| Errors::ConnectionError)?;

        Ok(Arc::new(MongoDB { client }))
    }
}

#[async_trait::async_trait]
impl Database for MongoDB {
    async fn ping(&self) -> Result<()> {
        // MongoDB ping을 위해 간단한 명령 실행
        self.client
            .database("admin")
            .run_command(mongodb::bson::doc! { "ping": 1 })
            .await
            .map_err(|_| Errors::ConnectionError)?;

        Ok(())
    }

    async fn setup(&self) -> Result<()> {
        Ok(())
    }

    async fn write(&self, key: &str, value: &str) -> Result<()> {
        let doc = KeyValue {
            key: key.to_string(),
            value: value.to_string(),
        };

        self.client
            .database("benchmark")
            .collection::<KeyValue>("key_value")
            .insert_one(doc)
            .await
            .map_err(|_| Errors::WriteError)?;

        Ok(())
    }
}
