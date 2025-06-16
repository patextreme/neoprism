use strum::VariantArray;

pub mod error;

#[cfg(feature = "oura")]
pub mod oura;

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
