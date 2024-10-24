use libset::routes::home;

use super::{settings::Settings, DevmodeError, Error};

#[derive(Debug)]
pub struct WorkspaceOptions {
    pub name: Option<String>,
    pub add: bool,
    pub delete: bool,
    pub rename: Option<String>,
    pub include: Option<String>,
    pub remove: Option<String>,
    pub list: bool,
}

pub struct Workspace {
    name: String,
    index: usize,
    settings: Settings,
}

impl Workspace {
    pub fn new(name: &str) -> Self {
        let settings = Settings::current().unwrap_or_default();
        let index = settings
            .workspaces
            .names
            .iter()
            .position(|ws| ws.eq(name))
            .unwrap();
        Self {
            name: name.to_string(),
            index,
            settings,
        }
    }

    pub fn delete(&mut self) -> Result<(), Error> {
        let dev = home().join("Developer");
        for provider in std::fs::read_dir(dev)? {
            for user in std::fs::read_dir(provider?.path())? {
                let user = user?;
                for repo_or_workspace in std::fs::read_dir(user.path())? {
                    let repo_or_workspace = repo_or_workspace?;
                    let repo_name = repo_or_workspace.file_name().to_str().unwrap().to_string();
                    if self.settings.workspaces.names.contains(&repo_name)
                        && repo_name.eq(&self.name)
                    {
                        for repo in std::fs::read_dir(repo_or_workspace.path())? {
                            let repo = repo?;
                            fs_extra::dir::move_dir(repo.path(), user.path(), &Default::default())?;
                        }
                        std::fs::remove_dir_all(repo_or_workspace.path())?;
                    }
                }
            }
        }
        self.settings.workspaces.names.remove(self.index);
        self.settings.write(true)?;
        Ok(())
    }

    pub fn rename(&mut self, rename: &str) -> Result<(), Error> {
        let dev = home().join("Developer");
        for provider in std::fs::read_dir(dev)? {
            for user in std::fs::read_dir(provider?.path())? {
                let user = user?;
                for repo_or_workspace in std::fs::read_dir(user.path())? {
                    let repo_or_workspace = repo_or_workspace?;
                    let name = repo_or_workspace.file_name().to_str().unwrap().to_string();
                    if self.settings.workspaces.names.contains(&name) {
                        std::fs::rename(
                            repo_or_workspace.path(),
                            repo_or_workspace.path().parent().unwrap().join(rename),
                        )?;
                    }
                }
            }
        }

        let name = self
            .settings
            .workspaces
            .names
            .get_mut(self.index)
            .ok_or(Error::Devmode(DevmodeError::WorkspaceMissing))?;
        *name = rename.to_string();
        self.settings.write(true)?;

        Ok(())
    }
}
