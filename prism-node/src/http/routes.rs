use rocket::response::Redirect;
use rocket::{get, uri, State};

use crate::http::response::SsrPage;
use crate::http::views;
use crate::AppState;

#[get("/")]
pub async fn index() -> Redirect {
    let redirect_uri = uri!(resolver());
    Redirect::temporary(redirect_uri)
}

#[get("/resolver")]
pub async fn resolver() -> SsrPage {
    SsrPage(views::resolver::ResolverPage())
}

#[get("/explorer")]
pub async fn explorer(state: &State<AppState>) -> SsrPage {
    let cursor = state.cursor_rx.as_ref().and_then(|rx| rx.borrow().to_owned());
    let dids = state.did_service.get_all_dids().await.unwrap(); // FIXME: unwrap
    SsrPage(views::explorer::ExplorerPage(cursor, dids))
}
pub mod hx {
    use rocket::form::Form;
    use rocket::{post, State};

    use crate::http::contract::form::{HxRpcForm, ResolveDidForm};
    use crate::http::contract::hx::HxRpc;
    use crate::http::response::SsrComponent;
    use crate::http::views;
    use crate::AppState;

    #[post("/hx/did-resolutions", data = "<form>")]
    pub async fn resolve_did(form: Form<ResolveDidForm>, state: &State<AppState>) -> SsrComponent {
        let result = state.did_service.resolve_did(&form.did).await;
        match result {
            Ok((result, debug)) => SsrComponent(views::resolver::ResolutionResultDisplay(result, debug)),
            Err(e) => SsrComponent(views::resolver::ResolutionErrorDisplay(e.to_string())),
        }
    }

    #[post("/hx/rpc", data = "<form>")]
    pub async fn rpc(form: Form<HxRpcForm>, state: &State<AppState>) -> SsrComponent {
        let rpc = serde_json::from_str::<HxRpc>(&form.rpc).unwrap(); // FIXME: unwrap
        match rpc {
            HxRpc::GetExplorerDltCursor {} => {
                let cursor = state.cursor_rx.as_ref().and_then(|rx| rx.borrow().to_owned());
                SsrComponent(views::explorer::DltCursorStat(views::explorer::DltCursorStatProps {
                    cursor,
                }))
            }
            HxRpc::GetExplorerDidList {} => {
                let dids = state.did_service.get_all_dids().await.unwrap(); // FIXME: unwrap
                SsrComponent(views::explorer::DidList(views::explorer::DidListProps { dids }))
            }
        }
    }
}
