use identus_did_prism::prelude::SignedPrismOperation;

pub trait DltSink {
    fn publish(&self, ops: Vec<SignedPrismOperation>) -> Result<(), String>;
}
