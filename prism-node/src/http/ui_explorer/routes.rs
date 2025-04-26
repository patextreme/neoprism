use axum::Router;
use axum::routing::get;
use maud::{DOCTYPE, Markup, html};

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/explorer", get(explorer_home))
}

async fn explorer_home() -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="UTF-8"
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { "UUID Search" }
                script src="https://cdn.tailwindcss.com";
                link href="https://cdn.jsdelivr.net/npm/daisyui@latest/dist/daisyui.css" rel="stylesheet" type="text/css";
                link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;600&display=swap" rel="stylesheet";
                style {
                    "body { font-family: 'Inter', sans-serif; }"
                }
            }
            body {
                p { "hello world" }
            }
        }
    }
}
