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

/// The application icon, shared with the Flathub packaging
/// (`dev.edfloreshz.Devmode`). Embedded so the running window and taskbar
/// entry carry it on every platform, independent of any desktop file.
const ICON: &[u8] = include_bytes!("../../../assets/img/devmode.png");

pub fn main() -> iced::Result {
    // `mut` is only used on macOS, below.
    #[cfg_attr(not(target_os = "macos"), allow(unused_mut))]
    let mut window = iced::window::Settings {
        icon: iced::window::icon::from_file_data(ICON, None).ok(),
        ..Default::default()
    };

    // Keeps the OS's own traffic lights (and their drag/maximize behavior)
    // while letting App::view draw the bar's background and label itself:
    // the real titlebar stays, just transparent, with our content free to
    // run up behind it. No equivalent exists on other platforms, so
    // decorations there are just the plain OS default.
    #[cfg(target_os = "macos")]
    {
        window.platform_specific = iced::window::settings::PlatformSpecific {
            title_hidden: true,
            titlebar_transparent: true,
            fullsize_content_view: true,
        };
    }

    iced::application(App::boot, App::update, App::view)
        .title(App::title)
        .theme(App::theme)
        .subscription(App::subscription)
        .window(window)
        .window_size((1180.0, 760.0))
        .centered()
        .run()
}
