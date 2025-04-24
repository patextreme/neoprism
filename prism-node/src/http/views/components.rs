use dioxus::prelude::*;
use prism_core::dlt::cardano::NetworkIdentifier;
use rocket::uri;

#[derive(PartialEq, Props, Clone)]
pub struct NavBarProps {
    #[props(!optional)]
    network: Option<NetworkIdentifier>,
}

pub fn NavBar(props: NavBarProps) -> Element {
    let resolver_uri = uri!(crate::http::routes::resolver(Option::<String>::None)).to_string();
    let explorer_uri = uri!(crate::http::routes::explorer(Option::<u32>::None)).to_string();
    let network_status = props
        .network
        .map(|i| {
            rsx! {
                p { class: "italic text-success", "network: {i.to_string()}" }
            }
        })
        .unwrap_or_else(|| {
            rsx! {
                p { class: "italic text-error", "disconnected" }
            }
        });
    rsx! {
        div { class: "navbar bg-neutral text-neutral-content flex-wrap-reverse",
            div { class: "flex-1",
                a { class: "btn btn-ghost text-xl", href: "{resolver_uri}", "Resolver" }
                a { class: "btn btn-ghost text-xl", href: "{explorer_uri}", "Explorer" }
            }
            div { class: "flex-1 px-2 justify-end", {network_status} }
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
