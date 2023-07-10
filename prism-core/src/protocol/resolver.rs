use super::DidStateOps;
use crate::{did::DidState, dlt::OperationMetadata, proto::SignedAtalaOperation};
use std::collections::VecDeque;

type OperationList = VecDeque<(OperationMetadata, SignedAtalaOperation)>;

#[derive(Debug, thiserror::Error)]
pub enum ResolutionError {
    #[error("The DID is not found")]
    DidNotFound,
}

pub fn resolve(
    mut operations: Vec<(OperationMetadata, SignedAtalaOperation)>,
) -> Result<DidState, ResolutionError> {
    operations.sort_by(|(t_a, _), (t_b, _)| t_a.cmp(t_b));
    let operations: OperationList = OperationList::from(operations);

    log::debug!("resolving DID data from {} operations", operations.len());

    let (mut state_ops, mut remaining) = init_state_ops(operations)?;
    while let Some((metadata, operation)) = remaining.pop_front() {
        state_ops = state_ops.process(operation, metadata);
    }

    Ok(state_ops.finalize())
}

fn init_state_ops(
    mut operations: OperationList,
) -> Result<(DidStateOps, OperationList), ResolutionError> {
    while let Some((metadata, operation)) = operations.pop_front() {
        let result = DidStateOps::new(operation, metadata);
        match result {
            Ok(state_ops) => return Ok((state_ops, operations)),
            Err(e) => log::debug!("unable to initialize DIDState from operation: {:?}", e,),
        }
    }
    Err(ResolutionError::DidNotFound)
}
