use strum::VariantArray;

pub mod error;

#[cfg(any(feature = "oura", feature = "dbsync"))]
mod common;

#[cfg(feature = "oura")]
pub mod oura;

#[cfg(feature = "dbsync")]
pub mod dbsync;

#[derive(Debug, Clone, PartialEq, Eq, strum::Display, strum::EnumString, strum::VariantArray)]
pub enum NetworkIdentifier {
    #[strum(serialize = "mainnet")]
    Mainnet,
    #[strum(serialize = "preprod")]
    Preprod,
    #[strum(serialize = "preview")]
    Preview,
}

impl NetworkIdentifier {
    pub fn variants() -> &'static [Self] {
        Self::VARIANTS
    }
}
