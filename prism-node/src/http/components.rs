use maud::{DOCTYPE, Markup, html};
use prism_core::dlt::cardano::NetworkIdentifier;

use crate::http::urls;

pub fn page_layout(title: &str, network: Option<NetworkIdentifier>, body: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "NeoPRISM UI" }
                link rel="stylesheet" href=(urls::AssetStyleSheet::url());
                script
                    src="https://unpkg.com/htmx.org@2.0.4"
                    integrity="sha384-HGfztofotfshcF7+8n44JQL2oJmowVChPTg48S+jvZoztPfvwD79OC/LTtG6dMp+"
                    crossorigin="anonymous"
                    {}
            }
            body class="bg-base-100 flex flex-col" {
                (navbar(title, network))
                (body)
            }
        }
    }
}

fn navbar(title: &str, network: Option<NetworkIdentifier>) -> Markup {
    html! {
        nav class="navbar bg-base-200" {
            div class="navbar-start" {
                div class="dropdown" {
                    label class="btn btn-ghost" tabindex="0" {
                        svg class="h-4 w-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor" {
                            path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" {
                            }
                        }
                        span { "Menu" }
                    }
                    ul class="menu menu-sm dropdown-content mt-3 z-[1] p-2 shadow bg-base-200 rounded-box w-36 border" tabindex="0" {
                        li { a class="btn btn-ghost" href=(urls::Resolver::url(None)) { "Resolver" } }
                        li { a class="btn btn-ghost" href=(urls::Explorer::url(None)) { "Explorer" } }
                        li { a class="btn btn-ghost" href=(urls::Swagger::url()) { "API Docs" } }
                    }
                }
            }
            div class="navbar-center" {
                p class="text-xl font-bold" { (title) }
            }
            div class="navbar-end" {
                div class="mr-4" {
                    @match network {
                        Some(nw) => span class="text-sm text-success" { (nw) },
                        None => span class="text-sm text-error" { "disconnected" }
                    }
                }
            }
        }
    }
}
