use std::error::Report;

use identus_apollo::jwk::EncodeJwk;
use identus_did_core::DidDocument;
use maud::{Markup, html};
use prism_core::did::operation::{self, PublicKey};
use prism_core::did::{DidState, PrismDid};
use prism_core::dlt::OperationMetadata;
use prism_core::dlt::cardano::NetworkIdentifier;
use prism_core::proto::SignedAtalaOperation;
use prism_core::protocol::error::ProcessError;

use crate::app::service::error::ResolutionError;
use crate::http::models::new_did_document;
use crate::http::{components, urls};

pub fn index(network: Option<NetworkIdentifier>) -> Markup {
    let body = search_box(None);
    components::page_layout("Resolver", network, body)
}

pub fn resolve(
    network: Option<NetworkIdentifier>,
    did: &str,
    did_state: Result<(PrismDid, DidState), ResolutionError>,
    did_debug: Vec<(OperationMetadata, SignedAtalaOperation, Option<ProcessError>)>,
) -> Markup {
    let resolution_body = match did_state.as_ref() {
        Err(e) => resolution_error_body(e),
        Ok((_, state)) => did_document_body(did, state),
    };
    let body = html! {
        (search_box(Some(did)))
        div class="flex flex-row w-screen justify-center" {
            div class="flex flex-col w-full max-w-4xl items-center" {
                (resolution_body)
                (did_debug_body(did_debug))
            }
        }
    };
    components::page_layout("Resolver", network, body)
}

fn search_box(did: Option<&str>) -> Markup {
    html! {
        div class="flex flex-col items-center min-w-screen py-8" {
            form
                method="GET"
                action=(urls::Resolver::new_uri(None))
                class="form-control w-full" {
                div class="flex flex-col flex-wrap items-center space-x-2 space-y-2" {
                    input
                        type="text"
                        name="did"
                        placeholder="Enter PRISM DID"
                        value=[did]
                        class="input input-bordered w-9/12 max-w-xl"
                        required;
                    button
                        type="submit"
                        class="btn btn-primary"
                        { "Resolve" }
                }
            }
        }
    }
}

fn resolution_error_body(error: &ResolutionError) -> Markup {
    let error_lines = Report::new(error)
        .pretty(true)
        .to_string()
        .split("\n")
        .map(|s| html! { (s) br; })
        .collect::<Vec<_>>();
    html! {
        div class="flex justify-center w-full" {
            div class="w-full m-4 space-y-4" {
                p class="text-2xl font-bold" { "Resolution error" }
                div class="card bg-base-200 border border-gray-700 font-mono text-sm p-3" {
                    @for line in error_lines { (line) }
                }
            }
        }
    }
}

fn did_document_body(did: &str, state: &DidState) -> Markup {
    let did_doc = new_did_document(did, state);
    let contexts = state.context.as_slice();
    let public_keys = state.public_keys.as_slice();
    let did_doc_url = urls::ApiDid::new_uri(did.to_string());
    html! {
        div class="flex justify-center w-full" {
            div class="w-full m-4 space-y-4" {
                p class="text-2xl font-bold" { "DID document" }
                a class="btn btn-xs btn-outline" href=(did_doc_url) target="_blank" { "Resolver API" }
                (context_card(contexts))
                (public_key_card(public_keys))
                (service_card(&did_doc))
            }
        }
    }
}

fn context_card(context: &[String]) -> Markup {
    html! {
        div class="card bg-base-200 border border-gray-700" {
            div class="card-body" {
                h2 class="card-title" { "@context" }
                @if context.is_empty() {
                    p class="text-neutral-content" { "Empty" }
                }
                ul class="list-disc list-inside" {
                    @for ctx in context {
                        li { (ctx) }
                    }
                }
            }
        }
    }
}

