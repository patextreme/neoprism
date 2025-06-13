pub mod did;
pub mod dlt;
pub mod error;
mod macros;
pub mod prelude;
pub mod protocol;
pub mod utils;

#[allow(clippy::doc_lazy_continuation)]
pub mod proto {
    use identus_apollo::hash::{Sha256Digest, sha256};
    use prost::Message;

    include!(concat!(env!("OUT_DIR"), "/proto.rs"));

    impl PrismOperation {
        pub fn operation_hash(&self) -> Sha256Digest {
            let bytes = self.encode_to_vec();
            sha256(bytes)
        }
    }

    impl SignedPrismOperation {
        pub fn operation_hash(&self) -> Option<Sha256Digest> {
            self.operation.as_ref().map(|op| op.operation_hash())
        }
    }
}
