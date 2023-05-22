use super::{
    get_did_from_signed_operation, OperationStore, OperationStoreError, SignedAtalaOperation,
};
use crate::did::CanonicalPrismDid;
use crate::dlt::OperationTimestamp;
use crate::util::MessageExt;
use chrono::Utc;
use prost::Message;
use serde::{Deserialize, Serialize};
use std::ops::Bound;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::{Database, Root};
use surrealdb::sql::Value;
use surrealdb::Surreal;

pub struct SurrealOperationStore {
    db: Surreal<Client>,
    namespace: String,
    db_name: String,
}

impl SurrealOperationStore {
    pub async fn ws(
        remote_url: &str,
        namespace: &str,
        database: &str,
        username: &str,
        password: &str,
    ) -> Result<Self, surrealdb::Error> {
        let db = Surreal::new::<Ws>(remote_url).await?;
        let db_auth = Database {
            namespace,
            database,
            username,
            password,
        };
        db.signin(db_auth).await?;
        Ok(Self {
            db,
            namespace: namespace.to_string(),
            db_name: database.to_string(),
        })
    }

    pub async fn ws_root(
        remote_url: &str,
        namespace: &str,
        database: &str,
        username: &str,
        password: &str,
    ) -> Result<Self, surrealdb::Error> {
        let db = Surreal::new::<Ws>(remote_url).await?;
        let db_auth = Root { username, password };
        db.signin(db_auth).await?;
        Ok(Self {
            db,
            namespace: namespace.to_string(),
            db_name: database.to_string(),
        })
    }

    pub async fn set_operation_ns(&self) -> Result<(), surrealdb::Error> {
        self.db.use_ns("test").use_db("test").await?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationRecord {
    timestamp: OperationTimestamp,
    raw_bytes: Vec<u8>,
}

#[async_trait::async_trait]
impl OperationStore for SurrealOperationStore {
    async fn get_by_did(
        &mut self,
        did: &CanonicalPrismDid,
    ) -> Result<Vec<(OperationTimestamp, SignedAtalaOperation)>, OperationStoreError> {
        self.set_operation_ns().await.unwrap();

        let range_from: Vec<Value> = vec![did.to_string().into(), Value::None];
        let range_to: Vec<Value> = vec![did.to_string().into(), Utc::now().into()];
        let records: Vec<OperationRecord> = self
            .db
            .select("signed_operation")
            .range((Bound::Included(range_from), Bound::Included(range_to)))
            .await
            .map_err(|e| OperationStoreError::StorageBackendError(e.into()))?;

        let mut result = Vec::with_capacity(records.len());
        for r in records {
            let timestamp = r.timestamp;
            let signed_operation = SignedAtalaOperation::decode(r.raw_bytes.as_slice())?;
            result.push((timestamp, signed_operation));
        }

        Ok(result)
    }

    async fn insert(
        &mut self,
        signed_operation: SignedAtalaOperation,
        timestamp: OperationTimestamp,
    ) -> Result<(), OperationStoreError> {
        self.set_operation_ns()
            .await
            .map_err(|e| OperationStoreError::StorageBackendError(e.into()))?;

        let did = get_did_from_signed_operation(&signed_operation)?;
        let bytes: Vec<u8> = signed_operation.encode_to_bytes()?.into();

        let record_id: Vec<Value> = vec![
            did.to_string().into(),
            timestamp.block_timestamp.cbt.into(),
            timestamp.block_timestamp.absn.into(),
            timestamp.osn.into(),
        ];

        let record = OperationRecord {
            timestamp,
            raw_bytes: bytes,
        };
        let record: OperationRecord = self
            .db
            .create(("signed_operation", record_id))
            .content(record)
            .await
            .map_err(|e| OperationStoreError::StorageBackendError(e.into()))?;

        Ok(())
    }
}
