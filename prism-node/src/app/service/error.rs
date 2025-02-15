use prism_core::{did, protocol};

#[derive(Debug, derive_more::From, derive_more::Display, derive_more::Error)]
pub enum ResolutionError {
    #[from]
    #[display("invalid did input")]
    InvalidDid { source: InvalidDid },
    #[display("did is not found")]
    NotFound,
    #[from]
    #[display("unexpected server error")]
    InternalError { source: anyhow::Error },
}

#[derive(Debug, derive_more::From, derive_more::Display, derive_more::Error)]
pub enum InvalidDid {
    #[from]
    #[display("cannot parse the did")]
    ParsingFail { source: did::Error },
    #[from]
    #[display("cannot process did state from did")]
    ProcessFail { source: protocol::error::ProcessError },
}
