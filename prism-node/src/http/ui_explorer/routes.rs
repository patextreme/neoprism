use axum::Router;
use axum::routing::get;
use maud::{DOCTYPE, Markup, html};
use sqlx::types::Uuid;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/explorer", get(explorer_home))
}

async fn explorer_home() -> Markup {
    let stats_number = 100_000_000;
    let uuids = vec![
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
    ];
    html! {
        (DOCTYPE)
        html data-theme="dark" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "Dashboard" }
                link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/daisyui@latest/dist/full.css";
                script src="https://cdn.tailwindcss.com" {}
            }
            body class="min-h-screen bg-base-100 text-base-content flex flex-col" {
                // Navbar
                nav class="navbar bg-neutral" {
                    div class="flex-1" {
                        a class="btn btn-ghost normal-case text-xl" href="#" { "My App" }
                    }
                }

                // Main content
                main class="flex flex-col items-center justify-center flex-grow p-8 space-y-8" {
                    // Statistic Number
                    div class="stats shadow" {
                        div class="stat" {
                            div class="stat-title" { "Total Records" }
                            div class="stat-value" { (stats_number) }
                        }
                    }

                    // UUID List
                    div class="card w-full max-w-md bg-neutral text-neutral-content" {
                        div class="card-body" {
                            h2 class="card-title" { "Available UUIDs" }
                            ul class="list-disc pl-5" {
                                @for uuid in uuids {
                                    li { (uuid) }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
