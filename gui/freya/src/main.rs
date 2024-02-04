#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use freya::hotreload::FreyaCtx;
use freya::prelude::*;

const CUSTOM_THEME: Theme = Theme {
    button: ButtonTheme {
        background: Cow::Borrowed("rgb(40, 40, 40)"),
        hover_background: Cow::Borrowed("rgb(50, 50, 50)"),
        border_fill: Cow::Borrowed("rgb(120, 120, 120)"),
        font_theme: FontTheme {
            color: Cow::Borrowed("white"),
        },
        ..DARK_THEME.button
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
        rect {
            width: "100%",
            height: "100%",
            ThemeProvider {
                theme: DARK_THEME,
                rect {
                    width: "100%",
                    background: "rgb(32, 32, 32)",
                    color: "white",
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
                    background: "rgb(32, 32, 32)",
                    cross_align: "center",
                    rect {
                        width: "100%",
                        height: "calc(100% - 65)",
                        padding: "15",
                        margin: "15",
                        background: "rgb(40, 40, 40)",
                        color: "white",
                        corner_radius: "10",
                        corner_smoothing: "75%",
                        Page {
                            page: app.read().page.clone()
                        }
                    }
                    rect {
                        width: "100%",
                        height: "65",
                        padding: "10",
                        background: "rgb(32, 32, 32)",
                        color: "white",
                        shadow: "0 0 10 1 rgb(0, 0, 0, 120)",
                        Button {
                            label {
                                "Clone"
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
