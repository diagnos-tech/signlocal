# Compatibility matrix

Results of **manual** tests with real devices. CI covers only software keys
(SoftHSM2, Windows software providers, a test keychain); everything that depends on hardware
is recorded here.

How to fill it in: run `websign-probe report --run-signatures --all --hash all --pss --out r.md`
(binaries are in the CI artifacts, `websign-probe-*`). For each device, fill in one row per
path and attach the report to the PR. The report contains no name, CPF/CNPJ, or serial number.

Legend: ✅ works · ❌ does not work · ⚠️ works with caveats (explain) · — not applicable · empty = not tested.

## Tokens and cards

| Device | Middleware (version) | OS | Path | Lists without PIN | RSA v1.5 | RSA-PSS | ECDSA | Foreground PIN | Wrong PIN / lockout | Date · who |
|---|---|---|---|---|---|---|---|---|---|---|
| SafeNet eToken 5110 | SafeNet Authentication Client | Windows 11 | CNG (`allow`) | | | | | | | |
| SafeNet eToken 5110 | SafeNet Authentication Client | Windows 11 | PKCS#11 (`eTPKCS11.dll`) | | | | | — | | |
| SafeSign (G&D StarSign) | SafeSign Identity Client | Windows 11 | CNG / CAPI | | | | | | | |
| Feitian ePass2003 | vendor driver / OpenSC | Windows 11 | CNG (minidriver) | | | | | | | |
| Watchdata ProxKey | Watchdata | Windows 11 | CAPI (CSP) | | | | | | | |
| Installed A1 (.pfx) | — | Windows 11 | CAPI → AES reopen | | | | — | — | — | |
| SafeNet eToken 5110 | SafeNet Authentication Client | macOS 15 | CryptoTokenKit | | | | | — | | |
| SafeSign | SafeSign 4.x | macOS 15 | CryptoTokenKit | | | | | — | | |
| ePass2003 | OpenSC (OpenSCToken) | macOS 15 | CryptoTokenKit | | | | | — | | |
| Cartão de Cidadão (PT) | Autenticação.gov 3.11+ | macOS 15 | CryptoTokenKit | | | | | — | | |
| DNIe (ES) | libpkcs11-dnie | macOS 15 | PKCS#11 (sandbox?) | | | | | — | | |
| SafeNet eToken 5110 | SafeNet Authentication Client | Ubuntu 24.04 | PKCS#11 via p11-kit | | | | | — | | |
| ePass2003 | OpenSC | Ubuntu 24.04 | PKCS#11 via p11-kit | | | | | — | | |

## Browsers (native messaging)

| OS | Browser (version) | App installation | Host starts | Signs | Notes | Date · who |
|---|---|---|---|---|---|---|
| Windows 11 | Chrome | MSIX (alias) | | | | |
| Windows 11 | Edge | MSIX (alias) | | | | |
| Windows 11 | Firefox | MSIX (alias) | | | | |
| Windows 10 22H2 | Chrome | MSIX (alias) | | | | |
| macOS 15 | Chrome | sandboxed app | | | | |
| macOS 15 | Safari | store app + appex | | | | |
| Ubuntu 24.04 | Chrome (deb) | .deb | | | | |
| Ubuntu 24.04 | Firefox (Snap, portal) | .deb | | | | |
| Fedora | Firefox (rpm) | .rpm | | | | |

## CI (software keys)

The latest result for each operating system is in the `report-linux`, `report-windows`, and
`report-macos` artifacts of the `prototypes` workflow, and is summarized in each document of
[`docs/prototypes/`](prototypes/).
