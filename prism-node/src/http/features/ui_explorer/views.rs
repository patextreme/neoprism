use maud::{Markup, html};
use prism_core::dlt::DltCursor;
use prism_core::dlt::cardano::NetworkIdentifier;
use prism_core::utils::codec::HexStr;

use crate::http::{components, urls};

pub fn index(network: Option<NetworkIdentifier>, cursor: Option<DltCursor>) -> Markup {
    let body = html! {
        (dlt_cursor_card(cursor))
    };
    components::page_layout("Explorer", network, body)
}

pub fn dlt_cursor_card(cursor: Option<DltCursor>) -> Markup {
    let slot_number = cursor
        .as_ref()
        .map(|i| i.slot.to_string())
        .unwrap_or_else(|| "-".to_string());
    let block_hash = cursor
        .as_ref()
        .map(|i| HexStr::from(&i.block_hash).to_string())
        .unwrap_or_else(|| "Sync not enabled".to_string());
    let timestamp = cursor
        .as_ref()
        .and_then(|i| i.cbt)
        .map(|i| i.to_rfc3339())
        .unwrap_or_else(|| "".to_string());
    html! {
        div
            id="dlt-cursor-card"
            class="flex flex-row justify-center min-w-screen py-8 px-2"
            hx-get=(urls::ExplorerDltCursor::url())
            hx-swap="outerHTML"
            hx-trigger="load delay:2s"
        {
            div class="card bg-base-200 w-full max-w-md" {
                div class="card-body text-center" {
                    p class="text-sm text-gray-400 mt-2" { "Current sync slot" }
                    h2 class="text-xl font-bold" { (slot_number) }
                    p class="text-xs text-gray-400 mt-2 truncate" {
                        (block_hash)
                        br;
                        (timestamp)
                    }
                }
            }
        }
    }
}
