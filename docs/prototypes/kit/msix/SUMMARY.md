# msix

Windows MSIX packaging of the probe, to prove that a packaged app can register the native messaging host and be started through its execution alias.

- `AppxManifest.xml` — package manifest template (full trust, unvirtualized registry and files, console execution alias, `websign:` protocol)
- `build-msix.ps1` — builds the layout and PNGs, packs with `makeappx`, signs with a self-signed certificate, and installs the package
- `common.ps1` — names derived from `project.toml` and small Windows helpers shared by the two scripts
- `test-msix.ps1` — runs `register` through the alias and checks registry keys, manifests, alias path, and optionally the browser end to end
