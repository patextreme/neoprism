use rocket::response::Redirect;
use rocket::{get, uri, State};

use crate::http::response::SsrPage;
use crate::http::views;
use crate::AppState;

#[get("/")]
pub async fn index() -> Redirect {
    let redirect_uri = uri!(resolver(Option::<String>::None));
    Redirect::temporary(redirect_uri)
}

#[get("/resolver?<did>")]
pub async fn resolver(did: Option<String>, state: &State<AppState>) -> SsrPage {
    let network = state.network.clone();
    let result = match did.as_ref() {
        Some(did) => {
            let result = state
                .did_service
                .resolve_did(did)
                .await
                .map_err(|e| anyhow::Error::new(e).chain().map(|e| e.to_string()).collect::<Vec<_>>())
                .map(|(result, debug)| {
                    let debug: Vec<_> = debug
                        .into_iter()
                        .map(|(meta, op, e)| {
                            let maybe_report = e
                                .map(|e| {
                                    let report = std::error::Report::new(e).pretty(true).show_backtrace(true);
                                    report
                                        .to_string()
                                        .split("\n")
                                        .map(|i| i.to_string())
                                        .collect::<Vec<_>>()
                                })
                                .unwrap_or_default();
                            (meta, op, maybe_report)
                        })
                        .collect();
                    (result, debug)
                });
            Some(result)
        }
        None => None,
    };
    SsrPage(views::resolver::ResolverPage(did, result, network))
}

#[get("/explorer?<page>")]
pub async fn explorer(state: &State<AppState>, page: Option<u64>) -> SsrPage {
    // UI use 1-index. internal pagination logic use 0-index.
    let page = page.map(|i| i.max(1) - 1);
    let cursor = state.cursor_rx.as_ref().and_then(|rx| rx.borrow().to_owned());
    let dids = state.did_service.get_all_dids(page).await.unwrap(); // FIXME: unwrap
    let network = state.network.clone();
    SsrPage(views::explorer::ExplorerPage(cursor, dids, network))
}

pub mod hx {
    use dioxus::prelude::*;
    use rocket::form::Form;
    use rocket::{post, State};

    use crate::http::model::form::HxRpcForm;
    use crate::http::model::hx::HxRpc;
    use crate::http::response::SsrComponent;
    use crate::http::views;
    use crate::AppState;

    #[post("/hx/rpc", data = "<form>")]
    pub async fn rpc(form: Form<HxRpcForm>, state: &State<AppState>) -> SsrComponent {
        let rpc = serde_json::from_str::<HxRpc>(&form.rpc).unwrap(); // FIXME: unwrap
        match rpc {
            HxRpc::GetExplorerDltCursor {} => {
                let cursor = state.cursor_rx.as_ref().and_then(|rx| rx.borrow().to_owned());
                SsrComponent(rsx! {
                    views::explorer::DltCursorStat { cursor }
                })
            }
            HxRpc::GetExplorerDidList { page } => {
                let dids = state.did_service.get_all_dids(page).await.unwrap(); // FIXME: unwrap
                SsrComponent(rsx! {
                    views::explorer::DidList { dids }
                })
            }
        }
    }
}

pub mod api {
    use rocket::http::Status;
    use rocket::serde::json::Json;
    use rocket::{get, State};

    use crate::app::service::error::ResolutionError;
    use crate::http::model::api::DidDocument;
    use crate::AppState;

    /// This endpoint is incomplete according to https://www.w3.org/TR/did-resolution/#bindings-https
    /// To be fully compliant, we need to add more format
    /// - application/ld+json;profile="https://w3id.org/did-resolution"
    /// - application/ld+json;profile="https://w3id.org/did-url-dereferencing"
    #[get("/api/dids/<did>", format = "application/json")]
    pub async fn resolver(did: String, state: &State<AppState>) -> Result<Json<DidDocument>, Status> {
        let result = state.did_service.resolve_did(&did).await;
        match result {
            Err(ResolutionError::InvalidDid { .. }) => Err(Status::BadRequest),
            Err(ResolutionError::NotFound) => Err(Status::NotFound),
            Err(ResolutionError::InternalError { .. }) => Err(Status::InternalServerError),
            Ok((did_state, _)) => Ok(Json(DidDocument::new(&did, did_state))),
        }
    }
}
