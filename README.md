<div align="center">
    <img width=200 src="../assets/logo.png"/>
    <h1>Dev(mode)</h1>
</div>

**Dev(mode)** is a project management utility for developers.

```
USAGE:
    dmd [SUBCOMMAND]

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
cargo install dmd
```

## Configuration

The `config` or `cf` command will help you configure `dmd` to your liking.

### Text Editor

You can set your favorite text editor running:

```
dmd config -e --editor
```

### Git Host

You can set your Git host running:

```
dmd config -h --host
```

### Git User

You can set your Git user running:

```
dmd config -o --owner
```

### Configure everything

You can configure everything running:

```
dmd config -a --all
```

### Show config

You can show your current config running:

```
dmd config -s --show

Current settings:
Host: GitHub
Owner: edfloreshz
Editor: Visual Studio Code
```

## Clone a repo

**Dev(mode)** facilitates repository storage and organization in your filesystem.

### How it works

When you clone a repository it will be stored to your filesystem using a specific folder structure.

You can also use `dmd cl`

```
$HOME
└── Developer
    └── host
        └── owner
            └── repo
```

This makes it easier for you to find repositories and allows `dmd` to open them by just specifying the name of the
project.

### Usage

```
USAGE:
    dmd clone <args>...

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <args>...    Provide either a Git <url> or a Git <host> <owner> <repo>.
```

#### Clone by URL

```bash
dmd clone https://github.com/edfloreshz/devmode
```

#### Clone without URL

```bash
dmd clone <host> <owner> <repository>
```

#### Clone with `config.toml`

Running `dmd config` asks you to specify your Git `host` and `user`, now just type one of your repos.

```bash
dmd clone <repo>
```

You can also clone multiple of your own repositories while using this option.

```bash
dmd clone <repo1> <repo2>
```

#### Just clone

You can clone without specifying the arguments.

```bash
dmd clone
```

You will be presented with the following setup:

```
ᐅ  dmd clone

? Choose your Git host: ›
❯ GitHub
  GitLab
? Git username: › user
? Git repo name: › repo

Cloning edfloreshz/blog from GitHub...
```

## Clone and set upstream (fork)

Clone a repo and set upstream, ideal for forks, when you clone a repository it will be stored to your filesystem using a
specific folder structure. Similar to how `dmd cl` works.

### Usage

```
    dmd fork --upstream <upstream> [args]...

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -u, --upstream <upstream>    Set the upstream to your fork <url>

ARGS:
    <args>...    Provide either a Git <url> or a Git <host> <owner> <repo>.

```

### Clone with url

Use the `--upstream` or `-u` to set the upstream repository, then specify the repository that you wish to modify.

```
dmd fork --upstream https://github.com/user/repo https://github.com/your-user/your-repo-fork
```

### Clone with upstream URL and remaining parameters.

```
dmd fk github <provider> <user> <forked-repo> -u <url>
```

### Just clone with the upstream URL

This will launch a clone setup and guide you throught the cloning process.

```
dmd fork -u https://github.com/user/repo
? Choose your Git host: ›
❯ GitHub
  GitLab
? Git username: › your-user
? Git repo name: › your-repo-fork

Cloning your-user/your-repo-fork from GitHub...
Setting https://github.com/user/repo how upstream
```

## Open a project

Opens a project with your selected text editor.

You can also use `dmd o`

```
USAGE:
    dmd open <project>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <project>    Provide a project name.
```

You can open a project with the following command:

```bash
dmd open <project>
```

If two or more projects with the same name are found, you will have to choose which one to open.

## Proposals

If you have a proposal for a new feature, open a new [issue](https://github.com/edfloreshz/devmode/issues).
