use dm_core::config::Config;
use dm_core::error::Error as CoreError;
use dm_core::registry::RegistryStore;
use dm_core::workspace::WorkspaceStore;

use crate::error::Result;

pub fn run(workspace: String, cd: bool) -> Result<()> {
    let registry = RegistryStore::open_default()?;
    let workspaces = WorkspaceStore::open_default()?;
    let ws = workspaces.get(&workspace)?;

    let repos = workspaces
        .members(&ws.id)?
        .into_iter()
        .map(|id| registry.get(id))
        .collect::<dm_core::Result<Vec<_>>>()?;

    if repos.is_empty() {
        println!(
            "workspace '{}' has no members yet — add one with `dm workspace add {} <repo>`",
            ws.id, ws.id
        );
        return Ok(());
    }

    if cd {
        // Shell-eval friendly: a caller wraps this in
        // `eval "$(dm workspace switch <id> --cd)"` to actually change
        // directory, since a child process can't affect the parent shell.
        println!("cd {}", shell_quote(&repos[0].path.display().to_string()));
        return Ok(());
    }

    let global_config = Config::load()?;
    let editor = ws.editor.clone().or(global_config.editor.clone());
    let Some(editor) = editor else {
        println!(
            "no editor configured for '{}' (or globally) — member repos:",
            ws.id
        );
        for repo in &repos {
            println!("  {}", repo.path.display());
        }
        println!(
            "\nset one with `dm workspace config set {} editor <cmd>` or `dm config set editor <cmd>`, \
             or use --cd to print a cd command",
            ws.id
        );
        return Ok(());
    };

    let mut parts = editor.split_whitespace();
    let Some(program) = parts.next() else {
        println!("editor command for '{}' is empty", ws.id);
        return Ok(());
    };

    let mut command = std::process::Command::new(program);
    command.args(parts);
    for repo in &repos {
        command.arg(&repo.path);
    }
    for (key, value) in workspaces.env_list(&ws.id)? {
        command.env(key, value);
    }

    let status = command.status().map_err(CoreError::from)?;
    if !status.success() {
        eprintln!("'{editor}' exited with a non-zero status");
    }
    Ok(())
}

fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}
