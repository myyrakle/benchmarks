use std::{fmt::Debug, sync::Arc};

#[async_trait::async_trait]
pub trait Database {
    async fn ping(&self) -> Result<()>;
    async fn write(&self, key: &str, value: &str) -> Result<()>;
}

pub enum Errors {
    ConnectionError,
    WriteError,
    ReadError,
}

impl Debug for Errors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Errors::ConnectionError => write!(f, "ConnectionError"),
            Errors::WriteError => write!(f, "WriteError"),
            Errors::ReadError => write!(f, "ReadError"),
        }
    }
}

pub type Result<T> = std::result::Result<T, Errors>;

#[derive(Clone, Debug)]
pub struct FakeDB {}

impl FakeDB {
    pub fn new() -> Arc<dyn Database + Send + Sync> {
        Arc::new(FakeDB {})
    }
}

#[async_trait::async_trait]
impl Database for FakeDB {
    async fn ping(&self) -> Result<()> {
        Ok(())
    }

    async fn write(&self, _key: &str, _value: &str) -> Result<()> {
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        Ok(())
    }
}
