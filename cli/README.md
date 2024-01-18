<div align="center">
    <img width=200 src="https://github.com/edfloreshz/devmode/blob/main/assets/img/logo.png?raw=true"/>
    <h1>Dev(mode)</h1>
    <a href="https://github.com/edfloreshz/devmode/actions/workflows/rust.yml">
    <img src="https://img.shields.io/github/workflow/status/edfloreshz/devmode/Rust?logo=GitHub" alt="build"/>
  </a>
  <a href="https://crates.io/crates/devmode">
    <img src="https://img.shields.io/crates/v/devmode?label=Devmode" alt="crate"/>
  </a>
   <a href="https://crates.io/crates/devmode">
    <img src="https://img.shields.io/crates/d/devmode" alt="downloads"/>
  </a>
  <a href="https://t.me/codewithed">
    <img src="https://img.shields.io/static/v1?label=chat&message=Telegram&color=blue&logo=telegram" alt="chat on telegram"/>
  </a>
  <a href="https://aur.archlinux.org/packages/devmode-git/">
    <img src="https://img.shields.io/aur/version/devmode-git" alt="devmode-git"/>
  </a>
</div>

**Dev(mode)** is a project management utility for developers.

```
USAGE:
   dm [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    clone        Clones a repository in a specific folder structure.
    config       Write changes to your configuration.
    fork         Clones a repo and sets the upstream to your fork.
    help         Print this message or the help of the given subcommand(s)
    open         Opens a project on your selected text editor.
    workspace    Create workspaces to store your projects.
```

## Installation

#### Cargo

```
cargo install devmode
```
#### Arch Linux
```
paru -S devmode-git
```

## Configuration
The `config` command will help you set your app preferences. 

When you first run `dm config` you will be prompted with a setup asking you to set your settings.

After that, you can:

| Syntax | Description |
| --------------------- | ----------------------------------------------------------|
| `dm config --all`     | Prompt the first-time setup to configure everything.    |
| `dm config --editor`  | Save your prefered text editor to the open projects with. |
| `dm config --host`    | Save your Git provider to clone projects from.            |
| `dm config --owner`   | Save your Git username to identify yourself.              |
| `dm config --show`    | Print the current settings.                               |
| `dm config --map`     | Save your currently cloned project paths.                 |


## Workspaces
Think of workspaces as containers for your repositories, you can classify and manipulate them in different ways.

To create a new workspace use:

`dm workspace <name>`

When you create a workspace, you can use it to clone a repository to that workspace.

`dm clone <repo> --workspace <name>` 

You can `add` and `remove` existing repositories to a workspace.

`dm workspace <name> --add | --remove <repo>`

If you no longer need a workspace, you can either move the repositories to another workspace manually or delete the workspace and all repositories inside it will return to the owner's folder.

`dm workspace <name> --delete`

You can also rename workspaces, the folders will be updated accordingly.

`dm workspace <name> --rename <name>`

You can list your existing workspaces.

`dm workspace --list`

| Syntax  | Description | Example |
| ------- | ----------- | ------- |
| `dm workspace <name>`                   | Create a new workspace.           | `dm workspace office`                             |
| `dm workspace <name> --add <repo>`      | Add repository to workspace.      | `dm workspace office --add devmode`               |
| `dm workspace <name> --remove <repo>`   | Remove repository from workspace. | `dm workspace office --remove devmode`            |
| `dm workspace <name> --delete`          | Delete a workspace.               | `dm workspace office --delete`                    |
| `dm workspace <name> --rename <name>`   | Rename a workspace.               | `dm workspace office --rename work`               |
| `dm workspace --list`                   | List all workspaces.              | `dm workspace --list`                             |

## Cloning

**Dev(mode)** facilitates repository storage and organization in your filesystem.

### How it works

When you clone a repository it will be stored to your filesystem using a specific folder structure.

You can also use ` dm cl`

```
$HOME
└── Developer
    └── host
        └── owner
            └── workspace?
                └── repo
```

This makes it easier for you to find repositories and allows `dm` to open them by just specifying the name of the
project.

| Syntax | Description | Example |
| --------------------------------------------- | ----------------------------------- | ------------------------------------------------- |
| `dm clone`                                    | Clone a repository using the setup. | `dm clone`                                        |
| `dm clone <url>`                              | Clone by providing a URL.           | `dm clone https://github.com/edfloreshz/devmode`  |
| `dm clone <provider> <owner> <repository>`    | Clone by providing parameters.      | `dm clone github edfloreshz devmode`              |
| `dm clone <args> --workspace <workspace>`     | Clone into a specified workspace.   | `dm clone gh edfloreshz devmode -w office`        |

The following commands only work when you run `dm config`.

| Syntax | Description | Example |
| ----------- | ------ | ------- |
| `dm config <repo>`            | Clone by providing a repo name.   | `dm clone devmode`        |
| `dm clone <repo1> <repo2>`    | Clone multiple repositories.      | `dm clone devmode sensei` |

## Fork

Clone a repo and set the upstream url in one command.

Use the `--upstream` or `-u` to set the upstream repository, then specify the repository that you wish to configure.

| Description | Syntax | Example |
| ----------------------------------------------------- | ------------------------------------------------- | ------------------------------------------------------------------ |
| `dm fork <provider> <owner> <repo> -u <upstream-url>` | Clone and set the upstream repository.            | `dm fork gh edfloreshz cosmic -u https://github.com/pop-os/cosmic` |
| `dm fork <url> -u <upstream-url>`                     | Using URLs.                                       | `dm fork https://github.com/edfloreshz/cosmic -u https://github.com/pop-os/cosmic` |
| `dm fork --upstream <upstream-url>`                   | Just using the upstream URL.                      | `dm fork --upstream https://github.com/pop-os/cosmic` |

## Open a project

Opens a project with your selected text editor.

You can also use ` dm o`

| Description | Syntax | Example |
| ------------------- | --------------- | ----------------- |
| `dm open <project>` | Open a project. | `dm open devmode` |

If two or more projects with the same name are found, you will have to choose which one to open.

# Dependencies
- openssl

## Fedora
```
sudo dnf install -y openssl-devel
```

## Ubuntu
```
sudo apt install -y libssl-sdev
```

## Proposals

If you have a proposal for a new feature, open a new [issue](https://github.com/edfloreshz/devmode/issues).
