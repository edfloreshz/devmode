use app::Devmode;
use iced::Application;

mod app;
mod pages;

fn main() -> iced::Result {
    Devmode::run(iced::Settings::default())
}
