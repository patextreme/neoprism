use super::{DltSource, PublishedAtalaObject};
use crate::dlt::cardano::model::MetadataEvent;
use oura::{
    model::{Event, EventData},
    pipelining::SourceProvider,
    sources::{n2n::Config, AddressArg, MagicArg, PointArg},
    utils::{ChainWellKnownInfo, Utils, WithUtils},
};
use std::{
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};
use tokio::sync::mpsc::{self, Receiver, Sender};

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
    fn receiver(self) -> Result<Receiver<PublishedAtalaObject>, String> {
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
                    break;
                }
            }
        });

        Ok(rx)
    }
}

mod model {
    use crate::{
        dlt::{BlockTimestamp, PublishedAtalaObject},
        proto::AtalaObject,
    };
    use bytes::BytesMut;
    use chrono::{DateTime, TimeZone, Utc};
    use oura::model::{EventContext, MetadataRecord};
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
        #[error("metadata malformed: {0}")]
        MalformedMetadata(String),
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
            let block_timestamp = BlockTimestamp {
                cbt: timestamp,
                absn: value.context.tx_idx,
            };
            let published_atala_object = PublishedAtalaObject {
                block_timestamp,
                atala_object,
            };
            Ok(published_atala_object)
        }
    }

    pub fn parse_oura_event(
        context: EventContext,
        metadata: MetadataRecord,
    ) -> Result<PublishedAtalaObject, ConversionError> {
        let hex_group = match metadata.content {
            oura::model::MetadatumRendition::MapJson(json) => {
                let meta = serde_json::from_value::<MetadataMapJson>(json)
                    .map_err(|e| ConversionError::MalformedMetadata(e.to_string()))?;
                meta.c
            }
            _ => Err(ConversionError::MalformedMetadata(
                "Metadata is not a MapJson type".to_string(),
            ))?,
        };
        let mut buf = BytesMut::with_capacity(64 * hex_group.len());

        for hex in hex_group {
            let b = hex::decode(hex)?;
            buf.extend(b);
        }

        let bytes = buf.freeze();
        let atala_object = AtalaObject::decode(bytes)?;
        let timestamp = context.timestamp.ok_or(ConversionError::MalformedMetadata(
            "Timestamp must be present in Cardano metadata".to_string(),
        ))? as i64;
        let timestamp: DateTime<Utc> =
            Utc.timestamp_opt(timestamp, 0).single().ok_or_else(|| {
                ConversionError::ProtoDecodeError(prost::DecodeError::new("timestamp is not valid"))
            })?;
        let block_timestamp = BlockTimestamp {
            cbt: timestamp,
            absn: context.tx_idx.ok_or(ConversionError::MalformedMetadata(
                "Transaction index must be present in Cardano metadata".to_string(),
            ))? as u32,
        };
        let published_atala_object = PublishedAtalaObject {
            block_timestamp,
            atala_object,
        };
        Ok(published_atala_object)
    }
}

pub enum NetworkIdentifier {
    Mainnet,
}

impl NetworkIdentifier {
    fn magic_args(&self) -> MagicArg {
        let chain_magic = match self {
            NetworkIdentifier::Mainnet => MagicArg::from_str("mainnet"),
        };
        chain_magic.expect("The chain magic value cannot be parsed")
    }

    fn chain_wellknown_info(&self) -> ChainWellKnownInfo {
        match self {
            NetworkIdentifier::Mainnet => ChainWellKnownInfo::mainnet(),
        }
    }
}

pub struct OuraN2NSource {
    with_utils: WithUtils<Config>,
}

impl OuraN2NSource {
    pub fn new(remote_addr: &str, chain: &NetworkIdentifier) -> Self {
        #[allow(deprecated)]
        let config = Config {
            address: AddressArg(oura::sources::BearerKind::Tcp, remote_addr.to_string()),
            magic: Some(NetworkIdentifier::Mainnet.magic_args()),
            since: None,
            intersect: Some(oura::sources::IntersectArg::Point(PointArg(
                71482683,
                "f3fd56f7e390d4e45d06bb797d83b7814b1d32c2112bc997779e34de1579fa7d".to_string(),
            ))), // TODO: use actual value
            well_known: None,
            mapper: Default::default(),
            min_depth: 112,
            retry_policy: None,
            finalize: None,
        };
        let utils = Utils::new(chain.chain_wellknown_info());
        let with_utils = WithUtils::new(config, Arc::new(utils));
        Self { with_utils }
    }
}

impl DltSource for OuraN2NSource {
    fn receiver(self) -> Result<Receiver<PublishedAtalaObject>, String> {
        let (_, oura_rx) = self.with_utils.bootstrap().map_err(|e| e.to_string())?;
        let (tx, rx) = tokio::sync::mpsc::channel::<PublishedAtalaObject>(1024);

        std::thread::spawn(move || {
            let mut last_slot_log = 0u64;
            loop {
                let event = oura_rx.recv();

                if let Ok(event) = &event {
                    let slot = event.context.slot.unwrap_or_default();
                    let slot_diff = slot - last_slot_log;
                    if slot_diff > 10_000 {
                        log::info!(
                            "Current cursor at: ({}, {:?})",
                            slot,
                            event.context.block_hash
                        );
                        last_slot_log = slot;
                    }
                }

                let handle_result = match event {
                    Ok(event) => handle_oura_event(event, &tx),
                    Err(e) => Err(e.into()),
                };
                if let Err(e) = handle_result {
                    log::error!("Error handling event from oura source. {}", e);
                    break;
                }
            }

            log::info!("oura event stream terminated");
        });

        Ok(rx)
    }
}

fn handle_oura_event(
    event: Event,
    tx: &Sender<PublishedAtalaObject>,
) -> Result<(), Box<dyn std::error::Error>> {
    let context = event.context;
    if let EventData::Metadata(meta) = event.data {
        if meta.label != "21325" {
            return Ok(());
        }

        log::info!(
            "Detect a new atala_block in slot ({}, {:?})",
            context.slot.unwrap_or_default(),
            context.block_hash.as_ref().unwrap(),
        );

        let atala_object = self::model::parse_oura_event(context, meta)?;
        tx.blocking_send(atala_object).map_err(|e| e.to_string())?;
    }

    Ok(())
}
