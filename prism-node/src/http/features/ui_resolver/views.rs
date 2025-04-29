use maud::{Markup, html};
use prism_core::crypto::EncodeJwk;
use prism_core::did::operation::{self, PublicKey};
use prism_core::did::{DidState, PrismDid};
use prism_core::dlt::cardano::NetworkIdentifier;

use crate::app::service::error::ResolutionError;
use crate::http::{components, urls};

pub fn index(network: Option<NetworkIdentifier>) -> Markup {
    let body = title_and_search_box(None);
    components::page_layout(network, body)
}

pub fn resolve(
    network: Option<NetworkIdentifier>,
    did: &str,
    did_state: Result<(PrismDid, DidState), ResolutionError>,
) -> Markup {
    let did_doc_body = match did_state.as_ref() {
        Err(_) => html! {},
        Ok((_, state)) => did_document_body(&state),
    };
    let body = html! {
        (title_and_search_box(Some(did)))
        div class="h-8" {}
        (did_doc_body)
    };
    components::page_layout(network, body)
}

fn title_and_search_box(did: Option<&str>) -> Markup {
    html! {
        div class="flex flex-col items-center min-w-screen" {
            div class="my-8 text-2xl font-bold" {
                "DID Resolver"
            }
            form
                method="GET"
                action=(urls::Resolver::url())
                class="form-control w-full" {
                div class="flex flex-wrap justify-center space-x-2 space-y-2" {
                    input
                        type="text"
                        name="did"
                        placeholder="Enter PRISM DID"
                        value=[did]
                        class="input input-bordered w-10/12 max-w-xl"
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

fn did_document_body(state: &DidState) -> Markup {
    let contexts = state.context.as_slice();
    let public_keys = state.public_keys.as_slice();
    html! {
        (context_card(contexts))
        (public_key_card(public_keys))
    }
}

fn context_card(context: &[String]) -> Markup {
    html! {
        div class="m-4" {
            div class="card bg-base-200 border border-gray-700" {
                div class="card-body" {
                    h2 class="card-title text-white" { "@context" }
                    @if context.is_empty() {
                        p class="text-info" { "Empty" }
                    }
                    ul class="list-disc list-inside text-white" {
                        @for ctx in context {
                            li { (ctx) }
                        }
                    }
                }
            }
        }
    }
}

fn public_key_card(public_keys: &[PublicKey]) -> Markup {
    let pk_elems = public_keys
        .iter()
        .map(|pk| {
            let jwk = match &pk.data {
                operation::PublicKeyData::Master { data } => data.encode_jwk(),
                operation::PublicKeyData::Other { data, .. } => data.encode_jwk(),
            };
            let key_id = pk.id.to_string();
            let key_usage = format!("{:?}", pk.usage());
            let curve = jwk.crv;
            html! {
                li class="border p-2 rounded-md border-gray-700" {
                    strong { "ID: " (key_id) }
                    br;
                    strong { "Usage: " (key_usage) }
                    br;
                    strong { "Curve: " (curve) }
                }
            }
        })
        .collect::<Vec<_>>();

    html! {
        div class="card bg-base-200 border border-gray-700 m-4" {
            div class="card-body" {
                h2 class="card-title" { "Public Keys" }
                @if pk_elems.is_empty() {
                    p class="text-info" { "Empty" }
                }
                ul class="space-y-2" {
                    @for elem in pk_elems { (elem) }
                }
            }
        }
    }
}
