use maud::{Markup, html};
use prism_core::dlt::cardano::NetworkIdentifier;

use crate::http::{components, urls};

pub fn index(network: Option<NetworkIdentifier>) -> Markup {
    let body = html! {
        div class="flex flex-col items-center min-w-screen" {
            div class="my-8 text-2xl font-bold" {
                "DID Resolver"
            }
            form
                method="POST"
                action=(urls::Resolver::new())
                class="form-control w-full" {
                div class="flex flex-wrap justify-center" {
                    input type="text" placeholder="Enter PRISM DID" class="input input-bordered w-10/12 max-w-xl mx-2 my-1" required;
                    button type="submit" class="btn btn-primary mx-2 my-1" { "Resolve" }
                }
            }
        }
    };
    components::page_layout(network, body)
}
