# prototypes

Risk proofs: each technical risk that could overturn an architecture decision is proven or refuted with real code, and documented with the answer, the evidence, and the decision.

- `README.md` — index of the proofs with their status, and a quick start for the kit
- `1-windows.md` — proof 1: CNG/CAPI with real tokens, foreground PIN dialog, MSIX registry writes and the execution alias
- `2-mac.md` — proof 2: Mac App Store sandbox, native messaging manifests, Keychain/CryptoTokenKit signing, Safari bridge design
- `3-tokens-mac.md` — proof 3: which token middlewares reach CryptoTokenKit, and whether PKCS#11 loads inside the sandbox
- `4-linux.md` — proof 4: signing through p11-kit and SoftHSM2, native messaging in Chromium, Firefox Snap portal
- `5-ui-screenshots.md` — proof 5: screenshots of the real egui window and headless kittest on Windows, macOS and four Linux distros; renderer fallback
- `kit/` — the Cargo workspace and scripts that produce the evidence (probe binary, core logic, test extension, per-OS scripts)
