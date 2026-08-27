// The GUI is a windowed app; on Windows it shouldn't also spawn a console.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod data;
mod design;
mod screen;
mod task;

#[cfg(test)]
mod tests;

use app::App;

pub fn main() -> iced::Result {
    let app = iced::application(App::boot, App::update, App::view)
        .title(App::title)
        .theme(App::theme)
        .subscription(App::subscription);

    // Keeps the OS's own traffic lights (and their drag/maximize behavior)
    // while letting App::view draw the bar's background and label itself:
    // the real titlebar stays, just transparent, with our content free to
    // run up behind it. No equivalent exists on other platforms, so
    // decorations there are just the plain OS default.
    #[cfg(target_os = "macos")]
    let app = app.window(iced::window::Settings {
        platform_specific: iced::window::settings::PlatformSpecific {
            title_hidden: true,
            titlebar_transparent: true,
            fullsize_content_view: true,
        },
        ..Default::default()
    });

    app.window_size((1180.0, 760.0)).centered().run()
}
