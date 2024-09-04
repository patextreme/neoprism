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
                div { class: "stat text-center",
                    div { class: "stat-title w-max-full", "Current Sync Slot" }
                    div { class: "stat-value w-max-full", "{slot}" }
                    div { class: "stat-desc w-max-full truncate", "{hash}" }
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
        rsx! {
            a {
                class: "link font-mono truncate text-center w-full",
                href: "{uri}",
                "{did}"
            }
        }
    });
    let pagination_items = (0..dids.total_pages)
        .map(|i| {
            // Only first, last and N pages before and after the current page
            let should_display = |i: u64| i == 0 || i == (dids.total_pages - 1) || i.abs_diff(dids.current_page) <= 2;
            (i, should_display(i), should_display(i + 1))
        })
        .filter_map(
            |(i, should_display, should_display_next)| match (should_display, should_display_next) {
                (true, _) => Some((i, false)),
                (false, true) => Some((i, true)),
                (false, false) => None,
            },
        )
        .map(|(i, is_3dots)| {
            let is_active = i == dids.current_page;
            let classes = if is_active {
                format!("join-item btn btn-active")
            } else {
                format!("join-item btn")
            };
            if is_3dots {
                rsx! { a { class: classes, "..." } }
            } else {
                let userfacing_page = i + 1;
                let goto_uri = uri!(crate::http::routes::explorer(Some(userfacing_page)));
                rsx! { a { href: "{goto_uri}", class: classes, "{userfacing_page}" } }
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
            class: "flex flex-col items-center w-full",
            "id": "did-list",
            "hx-post": "{rpc_uri}",
            "hx-vals": "{rpc}",
            "hx-trigger": "load delay:5s",
            "hx-swap": "outerHTML",
            {pagination.clone()},
            for elem in did_elems {
                {elem}
            }
            {pagination}
        }
    }
}
