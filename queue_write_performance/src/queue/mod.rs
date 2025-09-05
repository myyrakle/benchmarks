use std::{fmt::Debug, sync::Arc};

pub mod postgres;

#[async_trait::async_trait]
pub trait Queue {
    // connection ping
    async fn ping(&self) -> Result<()>;

    // create table if not exists
    // re-create table if exists
    async fn setup(&self) -> Result<()>;

    // write key, value
    async fn write(&self, key: &str, value: &str) -> Result<()>;
}

pub async fn new_queue(queue_type: &str) -> Result<Arc<dyn Queue + Send + Sync>> {
    match queue_type {
        "postgres" => postgres::PostgresDB::new().await,
        _ => Err(Errors::ConnectionError("Unknown database type".into())),
    }
}

pub enum Errors {
    ConnectionError(String),
    WriteError,
    ReadError,
}

impl Debug for Errors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Errors::ConnectionError(msg) => write!(f, "ConnectionError: {}", msg),
            Errors::WriteError => write!(f, "WriteError"),
            Errors::ReadError => write!(f, "ReadError"),
        }
    }
}

pub type Result<T> = std::result::Result<T, Errors>;

#[derive(Clone, Debug)]
pub struct FakeQueue {}

impl FakeQueue {
    pub fn new() -> Arc<dyn Queue + Send + Sync> {
        Arc::new(FakeQueue {})
    }
}

#[async_trait::async_trait]
impl Queue for FakeQueue {
    async fn ping(&self) -> Result<()> {
        Ok(())
    }

    async fn setup(&self) -> Result<()> {
        Ok(())
    }

    async fn write(&self, _key: &str, _value: &str) -> Result<()> {
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        Ok(())
    }
}
