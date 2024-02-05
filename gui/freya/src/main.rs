#![allow(non_snake_case)]
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
use freya::hotreload::FreyaCtx;
use freya::prelude::*;

#[derive(Debug, PartialEq, Props)]
struct Devmode {
    page: Page,
    is_light: bool,
}

impl Default for Devmode {
    fn default() -> Self {
        Self {
            page: Default::default(),
            is_light: dark_light::detect() == dark_light::Mode::Light,
        }
    }
}

#[derive(Debug, Default, PartialEq, Clone, Copy)]
enum Page {
    #[default]
    Clone,
    Open,
    Workspaces,
    Preferences,
}

fn main() {
    dioxus_hot_reload::hot_reload_init!(Config::<FreyaCtx>::default());
    launch_cfg(
        app,
        LaunchConfig::<()>::builder()
            .with_title("Devmode")
            .with_min_height(300.0)
            .with_min_width(500.0)
            .build(),
    );
}

fn app(cx: Scope) -> Element {
    let app = use_ref(cx, || Devmode::default());
    let theme = match dark_light::detect() {
        dark_light::Mode::Dark => DARK_THEME,
        dark_light::Mode::Light | dark_light::Mode::Default => LIGHT_THEME,
    };

    render!(
        ThemeProvider { theme: theme,
            Body {
                rect { width: "100%", height: "100%",
                    rect {
                        width: "100%",
                        padding: "10 0 0 0",
                        direction: "horizontal",
                        cross_align: "center",
                        main_align: "center",
                        Button { onclick: move |_| app.with_mut(|app| app.page = Page::Clone), label { "Clone" } }
                        Button { onclick: move |_| app.with_mut(|app| app.page = Page::Open), label { "Open" } }
                        Button { onclick: move |_| app.with_mut(|app| app.page = Page::Workspaces), label { "Workspaces" } }
                        Button { onclick: move |_| app.with_mut(|app| app.page = Page::Preferences), label { "Preferences" } }
                    }
                    Page { page: app.read().page, is_light: app.read().is_light }
                    Footer { page: app.read().page, is_light: app.read().is_light }
                }
            }
        }
    )
}

fn Page(cx: Scope<Devmode>) -> Element {
    let background = if cx.props.is_light {
        "rgb(240, 240, 240)"
    } else {
        "rgb(32, 32, 32)"
    };
    let shadow = if cx.props.is_light {
        "0 0 10 1 rgb(100, 100, 100, 120)"
    } else {
        "0 0 10 1 rgb(0, 0, 0, 120)"
    };
    let page = match cx.props.page {
        Page::Clone => render!(
            rect { label { "Remote repositories" } }
        ),
        Page::Open => render!(
            rect { label { "Local repositories" } }
        ),
        Page::Workspaces => render!(
            rect { label { "Workspaces" } }
        ),
        Page::Preferences => render!(
            rect { label { "Preferences" } }
        ),
    };
    render! {
        rect { width: "100%", cross_align: "center",
            rect {
                width: "100%",
                height: "calc(100% - 65)",
                padding: "15",
                margin: "15",
                corner_radius: "10",
                corner_smoothing: "75%",
                background: background,
                shadow: shadow,
                page
            }
        }
    }
}

fn Footer(cx: Scope<Devmode>) -> Element {
    render! {
        rect {
            width: "100%",
            height: "65",
            padding: "10",
            cross_align: "center",
            main_align: "center",
            direction: "horizontal",
            match cx.props.page {
                Page::Clone => render!(
                    Button { label { "Clone" } }
                ),
                Page::Open => render!(
                    Button { label { "Open" } },
                    Button { label { "Add to workspace" } }
                ),
                Page::Workspaces => render!(
                    Button { label { "Add" } },
                    Button { label { "Edit" } },
                    Button { label { "Remove" } }
                ),
                Page::Preferences => render! {
                    Button { label { "Save" } }
                }
            }
        }
    }
}
