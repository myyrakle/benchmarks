use std::{fmt::Debug, sync::Arc};

pub mod cassandra;
pub mod clickhouse;
pub mod cockroachdb;
pub mod couchdb;
pub mod etcd;
pub mod influxdb_v2;
pub mod mariadb;
pub mod mongodb;
pub mod mysql;
pub mod nats;
pub mod postgres;
pub mod redis;
pub mod scylla;
pub mod tidb;
pub mod tikv;
pub mod timescaledb;
pub mod yugabytedb;

#[async_trait::async_trait]
pub trait Database {
    // connection ping
    async fn ping(&self) -> Result<()>;

    // create table if not exists
    // re-create table if exists
    async fn setup(&self) -> Result<()>;

    // write key, value
    async fn write(&self, key: &str, value: &str) -> Result<()>;
}

pub async fn new_database(db_type: &str) -> Result<Arc<dyn Database + Send + Sync>> {
    match db_type {
        "postgres" => postgres::PostgresDB::new().await,
        "mysql" => mysql::MySqlDB::new().await,
        "mariadb" => mariadb::MariaDB::new().await,
        "mongodb" => mongodb::MongoDB::new().await,
        "scylla" => cassandra::ScyllaDB::new().await,
        "cassandra" => scylla::CassandraDB::new().await,
        "influxdb_v2" => influxdb_v2::InfluxDB::new().await,
        "timescaledb" => timescaledb::TimescaleDB::new().await,
        "couchdb" => couchdb::CouchDB::new().await,
        "yugabytedb" => yugabytedb::YugabyteDB::new().await,
        "cockroachdb" => cockroachdb::CockroachDB::new().await,
        "clickhouse" => clickhouse::ClickHouse::new().await,
        "etcd" => etcd::Etcd::new().await,
        "nats" => nats::NatsJetStream::new().await,
        "redis" => redis::Redis::new().await,
        "tidb" => tidb::TiDB::new().await,
        "tikv" => tikv::TiKV::new().await,
        _ => Err(Errors::ConnectionError("Unknown database type".into())),
    }
}

pub enum Errors {
    ConnectionError(String),
    WriteError(String),
    ReadError,
}

impl Debug for Errors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Errors::ConnectionError(msg) => write!(f, "ConnectionError: {}", msg),
            Errors::WriteError(msg) => write!(f, "WriteError: {}", msg),
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

    async fn setup(&self) -> Result<()> {
        Ok(())
    }

    async fn write(&self, _key: &str, _value: &str) -> Result<()> {
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        Ok(())
    }
}
