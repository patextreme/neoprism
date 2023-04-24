use crate::proto::AtalaBlock;
use tokio::sync::mpsc::{self, Receiver, Sender};

pub trait DltSource {
    fn receiver(self) -> Receiver<AtalaBlock>;
}

pub trait DltSink {
    fn send(&mut self, atala_object: AtalaBlock);
}

pub struct InMemoryDlt {
    delay: tokio::time::Duration,
    tx: Sender<AtalaBlock>,
    rx: Receiver<AtalaBlock>,
}

pub struct InMemoryDltSource {
    rx: Receiver<AtalaBlock>,
}

pub struct InMemoryDltSink {
    delay: tokio::time::Duration,
    tx: Sender<AtalaBlock>,
}

impl InMemoryDlt {
    pub fn new(delay: tokio::time::Duration) -> Self {
        let (tx, rx) = mpsc::channel::<AtalaBlock>(2048);
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
    fn receiver(self) -> Receiver<AtalaBlock> {
        self.rx
    }
}

impl DltSink for InMemoryDltSink {
    fn send(&mut self, atala_block: AtalaBlock) {
        let owned_delay = self.delay;
        let owned_tx = self.tx.clone();
        tokio::spawn(async move {
            tokio::time::sleep(owned_delay).await;
            let send_result = owned_tx.send(atala_block).await;
            if let Err(e) = send_result {
                log::error!("Error sending AtalaObject to InMemoryDlt: {}", e);
            }
        });
    }
}
