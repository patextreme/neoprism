use identus_did_prism::dlt::{DltCursor, PublishedPrismObject};
use identus_did_prism::location;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tokio::sync::{mpsc, watch};
use tokio::task::JoinHandle;

use crate::DltSource;
use crate::dlt::common::CursorPersistWorker;
use crate::dlt::dbsync::model::MetadataProjection;
use crate::dlt::error::DltError;
use crate::repo::DltCursorRepo;

mod model {
    use chrono::{DateTime, Utc};
    use sqlx::FromRow;

    #[derive(Debug, FromRow)]
    pub struct MetadataProjection {
        pub time: DateTime<Utc>,
        pub slot_no: i64,
        pub block_no: i32,
        pub block_hash: Vec<u8>,
        pub metadata: serde_json::Value,
    }
}

pub struct DbSyncSource<Store: DltCursorRepo + Send + 'static> {
    store: Store,
    dbsync_url: String,
    sync_cursor_tx: watch::Sender<Option<DltCursor>>,
}

impl<E, Store: DltCursorRepo<Error = E> + Send + 'static> DbSyncSource<Store> {
    pub fn new(store: Store, dbsync_url: &str) -> Self {
        let (cursor_tx, _) = tokio::sync::watch::channel::<Option<DltCursor>>(None);
        Self {
            store,
            dbsync_url: dbsync_url.to_string(),
            sync_cursor_tx: cursor_tx,
        }
    }
}

impl<E, Store: DltCursorRepo<Error = E> + Send + 'static> DltSource for DbSyncSource<Store> {
    fn sync_cursor(&self) -> watch::Receiver<Option<DltCursor>> {
        self.sync_cursor_tx.subscribe()
    }

    fn into_stream(self) -> Result<mpsc::Receiver<PublishedPrismObject>, String> {
        let (event_tx, rx) = tokio::sync::mpsc::channel::<PublishedPrismObject>(1024);

        let cursor_persist_worker = CursorPersistWorker::new(self.store, self.sync_cursor_tx.subscribe());
        let stream_worker = DbSyncStreamWorker {
            dbsync_url: self.dbsync_url,
            sync_cursor_tx: self.sync_cursor_tx,
            event_tx,
        };

        cursor_persist_worker.spawn();
        stream_worker.spawn();

        Ok(rx)
    }
}

struct DbSyncStreamWorker {
    dbsync_url: String,
    sync_cursor_tx: tokio::sync::watch::Sender<Option<DltCursor>>,
    event_tx: mpsc::Sender<PublishedPrismObject>,
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

    async fn stream_loop(pool: PgPool, event_tx: mpsc::Sender<PublishedPrismObject>) -> Result<(), DltError> {
        let mut slot_cursor = 0; // FIXME: load from last persisted cursor
        loop {
            let metadata_rows = Self::fetch_metadata(&pool, slot_cursor).await?;
            if let Some(latest_slot) = metadata_rows.iter().map(|i| i.slot_no).max() {
                slot_cursor = latest_slot;
            }
            for row in metadata_rows.iter() {
                println!("{:?}", row.slot_no);
            }
            println!("slot_cursor: {:?}", slot_cursor);

            // sleep if we don't find a new block to avoid spamming db sync
            if metadata_rows.is_empty() {
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            }
        }
    }

    async fn fetch_metadata(pool: &PgPool, from_slot: i64) -> Result<Vec<MetadataProjection>, DltError> {
        let rows = sqlx::query_as(
            r#"
SELECT
    b."time" AT TIME ZONE 'UTC' AS "time",
    b.slot_no,
    b.block_no,
    b.hash AS block_hash,
    tx_meta.json AS metadata
FROM tx_metadata AS tx_meta
LEFT JOIN tx ON tx_meta.tx_id = tx.id
LEFT JOIN block AS b ON block_id = b.id
WHERE tx_meta.key = 21325 AND b.slot_no > $1 AND b.block_no <= (SELECT max(block_no) - 112 FROM block)
ORDER BY b.block_no
LIMIT 200
            "#,
        )
        .bind(from_slot)
        .fetch_all(pool)
        .await
        .inspect_err(|e| tracing::error!("Failed to get data from dbsync: {}", e))
        .map_err(|_| DltError::Disconnected { location: location!() })?;
        Ok(rows)
    }
}
