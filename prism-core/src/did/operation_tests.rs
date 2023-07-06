use super::ServiceType;
use crate::{
    did::operation::{PublicKeyId, ServiceEndpoint, ServiceId},
    protocol::ProtocolParameter,
};

#[test]
fn parse_service_and_key_id() {
    let param = ProtocolParameter::default();
    let inputs = vec!["service-1", "service%E1", "123", "@", "."];

    for i in inputs.iter() {
        let result = ServiceId::parse(i, param.max_id_size);
        assert!(result.is_ok());
    }

    for i in inputs.iter() {
        let result = PublicKeyId::parse(i, param.max_id_size);
        assert!(result.is_ok());
    }
}

#[test]
fn parse_invalid_service_and_key_id() {
    let param = ProtocolParameter::default();
    let inputs = vec![
        "",
        "#",
        "%3",
        "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
        "service 1"
    ];

    for i in inputs.iter() {
        let result = ServiceId::parse(i, param.max_id_size);
        assert!(result.is_err());
    }

    for i in inputs.iter() {
        let result = PublicKeyId::parse(i, param.max_id_size);
        assert!(result.is_err());
    }
}

#[test]
fn parse_service_type() {
    let param = ProtocolParameter::default();
    let inputs = vec![
        "LinkedDomains",
        "Linked Domains",
        "L i n k e d    D o m a i n s",
        "[\"LinkedDomains\"]",
        "[\"LinkedDomains\",\"DIDCommMessaging\"]",
        "[\"Linked Domains\",\"DIDComm Messaging\"]",
        "true",
        "false",
        "null",
        "123",
        "-",
    ];

    for i in inputs {
        let result = ServiceType::parse(i, &param);
        assert!(result.is_ok());
    }
}

#[test]
fn parse_invalid_service_type() {
    let param = ProtocolParameter::default();
    let inputs = vec![
        "",
        " ",
        "  LinkedDomains",
        "LinkedDomains  ",
        "  LinkedDomains ",
        "Linked#Domains",
        "Linked@Domains",
        "[]",
        "[\"\"]",
        "[\"  \"]",
        "[123]",
        "[null]",
        "[true]",
        "[false]",
        "  [\"LinkedDomains\"]",
        "[\"LinkedDomains\"]  ",
        "[  \"LinkedDomains\"]",
        "[\"LinkedDomains\" ]  ",
        "[\"  LinkedDomains\"]",
        "[\"LinkedDomains  \"]",
        "[\"LinkedDomains\",\"\"]",
        "[\"LinkedDomains\",123]",
        "[\"LinkedDomains\",true]",
        "[\"LinkedDomains\",false]",
        "[\"LinkedDomains\",null]",
        "[\"LinkedDomains\",  \"DIDCommMessaging\"]",
        "LinkedDomainssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssss"
    ];

    for i in inputs {
        let result = ServiceType::parse(i, &param);
        assert!(result.is_err());
    }
}

#[test]
fn parse_service_endpoint() {
    let param = ProtocolParameter::default();
    let inputs = vec![
        "http://example.com",
        "https://example.com",
        "ws://example.com",
        "ftp://example.com",
        "https://example.com/path1/path2/../path3",
        "https://example.com/hello?page=1",
        "urn:resources",
        "did:prism:0000000000000000000000000000000000000000000000000000000000000000",
        "{\"uri\":\"https://example.com\"}",
        "{\"uri\": {}}",
        "{}",
        "{\"uri\": [\"https://example.com\"]}",
        "[\"https://example.com\"]",
        "[\"https://example.com\", \"https://example2.com\"]",
        "[{}]",
        "[{}, \"https://example.com\"]",
    ];

    for i in inputs {
        let result = ServiceEndpoint::parse(i, &param);
        assert!(result.is_ok());
    }
}

#[test]
fn parse_invalid_service_endopint() {
    let param = ProtocolParameter::default();
    let inputs = vec![
        "",
        "  ",
        "this is not a URI",
        "http",
        "123",
        "http://example.com/000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
        " https://example.com",
        "https://example.com ",
        "[]",
        "[\"https://example.com\", \"this is not a URI\"]",
        "[\"https://example.com\", 123]",
        "[\"https://example.com\", null]",
        "[\"https://example.com\", true]",
        "[\"https://example.com\", []]",
        "null"
    ];

    for i in inputs {
        let result = ServiceEndpoint::parse(i, &param);
        assert!(result.is_err());
    }
}
