use axum::Router;

use crate::AppState;

pub mod api;
pub mod home;
pub mod ui_explorer;
pub mod ui_resolver;

mod shared;
mod urls;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(home::router())
        .merge(api::router())
        .merge(ui_explorer::router())
        .merge(ui_resolver::router())
}
