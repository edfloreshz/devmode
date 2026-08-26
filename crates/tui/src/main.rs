mod actions;
mod app;
mod error;
mod event;
mod ui;

use std::io::{self, Stdout};
use std::time::Duration;

use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use app::App;
use error::Result;

type Tui = Terminal<CrosstermBackend<Stdout>>;

fn init_terminal() -> Result<Tui> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}

fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}

fn install_panic_hook() {
    let original = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = restore_terminal();
        original(panic_info);
    }));
}

fn run(terminal: &mut Tui) -> Result<Option<String>> {
    let mut app = App::new()?;

    while !app.should_quit {
        terminal.draw(|frame| ui::draw(frame, &app))?;

        app.poll_clone()?;
        app.spinner_tick = app.spinner_tick.wrapping_add(1);

        match event::next_action(Duration::from_millis(250), app.mode(), app.active_tab)? {
            event::Action::Quit => app.should_quit = true,
            event::Action::Back => app.back(),
            event::Action::NextTab => app.next_tab(),
            event::Action::MoveUp => app.move_up(),
            event::Action::MoveDown => app.move_down(),
            event::Action::StartFilter => app.start_filter(),
            event::Action::StopFilter => app.stop_filter(),
            event::Action::OpenClone => app.open_clone_form(),
            event::Action::OpenCreate => app.open_create_form(),
            event::Action::OpenTrack => app.open_track_form(),
            event::Action::OpenRemoveConfirm => app.open_remove_confirm(),
            event::Action::ApplyRelayout => app.apply_relayout()?,
            event::Action::OpenWorkspaceCreate => app.open_workspace_create_form(),
            event::Action::OpenWorkspaceConfig => app.open_workspace_config_form(),
            event::Action::OpenWorkspaceDelete => app.open_workspace_delete_confirm(),
            event::Action::OpenWorkspaceEnv => app.open_workspace_env_form(),
            event::Action::OpenAddMember => app.open_add_member_picker(),
            event::Action::ToggleDetailFocus => app.toggle_detail_focus(),
            event::Action::RemoveDetailItem => app.remove_detail_item(),
            event::Action::SwitchWorkspace => app.request_switch(),
            event::Action::FormNextField => app.form_next_field(),
            event::Action::FormToggleNoGit => app.form_toggle_no_git(),
            event::Action::FormSubmit => app.submit_form()?,
            event::Action::FormCancel => app.cancel_form(),
            event::Action::ConfirmYes => app.accept_confirm()?,
            event::Action::ConfirmNo => app.cancel_confirm(),
            event::Action::ConfirmToggleDelete => app.confirm_toggle_delete(),
            event::Action::PickerSubmit => app.submit_picker(),
            event::Action::PickerCancel => app.cancel_picker(),
            event::Action::Input(key) => app.handle_text_input(key),
            event::Action::None => {}
        }
    }

    Ok(app.pending_switch)
}

fn main() -> Result<()> {
    install_panic_hook();
    let mut terminal = init_terminal()?;

    let result = run(&mut terminal);

    restore_terminal()?;

    match result? {
        Some(workspace_id) => actions::switch_workspace(&workspace_id),
        None => Ok(()),
    }
}
