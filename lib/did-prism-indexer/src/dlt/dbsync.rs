use identus_did_prism::dlt::PublishedPrismObject;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{ConnectOptions, PgPool};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;
use tracing::Instrument;

use crate::DltSource;
use crate::dlt::error::DltError;
use crate::repo::DltCursorRepo;

pub struct DbSyncSource<Store: DltCursorRepo + Send + 'static> {
    store: Store,
    dbsync_url: String,
}

impl<E, Store: DltCursorRepo<Error = E> + Send + 'static> DbSyncSource<Store> {
    pub fn new(store: Store, dbsync_url: &str) -> Self {
        Self {
            store,
            dbsync_url: dbsync_url.to_string(),
        }
    }
}

impl<E, Store: DltCursorRepo<Error = E> + Send + 'static> DltSource for DbSyncSource<Store> {
    fn receiver(self) -> Result<Receiver<PublishedPrismObject>, String> {
        let (event_tx, rx) = tokio::sync::mpsc::channel::<PublishedPrismObject>(1024);

        let stream_worker = DbSyncStreamWorker {
            store: self.store,
            dbsync_url: self.dbsync_url,
            event_tx,
        };

        stream_worker.spawn();

        Ok(rx)
    }
}

struct DbSyncStreamWorker<Store> {
    store: Store,
    dbsync_url: String,
    event_tx: Sender<PublishedPrismObject>,
}

impl<E, Store: DltCursorRepo<Error = E> + Send + 'static> DbSyncStreamWorker<Store> {
    fn spawn(self) -> JoinHandle<Result<(), DltError>> {
        const RESTART_DELAY: std::time::Duration = std::time::Duration::from_secs(10);
        let db_url = self.dbsync_url;
        let event_tx = self.event_tx;
        tokio::spawn(async move {
            loop {
                // TODO: continue here ...
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

    async fn stream_loop(pool: PgPool, event_tx: Sender<PublishedPrismObject>) {
        // TODO: implement streaming logic
        ()
    }
}
