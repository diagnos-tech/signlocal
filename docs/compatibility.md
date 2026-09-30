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

### Cross-browser core, automated

What the automated run proves, per browser: `websign register --browser <name>` writes the registration a person's
install writes (the real folder or `HKCU` key, never a test-only one), the extension (unpacked development build;
Firefox: temporary add-on) finds the app, signs with an EC key (ECDSA SHA-256) and an RSA key (PKCS#1 v1.5 SHA-256,
PSS SHA-384) with every signature verified outside the app, the person's Cancel reaches the page as
`UserCancelled`, and a new site cancelled before **Continue** gets no certificate (D11). Software keys only.
Suite: `e2e/browsers/core.spec.ts`, run per browser by `e2e/run-browsers.sh`; CI runs it after the full e2e suite
on Ubuntu 24.04 and Windows for every push that touches the signing path, and on macOS nightly. Each CI run writes
its own table (OS, browser, version, result) to the job summary and the `browsers-<os>` artifact; copy the
results here when they change.

Legend as above; ⏳ = runs in CI, no result recorded here yet; 🔧 = failed in CI, fixed, waiting for the next run;
— = not offered on that OS. Brave and Opera versions are the Chromium version the run reports.

| Browser | Ubuntu 24.04 | Windows (latest runner) | macOS (latest runner) | How CI gets it |
|---|---|---|---|---|
| Google Chrome (stable) | ✅ 154 (local run, 2026-09-30) | ✅ 153 (CI, 2026-09-30) | ✅ 152 (CI, 2026-09-30) | preinstalled; else brew cask |
| Microsoft Edge (stable) | ✅ 154 (local run, 2026-09-30) | ✅ 153 (CI, 2026-09-30) | ✅ 152 (CI, 2026-09-30) | preinstalled; else Microsoft's apt repository / brew cask |
| Brave (stable) | ✅ 1.96 (local run, 2026-09-30) | ✅ Chromium 154 (CI, 2026-09-30) | 🔧 signs (CI, 2026-09-30) since registration writes Google Chrome's folder, which Brave reads; the run now turns Brave's updater off, which held its close, pending CI | Brave's apt repository; choco `brave`; brew cask |
| Opera (stable) | ✅ 136 (local run, 2026-09-30) | ✅ Chromium 152 (CI, 2026-09-30) | ✅ Chromium 151 (CI, 2026-09-30) | Opera's apt repository; choco `opera`; brew cask |
| Firefox (release) | ✅ 157 (local run, 2026-09-30) | 🔧 signs (156, CI, 2026-09-30); the harness now waits for Firefox's first tab to get its id, pending CI | ✅ 155 (CI, 2026-09-30) | Mozilla's tarball on Linux; preinstalled; brew cask |
| Firefox ESR | ✅ 140 (local run, 2026-09-30) | | | Mozilla's tarball (Linux only) |
| Vivaldi | ⚠️ registration traced, not run: Playwright crashes Vivaldi 8.2 | | | not in CI |
| Safari | — | — | see [install.md](install.md#safari) | not in this suite |

Not covered automatically: Chrome Beta/Dev/Canary, Edge Beta/Dev/Canary, Opera GX and Opera Beta/Developer, Brave
Beta/Nightly (registered by analogy with the stable channel), Snap and Flatpak browsers (the Firefox Snap reaches
the app through the desktop portal, which asks the person), and extensions installed from a store (none is
published yet). Where each browser reads its registration, and how that was established, is in
[research/native-messaging.md](research/native-messaging.md) §3.

### Real installations (manual)

| OS | Browser (version) | App installation | Host starts | Signs | Notes | Date · who |
|---|---|---|---|---|---|---|
| Windows 11 | Chrome | MSIX (alias) | | | | |
| Windows 11 | Edge | MSIX (alias) | | | | |
| Windows 11 | Firefox | MSIX (alias) | | | | |
| Windows 10 22H2 | Chrome | MSIX (alias) | | | | |
| macOS 15 | Chrome | sandboxed app | | | | |
| macOS 15 | Safari | `WebeSign.app` (direct, unsigned) with its appex; see [Safari](#safari-macos) | | | | |
| Ubuntu 24.04 | Chrome (deb) | .deb | | | | |
| Ubuntu 24.04 | Firefox (Snap, portal) | .deb | | | | |
| Fedora | Firefox (rpm) | .rpm | | | | |

## Safari (macOS)

Safari cannot be automated end to end: `safaridriver` drives pages but cannot install or enable an
extension, and an unsigned one needs **Allow unsigned extensions**, which asks for a password. So the
`safari` workflow proves everything up to Safari, and a person proves the rest with the script below.

What CI proves on every push (`.github/workflows/safari.yml`):

- The relay (`safari/Relay`) with fake hosts on macOS and Linux: framing, strict message shapes, sessions,
  polling, profiles, host exit, broken framing, backlog and session limits, idle reaping, a host that
  ignores SIGTERM being killed, and the client message types equal to the generated protocol.
- `cargo xtask package --target universal-apple-darwin` builds `WebeSign.app` with
  `Contents/PlugIns/WebeSignExtension.appex`: both CPU slices of the app, the appex and its host copy;
  the Safari extension point, principal class and host keys in the appex's `Info.plist`; the WXT safari
  build as its resources (MV3, `nativeMessaging`, no Chromium key); a strict signature check of the
  whole bundle; the appex sandboxed with the smart-card entitlement, its host copy inheriting it, the
  app itself not sandboxed.
- The packaged host binary, started with the arguments the appex uses, answers `hello` through the relay.
- PlugInKit registration is printed for information only.

Manual script (a Mac with Safari 17 or later; takes about five minutes):

1. Install the build under test with [`install.md`](install.md) (macOS), then follow its
   [Safari](install.md#safari) steps 1–4. Record: WebeSign listed in **Settings → Extensions** after
   opening the app once.
2. Open the site's test page (`/test/`) in Safari. The page reports the extension and the app as
   ready. Record "Host starts".
3. Sign with a Keychain certificate (a `.p12` imported into the login keychain): the WebeSign window
   opens in front, says "via Safari", shows the site and the verification code the page shows. Sign.
   The page verifies the signature. Record "Signs (Keychain)".
4. Repeat with a token that macOS sees through CryptoTokenKit, then with a PKCS#11-only driver if you
   have one. Record the PIN dialog's behavior.
5. Tick "Remember this site", sign again: no window when the certificate is remembered.
6. Leave the page idle for two minutes, then sign again (the session was closed and reopens).
7. Quit Safari, reopen it: the extension is off until **Allow unsigned extensions** is turned on again;
   then sign once more.
8. Run `websign doctor --json` and attach it, as for the other rows.

| macOS | Safari | Build | Listed after opening the app | Host starts | Signs (Keychain) | Signs (CTK token) | PKCS#11 token | Remember site | After idle / relaunch | Date · who |
|---|---|---|---|---|---|---|---|---|---|---|
| 15 | 18 | direct, unsigned (ad hoc) | | | | | | | | |
| 14 | 17 | direct, unsigned (ad hoc) | | | | | | | | |
| 13 | 17 | direct, unsigned (ad hoc) | | | | | | | | |

## CI (software keys)

The `ci` workflow runs the key-store contract suite on every push: SoftHSM2 on Linux, CNG and
CAPI software keys on Windows, and a temporary keychain on macOS. The earlier risk-proof results
are summarized in each document of [`docs/prototypes/`](prototypes/).
