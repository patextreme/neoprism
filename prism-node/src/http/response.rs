use dioxus::prelude::Element;
use rocket::http::Status;
use rocket::response::content::RawHtml;
use rocket::response::{self, Responder};
use rocket::{Request, Response};

pub struct SsrComponent(pub Element);
pub struct SsrPage(pub Element);
pub struct HxRedirect<'a>(pub rocket::http::uri::Origin<'a>);

impl<'r> Responder<'r, 'static> for SsrComponent {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        let html = dioxus_ssr::render_element(self.0);
        let body = RawHtml(html);
        body.respond_to(req)
    }
}

impl<'r> Responder<'r, 'static> for SsrPage {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        let html = dioxus_ssr::render_element(self.0);
        let body = RawHtml(super::views::html_page(html));
        body.respond_to(req)
    }
}

impl<'r, 'a> Responder<'r, 'static> for HxRedirect<'a> {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let resp = Response::build()
            .raw_header("hx-redirect", self.0.to_string())
            .status(Status::Ok)
            .finalize();
        Ok(resp)
    }
}
