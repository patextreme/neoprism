use crate::proto::AtalaObject;
use chrono::{DateTime, Utc};
use tokio::sync::mpsc::{self, Receiver, Sender};

#[cfg(feature = "cardano")]
pub mod cardano;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OperationMetadata {
    /// AtalaBlock metadata
    pub block_metadata: BlockMetadata,
    /// Operation sequence number
    ///
    /// This is used to order AtalaOperation within the same AtalaBlock
    pub osn: u32,
}

impl PartialOrd for BlockMetadata {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let tuple_self = (self.cbt, self.absn);
        let tuple_other = (other.cbt, other.absn);
        tuple_self.partial_cmp(&tuple_other)
    }
}

impl Ord for BlockMetadata {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let tuple_self = (self.cbt, self.absn);
        let tuple_other = (other.cbt, other.absn);
        tuple_self.cmp(&tuple_other)
    }
}

impl PartialOrd for OperationMetadata {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let tuple_self = (&self.block_metadata, self.osn);
        let tuple_other = (&other.block_metadata, other.osn);
        tuple_self.partial_cmp(&tuple_other)
    }
}

impl Ord for OperationMetadata {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let tuple_self = (&self.block_metadata, self.osn);
        let tuple_other = (&other.block_metadata, other.osn);
        tuple_self.cmp(&tuple_other)
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
                log::error!("Error sending AtalaObject to InMemoryDlt: {}", e);
            }
        });
        self.block_number += 1;
    }
}
