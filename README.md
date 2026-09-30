# WebeSign

Sign on any website — and from any desktop program — with your own digital
certificate: USB token, smart card, or a certificate installed on the
computer. Windows, macOS and Linux; Chrome, Edge, Firefox, Brave and Safari.

> **Status:** skeleton of the product (contracts defined, implementation in
> progress). Phase-0 risk proofs: [`docs/prototypes/`](docs/prototypes/).

## How it works

```
website ─ @websign/sdk ─▶ extension ─ native messaging ─▶ websign app ─▶ OS key store (CNG/CAPI, Keychain)
desktop program ─ websign connect / client libraries ──▶      │      └─▶ PKCS#11 token driver
                         ◀── raw signature (ECDSA r‖s, RSA), certificate, chain ──┘
```

- **Only the hash** leaves the caller; never the document. The caller builds
  PAdES/CAdES/XAdES/JAdES — so any format and any country (ICP-Brasil, eIDAS…).
- **The app asks the operating system, not the chip.** Any token, card or
  certificate the OS or a PKCS#11 driver can use works, with no model list.
- **Nothing runs in the background**, no port is opened: the browser or the
  calling program starts the app when needed. Every signature is confirmed in
  the app's own window.

## Parts

| Folder | What | License |
|---|---|---|
| [`sdk/`](sdk/) | `@websign/sdk`: TypeScript library for websites, zero dependencies | Apache-2.0 |
| [`clients/`](clients/) | `@websign/desktop` (Node) and `websign-client` (Rust) for desktop programs | Apache-2.0 |
| [`extension/`](extension/) | Browser extension (Chrome, Edge, Firefox, Safari) linking pages to the app | GPL-3.0-or-later |
| [`app/`](app/) | The `websign` desktop app (Rust): native messaging host, desktop API, confirmation and diagnostics windows | GPL-3.0-or-later |
| [`crates/`](crates/) | The app's libraries: core logic, protocol (Apache-2.0), key stores, devices, host engine, registration, i18n, UI model | GPL-3.0-or-later / Apache-2.0 |
| [`safari/`](safari/) | Swift bridge Apple requires for the Safari extension | GPL-3.0-or-later |
| [`devices.json`](devices.json) | Driver hints per device (USB and ATR), community-maintained | CC0-1.0 |
| [`i18n/`](i18n/) | Texts in en, pt-BR, pt-PT, es, fr, it, de | GPL-3.0-or-later |
| [`packaging/`](packaging/), [`scripts/install/`](scripts/install/) | Release artifacts and one-line installers | GPL-3.0-or-later |

The app's GPL does not extend to websites or programs that call it: they talk
through messages, not linking, and the SDK, the client libraries and the
protocol crate are Apache-2.0. The license includes an additional permission
for app-store distribution — see [`LICENSE`](LICENSE).

## Install

Prerelease builds are unsigned: follow [`docs/install.md`](docs/install.md)
(Windows, macOS, Linux, browser extension, uninstall). While the repository is
private, download with `gh release download --repo diagnos-tech/web-esign`.
Website: [`site/`](site/).

## Unsigned builds

Releases are not code-signed yet, so each system warns once. Windows
SmartScreen says "Windows protected your PC" (**More info → Run anyway**);
macOS Gatekeeper says the developer cannot be verified (right-click → **Open**);
release Firefox only loads the extension temporarily. The install scripts
avoid the first two; details and checksum verification in
[`docs/install.md`](docs/install.md#why-unsigned).

## Documentation

- [`docs/install.md`](docs/install.md) — install and uninstall per operating system
- [`docs/architecture/`](docs/architecture/) — contracts: protocol, web and desktop APIs, security, testing, packaging
- [`docs/plan.md`](docs/plan.md) — implementation plan and decisions
- [`docs/ux.md`](docs/ux.md) — interface specification
- [`docs/prototypes/`](docs/prototypes/) — risk proofs per operating system
- [`CONTRIBUTING.md`](CONTRIBUTING.md) — how to contribute (DCO required)

Provisional names and identifiers all live in [`project.toml`](project.toml).
