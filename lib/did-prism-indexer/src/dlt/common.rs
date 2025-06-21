use identus_apollo::hex::HexStr;
use identus_did_prism::dlt::DltCursor;
use tokio::task::JoinHandle;

use crate::dlt::error::DltError;
use crate::repo::DltCursorRepo;

pub struct CursorPersistWorker<Store: DltCursorRepo> {
    cursor_rx: tokio::sync::watch::Receiver<Option<DltCursor>>,
    store: Store,
}

impl<Store: DltCursorRepo + Send + 'static> CursorPersistWorker<Store> {
    pub fn new(store: Store, cursor_rx: tokio::sync::watch::Receiver<Option<DltCursor>>) -> Self {
        Self { cursor_rx, store }
    }

    pub fn spawn(mut self) -> JoinHandle<Result<(), DltError>> {
        const DELAY: tokio::time::Duration = tokio::time::Duration::from_secs(60);
        tracing::info!("Spawn cursor persist worker with {:?} interval", DELAY);
        tokio::spawn(async move {
            loop {
                let recv_result = self.cursor_rx.changed().await;
                tokio::time::sleep(DELAY).await;

                if let Err(e) = recv_result {
                    tracing::error!("Error getting cursor to persist: {}", e);
                }

                let cursor = self.cursor_rx.borrow_and_update().clone();
                let Some(cursor) = cursor else { continue };
                tracing::info!(
                    "Persisting cursor on slot ({}, {})",
                    cursor.slot,
                    HexStr::from(cursor.block_hash.as_slice()).to_string(),
                );

                let persist_result = self.store.set_cursor(cursor).await;
                if let Err(e) = persist_result {
                    tracing::error!("Error persisting cursor: {}", e);
                }
            }
        })
    }
}
