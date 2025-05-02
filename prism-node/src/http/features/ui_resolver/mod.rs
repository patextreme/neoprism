use axum::Router;
use axum::extract::{Query, State};
use axum::routing::get;
use maud::Markup;
use models::DidQuery;

use crate::AppState;
use crate::http::urls;

pub(in crate::http) mod models;
mod views;

pub fn router() -> Router<AppState> {
    Router::new().route(urls::Resolver::AXUM_PATH, get(index))
}

async fn index(Query(query): Query<DidQuery>, State(state): State<AppState>) -> Markup {
    let network = state.network;
    match query.did.as_ref() {
        None => views::index(network),
        Some(did) => {
            let (state, debug) = state.did_service.resolve_did(did).await;
            views::resolve(network, did, state, debug)
        }
    }
}
