use error::{InvalidDid, ResolutionError};
use identus_did_prism::did::{CanonicalPrismDid, DidState, PrismDid, PrismDidOps};
use identus_did_prism::protocol::resolver::{ResolutionDebug, resolve_published, resolve_unpublished};
use identus_did_prism::utils::paging::Paginated;
use identus_did_prism_indexer::repo::OperationRepo;
use indexer_storage::PostgresDb;

pub mod error;

#[derive(Clone)]
pub struct DidService {
    db: PostgresDb,
}

impl DidService {
    pub fn new(db: &PostgresDb) -> Self {
        Self { db: db.clone() }
    }

    pub async fn resolve_did(&self, did: &str) -> (Result<(PrismDid, DidState), ResolutionError>, ResolutionDebug) {
        let mut debug = vec![];
        let result = self.resolve_did_logic(did, &mut debug).await;
        (result, debug)
    }

    async fn resolve_did_logic(
        &self,
        did: &str,
        debug_acc: &mut ResolutionDebug,
    ) -> Result<(PrismDid, DidState), ResolutionError> {
        let did: PrismDid = did.parse().map_err(|e| InvalidDid::ParsingFail { source: e })?;
        let canonical_did = did.clone().into_canonical();

        todo!("get operation after indexing is implemented");
        let operations = vec![];
        // let operations = self
        //     .db
        //     .get_operations_by_did(&canonical_did)
        //     .await
        //     .map_err(|e| ResolutionError::InternalError { source: e.into() })?;

        if operations.is_empty() {
            match &did {
                PrismDid::Canonical(_) => Err(ResolutionError::NotFound)?,
                PrismDid::LongForm(long_form_did) => {
                    let operation = long_form_did
                        .operation()
                        .map_err(|e| InvalidDid::ParsingFail { source: e })?;
                    let did_state =
                        resolve_unpublished(operation).map_err(|e| InvalidDid::ProcessFail { source: e })?;
                    Ok((did, did_state))
                }
            }
        } else {
            let (did_state, debug) = resolve_published(operations);
            debug_acc.extend(debug);
            match did_state {
                Some(did_state) => Ok((did, did_state)),
                None => Err(ResolutionError::NotFound),
            }
        }
    }

    pub async fn get_all_dids(&self, page: Option<u32>) -> anyhow::Result<Paginated<CanonicalPrismDid>> {
        let page = page.unwrap_or(0);
        let dids = self.db.get_all_dids(page, 100).await?;
        Ok(dids)
    }
}
