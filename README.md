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
    clone     Clones a utils repository in a specific folder structure.
    config    Sets options for configuration.
    fork      Clone repo and set upstream to your fork
    help      Prints this message or the help of the given subcommand(s)
    open      Opens a project on your selected text editor.
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
            └── repo
```

This makes it easier for you to find repositories and allows `dm` to open them by just specifying the name of the
project.

| Syntax | Description | Example |
| --------------------------------------------- | --------------------------------- | ------------------------------------------------- |
| `dm clone`                                    | Clone a repository.               | `dm clone`                                        |
| `dm clone <url>`                              | Clone by providing a URL.         | `dm clone https://github.com/edfloreshz/devmode`  |
| `dm clone <provider> <owner> <repository>`    | Clone by providing parameters.    | `dm clone github edfloreshz devmode`              |

The following commands only work when you've specified a `provider` and `username`.

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

## Proposals

If you have a proposal for a new feature, open a new [issue](https://github.com/edfloreshz/devmode/issues).
