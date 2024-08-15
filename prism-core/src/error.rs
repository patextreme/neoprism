use crate::utils::Location;

pub(crate) type StdError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, derive_more::From, derive_more::Display, derive_more::Error)]
pub enum Error {
    #[from]
    #[display("{_0}")]
    InvalidInputSize(InvalidInputSizeError),
    #[from]
    #[display("{_0}")]
    Codec(crate::utils::codec::Error),
    #[display("unable to connect to DLT {location}")]
    DltConnection { source: StdError, location: Location },
}

#[derive(Debug, derive_more::Display, derive_more::Error)]
pub enum InvalidInputSizeError {
    #[display("expected input size of {expected} but got {actual} for type {type_name} {location}")]
    NotExact {
        expected: usize,
        actual: usize,
        type_name: &'static str,
        location: Location,
    },
    #[display("input size has a limit of {limit} but got {actual} for type {type_name} {location}")]
    TooBig {
        limit: usize,
        actual: usize,
        type_name: &'static str,
        location: Location,
    },
}
