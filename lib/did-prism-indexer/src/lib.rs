#![feature(error_reporter)]

use identus_did_prism::dlt::PublishedPrismObject;
use tokio::sync::mpsc::Receiver;

pub mod dlt;
pub mod repo;

pub trait DltSource {
    fn receiver(self) -> Result<Receiver<PublishedPrismObject>, String>;
}
