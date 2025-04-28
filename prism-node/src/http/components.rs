use maud::{DOCTYPE, Markup, html};
use prism_core::dlt::cardano::NetworkIdentifier;

use crate::http::urls;

pub fn page_layout(network: Option<NetworkIdentifier>, body: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html data-theme="dark" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "NeoPRISM UI" }
                link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/daisyui@latest/dist/full.css";
                script src="https://cdn.tailwindcss.com" {}
            }
            body class="bg-base-100 text-base-content flex flex-col" {
                (navbar(network))
                (body)
            }
        }
    }
}

fn navbar(network: Option<NetworkIdentifier>) -> Markup {
    html! {
        nav class="navbar bg-base-100 shadow-sm" {
            div class="flex-1" {
                a class="btn btn-ghost text-lg" href=(urls::Resolver::make_url()) { "Resolver" }
                a class="btn btn-ghost text-lg" href=(urls::Explorer::make_url()) { "Explore" }
                a class="btn btn-ghost text-lg" href=(urls::Swagger::make_url()) { "Swagger" }
            }
            div class="flex-none" {
                div class="mr-4" {
                    @match network {
                        Some(nw) => span class="text-success" { (nw) },
                        None => span class="text-error" { "offline" }
                    }
                }
            }
        }
    }
}
