use std::str::FromStr;
use std::sync::Arc;

use oura::model::{Event, EventData};
use oura::pipelining::{SourceProvider, StageReceiver};
use oura::sources::n2n::Config;
use oura::sources::{AddressArg, IntersectArg, MagicArg, PointArg};
use oura::utils::{ChainWellKnownInfo, Utils, WithUtils};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;

use super::{DltSource, PublishedAtalaObject};
use crate::store::{DltCursor, DltCursorStore};
use crate::utils::codec::HexStr;
use crate::utils::StdError;

mod model {
    use std::backtrace::Backtrace;
    use std::str::FromStr;

    use oura::model::{EventContext, MetadataRecord};
    use prost::Message;
    use serde::{Deserialize, Serialize};
    use time::OffsetDateTime;

    use crate::dlt::{BlockMetadata, PublishedAtalaObject};
    use crate::proto::AtalaObject;
    use crate::utils::codec::HexStr;

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

    #[derive(Debug, thiserror::Error)]
    pub enum ConversionError {
        #[error("hex conversion error: {source}")]
        HexDecodeError {
            #[from]
            source: crate::utils::codec::DecodeError,
            backtrace: Backtrace,
        },
        #[error("protobuf decode error: {source}")]
        ProtoDecodeError {
            #[from]
            source: prost::DecodeError,
            backtrace: Backtrace,
        },
        #[error("time component range error: {source}")]
        TimeRange {
            #[from]
            source: time::error::ComponentRange,
            backtrace: Backtrace,
        },
        #[error("metadata malformed: {0}")]
        MalformedMetadata(String),
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
        let hex = hex_group.join("");
        let bytes = HexStr::from_str(&hex)?.to_bytes();
        let atala_object = AtalaObject::decode(bytes.as_slice())?;
        let timestamp = context.timestamp.ok_or(ConversionError::MalformedMetadata(
            "Timestamp must be present in Cardano metadata".to_string(),
        ))? as i64;
        let timestamp = OffsetDateTime::from_unix_timestamp(timestamp)?;
        let block_metadata = BlockMetadata {
            cbt: timestamp,
            absn: context.tx_idx.ok_or(ConversionError::MalformedMetadata(
                "Transaction index must be present in Cardano metadata".to_string(),
            ))? as u32,
            block_number: context.block_number.ok_or(ConversionError::MalformedMetadata(
                "Block number must be present in Cardano metadata".to_string(),
            ))?,
            slot_number: context.slot.ok_or(ConversionError::MalformedMetadata(
                "Slot number must be present in Cardano metadata".to_string(),
            ))?,
        };
        let published_atala_object = PublishedAtalaObject {
            block_metadata,
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

pub struct OuraN2NSource<Store: DltCursorStore + Send + 'static> {
    with_utils: WithUtils<Config>,
    store: Store,
}

impl<E, Store: DltCursorStore<Error = E> + Send + 'static> OuraN2NSource<Store> {
    // 71482683 was about the slot that first AtalaBlock was observed on mainnet.
    // How can we support multiple network and define genesis slot / block?
    pub fn since_genesis(store: Store, remote_addr: &str, chain: &NetworkIdentifier) -> Self {
        let intersect = oura::sources::IntersectArg::Point(PointArg(
            71482683,
            "f3fd56f7e390d4e45d06bb797d83b7814b1d32c2112bc997779e34de1579fa7d".to_string(),
        ));
        Self::new(store, remote_addr, chain, intersect)
    }

    pub async fn since_persisted_cursor_or_genesis(
        store: Store,
        remote_addr: &str,
        chain: &NetworkIdentifier,
    ) -> Result<Self, E> {
        let cursor = store.get_cursor().await?;
        match cursor {
            Some(cursor) => {
                let blockhash_hex = HexStr::from(cursor.block_hash).to_string();
                log::info!(
                    "Persisted cursor found, starting syncing from ({}, {})",
                    cursor.slot,
                    blockhash_hex
                );
                let intersect = oura::sources::IntersectArg::Point(PointArg(cursor.slot, blockhash_hex));
                Ok(Self::new(store, remote_addr, chain, intersect))
            }
            None => {
                log::info!("Persisted cursor not found, staring syncing from PRISM genesis slot");
                Ok(Self::since_genesis(store, remote_addr, chain))
            }
        }
    }

