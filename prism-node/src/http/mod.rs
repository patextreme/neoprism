use std::path::Path;

use axum::Router;
use axum::response::Redirect;
use axum::routing::get;
use features::{api, ui_explorer, ui_resolver};
use tower_http::services::ServeDir;

use crate::AppState;

mod components;
mod features;
mod urls;

pub fn router(asset_dir: &Path) -> Router<AppState> {
    let serve_dir = ServeDir::new(asset_dir);
    Router::new()
        .nest_service(urls::AssetBase::AXUM, serve_dir)
        .route(urls::Home::AXUM, get(Redirect::temporary(&urls::Resolver::new())))
        .merge(api::router())
        .merge(ui_explorer::router())
        .merge(ui_resolver::router())
}