fn public_key_card(public_keys: &[PublicKey]) -> Markup {
    let mut sorted_pks = public_keys.to_vec();
    sorted_pks.sort_by_key(|i| i.id.to_string());

    let pk_elems = sorted_pks
        .iter()
        .map(|pk| {
            let jwk = match &pk.data {
                operation::PublicKeyData::Master { data } => data.encode_jwk(),
                operation::PublicKeyData::Other { data, .. } => data.encode_jwk(),
            };
            let key_id = pk.id.to_string();
            let key_usage = format!("{:?}", pk.usage());
            let curve = jwk.crv;
            let encoded_x = jwk.x.map(|i| i.to_string()).unwrap_or_default();
            let encoded_y = jwk.y.map(|i| i.to_string()).unwrap_or_default();
            html! {
                li class="border p-2 rounded-md border-gray-700 wrap-anywhere" {
                    strong { "ID: " } (key_id)
                    br;
                    strong { "Usage: " } (key_usage)
                    br;
                    strong { "Curve: " } (curve)
                    br;
                    strong { "X: " } (encoded_x)
                    br;
                    strong { "Y: " } (encoded_y)
                }
            }
        })
        .collect::<Vec<_>>();

    html! {
        div class="card bg-base-200 border border-gray-700" {
            div class="card-body" {
                h2 class="card-title" { "Public Keys" }
                @if pk_elems.is_empty() {
                    p class="text-neutral-content" { "Empty" }
                }
                ul class="space-y-2" {
                    @for elem in pk_elems { (elem) }
                }
            }
        }
    }
}

fn service_card(did_doc: &DidDocument) -> Markup {
    let mut services = did_doc.service.clone().unwrap_or_default();
    services.sort_by_key(|i| i.id.to_string());

    let svc_elems = services
        .iter()
        .map(|svc| {
            let svc_id = &svc.id;
            let svc_ty = serde_json::to_string_pretty(&svc.r#type).unwrap_or_default();
            let svc_ep = serde_json::to_string_pretty(&svc.service_endpoint).unwrap_or_default();
            html! {
                li class="border p-2 rounded-md border-gray-700 wrap-anywhere" {
                    strong { "ID: " } (svc_id)
                    br;
                    strong { "Type: " } span class="font-mono" { (svc_ty) }
                    br;
                    strong { "Endpoint: " } span class="font-mono" { (svc_ep) }
                }
            }
        })
        .collect::<Vec<_>>();

    html! {
        div class="card bg-base-200 border border-gray-700" {
            div class="card-body" {
                h2 class="card-title" { "Services" }
                @if svc_elems.is_empty() {
                    p class="text-neutral-content" { "Empty" }
                }
                ul class="space-y-2" {
                    @for elem in svc_elems { (elem) }
                }
            }
        }
    }
}

fn did_debug_body(did_debug: Vec<(OperationMetadata, SignedAtalaOperation, Option<ProcessError>)>) -> Markup {
    let op_elems = did_debug
        .iter()
        .map(|(metadata, signed_op, error)| {
            let block_time = metadata.block_metadata.cbt.to_rfc3339();
            let operation_payload = format!("{:?}", signed_op);
            let error_lines = error
                .as_ref()
                .map(|e| Report::new(e).pretty(true).to_string())
                .unwrap_or_else(|| "-".to_string())
                .split("\n")
                .map(|s| html! { (s) br; })
                .collect::<Vec<_>>();
            html! {
                li class="border p-2 rounded-md bg-base-200 border-gray-700 wrap-anywhere" {
                    strong { "Block time: " } (block_time)
                    br;
                    strong { "Slot no: " } (metadata.block_metadata.slot_number)
                    br;
                    strong { "Block no: " } (metadata.block_metadata.block_number)
                    br;
                    strong { "Block seq no: " } (metadata.block_metadata.absn)
                    br;
                    strong { "Operation seq no: " } (metadata.osn)
                    br;
                    strong { "Operation payload: " }
                    br;
                    div class="bg-base-300 font-mono text-sm text-neutral-content p-3" {
                        (operation_payload)
                    }
                    strong { "Error: " }
                    br;
                    div class="bg-base-300 font-mono text-sm text-neutral-content p-3" {
                        @for line in error_lines { (line) }
                    }
                }
            }
        })
        .collect::<Vec<_>>();

    html! {
        div class="flex justify-center w-full" {
            div class="w-full m-4 space-y-4" {
                p class="text-2xl font-bold" { "Operation debug" }
                ul class="space-y-2" {
                    @for elem in op_elems { (elem) }
                }
            }
        }
    }
}
