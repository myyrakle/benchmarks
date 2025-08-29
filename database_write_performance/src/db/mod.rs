#[async_trait::async_trait]
pub trait Database {
    async fn write(&self, key: &str, value: &str) -> Result<()>;
}

pub enum Errors {
    ConnectionError,
    WriteError,
    ReadError,
}

pub type Result<T> = std::result::Result<T, Errors>;
