use identus_did_prism::dlt::DltCursor;
use identus_did_prism_indexer::{DltSource, run_indexer_loop, run_sync_loop};
use node_storage::PostgresDb;
use tokio::sync::watch;

pub struct DltSyncWorker<Src> {
    store: PostgresDb,
    source: Src,
}

impl<Src> DltSyncWorker<Src>
where
    Src: DltSource,
{
    pub fn new(store: PostgresDb, source: Src) -> Self {
        Self { store, source }
    }

    pub fn sync_cursor(&self) -> watch::Receiver<Option<DltCursor>> {
        self.source.sync_cursor()
    }

    pub async fn run(self) -> anyhow::Result<()> {
        run_sync_loop(&self.store, self.source).await // block forever
    }
}

pub struct DltIndexWorker {
    store: PostgresDb,
    index_interval: u64,
}

impl DltIndexWorker {
    pub fn new(store: PostgresDb, index_interval: u64) -> Self {
        Self { store, index_interval }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        loop {
            let result = run_indexer_loop(&self.store).await;
            if let Err(e) = result {
                tracing::error!("{:?}", e);
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(self.index_interval)).await;
        }
    }
}
