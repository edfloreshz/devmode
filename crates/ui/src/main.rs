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
    iced::application(App::boot, App::update, App::view)
        .title(App::title)
        .theme(App::theme)
        .subscription(App::subscription)
        .window_size((1180.0, 760.0))
        .centered()
        .run()
}
