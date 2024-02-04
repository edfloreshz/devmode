#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use freya::hotreload::FreyaCtx;
use freya::prelude::*;

const LIGHT_CUSTOM: Theme = Theme {
    button: ButtonTheme {
        background: cow_borrowed!("rgb(230, 219, 204)"),
        hover_background: cow_borrowed!("rgb(132, 115, 108)"),
        border_fill: cow_borrowed!("rgb(230, 219, 204)"),
        font_theme: FontTheme {
            color: cow_borrowed!("black"),
        },
        ..LIGHT_THEME.button
    },
    body: BodyTheme {
        background: cow_borrowed!("rgb(240, 234, 226)"),
        ..LIGHT_THEME.body
    },
    ..LIGHT_THEME
};

const DARK_CUSTOM: Theme = Theme {
    button: ButtonTheme {
        background: cow_borrowed!("rgb(132, 115, 108)"),
        hover_background: cow_borrowed!("rgb(56, 49, 46)"),
        border_fill: cow_borrowed!("rgb(56, 49, 46)"),
        font_theme: FontTheme {
            color: cow_borrowed!("white"),
        },
        ..DARK_THEME.button
    },
    body: BodyTheme {
        background: cow_borrowed!("rgb(47, 42, 39)"),
        ..DARK_THEME.body
    },
    ..DARK_THEME
};

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

    render!(
        ThemeProvider {
            theme: LIGHT_CUSTOM,
            Body {
                rect {
                    width: "100%",
                    height: "100%",
                    rect {
                        width: "100%",
                        padding: "10 0 0 0",
                        direction: "horizontal",
                        cross_align: "center",
                        main_align: "center",
                        Button {
                            onclick: move |_| app.with_mut(|app| app.page = Page::Clone),
                            label {
                                "Clone"
                            }
                        }
                        Button {
                            onclick: move |_| app.with_mut(|app| app.page = Page::Open),
                            label {
                                "Open"
                            }
                        }
                        Button {
                            onclick: move |_| app.with_mut(|app| app.page = Page::Workspaces),
                            label {
                                "Workspaces"
                            }
                        }
                        Button {
                            onclick: move |_| app.with_mut(|app| app.page = Page::Preferences),
                            label {
                                "Preferences"
                            }
                        }
                    }
                    rect {
                        width: "100%",
                        cross_align: "center",
                        rect {
                            width: "100%",
                            height: "calc(100% - 65)",
                            padding: "15",
                            margin: "15",
                            corner_radius: "10",
                            corner_smoothing: "75%",
                            // background: "rgb(132, 115, 108)",
                            // shadow: "0 0 10 1 rgb(0, 0, 0, 120)",
                            background: "rgb(255, 255, 255)",
                            shadow: "0 0 10 1 rgb(100, 100, 100, 120)",
                            Page {
                                page: app.read().page.clone()
                            }
                        }
                        rect {
                            width: "100%",
                            height: "65",
                            padding: "10",
                            cross_align: "center",
                            main_align: "center",
                            Button {
                                label {
                                    "Clone"
                                }
                            }
                        }
                    }
                }
            }
        }
    )
}

fn Page(cx: Scope<Devmode>) -> Element {
    let value = use_state(cx, String::new);
    match cx.props.page {
        Page::Clone => render!(rect {
            width: "100%",
            label {
                "Remote repositories"
            }
        }),
        Page::Open => render!(rect {
            label {
                "Local repositories"
            }
        }),
        Page::Workspaces => render!(rect {
            label {
                "Workspaces"
            }
        }),
        Page::Preferences => render!(rect {
            label {
                "Preferences"
            }
        }),
    }
}
