# Compatibility matrix

Results of **manual** tests with real devices. CI covers only software keys
(SoftHSM2, Windows software providers, a test keychain); everything that depends on hardware
is recorded here.

How to fill it in, with the installed app (see [`install.md`](install.md)):

1. Plug in the device and run `websign doctor --json > doctor.json`. It lists the key stores,
   drivers, readers and certificates the app sees, without asking for a PIN.
2. Sign once per algorithm through the test page on the website (`/test`) or from a terminal, for
   example `websign sign --hash SHA-256 --digest <64 hex digits> --algorithm RSASSA-PSS`; this
   also exercises the PIN, the wrong-PIN and the lockout paths.
3. Fill in one row per device and path, and attach `doctor.json` to the PR. The report contains
   no name, CPF/CNPJ, or serial number; still read it before you attach it.

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

## Hardware checks

Behavior that only real devices can confirm; tick each one in the PR that records the result.

- [ ] Windows: asking a minidriver card for its reader (`PP_SMARTCARD_READER`, silent) shows no UI and adds
  no noticeable delay to listing — SafeNet, SafeSign, ePass2003, Watchdata
  (`crates/websign-keystores/src/windows/reader.rs`, SPEC §4.3).
- [ ] macOS CryptoTokenKit: a blocked PIN is reported as "PIN locked", not "wrong PIN" — which `userInfo`
  entry carries the attempts left (`crates/websign-keystores/src/macos/errors.rs`, SPEC §5.4).

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

The `ci` workflow runs the key-store contract suite on every push: SoftHSM2 on Linux, CNG and
CAPI software keys on Windows, and a temporary keychain on macOS. The earlier risk-proof results
are summarized in each document of [`docs/prototypes/`](prototypes/).
