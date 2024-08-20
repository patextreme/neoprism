use std::str::FromStr;
use std::sync::Arc;

use oura::model::{Event, EventData};
use oura::pipelining::{SourceProvider, StageReceiver};
use oura::sources::n2n::Config;
use oura::sources::{AddressArg, FinalizeConfig, IntersectArg, MagicArg, PointArg};
use oura::utils::{ChainWellKnownInfo, Utils, WithUtils};
use strum::VariantArray;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;

use super::error::DltError;
use super::{DltCursor, DltSource, PublishedAtalaObject};
use crate::location;
use crate::store::DltCursorStore;
use crate::utils::codec::HexStr;

mod model {
    use std::str::FromStr;

    use oura::model::{EventContext, MetadataRecord};
    use prost::Message;
    use serde::{Deserialize, Serialize};
    use time::OffsetDateTime;

    use crate::dlt::error::MetadataReadError;
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

    pub fn parse_oura_event(
        context: EventContext,
        metadata: MetadataRecord,
    ) -> Result<PublishedAtalaObject, MetadataReadError> {
        // parse metadata
        let block_hash = context.block_hash;
        let tx_idx = context.tx_idx;
        let timestamp = context.timestamp.ok_or(MetadataReadError::MissingBlockProperty {
            block_hash: block_hash.clone(),
            tx_idx,
            name: "timestamp",
        })? as i64;
        let timestamp =
            OffsetDateTime::from_unix_timestamp(timestamp).map_err(|e| MetadataReadError::InvalidBlockTimestamp {
                source: e,
                block_hash: block_hash.clone(),
                timestamp,
                tx_idx,
            })?;
        let block_metadata = BlockMetadata {
            cbt: timestamp,
            absn: context.tx_idx.ok_or(MetadataReadError::MissingBlockProperty {
                block_hash: block_hash.clone(),
                tx_idx,
                name: "tx_idx",
            })? as u32,
            block_number: context.block_number.ok_or(MetadataReadError::MissingBlockProperty {
                block_hash: block_hash.clone(),
                tx_idx,
                name: "block_number",
            })?,
            slot_number: context.slot.ok_or(MetadataReadError::MissingBlockProperty {
                block_hash: block_hash.clone(),
                tx_idx,
                name: "slot",
            })?,
        };

        // parse atala_block
        let hex_group = match metadata.content {
            oura::model::MetadatumRendition::MapJson(json) => {
                let meta = serde_json::from_value::<MetadataMapJson>(json).map_err(|e| {
                    MetadataReadError::InvalidJsonType {
                        source: e.into(),
                        block_hash: block_hash.clone(),
                        tx_idx,
                    }
                })?;
                meta.c
            }
            _ => Err(MetadataReadError::InvalidJsonType {
                source: "Metadata is not a MapJson type".to_string().into(),
                block_hash: block_hash.clone(),
                tx_idx,
            })?,
        };
        let hex = hex_group.join("");
        let bytes = HexStr::from_str(&hex)
            .map_err(|e| MetadataReadError::AtalaBlockHexDecode {
                source: e,
                block_hash: block_hash.clone(),
                tx_idx,
            })?
            .to_bytes();
        let atala_object =
            AtalaObject::decode(bytes.as_slice()).map_err(|e| MetadataReadError::AtalaBlockProtoDecode {
                source: e,
                block_hash,
                tx_idx,
            })?;

        Ok(PublishedAtalaObject {
            block_metadata,
            atala_object,
        })
    }
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

    fn magic_args(&self) -> MagicArg {
        let chain_magic = match self {
            NetworkIdentifier::Mainnet => MagicArg::from_str("mainnet"),
            NetworkIdentifier::Preprod => MagicArg::from_str("preprod"),
            NetworkIdentifier::Preview => MagicArg::from_str("preview"),
        };
        chain_magic.expect("The chain magic value cannot be parsed")
    }

    fn chain_wellknown_info(&self) -> ChainWellKnownInfo {
        match self {
            NetworkIdentifier::Mainnet => ChainWellKnownInfo::mainnet(),
            NetworkIdentifier::Preprod => ChainWellKnownInfo::preprod(),
            NetworkIdentifier::Preview => ChainWellKnownInfo::preview(),
        }
    }
}

pub struct OuraN2NSource<Store: DltCursorStore + Send + 'static> {
    with_utils: WithUtils<Config>,
    store: Store,
    cursor_tx: tokio::sync::watch::Sender<Option<DltCursor>>,
}

