use tokio::sync::mpsc::{self, Receiver};

use super::{DltSource, PublishedAtalaObject};
use crate::dlt::cardano::model::MetadataEvent;
use std::path::{Path, PathBuf};

pub struct OuraFileSource {
    path: PathBuf,
}

impl OuraFileSource {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    fn parse(self) -> Result<Vec<PublishedAtalaObject>, Box<dyn std::error::Error>> {
        let lines = std::fs::read_to_string(self.path)?;
        let mut atala_objects = Vec::with_capacity(lines.len());
        for line in lines.trim().split('\n') {
            let metadata_event: MetadataEvent = serde_json::from_str(line)?;
            let atala_object: PublishedAtalaObject = metadata_event.try_into()?;
            atala_objects.push(atala_object)
        }
        Ok(atala_objects)
    }
}

impl DltSource for OuraFileSource {
    fn receiver(self) -> Receiver<PublishedAtalaObject> {
        let (tx, rx) = mpsc::channel(2048);

        tokio::spawn(async move {
            let atala_objects = match self.parse() {
                Ok(atala_objects) => atala_objects,
                Err(e) => {
                    log::error!("Error parsing OuraFileSource: {}", e);
                    return;
                }
            };
            for atala_object in atala_objects {
                if let Err(e) = tx.send(atala_object).await {
                    log::error!("Error sending AtalaObject from OuraFileSource: {}", e);
                }
            }
        });

        rx
    }
}

mod model {
    use crate::{
        dlt::{DltTimestamp, PublishedAtalaObject},
        proto::AtalaObject,
    };
    use bytes::BytesMut;
    use chrono::{DateTime, TimeZone, Utc};
    use prost::Message;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct MetadataEvent {
        pub context: MetadataContext,
        pub metadata: Metadata,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct MetadataContext {
        pub block_hash: String,
        pub block_number: u64,
        pub slot: u64,
        pub timestamp: i64,
        pub tx_hash: String,
        pub tx_idx: u32,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct Metadata {
        pub label: String,
        pub map_json: MetadataMapJson,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct MetadataMapJson {
        pub c: Vec<String>,
        pub v: u64,
    }

    #[derive(Debug, Clone, thiserror::Error)]
    pub enum ConversionError {
        #[error("hex conversion error: {0}")]
        HexDecodeError(#[from] hex::FromHexError),
        #[error("protobuf decode error: {0}")]
        ProtoDecodeError(#[from] prost::DecodeError),
    }

    impl TryFrom<MetadataEvent> for AtalaObject {
        type Error = ConversionError;

        fn try_from(value: MetadataEvent) -> Result<Self, Self::Error> {
            let published_atala_object = PublishedAtalaObject::try_from(value)?;
            Ok(published_atala_object.atala_object)
        }
    }

    impl TryFrom<MetadataEvent> for PublishedAtalaObject {
        type Error = ConversionError;

        fn try_from(value: MetadataEvent) -> Result<Self, Self::Error> {
            let hex_group = value.metadata.map_json.c;
            let mut buf = BytesMut::with_capacity(64 * hex_group.len());

            for hex in hex_group {
                let b = hex::decode(hex)?;
                buf.extend(b);
            }

            let bytes = buf.freeze();
            let atala_object = AtalaObject::decode(bytes)?;
            let timestamp: DateTime<Utc> = Utc
                .timestamp_opt(value.context.timestamp, 0)
                .single()
                .ok_or_else(|| {
                ConversionError::ProtoDecodeError(prost::DecodeError::new("timestamp is not valid"))
            })?;
            let dlt_timestamp = DltTimestamp {
                timestamp,
                tx_idx: value.context.tx_idx,
            };
            let published_atala_object = PublishedAtalaObject {
                dlt_timestamp,
                atala_object,
            };
            Ok(published_atala_object)
        }
    }
}
