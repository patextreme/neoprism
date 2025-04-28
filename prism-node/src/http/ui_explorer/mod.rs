use axum::Router;
use axum::extract::{Query, State};
use axum::routing::get;
use maud::Markup;
use models::PageQueryParams;

use crate::AppState;
use crate::http::urls;

mod models;
mod views;

pub fn router() -> Router<AppState> {
    Router::new().route(&urls::Explorer::axum_url(), get(explorer_home))
}

async fn explorer_home(Query(page): Query<PageQueryParams>, State(state): State<AppState>) -> Markup {
    let network = state.network;
    let _cursor = state.cursor_rx.as_ref().and_then(|rx| rx.borrow().to_owned());
    let _dids = state.did_service.get_all_dids(page.page).await.unwrap();
    views::page(network)
}
