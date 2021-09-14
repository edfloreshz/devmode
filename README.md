<div align="center">
    <img width=200 src="assets/logo.png"/>
    <h1>Dev(mode)</h1>
</div>

**Dev(mode)** is a project management utility for developers.

```bash
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

## Clone a repo
**Dev(mode)** facilitates repository organization in your filesystem by using the following structure.
```
home
└── Developer
    └── host
        └── owner
            └── repo
```
### Usage
```bash
Clones a utils repository in a specific folder structure.

USAGE:
    devmode clone <args>...

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <args>...    Provide either a Git <url> or a Git <host> <owner> <repo>.
```
#### URL
You can clone by using a URL.
```bash
devmode clone https://github.com/edfloreshz/devmode
```
#### No URL
You can also type `<host>` `<owner>` `<repository>`
```bash
devmode clone github edfloreshz devmode
```

## Open a project
```
Opens a project on your selected text editor.

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

## Configuration
```bash
Sets options for configuration.

USAGE:
    devmode config [FLAGS]

FLAGS:
    -e, --editor     Sets the favorite editor to open projects.
    -h, --help       Prints help information
    -V, --version    Prints version information
```
You can configure your favorite text editor by executing:
```bash
devmode config -e
```

You will be presented with the following:
```bash
ᐅ  devmode config -e
? Choose your favorite editor: (vcnxH)
  v) Vim
  c) VSCode
  n) Nano
   ──────────────
  x) Abort
  h) Help, list all options
  Answer:
```

Choose your prefered text editor

```bash 
✔ Choose your favorite editor: · Vim
Settings updated.
```
