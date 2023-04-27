use crate::proto::AtalaObject;
use chrono::{DateTime, Utc};
use tokio::sync::mpsc::{self, Receiver, Sender};

#[cfg(feature = "cardano")]
pub mod cardano;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DltTimestamp {
    pub timestamp: DateTime<Utc>,
    pub tx_idx: u32,
}

impl PartialOrd for DltTimestamp {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let tuple_self = (self.timestamp, self.tx_idx);
        let tuple_other = (other.timestamp, other.tx_idx);
        tuple_self.partial_cmp(&tuple_other)
    }
}

impl Ord for DltTimestamp {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let tuple_self = (self.timestamp, self.tx_idx);
        let tuple_other = (other.timestamp, other.tx_idx);
        tuple_self.cmp(&tuple_other)
    }
}

#[derive(Debug, Clone)]
pub struct PublishedAtalaObject {
    pub dlt_timestamp: DltTimestamp,
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
        tokio::spawn(async move {
            tokio::time::sleep(owned_delay).await;
            let dlt_timestamp = DltTimestamp {
                timestamp: Utc::now(),
                tx_idx: 0,
            };
            let published_atala_object = PublishedAtalaObject {
                dlt_timestamp,
                atala_object,
            };
            let send_result = owned_tx.send(published_atala_object).await;
            if let Err(e) = send_result {
                log::error!("Error sending AtalaObject to InMemoryDlt: {}", e);
            }
        });
    }
}
