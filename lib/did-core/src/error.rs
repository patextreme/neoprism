#[derive(Debug, derive_more::From, derive_more::Display, derive_more::Error)]
#[display("{source}")]
pub struct Error {
    source: identity_did::Error,
}
