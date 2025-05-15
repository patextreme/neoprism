use identus_did_core::DidDocument;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn add(a: u32, b: u32) -> u32 {
    a + b
}

#[wasm_bindgen]
pub fn resolve_did(did: String) -> JsValue {
    let did_doc = mock_did_doc(&did);
    serde_wasm_bindgen::to_value(&did_doc).unwrap()
}

fn mock_did_doc(did: &str) -> DidDocument {
    DidDocument {
        context: Default::default(),
        id: did.to_string(),
        verification_method: Default::default(),
        authentication: Some(Default::default()),
        assertion_method: Some(Default::default()),
        key_agreement: Some(Default::default()),
        capability_invocation: Some(Default::default()),
        capability_delegation: Some(Default::default()),
        service: Some(Default::default()),
    }
}
