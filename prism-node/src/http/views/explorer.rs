use dioxus::prelude::*;
use prism_core::did::CanonicalPrismDid;
use prism_core::dlt::DltCursor;
use prism_core::utils::codec::HexStr;
use rocket::uri;

use crate::http::contract::hx::HxRpc;
use crate::http::views::components::{NavBar, PageTitle};
use crate::http::views::escape_html_rpc;

pub fn ExplorerPage(cursor: Option<DltCursor>, dids: Vec<CanonicalPrismDid>) -> Element {
    rsx! {
        NavBar {}
        PageTitle { title: "Operation Explorer".to_string() }
        DltCursorStat { cursor }
        DidList { dids }
    }
}

#[component]
pub fn DltCursorStat(cursor: Option<DltCursor>) -> Element {
    let rpc_uri = uri!(crate::http::routes::hx::rpc());
    let rpc = escape_html_rpc(&HxRpc::GetExplorerDltCursor {});
    let slot = cursor
        .as_ref()
        .map(|i| format!("{}", i.slot))
        .unwrap_or_else(|| "-".to_string());
    let hash = cursor
        .map(|i| HexStr::from(i.block_hash).to_string())
        .unwrap_or_else(|| "Sync is not enabled".to_string());
    rsx! {
        div {
            class: "stats shadow",
            "hx-post": "{rpc_uri}",
            "hx-vals": "{rpc}",
            "hx-trigger": "load delay:2s",
            "hx-swap": "outerHTML",
            div { class: "stat",
                div { class: "stat-title", "Current Sync Slot" }
                div { class: "stat-value", "{slot}" }
                div { class: "stat-desc", "{hash}" }
            }
        }
    }
}

#[component]
pub fn DidList(dids: Vec<CanonicalPrismDid>) -> Element {
    let rpc_uri = uri!(crate::http::routes::hx::rpc());
    let rpc = escape_html_rpc(&HxRpc::GetExplorerDidList {});
    let did_elems = dids.iter().map(|did| {
        let uri = uri!(crate::http::routes::resolver(Some(did.to_string())));
        rsx! { a { class: "link font-mono", href: "{uri}", "{did}" } }
    });
    rsx! {
        div {
            class: "my-2",
            "hx-post": "{rpc_uri}",
            "hx-vals": "{rpc}",
            "hx-trigger": "load delay:5s",
            "hx-swap": "outerHTML",
            ul {
                for elem in did_elems {
                    li { {elem} }
                }
            }
        }
    }
}
