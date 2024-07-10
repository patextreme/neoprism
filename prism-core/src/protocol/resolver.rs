use std::collections::VecDeque;

use super::{DidStateProcessingContext, ProcessError};
use crate::did::DidState;
use crate::dlt::OperationMetadata;
use crate::proto::SignedAtalaOperation;

type OperationList = VecDeque<(OperationMetadata, SignedAtalaOperation)>;
pub type ResolutionDebug = Vec<(SignedAtalaOperation, Option<ProcessError>)>;

#[derive(Debug, Clone)]
pub enum ResolutionResult {
    Ok(DidState),
    NotFound,
}

pub fn resolve(mut operations: Vec<(OperationMetadata, SignedAtalaOperation)>) -> (ResolutionResult, ResolutionDebug) {
    log::debug!("resolving DID data from {} operations", operations.len());
    operations.sort_by(|a, b| OperationMetadata::compare_time_asc(&a.0, &b.0));
    let mut operations: OperationList = operations.into();

    // Initialize first valid CreateOperation
    let (state_ctx, mut debug) = init_state_ops(&mut operations);
    let Some(mut state_ctx) = state_ctx else {
        return (ResolutionResult::NotFound, debug);
    };

    // Iterate all remaning operations and apply new state
    while let Some((metadata, operation)) = operations.pop_front() {
        let (new_ctx, error) = state_ctx.process(operation.clone(), metadata);
        state_ctx = new_ctx;
        debug.push((operation, error));
    }

    (ResolutionResult::Ok(state_ctx.finalize()), debug)
}

fn init_state_ops(operations: &mut OperationList) -> (Option<DidStateProcessingContext>, ResolutionDebug) {
    let mut debug = Vec::with_capacity(operations.len());
    while let Some((metadata, operation)) = operations.pop_front() {
        let result = DidStateProcessingContext::new(operation.clone(), metadata);
        match result {
            Ok(state_ctx) => {
                debug.push((operation, None));
                return (Some(state_ctx), debug);
            }
            Err(e) => {
                log::debug!("unable to initialize DIDState from operation: {:?}", e);
                debug.push((operation, Some(e)));
            }
        }
    }
    (None, debug)
}
