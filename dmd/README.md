<div align="center">
    <img width=200 src="../assets/logo.png"/>
    <h1>Dev(mode)</h1>
</div>

**Dev(mode)** is a project management utility for developers.

```
USAGE:
    devmode [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    clone     Clones a utils repository in a specific folder structure.
    config    Sets options for configuration.
    help      Prints this message or the help of the given subcommand(s)
    open      Opens a project on your selected text editor.
```

## Installation

#### Arch Linux
```
paru -S devmode-git
```

#### Cargo
```
cargo install devmode
```


## Configuration

The `config` command will help you configure `devmode` to your liking.

### Text Editor

You can set your favorite text editor running:

```
devmode config -e | devmode config --editor
``` 

### Git Host

You can set your Git host running:

```
devmode config -h | devmode config --host
``` 

### Git User

You can set your Git user running:

```
devmode config -o | devmode config --owner
``` 

### Configure everything

You can configure everything running:

```
devmode config -a | devmode config --all
``` 

### Show config

You can show your current config running:

```
devmode config -s | devmode config --show

Current settings:
Host: GitHub
Owner: edfloreshz
Editor: Visual Studio Code
``` 

## Clone a repo

**Dev(mode)** facilitates repository storage and organization in your filesystem.

### How it works

When you clone a repository it will be stored to your filesystem using a specific folder structure.

```
$HOME
└── Developer
    └── host
        └── owner
            └── repo
```

This makes it easier for you to find repositories and allows `devmode` to open them by just specifying the name of the
project.

### Usage

```
USAGE:
    devmode clone <args>...

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <args>...    Provide either a Git <url> or a Git <host> <owner> <repo>.
```

#### Clone by URL

```bash
devmode clone https://github.com/edfloreshz/devmode
```

#### Clone without URL

```bash
devmode clone <host> <owner> <repository>
```

#### Clone with `config.toml`

Running `devmode config` asks you to specify your Git `host` and `user`, now just type one of your repos.

```bash
devmode clone <repo>
```

#### Just clone

You can clone without specifying the arguments.

```bash
devmode clone
```

You will be presented with the following setup:

```
ᐅ  devmode clone

? Choose your Git host: ›
❯ GitHub
  GitLab
? Git username: › user
? Git repo name: › repo

Cloning edfloreshz/blog from GitHub...
```

## Open a project

Opens a project with your selected text editor.

```
USAGE:
    devmode open <project>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <project>    Provide a project name.
```

You can open a project with the following command:

```bash
devmode open <project>
```

If two or more projects with the same name are found, you will have to choose which one to open.

## Proposals
If you have a proposal for a new feature, open a new [issue](https://github.com/edfloreshz/devmode/issues).
