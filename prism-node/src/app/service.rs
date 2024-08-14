use prism_core::did::{CanonicalPrismDid, PrismDid, PrismDidLike};
use prism_core::protocol::resolver::{resolve, ResolutionDebug, ResolutionResult};
use prism_core::store::OperationStore;
use prism_storage::PostgresDb;

pub struct DidService {
    db: PostgresDb,
}

impl DidService {
    pub fn new(db: &PostgresDb) -> Self {
        Self { db: db.clone() }
    }

    pub async fn resolve_did(&self, did: &str) -> anyhow::Result<(ResolutionResult, ResolutionDebug)> {
        let did: PrismDid = did.parse()?;
        let canonical_did = did.into_canonical();

        let tx = self.db.begin().await?;
        let operations = tx.get_operations_by_did(&canonical_did).await?;
        tx.commit().await?;
        Ok(resolve(operations))
    }

    pub async fn get_all_dids(&self) -> anyhow::Result<Vec<CanonicalPrismDid>> {
        let tx = self.db.begin().await?;
        let dids = tx.get_all_dids().await?;
        tx.commit().await?;
        Ok(dids)
    }
}
