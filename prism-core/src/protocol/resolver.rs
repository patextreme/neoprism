use super::DidStateOps;
use crate::{did::DidState, dlt::OperationTimestamp, proto::SignedAtalaOperation};
use std::collections::VecDeque;

type OperationList = VecDeque<(OperationTimestamp, SignedAtalaOperation)>;

#[derive(Debug, Clone, thiserror::Error)]
pub enum ResolutionError {
    #[error("The DID is not found")]
    DidNotFound,
}

pub fn resolve(
    mut operations: Vec<(OperationTimestamp, SignedAtalaOperation)>,
) -> Result<DidState, ResolutionError> {
    operations.sort_by_key(|i| i.0.clone());
    let operations: OperationList = OperationList::from(operations);

    let (mut state_ops, mut remaining) = init_state_ops(operations)?;
    while let Some((timestamp, operation)) = remaining.pop_front() {
        state_ops = state_ops.process(operation, timestamp);
    }

    Ok(state_ops.finalize())
}

fn init_state_ops(
    mut operations: OperationList,
) -> Result<(DidStateOps, OperationList), ResolutionError> {
    while let Some((timestamp, operation)) = operations.pop_front() {
        if let Ok(state_ops) = DidStateOps::new(operation, timestamp) {
            return Ok((state_ops, operations));
        }
    }
    Err(ResolutionError::DidNotFound)
}
