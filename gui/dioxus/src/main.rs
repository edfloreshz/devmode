#![allow(non_snake_case)]
use dioxus::prelude::*;

#[derive(Debug, Default, PartialEq, Props)]
struct Devmode {
    page: Page,
}

#[derive(Debug, Default, PartialEq, Clone)]
enum Page {
    #[default]
    Clone,
    Open,
    Workspaces,
    Preferences,
}

fn main() {
    dioxus_desktop::launch(App);
}

fn App(cx: Scope) -> Element {
    let app = use_ref(cx, || Devmode::default());

    render! {
        div { width: "100%", height: "100%",
            div { width: "100%", padding: "10 0 0 0",
                button { onclick: move |_| app.with_mut(|app| app.page = Page::Clone), label { "Clone" } }
                button { onclick: move |_| app.with_mut(|app| app.page = Page::Open), label { "Open" } }
                button { onclick: move |_| app.with_mut(|app| app.page = Page::Workspaces), label { "Workspaces" } }
                button { onclick: move |_| app.with_mut(|app| app.page = Page::Preferences), label { "Preferences" } }
            }
            div {
                width: "100%",
                height: "calc(100% - 65)",
                padding: "15",
                margin: "15",
                // background: "rgb(132, 115, 108)",
                // shadow: "0 0 10 1 rgb(0, 0, 0, 120)",
                background: "rgb(255, 255, 255)",
                Page { page: app.read().page.clone() }
            }
            div { width: "100%", height: "65", padding: "10",
                button { label { "Clone" } }
            }
        }
    }
}

fn Page(cx: Scope<Devmode>) -> Element {
    match cx.props.page {
        Page::Clone => render!(
            div { width: "100%", label { "Remote repositories" } }
        ),
        Page::Open => render!(
            div { label { "Local repositories" } }
        ),
        Page::Workspaces => render!(
            div { label { "Workspaces" } }
        ),
        Page::Preferences => render!(
            div { label { "Preferences" } }
        ),
    }
}
