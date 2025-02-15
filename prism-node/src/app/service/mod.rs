use error::{InvalidDid, ResolutionError};
use prism_core::did::{CanonicalPrismDid, DidState, PrismDid, PrismDidLike};
use prism_core::protocol::resolver::{resolve_published, resolve_unpublished, ResolutionDebug};
use prism_core::store::OperationStore;
use prism_core::utils::paging::Paginated;
use prism_storage::PostgresDb;

pub mod error;

pub struct DidService {
    db: PostgresDb,
}

impl DidService {
    pub fn new(db: &PostgresDb) -> Self {
        Self { db: db.clone() }
    }

    pub async fn resolve_did(&self, did: &str) -> Result<(PrismDid, DidState, ResolutionDebug), ResolutionError> {
        let did: PrismDid = did.parse().map_err(|e| InvalidDid::ParsingFail { source: e })?;
        let canonical_did = did.clone().into_canonical();

        let tx = self
            .db
            .begin()
            .await
            .map_err(|e| ResolutionError::InternalError { source: e.into() })?;
        let operations = tx
            .get_operations_by_did(&canonical_did)
            .await
            .map_err(|e| ResolutionError::InternalError { source: e.into() })?;
        tx.commit()
            .await
            .map_err(|e| ResolutionError::InternalError { source: e.into() })?;

        if operations.is_empty() {
            match &did {
                PrismDid::Canonical(_) => Err(ResolutionError::NotFound)?,
                PrismDid::LongForm(long_form_did) => {
                    let operation = long_form_did
                        .operation()
                        .map_err(|e| InvalidDid::ParsingFail { source: e })?;
                    let did_state =
                        resolve_unpublished(operation).map_err(|e| InvalidDid::ProcessFail { source: e })?;
                    Ok((did, did_state, vec![]))
                }
            }
        } else {
            let (did_state, debug) = resolve_published(operations);
            match did_state {
                Some(did_state) => Ok((did, did_state, debug)),
                None => Err(ResolutionError::NotFound),
            }
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
