use dioxus::prelude::*;
use prism_core::did::CanonicalPrismDid;
use prism_core::dlt::cardano::NetworkIdentifier;
use prism_core::dlt::DltCursor;
use prism_core::utils::codec::HexStr;
use prism_core::utils::paging::Paginated;
use rocket::uri;

use crate::http::contract::hx::HxRpc;
use crate::http::views::components::{NavBar, PageContent, PageTitle};
use crate::http::views::escape_html_rpc;

pub fn ExplorerPage(
    cursor: Option<DltCursor>,
    dids: Paginated<CanonicalPrismDid>,
    network: Option<NetworkIdentifier>,
) -> Element {
    rsx! {
        NavBar { network }
        PageTitle { title: "Operation Explorer".to_string() }
        PageContent {
            DltCursorStat { cursor }
            DidList { dids }
        }
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
        div { class: "flex flex-col items-center",
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
}

#[component]
pub fn DidList(dids: Paginated<CanonicalPrismDid>) -> Element {
    let rpc_uri = uri!(crate::http::routes::hx::rpc());
    let rpc = escape_html_rpc(&HxRpc::GetExplorerDidList {
        page: Some(dids.current_page),
    });
    let did_elems = dids.items.iter().map(|did| {
        let uri = uri!(crate::http::routes::resolver(Some(did.to_string())));
        rsx! { a { class: "link font-mono", href: "{uri}", "{did}" } }
    });
    let pagination_items = (0..dids.total_pages).map(|i: u64| {
        let is_active = i == dids.current_page;
        let goto_page_rpc = escape_html_rpc(&HxRpc::GetExplorerDidList { page: Some(i) });
        let classes = if is_active {
            format!("join-item btn btn-active")
        } else {
            format!("join-item btn")
        };
        rsx! {
            button {
                class: classes,
                "hx-post": "{rpc_uri}",
                "hx-vals": "{goto_page_rpc}",
                "hx-target": "closest #did-list",
                "hx-swap": "outerHTML",
                "{i + 1}"
            }
        }
    });
    let pagination = rsx! {
        div { class: "join my-2",
            for i in pagination_items {
                {i}
            }
        }
    };
    rsx! {
        div {
            class: "flex flex-col items-center",
            "id": "did-list",
            "hx-post": "{rpc_uri}",
            "hx-vals": "{rpc}",
            "hx-trigger": "load delay:5s",
            "hx-swap": "outerHTML",
            {pagination.clone()},
            ul {
                for elem in did_elems {
                    li { {elem} }
                }
            }
            {pagination}
        }
    }
}
