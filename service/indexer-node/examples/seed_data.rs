use identus_apollo::crypto::secp256k1::Secp256k1PrivateKey;
use identus_apollo::hash::Sha256Digest;
use identus_did_prism::did::{CanonicalPrismDid, PrismDidOps};
use identus_did_prism::{proto, test_utils};
use identus_did_prism_indexer::repo::OperationRepo;
use identus_did_prism_indexer::run_indexer_loop;
use indexer_storage::PostgresDb;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let db = PostgresDb::connect("postgres://postgres:postgres@localhost:5432/postgres")
        .await
        .expect("Unable to connect to database");

    db.migrate().await.expect("Failed to apply migrations");

    let master_sk = Secp256k1PrivateKey::from_slice(&[1; 32]).unwrap();
    let vdr_sk = Secp256k1PrivateKey::from_slice(&[2; 32]).unwrap();

    let (op_1, _, did_1) = create_did_with_vdr_key(&vdr_sk, "vdr-1");
    let (op_2, _, did_2) = create_did_with_vdr_key(&vdr_sk, "vdr-2");

    let (op_1_2, op_hash_1_2) = test_utils::new_signed_operation(
        "vdr-1",
        &vdr_sk,
        proto::prism_operation::Operation::CreateStorageEntry(proto::ProtoCreateStorageEntry {
            did_prism_hash: did_1.suffix.to_vec(),
            nonce: vec![1],
            data: Some(proto::proto_create_storage_entry::Data::Bytes(vec![1, 2, 3])),
        }),
    );

    let (op_1_3, op_hash_1_3) = test_utils::new_signed_operation(
        "vdr-1",
        &vdr_sk,
        proto::prism_operation::Operation::UpdateStorageEntry(proto::ProtoUpdateStorageEntry {
            previous_operation_hash: op_hash_1_2.to_vec(),
            data: Some(proto::proto_update_storage_entry::Data::Bytes(vec![4, 5, 6])),
        }),
    );

    let (op_1_4, _) = test_utils::new_signed_operation(
        "vdr-1",
        &vdr_sk,
        proto::prism_operation::Operation::UpdateStorageEntry(proto::ProtoUpdateStorageEntry {
            previous_operation_hash: op_hash_1_3.to_vec(),
            data: Some(proto::proto_update_storage_entry::Data::Bytes(vec![7, 8, 9])),
        }),
    );

    let (op_1_5, _) = test_utils::new_signed_operation(
        "vdr-1",
        &vdr_sk,
        proto::prism_operation::Operation::CreateStorageEntry(proto::ProtoCreateStorageEntry {
            did_prism_hash: did_1.suffix.to_vec(),
            nonce: vec![1, 2],
            data: Some(proto::proto_create_storage_entry::Data::Bytes(vec![42])),
        }),
    );

    let (op_2_2, op_hash_2_2) = test_utils::new_signed_operation(
        "vdr-2",
        &vdr_sk,
        proto::prism_operation::Operation::CreateStorageEntry(proto::ProtoCreateStorageEntry {
            did_prism_hash: did_2.suffix.to_vec(),
            nonce: vec![1],
            data: Some(proto::proto_create_storage_entry::Data::Bytes(vec![1, 2, 3])),
        }),
    );

    let (op_2_3, _) = test_utils::new_signed_operation(
        "master-0",
        &master_sk,
        proto::prism_operation::Operation::DeactivateDid(proto::ProtoDeactivateDid {
            previous_operation_hash: op_hash_2_2.to_vec(),
            id: did_2.suffix_hex().to_string(),
        }),
    );

    let operations = test_utils::populate_metadata(vec![op_1, op_2, op_1_2, op_1_3, op_1_4, op_1_5, op_2_2, op_2_3]);

    db.insert_raw_operations(operations).await.unwrap();
    run_indexer_loop(&db).await?;
    Ok(())
}

fn create_did_with_vdr_key(
    vdr_sk: &Secp256k1PrivateKey,
    vdr_key_name: &str,
) -> (proto::SignedPrismOperation, Sha256Digest, CanonicalPrismDid) {
    let options = test_utils::CreateDidOptions {
        public_keys: Some(vec![test_utils::new_public_key(
            vdr_key_name,
            proto::KeyUsage::VdrKey,
            vdr_sk,
        )]),
        ..Default::default()
    };
    let (signed_operation, operation_hash, _) = test_utils::new_create_did_operation(Some(options));
    let did = CanonicalPrismDid::from_operation(signed_operation.operation.as_ref().unwrap()).unwrap();
    (signed_operation, operation_hash, did)
}
