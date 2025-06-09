use std::str::FromStr;

use identus_did_core::{Did, DidOps, DidUrl};

#[test]
fn parse_did() {
    let did: Did = "did:example:abcdefghi".parse().unwrap();
    assert_eq!(did.to_string(), "did:example:abcdefghi");
    assert_eq!(did.method(), "example");
    assert_eq!(did.method_id(), "abcdefghi");

    let did: Did = "did:prism:9bf36a6dd4090ad66e359a0c041e25662c3f84c00467e9a61eeba68477c8a595"
        .parse()
        .unwrap();
    assert_eq!(
        did.to_string(),
        "did:prism:9bf36a6dd4090ad66e359a0c041e25662c3f84c00467e9a61eeba68477c8a595"
    );
    assert_eq!(did.method(), "prism");
    assert_eq!(
        did.method_id(),
        "9bf36a6dd4090ad66e359a0c041e25662c3f84c00467e9a61eeba68477c8a595"
    );
}

#[test]
fn parse_did_fail() {
    assert!(Did::from_str("did").is_err());
    assert!(Did::from_str("did:").is_err());
    assert!(Did::from_str("did::").is_err());
    assert!(Did::from_str("did:example").is_err());
    assert!(Did::from_str("did:example:").is_err());
    assert!(Did::from_str("did:_______:abcdefghi").is_err());
    assert!(Did::from_str("did:example:abcdefghi?service=abc").is_err());
    assert!(Did::from_str("did:example:abcdefghi#key-1").is_err());
}

#[test]
fn parse_did_url() {
    let did: DidUrl = "did:example:abcdefghi".parse().unwrap();
    assert_eq!(did.to_string(), "did:example:abcdefghi");
    // assert_eq!(did.method(), "example");
    // assert_eq!(did.method_id(), "abcdefghi");

    let did: DidUrl = "did:prism:9bf36a6dd4090ad66e359a0c041e25662c3f84c00467e9a61eeba68477c8a595"
        .parse()
        .unwrap();
    assert_eq!(
        did.to_string(),
        "did:prism:9bf36a6dd4090ad66e359a0c041e25662c3f84c00467e9a61eeba68477c8a595"
    );
    // assert_eq!(did.method(), "prism");
    // assert_eq!(did.method_id(), "9bf36a6dd4090ad66e359a0c041e25662c3f84c00467e9a61eeba68477c8a595");
}
