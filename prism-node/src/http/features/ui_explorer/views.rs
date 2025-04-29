use maud::{Markup, html};
use prism_core::dlt::cardano::NetworkIdentifier;

use crate::http::components;

pub fn index(network: Option<NetworkIdentifier>) -> Markup {
    let body = html! {
    };
    components::page_layout("Explorer", network, body)
}
