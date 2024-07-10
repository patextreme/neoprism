use rocket::response::Redirect;
use rocket::{get, uri};

use crate::http::response::SsrPage;
use crate::http::views;

#[get("/")]
pub async fn index() -> Redirect {
    let redirect_uri = uri!(resolver());
    Redirect::temporary(redirect_uri)
}

#[get("/resolver")]
pub async fn resolver() -> SsrPage {
    SsrPage(views::resolver::ResolverPage())
}

pub mod hx {
    use rocket::form::Form;
    use rocket::{post, State};

    use crate::app::service::DidService;
    use crate::http::contract::form::ResolveDidForm;
    use crate::http::response::SsrComponent;
    use crate::http::views;

    #[post("/hx/resolve-did", data = "<form>")]
    pub async fn resolve_did(form: Form<ResolveDidForm>, service: &State<DidService>) -> SsrComponent {
        let result = service.resolve_did(&form.did).await;
        match result {
            Ok((result, debug)) => SsrComponent(views::resolver::ResolutionResultDisplay(result, debug)),
            Err(e) => SsrComponent(views::resolver::ResolutionErrorDisplay(e.to_string())),
        }
    }
}
