use maud::{Markup, html};
use prism_core::dlt::cardano::NetworkIdentifier;

use crate::http::components;

pub fn page(network: Option<NetworkIdentifier>) -> Markup {
    let body = html! {
        h1 class="text-lg" { "hello resolver" }
    };
    components::page_layout(network, body)
}