impl<E, Store: DltCursorStore<Error = E> + Send + 'static> OuraN2NSource<Store> {
    pub fn since_genesis(store: Store, remote_addr: &str, chain: &NetworkIdentifier, sync_block_quantity: u64) -> Self {
        let intersect = match chain {
            NetworkIdentifier::Mainnet => oura::sources::IntersectArg::Point(PointArg(
                71482683,
                "f3fd56f7e390d4e45d06bb797d83b7814b1d32c2112bc997779e34de1579fa7d".to_string(),
            )),
            NetworkIdentifier::Preprod => oura::sources::IntersectArg::Point(PointArg(
                10718532,
                "cb95a5effb12871b69c27c184ffb1355e6208c4071956df67248bad1cc329ca4".to_string(),
            )),
            _ => oura::sources::IntersectArg::Origin,
        };
        Self::new(store, remote_addr, chain, intersect, sync_block_quantity)
    }

    pub async fn since_persisted_cursor_or_genesis(
        store: Store,
        remote_addr: &str,
        chain: &NetworkIdentifier,
        sync_block_quantity: u64,
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
                Ok(Self::new(store, remote_addr, chain, intersect, sync_block_quantity))
            }
            None => {
                log::info!("Persisted cursor not found, staring syncing from PRISM genesis slot");
                Ok(Self::since_genesis(store, remote_addr, chain, sync_block_quantity))
            }
        }
    }

    pub fn new(
        store: Store,
        remote_addr: &str,
        chain: &NetworkIdentifier,
        intersect: IntersectArg,
        sync_block_quantity: u64,
    ) -> Self {
        // When oura pipeline fails, it will be restarted from original intersect config.
        // If the pipeline runs for a long time, it can replay lots of block which has been synced.
        // This workaround makes the pipeline lifetime finite so the restart doesn't replay too many blocks.
        // Once the pipeline sync up to the max_block_quantity, it will be terminated and new pipeline will be created with new intersect config.
        let finalize_config: FinalizeConfig = serde_json::from_value(serde_json::json!({
            "max_block_quantity": sync_block_quantity
        }))
        .expect("json config for FinalizeConfig is not valid");

        #[allow(deprecated)]
        let config = Config {
            address: AddressArg(oura::sources::BearerKind::Tcp, remote_addr.to_string()),
            magic: Some(chain.magic_args()),
            since: None,
            intersect: Some(intersect),
            well_known: None,
            mapper: Default::default(),
            min_depth: 112,
            retry_policy: Some(oura::sources::RetryPolicy {
                chainsync_max_retries: u32::MAX,
                chainsync_max_backoff: 60,
                connection_max_retries: u32::MAX,
                connection_max_backoff: 60,
            }),
            finalize: Some(finalize_config),
        };
        let utils = Utils::new(chain.chain_wellknown_info());
        let with_utils = WithUtils::new(config, Arc::new(utils));
        let (cursor_tx, _) = tokio::sync::watch::channel::<Option<DltCursor>>(None);
        Self {
            with_utils,
            store,
            cursor_tx,
        }
    }

    pub fn cursor_receiver(&self) -> tokio::sync::watch::Receiver<Option<DltCursor>> {
        self.cursor_tx.subscribe()
    }
}

impl<Store: DltCursorStore + Send> DltSource for OuraN2NSource<Store> {
    fn receiver(self) -> Result<Receiver<PublishedAtalaObject>, String> {
        let (event_tx, rx) = tokio::sync::mpsc::channel::<PublishedAtalaObject>(1024);

        let cursor_persist_worker = CursorPersistWorker {
            cursor_rx: self.cursor_tx.subscribe(),
            store: self.store,
        };

        let oura_stream_worker = OuraStreamWorker {
            with_utils: self.with_utils,
            cursor_tx: self.cursor_tx,
            event_tx,
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
    fn spawn(self) -> std::thread::JoinHandle<Result<(), DltError>> {
        std::thread::spawn(move || loop {
            let with_utils = self.build_with_util();
            log::info!("Bootstraping oura pipeline thread");
            let (_, oura_rx) = with_utils.bootstrap().map_err(|e| DltError::Bootstrap {
                source: e.to_string().into(),
            })?;
            let _exit_err = self.stream_loop(oura_rx);
            log::error!("Oura pipeline terminated. Retry in 10 seconds");
            std::thread::sleep(std::time::Duration::from_secs(10));
        })
    }

    /// Construct WithUtils instance from the last event sent to persist worker.
    fn build_with_util(&self) -> WithUtils<Config> {
        let mut owned_with_utils = self.with_utils.clone();
        let rx = self.cursor_tx.subscribe();
        let prev_cursor = rx.borrow();
        let prev_intersect = prev_cursor
            .as_ref()
            .map(|c| oura::sources::IntersectArg::Point(PointArg(c.slot, HexStr::from(&c.block_hash).to_string())));
        let intersect = prev_intersect
            .map(Some)
            .unwrap_or_else(|| owned_with_utils.inner.intersect.clone());
        owned_with_utils.inner.intersect = intersect;
        owned_with_utils
    }

    fn stream_loop(&self, receiver: StageReceiver) -> DltError {
        let timeout = std::time::Duration::from_secs(300);
        loop {
            let received_event = receiver.recv_timeout(timeout);
            let handle_result = match received_event {
                Ok(event) => {
                    let handle_result = self.handle_atala_event(event.clone());
                    self.persist_cursor(&event);
                    handle_result
                }
                Err(e) => Err(DltError::DisconnectedOrTimeout {
                    source: e.to_string().into(),
                    location: location!(),
                }),
            };
            if let Err(e) = handle_result {
                match &e {
                    DltError::DisconnectedOrTimeout { .. } => {
                        log::error!("Oura pipeline has disconnected or timeout");
                    }
                    e => {
                        log::error!("Error handling event from oura source");
                        let report = std::error::Report::new(&e).pretty(true);
                        log::error!("{}", report);
                    }
                };
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

    fn handle_atala_event(&self, event: Event) -> Result<(), DltError> {
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
            Ok(atala_object) => self
                .event_tx
                .blocking_send(atala_object)
                .map_err(|e| DltError::EventHandling {
                    source: e.to_string().into(),
                    location: location!(),
                })?,
            Err(e) => {
                // TODO: add debug level error report
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
    fn spawn(mut self) -> JoinHandle<Result<(), DltError>> {
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
                log::info!(
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
