<div align="center">
  <img width=200 src="https://github.com/edfloreshz/devmode/blob/main/assets/img/devmode.png?raw=true"/>
  <h1>Dev(mode)</h1>
  <a href="https://crates.io/crates/devmode">
    <img src="https://img.shields.io/crates/v/devmode?label=Devmode" alt="crate"/>
  </a>
   <a href="https://crates.io/crates/devmode">
    <img src="https://img.shields.io/crates/d/devmode" alt="downloads"/>
  </a>
  <a href="https://aur.archlinux.org/packages/devmode-git/">
    <img src="https://img.shields.io/aur/version/devmode-git" alt="devmode-git"/>
  </a>
</div>

**Devmode** is a project management utility for developers.

```
Usage: dm <COMMAND>

Commands:
  cl    Clones a repository in a specific folder structure.
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
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
## Cloning

When you clone a repository it will be stored to your filesystem using a specific folder structure.

You can also use ` dm cl`

```
$HOME
└── Developer
    └── host
        └── owner
            └── repo
```

| Syntax           | Description               | Example                                          |
| ---------------- | ------------------------- | ------------------------------------------------ |
| `dm clone <url>` | Clone by providing a URL. | `dm clone https://github.com/edfloreshz/devmode` |

# Dependencies
- openssl

## Proposals

If you have a proposal for a new feature, open a new [issue](https://github.com/edfloreshz/devmode/issues).
