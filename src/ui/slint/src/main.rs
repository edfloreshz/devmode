slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    ui.global::<Logic>().on_clone(move |url| {
        println!("{url}");
    });

    ui.run()
}
