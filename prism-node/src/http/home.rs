use axum::Router;
use axum::response::Redirect;
use axum::routing::get;

use super::urls;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route(
        &urls::Home::axum_url(),
        get(|| async { Redirect::permanent(&urls::Explorer::make_url()) }),
    )
}
