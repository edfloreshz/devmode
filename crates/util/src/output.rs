use std::io::IsTerminal;

/// Whether colored output should be used: respects `NO_COLOR` and falls back
/// to plain output when stdout isn't a terminal.
pub fn use_color() -> bool {
    std::env::var_os("NO_COLOR").is_none() && std::io::stdout().is_terminal()
}
