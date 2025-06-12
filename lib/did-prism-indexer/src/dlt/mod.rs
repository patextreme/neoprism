use identus_did_prism::dlt::PublishedPrismObject;
use identus_did_prism::proto::PrismObject;
use strum::VariantArray;
use tokio::sync::mpsc::Receiver;

pub mod error;

#[cfg(feature = "oura")]
pub mod oura;

pub trait DltSource {
    fn receiver(self) -> Result<Receiver<PublishedPrismObject>, String>;
}

pub trait DltSink {
    fn send(&mut self, prism_object: PrismObject);
}

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
