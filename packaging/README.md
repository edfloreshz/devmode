# Packaging

## Flathub (`dev.edfloreshz.Devmode`)

Files in `flatpak/`:

| File | Purpose |
|------|---------|
| `dev.edfloreshz.Devmode.yaml` | flatpak-builder manifest |
| `dev.edfloreshz.Devmode.desktop` | desktop entry |
| `dev.edfloreshz.Devmode.metainfo.xml` | AppStream metadata |
| `generated-sources.json` | offline vendor listing for Flathub; generated, committed |

Flathub builds are offline, so every crate has to be listed explicitly in
`generated-sources.json`. It is kept up to date automatically: the
`.github/workflows/cargo-sources.yml` workflow regenerates and commits it
whenever `Cargo.lock` changes on `main`/`rewrite` (or on manual dispatch).

Local test build:

```bash
flatpak install flathub org.freedesktop.Sdk//25.08 \
  org.freedesktop.Platform//25.08 \
  org.freedesktop.Sdk.Extension.rust-stable//25.08
flatpak-builder --user --install --force-clean build \
  packaging/flatpak/dev.edfloreshz.Devmode.yaml
flatpak run dev.edfloreshz.Devmode
```

### Still TODO before submitting

- Add real screenshots to the repo, reference their raw URLs in the metainfo
  file, and keep `appstreamcli validate --strict` passing.
- Replace the manifest's `type: dir` source with a tagged `type: git` source.
- Open the submission PR against `flathub/flathub`.
