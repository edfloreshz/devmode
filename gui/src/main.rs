use devmode_shared::clone::CloneAction;

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    ui.global::<CloneLogic>().on_clone(move |url| {
        match CloneAction::new().set_url(Some(url.to_string())).run() {
            Ok(_) => {}
            Err(err) => eprintln!("{err}"),
        };
    });

    ui.run()
}
