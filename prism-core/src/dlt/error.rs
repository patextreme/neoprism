use crate::error::StdError;

#[derive(Debug, derive_more::From, derive_more::Display, derive_more::Error)]
pub enum MetadataReadError {
    #[display("metadata is not a valid json on block {block_hash:?} tx {tx_idx:?}")]
    InvalidJsonType {
        source: StdError,
        block_hash: Option<String>,
        tx_idx: Option<usize>,
    },
    #[display("cannot decode atala_block hex on block {block_hash:?} tx {tx_idx:?}")]
    AtalaBlockHexDecode {
        source: crate::utils::codec::Error,
        block_hash: Option<String>,
        tx_idx: Option<usize>,
    },
    #[display("cannot decode atala_block protobuf on block {block_hash:?} tx {tx_idx:?}")]
    AtalaBlockProtoDecode {
        source: prost::DecodeError,
        block_hash: Option<String>,
        tx_idx: Option<usize>,
    },
    #[display("timestamp {timestamp} is invalid on block {block_hash:?} tx {tx_idx:?}")]
    InvalidBlockTimestamp {
        source: time::error::ComponentRange,
        block_hash: Option<String>,
        tx_idx: Option<usize>,
        timestamp: i64,
    },
    #[display("block property '{name}' is missing on block {block_hash:?} tx {tx_idx:?}")]
    MissingBlockProperty {
        block_hash: Option<String>,
        tx_idx: Option<usize>,
        name: &'static str,
    },
}
