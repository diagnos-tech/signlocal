# Risk proofs

Before building the product, every technical risk that could overturn an architecture
decision is proven (or refuted) with real code. Each proof has a document with the
**yes/no answer, the evidence, and the decision**.

| # | Proof | Document | Status |
|---|---|---|---|
| 1 | Windows: CNG/CAPI with real tokens, foreground PIN, MSIX writes HKCU and the browser starts the host through the alias | [1-windows.md](1-windows.md) | CI ✅ CNG/CAPI (legacy A1; PSS through the CNG bridge) and ✅ MSIX (real HKCU + Chromium starts the host through the alias); tokens, Windows client edition, and store acceptance still missing |
| 2 | Mac: sandboxed app writes manifests, Chrome starts the host and signs through CryptoTokenKit, Safari bridge | [2-mac.md](2-mac.md) | CI ✅ (sandbox, manifests, host, Keychain); real Mac and Safari still missing |
| 3 | Tokens on Mac: which middlewares expose the token to CryptoTokenKit; PKCS#11 inside the sandbox | [3-tokens-mac.md](3-tokens-mac.md) | CI: PKCS#11 ❌ in the sandbox → complement for PKCS#11-only tokens; token matrix still missing |
| 4 | Linux: signing through p11-kit, native messaging (including Firefox Snap) | [4-linux.md](4-linux.md) | ✅ SoftHSM2/p11-kit + Chromium end to end (local and CI); real token and Firefox Snap still missing |

## The kit

[`kit/`](kit/) is a Cargo workspace separate from the product:

- [`kit/probe-core/`](kit/probe-core/): pure, tested logic (algorithms, signature encoding,
  certificate summaries with ICP-Brasil and eIDAS, verification, deduplication).
  Specification in [`SPEC.md`](kit/probe-core/SPEC.md). It will be promoted into the app.
- [`kit/probe/`](kit/probe/): the `websign-probe` binary. It lists and signs through **every**
  path the app will use (CNG/CAPI, Keychain/CryptoTokenKit, PKCS#11), checks each
  signature, and also works as a native messaging host.
- `kit/extension/`, `kit/nm-e2e/`: minimal extension and end-to-end native messaging test.
- `kit/windows/`, `kit/msix/`, `kit/macos/`, `kit/linux/`: proof scripts per operating system.

CI ([`.github/workflows/prototypes.yml`](../../.github/workflows/prototypes.yml)) runs the
proofs with software keys on all three systems. Real tokens: see the script in each document.

### Quick start

```sh
cd docs/prototypes/kit
cargo run -p websign-probe -- list                 # all certificates, by origin
cargo run -p websign-probe -- sign --all --hash all --pss
cargo run -p websign-probe -- devices              # USB devices and readers with ATR
cargo run -p websign-probe -- report --run-signatures --all --out report.md
```

The report contains no names, CPF/CNPJ, or serial numbers, so it is safe to paste here.
