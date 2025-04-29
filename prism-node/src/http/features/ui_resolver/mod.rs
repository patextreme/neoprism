use axum::Router;
use axum::extract::{Query, State};
use axum::routing::get;
use maud::Markup;
use models::ResolveQuery;

use crate::AppState;
use crate::http::urls;

mod models;
mod views;

pub fn router() -> Router<AppState> {
    Router::new().route(urls::Resolver::AXUM, get(index).post(resolve))
}

async fn index(State(state): State<AppState>) -> Markup {
    let network = state.network;
    views::index(network)
}

async fn resolve(Query(query): Query<ResolveQuery>, State(state): State<AppState>) -> Markup {
    let network = state.network;
    let (result, debug) = state.did_service.resolve_did(&query.did).await; // TODO: display this in the UI
    views::index(network)
}
