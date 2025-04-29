use maud::{Markup, html};
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
    _did_state: Result<(PrismDid, DidState), ResolutionError>,
) -> Markup {
    let body = html! {
        (title_and_search_box(Some(did)))
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
                div class="flex flex-wrap justify-center" {
                    input
                        type="text"
                        name="did"
                        placeholder="Enter PRISM DID"
                        value=[did]
                        class="input input-bordered w-10/12 max-w-xl mx-2 my-1"
                        required;
                    button
                        type="submit"
                        class="btn btn-primary mx-2 my-1"
                        { "Resolve" }
                }
            }
        }
    }
}
