use chrono::{DateTime, Utc};
use strum::VariantArray;

use crate::proto::PrismObject;

pub mod error;

#[cfg(feature = "oura")]
pub mod oura;

#[cfg(feature = "oura")]
pub trait DltSource {
    fn receiver(self) -> Result<tokio::sync::mpsc::Receiver<PublishedAtalaObject>, String>;
}

#[cfg(feature = "oura")]
pub trait DltSink {
    fn send(&mut self, atala_object: PrismObject);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DltCursor {
    pub slot: u64,
    pub block_hash: Vec<u8>,
    pub cbt: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockMetadata {
    /// Cardano slot number
    pub slot_number: u64,
    /// Cardano block number
    pub block_number: u64,
    /// Cardano block timestamp
    pub cbt: DateTime<Utc>,
    /// AtalaBlock seqeuence number
    ///
    /// This is used to order AtalaBlock within the same Cardano block
    pub absn: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationMetadata {
    /// AtalaBlock metadata
    pub block_metadata: BlockMetadata,
    /// Operation sequence number
    ///
    /// This is used to order AtalaOperation within the same AtalaBlock
    pub osn: u32,
}

impl OperationMetadata {
    pub fn compare_time_asc(a: &Self, b: &Self) -> std::cmp::Ordering {
        let a_tup = (a.block_metadata.block_number, a.block_metadata.absn, a.osn);
        let b_tup = (b.block_metadata.block_number, b.block_metadata.absn, b.osn);
        a_tup.cmp(&b_tup)
    }

    pub fn compare_time_desc(a: &Self, b: &Self) -> std::cmp::Ordering {
        Self::compare_time_asc(b, a)
    }
}

#[derive(Debug, Clone)]
pub struct PublishedAtalaObject {
    pub block_metadata: BlockMetadata,
    pub atala_object: PrismObject,
}

#[derive(Debug, Clone, PartialEq, Eq, strum::Display, strum::EnumString, strum::VariantArray)]
pub enum NetworkIdentifier {
    #[strum(serialize = "mainnet")]
    Mainnet,
    #[strum(serialize = "preprod")]
    Preprod,
    #[strum(serialize = "preview")]
    Preview,
}

impl NetworkIdentifier {
    pub fn variants() -> &'static [Self] {
        Self::VARIANTS
    }
}
