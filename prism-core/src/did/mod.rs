use crate::crypto::{codec::Base64UrlStrNoPad, hash::Sha256Digest};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PrismDid {
    Canonical(CanonicalPrismDid),
    LongForm(LongFormPrismDid),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CanonicalPrismDid {
    pub suffix: Sha256Digest,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LongFormPrismDid {
    pub suffix: Sha256Digest,
    pub encoded_state: Base64UrlStrNoPad,
}

pub trait Did {
    fn method(&self) -> &str;
    fn suffix(&self) -> &Sha256Digest;
}

impl Did for PrismDid {
    fn method(&self) -> &str {
        "prism"
    }

    fn suffix(&self) -> &Sha256Digest {
        match self {
            PrismDid::Canonical(did) => &did.suffix,
            PrismDid::LongForm(did) => &did.suffix,
        }
    }
}

impl Did for CanonicalPrismDid {
    fn method(&self) -> &str {
        "prism"
    }

    fn suffix(&self) -> &Sha256Digest {
        &self.suffix
    }
}

impl Did for LongFormPrismDid {
    fn method(&self) -> &str {
        "prism"
    }

    fn suffix(&self) -> &Sha256Digest {
        &self.suffix
    }
}

impl From<CanonicalPrismDid> for PrismDid {
    fn from(did: CanonicalPrismDid) -> Self {
        PrismDid::Canonical(did)
    }
}

impl From<LongFormPrismDid> for PrismDid {
    fn from(did: LongFormPrismDid) -> Self {
        PrismDid::LongForm(did)
    }
}
