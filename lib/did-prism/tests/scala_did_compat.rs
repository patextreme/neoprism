use std::ops::Deref;
use std::str::FromStr;

use identus_apollo::hex::HexStr;
use identus_did_prism::did::CanonicalPrismDid;
use identus_did_prism::did::operation::StorageData;
use identus_did_prism::proto;
use identus_did_prism::protocol::resolver;
use prost::Message;

mod test_utils;

#[test]
fn input_from_scala_did() {
    let parse_signed_operation = |hex_str: &str| {
        let bytes = HexStr::from_str(hex_str).unwrap();
        proto::SignedPrismOperation::decode(bytes.to_bytes().as_slice()).unwrap()
    };

    let create_did_op = parse_signed_operation(
        "0a076d61737465723112473045022100fd1f2ea66ea9e7f37861dbe1599fb12b7ca3297e9efa872504bfc54f1daebec502205f0152d45b266b5524d2fe8eb38aaa1d3e78dc053b4f50d98fe4564f50c4c0da1a7b0a790a77123b0a076d61737465723110014a2e0a09736563703235366b311221028210fd4c42b148df2b908eb6a5c507822f63c440facc283b30c84859fde2e30f12380a047664723110084a2e0a09736563703235366b311221028210fd4c42b148df2b908eb6a5c507822f63c440facc283b30c84859fde2e30f",
    );
    let create_storage_op = parse_signed_operation(
        "0a0476647231124730450221008b7d8eab69f8fe25c496d04545a0c87c1869de12fcd77e2be6746286c499858902200f5351773a4720f5ece5ff60f7912f67ac82d3f999a0772ff8477ec1fce1d4621a293a270a2051d47b13393a7cc5c1afc47099dcbecccf0c8a70828c072ac82f55225b42d4f4520300ff11",
    );
    let update_storage_op = parse_signed_operation(
        "0a04766472311246304402206973afd6b82a1f94a952d279310c5ba3e1afc8462104506c0e5299df49268b9d02202c5c250a82288e5f392261014167bac8b61ca9d4173b0f7953386e8cb389ca041a2a42280a203ade633ab371f00687b9e23431d10b9dc1943484d364c48608d5c1a985357a3b52043300ffcc",
    );

    let did = CanonicalPrismDid::from_operation(create_did_op.operation.as_ref().unwrap()).unwrap();
    assert_eq!(
        did.to_string(),
        "did:prism:51d47b13393a7cc5c1afc47099dcbecccf0c8a70828c072ac82f55225b42d4f4"
    );

    let operations = test_utils::populate_metadata(vec![
        create_did_op.clone(),
        create_storage_op.clone(),
        update_storage_op.clone(),
    ]);

    let state_0 = resolver::resolve_published(operations[..1].to_vec()).0.unwrap();
    let state_1 = resolver::resolve_published(operations[..2].to_vec()).0.unwrap();
    let state_2 = resolver::resolve_published(operations).0.unwrap();

    assert!(state_0.storage.is_empty());
    assert_eq!(state_1.storage[0].data.deref(), &StorageData::Bytes(vec![0, 255, 17]));
    assert_eq!(
        state_2.storage[0].data.deref(),
        &StorageData::Bytes(vec![51, 0, 255, 204])
    );
}
