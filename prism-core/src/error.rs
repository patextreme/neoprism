use crate::utils::Location;

type StdError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, derive_more::From, derive_more::Display, derive_more::Error)]
pub enum Error {
    #[display("{_0}")]
    Codec(crate::utils::codec::Error),
    #[display("unable to connect to DLT")]
    DltConnection { source: StdError, location: Location },
}
