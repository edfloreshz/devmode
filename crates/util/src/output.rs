use std::io::IsTerminal;

/// Whether colored output should be used on stdout: respects `NO_COLOR` and
/// falls back to plain output when stdout isn't a terminal.
pub fn use_color() -> bool {
    std::env::var_os("NO_COLOR").is_none() && std::io::stdout().is_terminal()
}

/// Same as `use_color`, but checks stderr, for error messages, which are
/// printed there and may be piped independently of stdout.
pub fn use_color_stderr() -> bool {
    std::env::var_os("NO_COLOR").is_none() && std::io::stderr().is_terminal()
}
