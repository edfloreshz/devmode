use devmode_shared::{action::Action, clone::CloneAction};

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    ui.global::<CloneLogic>().on_clone(move |url| {
        let mut clone = CloneAction::new(&url);
        match clone.run() {
            Ok(_) => (),
            Err(err) => {
                if let Some(error) = err.downcast_ref::<git2::Error>() {
                    match error.code() {
                        git2::ErrorCode::Exists => {
                            // if overwrite(clone.get_local_path()?)? {
                            //     clone.run()?;
                            // }
                        }
                        _ => eprint!("{error}"),
                    }
                }
            }
        };
    });

    ui.run()
}
