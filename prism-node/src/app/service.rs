use prism_core::did::{CanonicalPrismDid, PrismDid, PrismDidLike};
use prism_core::protocol::resolver::{resolve_published, resolve_unpublished, ResolutionDebug, ResolutionResult};
use prism_core::store::OperationStore;
use prism_core::utils::paging::Paginated;
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
        let canonical_did = did.clone().into_canonical();

        let tx = self.db.begin().await?;
        let operations = tx.get_operations_by_did(&canonical_did).await?;
        tx.commit().await?;

        if operations.is_empty() {
            match did {
                PrismDid::Canonical(_) => Ok((ResolutionResult::NotFound, vec![])),
                PrismDid::LongForm(long_form_did) => {
                    // TODO: make error more consistent for not found and other error. It's good enough for now.
                    let operation = long_form_did.operation()?;
                    let (resolution_result, error) = resolve_unpublished(operation);
                    match error {
                        Some(e) => Err(e)?,
                        None => Ok((resolution_result, vec![])),
                    }
                }
            }
        } else {
            Ok(resolve_published(operations))
        }
    }

    pub async fn get_all_dids(&self, page: Option<u64>) -> anyhow::Result<Paginated<CanonicalPrismDid>> {
        let page = page.unwrap_or(0);
        let tx = self.db.begin().await?;
        let dids = tx.get_all_dids(page, 100).await?;
        tx.commit().await?;
        Ok(dids)
    }
}
