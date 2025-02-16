use std::collections::VecDeque;

use super::{init_published_context, DidStateProcessingContext, ProcessError, Published};
use crate::did::DidState;
use crate::dlt::OperationMetadata;
use crate::prelude::AtalaOperation;
use crate::proto::SignedAtalaOperation;
use crate::protocol::init_unpublished_context;

type OperationList = VecDeque<(OperationMetadata, SignedAtalaOperation)>;
pub type ResolutionDebug = Vec<(OperationMetadata, SignedAtalaOperation, Option<ProcessError>)>;

pub fn resolve_unpublished(operation: AtalaOperation) -> Result<DidState, ProcessError> {
    log::debug!("resolving unpublished DID data");
    init_unpublished_context(operation).map(|ctx| ctx.finalize())
}

pub fn resolve_published(
    mut operations: Vec<(OperationMetadata, SignedAtalaOperation)>,
) -> (Option<DidState>, ResolutionDebug) {
    log::debug!("resolving published DID data from {} operations", operations.len());
    operations.sort_by(|a, b| OperationMetadata::compare_time_asc(&a.0, &b.0));
    let mut operations: OperationList = operations.into();

    // Initialize first valid CreateOperation
    let (state_ctx, mut debug) = init_state_ops(&mut operations);
    let Some(mut state_ctx) = state_ctx else {
        return (None, debug);
    };

    // Iterate all remaining operations and apply new state
    while let Some((metadata, operation)) = operations.pop_front() {
        let (new_ctx, error) = state_ctx.process(operation.clone(), metadata.clone());
        state_ctx = new_ctx;
        debug.push((metadata, operation, error));
    }

    (Some(state_ctx.finalize()), debug)
}

fn init_state_ops(operations: &mut OperationList) -> (Option<DidStateProcessingContext<Published>>, ResolutionDebug) {
    let mut debug = Vec::with_capacity(operations.len());
    while let Some((metadata, operation)) = operations.pop_front() {
        let result = init_published_context(operation.clone(), metadata.clone());
        match result {
            Ok(state_ctx) => {
                debug.push((metadata, operation, None));
                return (Some(state_ctx), debug);
            }
            Err(e) => {
                log::debug!("unable to initialize DIDState from operation: {:?}", e);
                debug.push((metadata, operation, Some(e)));
            }
        }
    }
    (None, debug)
}
