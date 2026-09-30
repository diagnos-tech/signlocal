# WebeSign

Sign on any website with your digital certificate: a USB token, a smart card, or a
certificate installed in the operating system. Works with Chrome, Edge, Firefox, and
Safari on Windows, macOS, and Linux.

> **Status:** under technical validation (risk proofs). There is no usable release yet.
> See [`docs/prototypes/`](docs/prototypes/).

## Install

See docs/install.md.

## How it works

```
site ──hash──▶ SDK ──▶ extension ──native messaging──▶ app ──▶ operating system
                                                        │      (CNG/CAPI, Keychain)
                                                        └────▶ PKCS#11 driver
site ◀──────────── raw signature (ECDSA as r‖s, RSA) ◀──┘
```

- **Only the hash** leaves the site; the document never does. The site assembles
  PAdES/CAdES/XAdES, so WebeSign works with any standard (ICP-Brasil, eIDAS, ...) and
  any country.
- **The app asks the operating system, not the chip.** Every token, card, or certificate
  that the operating system or a PKCS#11 driver can see works, with no list of supported models.
- **Nothing runs in the background** and no port is opened: the browser starts the app
  when it needs it. Every signature is confirmed in a window of the app itself.

## Components

| Folder | What it is | License |
|---|---|---|
| `sdk/` | TypeScript library for websites, zero dependencies | Apache-2.0 |
| `extension/` | Extension (Chrome, Edge, Firefox, Safari) that connects the page to the app | GPL-3.0-or-later |
| `app/` | Rust desktop app: native messaging host plus confirmation and diagnostics windows | GPL-3.0-or-later |
| `safari/` | Swift bridge that Apple requires for the Safari extension | GPL-3.0-or-later |
| `devices.json` | Per-device driver hints (USB and ATR), maintained by the community | CC0-1.0 |
| `packaging/`, `site/` | Per-store and per-OS packaging, and the download page | GPL-3.0-or-later |

The GPL of the app and the extension does not extend to websites: they talk to it through
messages, not through code linking, and the SDK that a site bundles is Apache-2.0.
The license includes an additional permission for distribution through app stores; see [`LICENSE`](LICENSE).

## Documentation

- [`docs/prototypes/`](docs/prototypes/): risk proofs per operating system
- [`docs/plan.md`](docs/plan.md): implementation plan
- [`docs/ux.md`](docs/ux.md): interface specification
- [`CONTRIBUTING.md`](CONTRIBUTING.md): how to contribute (DCO required)

All provisional names and identifiers live in [`project.toml`](project.toml).
