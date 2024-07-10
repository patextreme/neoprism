use dioxus::prelude::Element;
use rocket::response::content::RawHtml;
use rocket::response::{self, Responder};
use rocket::Request;

pub struct SsrComponent(pub Element);
pub struct SsrPage(pub Element);

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
