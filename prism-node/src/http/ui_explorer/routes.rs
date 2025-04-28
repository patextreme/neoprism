use axum::Router;
use axum::extract::{Query, State};
use axum::routing::get;
use maud::{DOCTYPE, Markup, html};

use super::models::PageQueryParams;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/explorer", get(explorer_home))
}

async fn explorer_home(Query(page): Query<PageQueryParams>, State(state): State<AppState>) -> Markup {
    let cursor = state.cursor_rx.as_ref().and_then(|rx| rx.borrow().to_owned());
    let dids = state.did_service.get_all_dids(page.page).await.unwrap();

    html! {
        (DOCTYPE)
        html data-theme="black" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "Dashboard" }
                link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/daisyui@latest/dist/full.css";
                script src="https://cdn.tailwindcss.com" {}
            }
            body class="min-h-screen bg-base-100 text-base-content flex flex-col" {
                nav class="navbar bg-neutral" {
                    div class="flex-1" {
                        a class="btn btn-ghost normal-case text-xl" href="#" { "My App" }
                    }
                }

                main class="flex flex-col items-center justify-center flex-grow p-8 space-y-8" {
                    div class="stats shadow" {
                        div class="stat" {
                            div class="stat-title" { "Sync cursor" }
                            div class="stat-value" {
                                @match cursor {
                                    Some(cursor) => (cursor.slot)
                                    None => "-"
                                }
                            }
                        }
                    }

                    div class="card w-full max-w-md bg-neutral text-neutral-content" {
                        div class="card-body" {
                            h2 class="card-title" { "Available DIDs" }
                            ul class="list-disc pl-5" {
                                @for did in dids.items {
                                    li class="truncate" { (did) }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
