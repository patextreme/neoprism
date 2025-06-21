use identus_did_prism::dlt::{DltCursor, PublishedPrismObject};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;

use crate::DltSource;
use crate::dlt::common::CursorPersistWorker;
use crate::dlt::error::DltError;
use crate::repo::DltCursorRepo;

pub struct DbSyncSource<Store: DltCursorRepo + Send + 'static> {
    store: Store,
    dbsync_url: String,
    cursor_tx: tokio::sync::watch::Sender<Option<DltCursor>>,
}

impl<E, Store: DltCursorRepo<Error = E> + Send + 'static> DbSyncSource<Store> {
    pub fn new(store: Store, dbsync_url: &str) -> Self {
        let (cursor_tx, _) = tokio::sync::watch::channel::<Option<DltCursor>>(None);
        Self {
            store,
            dbsync_url: dbsync_url.to_string(),
            cursor_tx,
        }
    }
}

impl<E, Store: DltCursorRepo<Error = E> + Send + 'static> DltSource for DbSyncSource<Store> {
    fn receiver(self) -> Result<Receiver<PublishedPrismObject>, String> {
        let (event_tx, rx) = tokio::sync::mpsc::channel::<PublishedPrismObject>(1024);

        let cursor_persist_worker = CursorPersistWorker::new(self.store, self.cursor_tx.subscribe());
        let stream_worker = DbSyncStreamWorker {
            dbsync_url: self.dbsync_url,
            cursor_tx: self.cursor_tx,
            event_tx,
        };

        cursor_persist_worker.spawn();
        stream_worker.spawn();

        Ok(rx)
    }
}

struct DbSyncStreamWorker {
    dbsync_url: String,
    cursor_tx: tokio::sync::watch::Sender<Option<DltCursor>>,
    event_tx: Sender<PublishedPrismObject>,
}

impl DbSyncStreamWorker {
    fn spawn(self) -> JoinHandle<Result<(), DltError>> {
        const RESTART_DELAY: std::time::Duration = std::time::Duration::from_secs(10);
        tokio::spawn(async move {
            let db_url = self.dbsync_url;
            let event_tx = self.event_tx;
            loop {
                let pool = PgPoolOptions::new().max_connections(1).connect(&db_url).await;
                match pool {
                    Ok(pool) => {
                        Self::stream_loop(pool, event_tx.clone()).await;
                    }
                    Err(e) => {
                        tracing::error!("Unable to connect to dbsync database: {}", e);
                    }
                }

                tracing::error!(
                    "DbSync pipeline terminated, Restarting in {} seconds",
                    RESTART_DELAY.as_secs()
                );

                std::thread::sleep(RESTART_DELAY);
            }
        })
    }

    async fn stream_loop(pool: PgPool, event_tx: Sender<PublishedPrismObject>) -> DltError {
        // TODO: implement dbsync query here ...
        loop {}
    }
}
