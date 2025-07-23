use identus_did_prism::prelude::SignedPrismOperation;

pub mod repo;

pub trait DltSink {
    fn publish(&self, ops: Vec<SignedPrismOperation>) -> Result<(), String>;
}
