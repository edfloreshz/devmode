use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};

use crate::app::Tab;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Filtering,
    Form,
    Confirm,
    Picker,
}

pub enum Action {
    Quit,
    Back,
    NextTab,
    MoveUp,
    MoveDown,
    StartFilter,
    StopFilter,
    OpenClone,
    OpenCreate,
    OpenTrack,
    OpenRemoveConfirm,
    OpenWorkspaceCreate,
    OpenWorkspaceConfig,
    OpenWorkspaceDelete,
    OpenWorkspaceEnv,
    OpenAddMember,
    ToggleDetailFocus,
    RemoveDetailItem,
    SwitchWorkspace,
    ApplyRelayout,
    OpenSettingsEdit,
    DiscoveryScan,
    DiscoveryTrackAll,
    DiscoveryActivate,
    FormNextField,
    FormToggleNoGit,
    FormSubmit,
    FormCancel,
    ConfirmYes,
    ConfirmNo,
    ConfirmToggleDelete,
    PickerSubmit,
    PickerCancel,
    Input(KeyEvent),
    None,
}

pub fn next_action(timeout: Duration, mode: Mode, tab: Tab) -> std::io::Result<Action> {
    if !event::poll(timeout)? {
        return Ok(Action::None);
    }

    let Event::Key(key) = event::read()? else {
        return Ok(Action::None);
    };

    if key.kind != KeyEventKind::Press {
        return Ok(Action::None);
    }

    Ok(match mode {
        Mode::Filtering => match key.code {
            KeyCode::Esc | KeyCode::Enter => Action::StopFilter,
            _ => Action::Input(key),
        },
        Mode::Form => match key.code {
            KeyCode::Esc => Action::FormCancel,
            KeyCode::Enter => Action::FormSubmit,
            KeyCode::Tab | KeyCode::BackTab => Action::FormNextField,
            KeyCode::F(2) => Action::FormToggleNoGit,
            _ => Action::Input(key),
        },
        Mode::Confirm => match key.code {
            KeyCode::Char('y') | KeyCode::Enter => Action::ConfirmYes,
            KeyCode::Char('n') | KeyCode::Esc => Action::ConfirmNo,
            KeyCode::Char('d') => Action::ConfirmToggleDelete,
            _ => Action::None,
        },
        Mode::Picker => match key.code {
            KeyCode::Esc => Action::PickerCancel,
            KeyCode::Enter => Action::PickerSubmit,
            KeyCode::Up => Action::MoveUp,
            KeyCode::Down => Action::MoveDown,
            _ => Action::Input(key),
        },
        Mode::Normal => match (tab, key.code) {
            (_, KeyCode::Char('q')) => Action::Quit,
            (_, KeyCode::Esc) => Action::Back,
            (_, KeyCode::Tab) | (_, KeyCode::BackTab) => Action::NextTab,
            (_, KeyCode::Up) | (_, KeyCode::Char('k')) => Action::MoveUp,
            (_, KeyCode::Down) | (_, KeyCode::Char('j')) => Action::MoveDown,
            (Tab::Repos, KeyCode::Char('/')) => Action::StartFilter,
            (Tab::Repos, KeyCode::Char('c')) => Action::OpenClone,
            (Tab::Repos, KeyCode::Char('n')) => Action::OpenCreate,
            (Tab::Repos, KeyCode::Char('t')) => Action::OpenTrack,
            (Tab::Repos, KeyCode::Char('r')) => Action::OpenRemoveConfirm,
            (Tab::Repos, KeyCode::Char('l')) => Action::ApplyRelayout,
            (Tab::Settings, KeyCode::Char('e') | KeyCode::Enter) => Action::OpenSettingsEdit,
            (Tab::Discovery, KeyCode::Char('s')) => Action::DiscoveryScan,
            (Tab::Discovery, KeyCode::Char('a')) => Action::DiscoveryTrackAll,
            (Tab::Discovery, KeyCode::Enter) => Action::DiscoveryActivate,
            (Tab::Workspaces, KeyCode::Char('c')) => Action::OpenWorkspaceCreate,
            (Tab::Workspaces, KeyCode::Char('r')) => Action::OpenWorkspaceConfig,
            (Tab::Workspaces, KeyCode::Char('d')) => Action::OpenWorkspaceDelete,
            (Tab::Workspaces, KeyCode::Char('a')) => Action::OpenAddMember,
            (Tab::Workspaces, KeyCode::Char('v')) => Action::OpenWorkspaceEnv,
            (Tab::Workspaces, KeyCode::Char('s')) => Action::SwitchWorkspace,
            (Tab::Workspaces, KeyCode::Enter) => Action::ToggleDetailFocus,
            (Tab::Workspaces, KeyCode::Char('x')) => Action::RemoveDetailItem,
            _ => Action::None,
        },
    })
}
