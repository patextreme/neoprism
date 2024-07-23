use dioxus::prelude::*;
use rocket::uri;

#[component]
pub fn NavBar() -> Element {
    let resolver_uri = uri!(crate::http::routes::resolver(Option::<String>::None)).to_string();
    let explorer_uri = uri!(crate::http::routes::explorer).to_string();
    rsx! {
        div { class: "navbar bg-neutral text-neutral-content",
            div { class: "flex-1",
                a { class: "btn btn-ghost text-xl", href: "{resolver_uri}", "Resolver" }
                a { class: "btn btn-ghost text-xl", href: "{explorer_uri}", "Explorer" }
            }
        }
    }
}

#[component]
pub fn PageTitle(title: String) -> Element {
    rsx! {
        div { class: "text-center text-3xl font-bold py-5", {title} }
    }
}

#[component]
pub fn PageContent(children: Element) -> Element {
    rsx! {
        div { class: "px-4", {children} }
    }
}
