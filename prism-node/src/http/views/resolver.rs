use std::rc::Rc;

use dioxus::prelude::*;
use prism_core::crypto::EncodeVec;
use prism_core::did::operation::{PublicKey, Service};
use prism_core::did::DidState;
use prism_core::proto::SignedAtalaOperation;
use prism_core::protocol::resolver::ResolutionResult;
use prism_core::utils::codec::HexStr;
use rocket::uri;

use crate::http::views::components::{NavBar, PageContent, PageTitle};

pub type ResolutionDebug = Vec<(SignedAtalaOperation, Option<String>)>;

pub fn ResolverPage(
    did: Option<String>,
    resolution_result: Option<Result<(ResolutionResult, ResolutionDebug), String>>,
) -> Element {
    let content = match resolution_result {
        Some(Ok((result, debug))) => rsx! { ResolutionResultSection { result, debug: Rc::new(debug) } },
        Some(Err(e)) => rsx! { ResolutionErrorSection { message: e } },
        None => None,
    };
    rsx! {
        NavBar {}
        PageTitle { title: "DID Resolver".to_string() }
        PageContent {
            SearchBox { did }
            {content}
        }
    }
}

#[component]
fn SearchBox(did: Option<String>) -> Element {
    let resolve_uri = uri!(crate::http::routes::resolver(Option::<String>::None));
    rsx! {
        form {
            class: "flex flex-row flex-wrap justify-center gap-2 py-2",
            action: "{resolve_uri}",
            method: "get",
            input {
                class: "input input-bordered input-primary w-9/12 lg:w-6/12",
                r#type: "text",
                name: "did",
                placeholder: "Enter Prism DID",
                value: did,
                required: true
            }
            button { class: "btn btn-primary", r#type: "submit", "Resolve" }
        }
    }
}

#[component]
fn ResolutionErrorSection(message: String) -> Element {
    rsx! { p { class: "text-lg", "{message}" } }
}

#[component]
fn ResolutionResultSection(result: ResolutionResult, debug: Rc<ResolutionDebug>) -> Element {
    let did_doc = match result {
        ResolutionResult::Ok(did_state) => rsx! { DidDocumentCardContainer { did_state } },
        ResolutionResult::NotFound => rsx! { p { class: "text-lg", "DID not found" } },
    };
    let debug = debug.iter().map(|(operation, error)| {
        rsx! {
            div { class: "flex flex-col gap-2 py-3 bg-base-300",
                p { class: "font-mono", "{operation:?}" }
                p { "Error: {error:?}" }
            }
        }
    });
    rsx! {
        {did_doc},
        div { class: "divider divider-neutral", "Operation Debug" }
        for d in debug {
            {d}
        }
    }
}

#[component]
fn DidDocumentCardContainer(did_state: DidState) -> Element {
    let contexts = did_state.context.into_iter().map(|c| {
        rsx! { li { "{c}" } }
    });

    let mut keys = did_state.public_keys;
    keys.sort_by_key(|i| i.id.to_string());
    let keys = keys.into_iter().map(|pk| rsx! { DidDocumentPublicKeyCard { pk } });

    let mut services = did_state.services;
    services.sort_by_key(|i| i.id.to_string());
    let services = services.into_iter().map(|s| rsx! { DidDocumentServiceCard { svc: s } });

    rsx! {
        div {
            div { class: "divider divider-neutral", "Contexts" }
            ul {
                for c in contexts {
                    {c}
                }
            }
            div { class: "divider divider-neutral", "Public Keys" }
            div { class: "flex flex-row gap-2",
                for k in keys {
                    {k}
                }
            }
            div { class: "divider divider-neutral", "Services" }
            div { class: "flex flex-row gap-2",
                for s in services {
                    {s}
                }
            }
        }
    }
}

#[component]
fn DidDocumentPublicKeyCard(pk: PublicKey) -> Element {
    let usage = format!("{:?}", pk.usage());
    let curve = match &pk.data {
        prism_core::did::operation::PublicKeyData::Master { .. } => "secp256k1",
        prism_core::did::operation::PublicKeyData::Other { data, .. } => match data {
            prism_core::did::operation::SupportedPublicKey::Secp256k1(_) => "secp256k1",
            prism_core::did::operation::SupportedPublicKey::Ed25519(_) => "Ed25519",
            prism_core::did::operation::SupportedPublicKey::X25519(_) => "X25519",
        },
    };
    let public_key_hex: HexStr = match pk.data {
        prism_core::did::operation::PublicKeyData::Master { data } => data.encode_vec().into(),
        prism_core::did::operation::PublicKeyData::Other { data, .. } => match data {
            prism_core::did::operation::SupportedPublicKey::Secp256k1(k) => k.encode_vec().into(),
            prism_core::did::operation::SupportedPublicKey::Ed25519(k) => k.encode_vec().into(),
            prism_core::did::operation::SupportedPublicKey::X25519(k) => k.encode_vec().into(),
        },
    };
    rsx! {
        div { class: "card bg-base-200 w-96 shadow-xl",
            div { class: "card-body",
                h2 { class: "card-title", "ID: {pk.id}" }
                div { class: "badge badge-outline", "usage: {usage}" }
                div { class: "badge badge-primary badge-outline", "curve: {curve}" }
                p { class: "font-bold", "key data" }
                p { class: "bg-base-300 font-mono break-words", "0x{public_key_hex}" }
            }
        }
    }
}

#[component]
fn DidDocumentServiceCard(svc: Service) -> Element {
    rsx! {
        div { class: "card bg-base-200 w-96 shadow-xl",
            div { class: "card-body",
                h2 { class: "card-title", "ID: {svc.id}" }
                p { class: "font-bold", "service type" }
                p { class: "bg-base-300 font-mono break-words", "{svc.r#type:?}" }
                p { class: "font-bold", "service endpoint" }
                p { class: "bg-base-300 font-mono break-words", "{svc.service_endpoints:?}" }
            }
        }
    }
}

#[component]
fn DidDocumentContextCard(ctx: String) -> Element {
    rsx! {
        div { class: "card bg-base-200 w-96 shadow-xl",
            div { class: "card-body", h2 { class: "card-title font-mono", "{ctx}" } }
        }
    }
}
