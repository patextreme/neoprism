use axum::Router;
use axum::extract::{Query, State};
use axum::routing::get;
use maud::Markup;
use models::PageQueryParams;

use crate::AppState;
use crate::http::urls;

pub(in crate::http) mod models;
mod views;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(urls::Explorer::AXUM, get(index))
        .route(urls::ExplorerDltCursor::AXUM, get(dlt_cursor))
}

async fn index(Query(page): Query<PageQueryParams>, State(state): State<AppState>) -> Markup {
    let network = state.network;
    let cursor = state.cursor_rx.as_ref().and_then(|rx| rx.borrow().to_owned());
    let _dids = state.did_service.get_all_dids(page.page).await.unwrap(); // FIXME: unwrap
    views::index(network, cursor)
}

async fn dlt_cursor(State(state): State<AppState>) -> Markup {
    let cursor = state.cursor_rx.as_ref().and_then(|rx| rx.borrow().to_owned());
    views::dlt_cursor_card(cursor)
}
