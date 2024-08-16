use crate::utils::Location;

pub(crate) type StdError = Box<dyn std::error::Error + Send + Sync>;

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
