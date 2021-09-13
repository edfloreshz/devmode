<div align="center">
    <img width=200 src="assets/logo.png"/>
    <h1>Dev(mode)</h1>
</div>

**Dev(mode)** is a project management utility for developers.

## Clone a repo
**Dev(mode)** facilitates repository organization in your filesystem by using the following structure.
```
home
└── Developer
    └── host
        └── owner
            └── repo
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
You can open a project with the following command:
```bash
devmode open <project>
```
If two or more projects with the same name are found, you will have to choose which one to open.