    pub fn new(store: Store, remote_addr: &str, chain: &NetworkIdentifier, intersect: IntersectArg) -> Self {
        #[allow(deprecated)]
        let config = Config {
            address: AddressArg(oura::sources::BearerKind::Tcp, remote_addr.to_string()),
            magic: Some(NetworkIdentifier::Mainnet.magic_args()),
            since: None,
            intersect: Some(intersect),
            well_known: None,
            mapper: Default::default(),
            min_depth: 112,
            retry_policy: None,
            finalize: None,
        };
        let utils = Utils::new(chain.chain_wellknown_info());
        let with_utils = WithUtils::new(config, Arc::new(utils));
        Self { with_utils, store }
    }
}

impl<Store: DltCursorStore + Send> DltSource for OuraN2NSource<Store> {
    fn receiver(self) -> Result<Receiver<PublishedAtalaObject>, String> {
        let (event_tx, rx) = tokio::sync::mpsc::channel::<PublishedAtalaObject>(1024);
        let (cursor_tx, cursor_rx) = tokio::sync::watch::channel::<Option<DltCursor>>(None);

        let oura_stream_worker = OuraStreamWorker {
            with_utils: self.with_utils,
            cursor_tx,
            event_tx,
        };

        let cursor_persist_worker = CursorPersistWorker {
            cursor_rx,
            store: self.store,
        };

        oura_stream_worker.spawn();
        cursor_persist_worker.spawn();

        Ok(rx)
    }
}

struct OuraStreamWorker {
    with_utils: WithUtils<Config>,
    cursor_tx: tokio::sync::watch::Sender<Option<DltCursor>>,
    event_tx: Sender<PublishedAtalaObject>,
}

impl OuraStreamWorker {
    fn spawn(self) -> std::thread::JoinHandle<Result<(), StdError>> {
        std::thread::spawn(move || loop {
            log::info!("Bootstraping oura pipeline thread");
            let (_, oura_rx) = self.with_utils.bootstrap().map_err(|e| e.to_string())?;
            let exit_err = self.stream_loop(oura_rx);
            log::error!("Oura pipeline terminated. Retry in 10 seconds. ({})", exit_err);
            std::thread::sleep(std::time::Duration::from_secs(10));
        })
    }

    fn stream_loop(&self, receiver: StageReceiver) -> StdError {
        let timeout = std::time::Duration::from_secs(300);
        loop {
            let received_event = receiver.recv_timeout(timeout);
            let handle_result = match received_event {
                Ok(event) => {
                    self.persist_cursor(&event);
                    self.handle_atala_event(event.clone())
                }
                Err(e) => Err(e.into()),
            };
            if let Err(e) = handle_result {
                log::error!("Error handling event from oura source. {}", e);
                return e;
            }
        }
    }

    fn persist_cursor(&self, event: &Event) {
        let Some(slot) = event.context.slot else {
            return;
        };
        let Some(block_hash_hex) = &event.context.block_hash else {
            return;
        };
        let Ok(block_hash) = HexStr::from_str(block_hash_hex) else {
            return;
        };
        let cursor = DltCursor {
            slot,
            block_hash: block_hash.to_bytes(),
        };
        let _ = self.cursor_tx.send(Some(cursor));
    }

    fn handle_atala_event(&self, event: Event) -> Result<(), StdError> {
        let EventData::Metadata(meta) = event.data else {
            return Ok(());
        };
        if meta.label != "21325" {
            return Ok(());
        }

        let context = event.context;
        log::info!(
            "Detect a new atala_block on slot ({}, {})",
            context.slot.unwrap_or_default(),
            context.block_hash.as_deref().unwrap_or_default(),
        );

        let parsed_atala_object = self::model::parse_oura_event(context, meta);
        match parsed_atala_object {
            Ok(atala_object) => self.event_tx.blocking_send(atala_object).map_err(|e| e.to_string())?,
            Err(e) => {
                log::warn!("Unable to parse oura event into AtalaObject. ({})", e);
            }
        }

        Ok(())
    }
}

struct CursorPersistWorker<Store: DltCursorStore> {
    cursor_rx: tokio::sync::watch::Receiver<Option<DltCursor>>,
    store: Store,
}

impl<Store: DltCursorStore + Send + 'static> CursorPersistWorker<Store> {
    fn spawn(mut self) -> JoinHandle<Result<(), StdError>> {
        let delay_sec = 30;
        log::info!("Spawn cursor persist worker with {} seconds interval", delay_sec);
        tokio::spawn(async move {
            loop {
                let recv_result = self.cursor_rx.changed().await;
                tokio::time::sleep(tokio::time::Duration::from_secs(delay_sec)).await;

                if let Err(e) = recv_result {
                    log::error!("Error getting cursor to persist: {}", e);
                }

                let cursor = self.cursor_rx.borrow_and_update().clone();
                let Some(cursor) = cursor else { continue };
                log::debug!(
                    "Persisting cursor on slot ({}, {})",
                    cursor.slot,
                    HexStr::from(cursor.block_hash.as_slice()).to_string(),
                );

                let persist_result = self.store.set_cursor(cursor).await;
                if let Err(e) = persist_result {
                    log::error!("Error persisting cursor: {}", e);
                }
            }
        })
    }
}
