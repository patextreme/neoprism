use chrono::{DateTime, Utc};
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::proto::AtalaObject;

pub mod error;

#[cfg(feature = "cardano")]
pub mod cardano;

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
    pub atala_object: AtalaObject,
}

pub trait DltSource {
    fn receiver(self) -> Result<Receiver<PublishedAtalaObject>, String>;
}

pub trait DltSink {
    fn send(&mut self, atala_object: AtalaObject);
}

pub struct InMemoryDlt {
    delay: tokio::time::Duration,
    tx: Sender<PublishedAtalaObject>,
    rx: Receiver<PublishedAtalaObject>,
}

pub struct InMemoryDltSource {
    rx: Receiver<PublishedAtalaObject>,
}

pub struct InMemoryDltSink {
    delay: tokio::time::Duration,
    tx: Sender<PublishedAtalaObject>,
    block_number: u64,
}

impl InMemoryDlt {
    pub fn new(delay: tokio::time::Duration) -> Self {
        let (tx, rx) = mpsc::channel(2048);
        Self { delay, tx, rx }
    }

    pub fn split(self) -> (InMemoryDltSource, InMemoryDltSink) {
        let source = InMemoryDltSource { rx: self.rx };
        let sink = InMemoryDltSink {
            delay: self.delay,
            tx: self.tx,
            block_number: 0,
        };
        (source, sink)
    }
}

impl DltSource for InMemoryDltSource {
    fn receiver(self) -> Result<Receiver<PublishedAtalaObject>, String> {
        Ok(self.rx)
    }
}

impl DltSink for InMemoryDltSink {
    fn send(&mut self, atala_object: AtalaObject) {
        let owned_delay = self.delay;
        let owned_tx = self.tx.clone();
        let block_number = self.block_number;
        tokio::spawn(async move {
            tokio::time::sleep(owned_delay).await;
            let block_metadata = BlockMetadata {
                slot_number: block_number,
                block_number,
                cbt: Utc::now(),
                absn: 0,
            };
            let published_atala_object = PublishedAtalaObject {
                block_metadata,
                atala_object,
            };
            let send_result = owned_tx.send(published_atala_object).await;
            if let Err(e) = send_result {
                tracing::error!("Error sending AtalaObject to InMemoryDlt: {}", e);
            }
        });
        self.block_number += 1;
    }
}
