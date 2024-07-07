use axum::{routing::get, Router};
use prism_core::{
    dlt::{DltSource, OperationMetadata},
    store::OperationStore,
};
use prism_storage::PostgresDb;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

pub struct PrismNodeApp<Src> {
    store: PostgresDb,
    source: Src,
}

impl<Src> PrismNodeApp<Src>
where
    Src: DltSource,
{
    pub fn new(store: PostgresDb, source: Src) -> Self {
        Self { store, source }
    }

    pub async fn run(self, bind: Ipv4Addr, port: u16) {
        let sync_app = CardanoSyncApp {
            store: self.store.clone(),
            source: self.source,
        };
        let server_app = ServerApp { store: self.store };
        let exit = tokio::try_join!(sync_app.run(), server_app.run(bind, port));
        log::info!("PrismNodeApp terminated. Exit reason {:?}", exit);
    }
}

struct ServerApp {
    store: PostgresDb,
}

impl ServerApp {
    pub async fn run(self, bind: Ipv4Addr, port: u16) -> anyhow::Result<()> {
        let app = Router::new()
            .route("/", get(|| async { "hello" }))
            .with_state(self.store);

        let server = axum::Server::bind(&SocketAddr::new(IpAddr::V4(bind), port));

        log::info!("Starting http server on {}:{}", bind, port);
        server.serve(app.into_make_service()).await?;
        Ok(())
    }
}

struct CardanoSyncApp<Src> {
    store: PostgresDb,
    source: Src,
}

impl<Src> CardanoSyncApp<Src>
where
    Src: DltSource,
{
    pub async fn run(self) -> anyhow::Result<()> {
        let mut rx = self
            .source
            .receiver()
            .expect("Unable to create a DLT source");

        while let Some(published_atala_object) = rx.recv().await {
            let block = published_atala_object.atala_object.block_content;
            let block_metadata = published_atala_object.block_metadata;
            let signed_operations = block.map(|i| i.operations).unwrap_or_default();
            let tx = self.store.begin().await?;
            for (idx, signed_operation) in signed_operations.into_iter().enumerate() {
                if signed_operation
                    .operation
                    .as_ref()
                    .and_then(|i| i.operation.as_ref())
                    .is_none()
                {
                    continue;
                }

                let insert_result = tx
                    .insert(
                        signed_operation,
                        OperationMetadata {
                            block_metadata: block_metadata.clone(),
                            osn: idx as u32,
                        },
                    )
                    .await;

                match insert_result {
                    Err(e) => log::error!("{:?}", e),
                    _ => {}
                };
            }
            tx.commit().await?;
        }
        Ok(())
    }
}
