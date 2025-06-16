#![feature(error_reporter)]

use identus_did_prism::dlt::PublishedPrismObject;
use tokio::sync::mpsc::Receiver;

pub mod dlt;
mod indexing;
pub mod repo;

pub use indexing::{run_indexer_loop, run_sync_loop};

pub trait DltSource {
    fn receiver(self) -> Result<Receiver<PublishedPrismObject>, String>;
}
