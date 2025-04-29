use axum::Router;
use axum::extract::State;
use axum::routing::get;
use maud::Markup;

use crate::AppState;
use crate::http::urls;

mod views;

pub fn router() -> Router<AppState> {
    Router::new().route(urls::Resolver::AXUM, get(resolver_home))
}

async fn resolver_home(State(state): State<AppState>) -> Markup {
    let network = state.network;
    views::page(network)
}
