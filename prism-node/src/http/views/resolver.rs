use dioxus::prelude::*;
use prism_core::crypto::EncodeVec;
use prism_core::did::DidState;
use prism_core::protocol::resolver::{ResolutionDebug, ResolutionResult};
use prism_core::utils::codec::HexStr;
use rocket::uri;

use crate::http::views::components::{NavBar, PageTitle};

pub fn ResolverPage() -> Element {
    rsx! {
        NavBar {}
        PageTitle { title: "DID Resolver".to_string() }
        SearchBox {}
        div { id: "resolution-result" }
    }
}

#[component]
fn SearchBox() -> Element {
    let resolve_uri = uri!(crate::http::routes::hx::resolve_did);
    rsx! {
        form {
            class: "flex flex-row flex-wrap justify-center gap-2 py-2",
            "hx-post": "{resolve_uri}",
            "hx-target": "#resolution-result",
            input {
                class: "input input-bordered input-primary w-9/12 lg:w-6/12",
                r#type: "text",
                name: "did",
                placeholder: "Enter Prism DID"
            }
            button { class: "btn btn-primary", r#type: "submit", "Resolve" }
        }
    }
}

pub fn ResolutionResultDisplay(resolution_result: ResolutionResult, debug: ResolutionDebug) -> Element {
    let did_doc = match resolution_result {
        ResolutionResult::Ok(did_state) => rsx! { DidDocumentDisplay { did_state } },
        ResolutionResult::NotFound => rsx! { p { class: "text-lg", "DID not found" } },
    };
    let debug = debug.into_iter().map(|(operation, error)| {
        let operation_str = format!("{:?}", operation);
        let error_str = format!("{:?}", error);
        rsx! {
            div { class: "flex flex-col gap-2 py-3",
                p { "{operation_str}" }
                p { "{error_str}" }
            }
        }
    });
    rsx! {
        {did_doc},
        div { class: "divider divider-neutral", "Operation Debug" }
        for dbg in debug {
            {dbg}
        }
    }
}

#[component]
fn DidDocumentDisplay(did_state: DidState) -> Element {
    let contexts = did_state.context.into_iter().map(|c| {
        rsx! { li { "{c}" } }
    });
    let mut keys = did_state.public_keys;
    keys.sort_by_key(|i| i.id.to_string());
    let keys = keys.into_iter().map(|pk| {
        let usage = format!("{:?}", pk.usage());
        let curve = match &pk.data {
            prism_core::did::operation::PublicKeyData::Master { .. } => "secp256k1",
            prism_core::did::operation::PublicKeyData::Other { data, .. } => match data {
                prism_core::did::operation::SupportedPublicKey::Secp256k1(_) => "secp256k1",
                prism_core::did::operation::SupportedPublicKey::Ed25519(_) => "Ed25519",
                prism_core::did::operation::SupportedPublicKey::X25519(_) => "X25519",
            },
        };
        let public_key = match pk.data {
            prism_core::did::operation::PublicKeyData::Master { data } => data.encode_vec(),
            prism_core::did::operation::PublicKeyData::Other { data, .. } => match data {
                prism_core::did::operation::SupportedPublicKey::Secp256k1(k) => k.encode_vec(),
                prism_core::did::operation::SupportedPublicKey::Ed25519(k) => k.encode_vec(),
                prism_core::did::operation::SupportedPublicKey::X25519(k) => k.encode_vec(),
            },
        };
        let public_key = HexStr::from(public_key);
        rsx! {
            ul { class: "py-2",
                li { "ID: {pk.id}" }
                li { "Usage: {usage}" }
                li { "Curve: {curve}" }
                li { "PublicKey: 0x{public_key}" }
            }
        }
    });
    let services = did_state.services.into_iter().map(|svc| {
        let svc_str = format!("{:?}", svc);
        rsx! { p { "{svc_str}" } }
    });
    rsx! {
        div {
            div { class: "divider divider-neutral", "Contexts" }
            ul {
                for c in contexts {
                    {c}
                }
            }
            div { class: "divider divider-neutral", "Public Keys" }
            for k in keys {
                {k}
            }
            div { class: "divider divider-neutral", "Services" }
            for s in services {
                {s}
            }
        }
    }
}

pub fn ResolutionErrorDisplay(message: String) -> Element {
    rsx! { p { class: "text-lg", "{message}" } }
}
