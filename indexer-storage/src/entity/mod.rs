use chrono::{DateTime, Utc};
use identus_apollo::hex::HexStr;
use identus_did_prism::did::CanonicalPrismDid;
use lazybe::macros::{Entity, Newtype};
use lazybe::uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Newtype, derive_more::From)]
pub struct DidSuffix(Vec<u8>);

impl From<CanonicalPrismDid> for DidSuffix {
    fn from(value: CanonicalPrismDid) -> Self {
        value.suffix.to_vec().into()
    }
}

impl TryFrom<DidSuffix> for CanonicalPrismDid {
    type Error = crate::Error;

    fn try_from(value: DidSuffix) -> Result<Self, Self::Error> {
        let suffix = HexStr::from(value.0);
        let did = CanonicalPrismDid::from_suffix(suffix)?;
        Ok(did)
    }
}

#[derive(Entity)]
#[lazybe(table = "dlt_cursor")]
pub struct DltCursor {
    #[lazybe(primary_key)]
    pub id: Uuid,
    pub slot: i64,
    pub block_hash: Vec<u8>,
}

#[derive(Entity)]
#[lazybe(table = "raw_operation")]
#[allow(unused)]
pub struct RawOperation {
    #[lazybe(primary_key)]
    pub id: Uuid,
    pub signed_operation_data: Vec<u8>,
    pub slot: i64,
    pub block_number: i64,
    pub cbt: DateTime<Utc>,
    pub absn: i32,
    pub osn: i32,
    pub is_indexed: bool,
}

#[derive(Entity)]
#[lazybe(table = "indexed_ssi_operation")]
#[allow(unused)]
pub struct IndexedSsiOperation {
    #[lazybe(primary_key)]
    pub id: Uuid,
    pub raw_operation_id: Uuid,
    pub did: DidSuffix,
}

#[derive(Entity)]
#[lazybe(table = "indexed_vdr_operation")]
#[allow(unused)]
pub struct IndexedVdrOperation {
    #[lazybe(primary_key)]
    pub id: Uuid,
    pub raw_operation_id: Uuid,
    pub operation_hash: Vec<u8>,
    pub prev_operation_hash: Option<Vec<u8>>,
    pub did: Option<DidSuffix>,
}

#[derive(Entity)]
#[lazybe(table = "did_stats")]
#[allow(unused)]
pub struct DidStats {
    #[lazybe(primary_key)]
    pub did: DidSuffix,
    pub operation_count: i64,
    pub last_block: i64,
    pub last_slot: i64,
    pub last_cbt: DateTime<Utc>,
    pub first_block: i64,
    pub first_slot: i64,
    pub first_cbt: DateTime<Utc>,
}
