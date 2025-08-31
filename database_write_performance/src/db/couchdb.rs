use std::borrow::Cow;
use std::sync::Arc;

use couch_rs::{Client, database::Database, document::TypedCouchDocument};
use serde::{Deserialize, Serialize};

use crate::db::{Database as DbTrait, Errors, Result};

#[derive(Clone)]
pub struct CouchDB {
    db: Database,
}

#[derive(Serialize, Deserialize)]
struct BenchmarkDoc {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _rev: Option<String>,
    pub key: String,
    pub value: String,
    pub timestamp: i64,
}

impl TypedCouchDocument for BenchmarkDoc {
    fn get_id(&self) -> Cow<'_, str> {
        self._id
            .as_ref()
            .map(|s| Cow::Borrowed(s.as_str()))
            .unwrap_or(Cow::Borrowed(""))
    }

    fn get_rev(&self) -> Cow<'_, str> {
        self._rev
            .as_ref()
            .map(|s| Cow::Borrowed(s.as_str()))
            .unwrap_or(Cow::Borrowed(""))
    }

    fn set_id(&mut self, id: &str) {
        self._id = Some(id.to_string());
    }

    fn set_rev(&mut self, rev: &str) {
        self._rev = Some(rev.to_string());
    }

    fn merge_ids(&mut self, other: &Self) {
        if self._id.is_none() {
            self._id = other._id.clone();
        }
        if self._rev.is_none() {
            self._rev = other._rev.clone();
        }
    }
}

impl CouchDB {
    pub async fn new() -> Result<Arc<dyn DbTrait + Send + Sync>> {
        // 연결 설정 개선
        let client = Client::new("http://localhost:15984", "admin", "q1w2e3r4").map_err(|e| {
            eprintln!("CouchDB client creation error: {:?}", e);
            Errors::ConnectionError(format!("CouchDB client creation failed: {}", e))
        })?;

        // 데이터베이스 객체를 미리 가져와서 재사용
        let db = client.db("benchmark").await.map_err(|e| {
            eprintln!("CouchDB database connection error: {:?}", e);
            Errors::ConnectionError(format!("CouchDB database connection failed: {}", e))
        })?;

        let couchdb = CouchDB { db };

        Ok(Arc::new(couchdb))
    }
}

#[async_trait::async_trait]
impl DbTrait for CouchDB {
    async fn ping(&self) -> Result<()> {
        // 데이터베이스 존재 확인
        let exists = self.db.exists("").await;
        match exists {
            true => Ok(()),
            false => Err(Errors::ConnectionError(
                "Database not accessible".to_string(),
            )),
        }
    }

    async fn setup(&self) -> Result<()> {
        // CouchDB는 데이터베이스가 자동으로 생성되므로 ping으로 확인만 함
        let exists = self.db.exists("").await;

        if exists {
            println!("CouchDB database is ready");
        } else {
            eprintln!("CouchDB database not accessible");
        }

        Ok(())
    }

    async fn write(&self, key: &str, value: &str) -> Result<()> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as i64;

        let mut doc = BenchmarkDoc {
            _id: Some(format!("{}_{}", key, timestamp)), // 고유 ID 생성
            _rev: None,
            key: key.to_string(),
            value: value.to_string(),
            timestamp,
        };

        // 기존 데이터베이스 객체 재사용으로 성능 개선
        match self.db.create(&mut doc).await {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!("CouchDB write error: {:?}", e);
                Err(Errors::WriteError)
            }
        }
    }
}
