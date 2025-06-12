use identus_did_prism_indexer::{DltSource, run_sync_loop};
use indexer_storage::PostgresDb;

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

    pub async fn run(self) -> anyhow::Result<()> {
        run_sync_loop(self.store, self.source).await
    }
}
