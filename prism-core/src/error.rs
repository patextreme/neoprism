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

#[derive(Debug, derive_more::From, derive_more::Display, derive_more::Error)]
#[display("unexpected input size: expected {expected} but got {actual} for type {type_name} {location}")]
pub struct InvalidInputSizeError {
    pub expected: usize,
    pub actual: usize,
    pub type_name: &'static str,
    pub location: Location,
}
