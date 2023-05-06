use crate::proto::AtalaObject;
use chrono::{DateTime, Utc};
use tokio::sync::mpsc::{self, Receiver, Sender};

#[cfg(feature = "cardano")]
pub mod cardano;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockTimestamp {
    /// Cardano block timestamp
    pub cbt: DateTime<Utc>,
    /// AtalaBlock seqeuence number
    ///
    /// This is used to order AtalaBlock within the same Cardano block
    pub absn: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationTimestamp {
    /// AtalaBlock timestamp
    pub block_timestamp: BlockTimestamp,
    /// Operation sequence number
    ///
    /// This is used to order AtalaOperation within the same AtalaBlock
    pub osn: usize,
}

impl BlockTimestamp {
    pub fn into_operation_ts(self, atala_block_idx: usize) -> OperationTimestamp {
        OperationTimestamp {
            block_timestamp: self,
            osn: atala_block_idx,
        }
    }
}

impl PartialOrd for BlockTimestamp {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let tuple_self = (self.cbt, self.absn);
        let tuple_other = (other.cbt, other.absn);
        tuple_self.partial_cmp(&tuple_other)
    }
}

impl Ord for BlockTimestamp {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let tuple_self = (self.cbt, self.absn);
        let tuple_other = (other.cbt, other.absn);
        tuple_self.cmp(&tuple_other)
    }
}

impl PartialOrd for OperationTimestamp {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let tuple_self = (&self.block_timestamp, self.osn);
        let tuple_other = (&other.block_timestamp, other.osn);
        tuple_self.partial_cmp(&tuple_other)
    }
}

impl Ord for OperationTimestamp {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let tuple_self = (&self.block_timestamp, self.osn);
        let tuple_other = (&other.block_timestamp, other.osn);
        tuple_self.cmp(&tuple_other)
    }
}

#[derive(Debug, Clone)]
pub struct PublishedAtalaObject {
    pub block_timestamp: BlockTimestamp,
    pub atala_object: AtalaObject,
}

pub trait DltSource {
    fn receiver(self) -> Receiver<PublishedAtalaObject>;
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
    slot: u64,
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
            slot: 0,
        };
        (source, sink)
    }
}

impl DltSource for InMemoryDltSource {
    fn receiver(self) -> Receiver<PublishedAtalaObject> {
        self.rx
    }
}

impl DltSink for InMemoryDltSink {
    fn send(&mut self, atala_object: AtalaObject) {
        let owned_delay = self.delay;
        let owned_tx = self.tx.clone();
        let owned_slot = self.slot;
        tokio::spawn(async move {
            tokio::time::sleep(owned_delay).await;
            let block_timestamp = BlockTimestamp {
                cbt: Utc::now(),
                absn: 0,
            };
            let published_atala_object = PublishedAtalaObject {
                block_timestamp,
                atala_object,
            };
            let send_result = owned_tx.send(published_atala_object).await;
            if let Err(e) = send_result {
                log::error!("Error sending AtalaObject to InMemoryDlt: {}", e);
            }
        });
        self.slot += 1;
    }
}
