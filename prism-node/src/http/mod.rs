use axum::Router;
use axum::response::Redirect;
use axum::routing::get;
use features::{api, ui_explorer, ui_resolver};

use crate::AppState;

mod components;
mod features;
mod urls;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            urls::Home::axum_url(),
            get(Redirect::temporary(&urls::Resolver::make_url())),
        )
        .merge(api::router())
        .merge(ui_explorer::router())
        .merge(ui_resolver::router())
}
