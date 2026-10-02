# Proof 1: Windows: CNG/CAPI, foreground PIN, and MSIX

**Status:** CNG/CAPI and MSIX proven in CI (Windows Server 2025); real tokens and Windows client edition pending · **Partial result:** YES for CNG and CAPI (including A1 on `PROV_RSA_FULL`) · **Decision:** see [Proposed decision](#proposed-decision)

## Goal

Answer with evidence, before building the app:

1. The Windows certificate store signs with **any** key the user has: minidriver (CNG),
   legacy CSP (CAPI), A1 installed from a `.pfx`, with real tokens
   (SafeSign, SafeNet, ePass2003, Watchdata)?
2. The system/middleware PIN dialog opens **in front of** the browser
   (`NCRYPT_WINDOW_HANDLE_PROPERTY` / `PP_CLIENT_HWND`)?
3. MSIX with `unvirtualizedResources` writes the **real** HKCU, and Chrome, Edge, and Firefox start the
   host through the *app execution alias* and exchange messages with it?
4. Does the app's pre-registration of the extension work?
5. Does the egui window open quickly over RDP?

If MSIX fails (item 3, or the store rejects the restricted capability), plan B is an MSI signed
through SignPath.

## How the kit proves it

### Certificate store (`probe/src/keystores/windows/`)

| File | Role |
|---|---|
| `store.rs`, `thumbprint.rs` | Opens `CurrentUser\MY` read-only, with `CERT_STORE_CTRL_AUTO_RESYNC` (tokens come and go without reopening); enumerates; finds again by SHA-1 thumbprint (`CERT_FIND_HASH`) |
| `key_info.rs` | Reads `CERT_KEY_PROV_INFO` (provider, type, container, keyspec) **without opening the key** |
| `hardware.rs` | Decides whether the provider is hardware by asking the provider, never the key |
| `acquire.rs` | `CryptAcquireCertificatePrivateKey` with `COMPARE_KEY`, `WINDOW_HANDLE`, and `ALLOW`/`PREFER`/`ONLY_NCRYPT_KEY` according to `--ncrypt` |
| `ncrypt.rs` | `NCryptSignHash`: PKCS#1 v1.5, PSS (salt = digest length), ECDSA (already comes out as `r‖s`) |
| `capi.rs` | `CryptCreateHash` + `HP_HASHVAL` + `CryptSignHashW`, bytes reversed (CAPI returns little-endian) |
| `errors.rs` | Cancellation → `Cancelled`; wrong PIN → `WrongPin`; locked → `PinLocked`; everything else → code + system message |
| `window.rs` | Owner of the PIN dialog: the caller's window (Chrome passes `--parent-window=<HWND>` to the host, but `0` when the request comes from an MV3 extension's service worker) or, without it, the console's window, which for a browser-started host is a **hidden** console (Chromium starts the host with `start_hidden`) |

Decisions and reasons:

- **`list` never asks for a PIN.** It only reads certificate properties. To learn whether a key is hardware, it asks the
  *provider* (not the key): `NCRYPT_IMPL_TYPE_PROPERTY` on the KSP opened with `NCryptOpenStorageProvider`,
  or `PP_IMPTYPE` on a `CRYPT_VERIFYCONTEXT | CRYPT_SILENT` context of the CSP. Neither touches the
  card. If the provider does not answer, a name heuristic applies: "smart card" or "Platform Crypto Provider"
  (TPM) → hardware; other "Microsoft …" providers → software; third parties → unknown.
  Accepted consequence: a certificate whose `KEY_PROV_INFO` points to a deleted key shows up in the
  list and fails only when signing.
- **What appears in `provider`:** the KSP/CSP name, `[CNG]` or `[CAPI type N, AT_…]`, and the reader when
  the container is fully qualified (`\\.\reader\…`). The container name is never shown: some
  middlewares name it after the holder.
- **Key opened once per session.** Without `CRYPT_ACQUIRE_CACHE_FLAG`: the keystore keeps the handle
  (and the certificate, because a Windows-cached handle only lives as long as the certificate) and
  releases it in `Drop`. Middlewares tie the PIN cache to the open handle, so the session asks for the
  PIN once. If a signature fails with a native error (card removed, middleware restarted), the
  handle is discarded and the next signature reopens it.
- **`--ncrypt allow` is what the app will use** (CNG when the key is CNG, CAPI when it is a CSP), like
  Windows does by default. chrome-token-signing uses `PREFER`, delegating the CAPI→CNG bridge to Windows;
  the proof runs all three modes to compare.
- **A1 on the `PROV_RSA_FULL` CSP.** The import wizard puts the `.pfx` in the
  "Microsoft Enhanced Cryptographic Provider v1.0" (`PROV_RSA_FULL`), which does not know SHA-2
  (`NTE_BAD_ALGID`). For the three Microsoft software CSPs of this type, the kit reopens the same
  container in the "Microsoft Enhanced RSA and AES Cryptographic Provider" (`PROV_RSA_AES`), which reads the
  same containers; this is what .NET does. The output shows `CryptSignHash (PROV_RSA_AES)`.
  A **third-party** `PROV_RSA_FULL` CSP without SHA-2 has no such shortcut: the error becomes
  `Unsupported("… try --ncrypt prefer")` and the real way out is PKCS#11. **Unproven until tokens are tested.**
- **PSS and ECDSA via CAPI** → `Unsupported` (CAPI has neither PSS nor ECC).
- **One signing call, not two.** A buffer sized for RSA-16384, and a retry only if the provider
  says it is too small (like .NET). This avoids a second round trip to the card.
- **Foreground PIN.** Three layers: `CRYPT_ACQUIRE_WINDOW_HANDLE_FLAG` when opening the key;
  `NCRYPT_WINDOW_HANDLE_PROPERTY` on the CNG handle before each signature; `PP_CLIENT_HWND` (null
  context = whole process) before each CAPI signature.

### MSIX (`msix/`)

- `AppxManifest.xml` (template filled in from `project.toml`): `runFullTrust`;
  `unvirtualizedResources` + `desktop6:RegistryWriteVirtualization` and
  `desktop6:FileSystemWriteVirtualization` turned off (without this, writes to HKCU and to
  `%LOCALAPPDATA%` would go to the package's private copy and no browser would see them);
  `uap5:AppExecutionAlias` with `desktop4:Subsystem="console"` (same pattern as the Microsoft
  Store Python, which works with stdin/stdout); `websign:` protocol that runs `register` (the store does not run an
  install script). Identity separate from the future app (`WebeSign.Probe`) and a test publisher.
- `build-msix.ps1`: layout, generated PNGs, `makeappx pack` (validates the manifest), self-signed
  certificate with Subject = Publisher, `signtool`, trust in `LocalMachine\TrustedPeople`,
  `Add-AppxPackage`.
- `test-msix.ps1`: deletes the host's keys and manifests; runs `register` **through the alias**; checks
  (a) the keys in the real HKCU with `reg.exe` from outside the package, (b) the manifest at the real, freshly
  written path (if missing, it looks for the virtualized copy to diagnose), (c) `path` = alias and the
  alias runs; (d) optional: `node nm-e2e/run.mjs --probe <alias> --no-register`.
- `msix_alias_path()` (`probe/src/platform/windows.rs`): with package identity
  (`GetCurrentPackageFamilyName`), returns `%LOCALAPPDATA%\Microsoft\WindowsApps\<family>\websign-probe.exe`
  if it exists (it cannot be taken over by another package with the same alias), otherwise the one at the `WindowsApps` root.

## CI evidence (software keys)

Workflow: `.github/workflows/prototypes.yml`, job `windows` (`windows-latest`).
Run: [36635313691](https://github.com/diagnos-tech/web-esign/actions/runs/36635313691) · Commit: `bdfa98f` · Runner: Windows Server 2025 Datacenter 10.0.26100, PowerShell 7.6.6

### Cases and expectations (`windows/ci-windows.ps1`)

Certificates created by `windows/make-test-certs.ps1` (removed at the end):

| Certificate | Provider | `allow` | `prefer` / `only` |
|---|---|---|---|
| `cng-rsa2048` | Software KSP (CNG) | 6/6 OK via `NCryptSignHash` | same |
| `cng-p256`, `cng-p384` | Software KSP (CNG) | 3/3 ECDSA OK | same |
| `capi-aes-rsa2048` | Enhanced RSA and AES CSP, `AT_SIGNATURE` | PKCS#1 3/3 OK via `CryptSignHash`; PSS = `not supported` (**expected failure**) | **6/6 OK via `NCryptSignHash`** (CAPI→CNG bridge), including PSS |
| `a1-rsa2048` | `AT_KEYEXCHANGE` key created in Enhanced CSP v1.0 (`PROV_RSA_FULL`), the way an imported `.pfx` ends up; `certutil -importpfx` hangs on the runner | PKCS#1 3/3 OK via `CryptSignHash (PROV_RSA_AES)`; PSS = `not supported` | **6/6 OK via `NCryptSignHash`**, including PSS |
| `capi-base-rsa2048` | Base CSP v1.0 (`PROV_RSA_FULL`) | same as A1 | **6/6 OK via `NCryptSignHash`**, including PSS |

All signatures above were verified with `probe-core::verify`. With `--silent`, no call
opened a dialog (the trace confirms `VERIFYCONTEXT | SILENT` when listing and `silent: true` on the signatures).

**Finding:** for keys in **Microsoft** CSPs, `prefer`/`only` makes Windows open the legacy key
through CNG and sign **RSASSA-PSS as well**. Proposal for the app: `prefer` as the default, falling back to
`allow` (CAPI) when a third-party CSP refuses. This can only be confirmed with real tokens: third-party CSPs
(SafeSign, Watchdata) may not cross the bridge.

"Observed" = recorded without failing CI: this is where Windows' CAPI→CNG bridge comes in.
Also mandatory: `list` shows each certificate with the right provider and marked as software.

```text
Summary (abridged; the full table is in the job log)
cng-rsa2048       allow|prefer|only  SHA-256/384/512  PKCS1 + PSS   OK  NCryptSignHash
cng-p256/p384     allow|prefer|only  SHA-256/384/512  ECDSA         OK  NCryptSignHash
capi-aes-rsa2048  allow              SHA-256/384/512  PKCS1         OK  CryptSignHash
capi-aes-rsa2048  allow              SHA-256/384/512  PSS           FAIL (expected) legacy CAPI key: RSASSA-PSS needs CNG
a1-rsa2048        allow              SHA-256/384/512  PKCS1         OK  CryptSignHash (PROV_RSA_AES)
capi-base-rsa2048 allow              SHA-256/384/512  PKCS1         OK  CryptSignHash (PROV_RSA_AES)
capi-*, a1        prefer|only        SHA-256/384/512  PKCS1 + PSS   OK  NCryptSignHash
All required checks passed.
```

Probe report: `report-windows` artifact of the same run (no names or serial numbers).

### MSIX on the runner (`msix/build-msix.ps1` + `msix/test-msix.ps1`)

The step is `continue-on-error`: the result is evidence in both directions.

Run [36637051578](https://github.com/diagnos-tech/web-esign/actions/runs/36637051578) (Windows Server 2025),
test package signed with a self-signed certificate and installed with `Add-AppxPackage`:

| Check | Result |
|---|---|
| `makeappx` accepts the manifest (rescap `runFullTrust` + `unvirtualizedResources`, desktop6 without virtualization, console alias) | ✅ |
| `Add-AppxPackage` installs | ✅ |
| alias exists (`WindowsApps` root and family folder) and runs with stdout captured | ✅ `websign-probe 0.1.0` |
| (a) keys in the **real** HKCU: Chrome, Edge, Firefox, Chromium, Brave, Vivaldi | ✅ all 6 |
| (b) manifests at the real path (`%LOCALAPPDATA%\websign\NativeMessagingHosts\`) | ✅ Chromium and Firefox |
| (c) `path` = family-folder alias (`WindowsApps\WebeSign.Probe_…\websign-probe.exe`), and it runs | ✅ |
| (d) Chromium starts the host **through the MSIX alias** and exchanges a message | ✅ `NM-E2E: PASS`: ping, list, digest validation; host log with `family=chromium` |

Reading: **the Microsoft Store path works technically**. A packaged app with
`unvirtualizedResources` writes the host registration to the real HKCU, and the browser starts it through the alias.
What the runner does not prove is still missing: client Windows 10/11, **branded** Chrome and Edge (CI uses Chromium), and
acceptance of the restricted capability in store certification.

## What CI does not prove

The runner is Windows Server, with no reader, no tokens, and no user in front of the screen. Still to be proven
with hardware and people:

- signing with SafeSign, SafeNet, ePass2003, and Watchdata, through CNG and through CAPI, and whether any
  third-party CSP is `PROV_RSA_FULL` without SHA-2;
- A1 imported by the wizard on Windows 10 and 11 (and with "strong protection" on);
- the PIN dialog in front of the terminal and in front of the browser;
- how many PINs a session asks for (one per session? one per signature?);
- MSIX on Windows 10 22H2 and 11 24H2 with a regular user; **real** Chrome, Edge, and Firefox
  starting the host through the alias;
- extension pre-registration (`HKCU\Software\Google\Chrome\Extensions\<id>` and Edge equivalents);
  it depends on the extension being published;
- egui window over RDP (the kit does not have a window yet; see step 8 of the script).

## Script for real tokens

Write everything down in a copy of the table at the end. **Careful with the PIN:** enter a wrong PIN on purpose at most
once per token, and then enter the right one, so it does not lock.

### 0. Preparation

1. Windows 10 22H2 **and** Windows 11 (24H2 or newer), regular user. Note the `winver` output.
2. Binary: download `websign-probe.exe` from CI or build it (`cargo build --release -p websign-probe`
   in `docs/prototypes/kit`). Open a terminal (Windows Terminal or `cmd`) in its folder.
3. Per token: model, installed middleware and version (Control Panel → Programs).
4. With the token connected:
   ```powershell
   .\websign-probe.exe devices
   .\websign-probe.exe report --out report-<token>.md
   ```
   The report has no personal data; attach it here.

### 1. List without a PIN

```powershell
.\websign-probe.exe list
.\websign-probe.exe list --every-path
```

Note: does the certificate appear? Which `provider` (`[CNG]` or `[CAPI type …]`)? `hardware`? **Did
any PIN prompt appear?** (it must not). With `--every-path`, does the same certificate also appear through
PKCS#11 as an alternative path?

### 2. Sign through the path the app uses

```powershell
.\websign-probe.exe sign --cert <8+ digits of the thumbprint> --hash all --pss
```

Note per case: OK/FAIL, `via NCryptSignHash` or `via CryptSignHash`, time. And about the PIN:
how many times it was asked; **did the dialog open in front of the terminal?** (if it opens behind or only flashes in the
taskbar, note that); dialog text/look (is it from Windows or from the middleware?).

### 3. Cancel and enter a wrong PIN

Repeat step 2 with SHA-256 only, clicking **Cancel** in the dialog → expected `cancelled by the user`.
Repeat entering the wrong PIN **once** → expected `wrong PIN` (note the code if `… failed with 0x…`
comes back). Then sign with the right PIN.

### 4. The other ways of opening the key

```powershell
.\websign-probe.exe sign --cert <fp> --hash all --pss --ncrypt prefer
.\websign-probe.exe sign --cert <fp> --hash all --pss --ncrypt only
```

Note whether the path changed (CAPI → CNG), whether PSS started working, and whether the dialog changed.

### 5. The same token through PKCS#11

```powershell
.\websign-probe.exe sign --cert <fp> --hash all --pss --every-path --module <dll>
```

Usual DLLs (confirm the path on the machine): SafeNet `C:\Windows\System32\eTPKCS11.dll`;
SafeSign `C:\Windows\System32\aetpkss1.dll`; ePass2003 `C:\Windows\System32\eps2003csp11.dll`;
Watchdata: look for `*pkcs11*.dll`/`WD*.dll` in `System32`; OpenSC
`C:\Program Files\OpenSC Project\OpenSC\pkcs11\opensc-pkcs11.dll`.
Note whether the PIN was asked in the terminal (expected, PKCS#11) and whether everything verified.

### 6. Installed A1

1. Double-click a test `.pfx` → wizard with the default options → "Current user".
2. Steps 1 and 2. Expected: `Microsoft Enhanced Cryptographic Provider v1.0 [CAPI type 1, AT_KEYEXCHANGE]`
   (or the software KSP) and `via CryptSignHash (PROV_RSA_AES)`.
3. Remove it (`certmgr.msc`), reimport checking **"Enable strong private key protection"** and
   repeat step 2: did the consent/password dialog open in front?

### 7. MSIX and browsers

In PowerShell 7 **as administrator**, in the `docs/prototypes/kit` folder:

```powershell
$env:PROBE_EXE = "<path>\websign-probe.exe"
./msix/build-msix.ps1
./msix/test-msix.ps1
```

(If the kit came in a ZIP, unblock the scripts first: `Get-ChildItem -Recurse *.ps1 | Unblock-File`.)
Note the final table. Then, **without** administrator privileges:

1. Load the test extension (`docs/prototypes/kit/extension`) unpacked in Chrome and Edge
   (developer mode) and as a temporary add-on in Firefox (`about:debugging`).
2. From the extension's test page/action, ask for the version, the list, and a signature. Note in each
   browser: did it connect? did the message come back? **did the PIN dialog open in front of the browser?**
3. In Settings → Apps → App execution aliases, **turn off** the WebeSign Probe alias and repeat: does the host still open (the manifest points to the copy in the package's
   family folder)?
4. Uninstall (`Get-AppxPackage WebeSign.Probe | Remove-AppxPackage`) and note what is left in
   `HKCU\Software\Google\Chrome\NativeMessagingHosts` and in `%LOCALAPPDATA%\websign`.

### 8. egui over RDP

The kit does not have a window yet. Until the app has the confirmation window, use a minimal eframe example
with `wgpu` (TODO(gustavo): decide whether a `websign-probe window` command goes into the kit) and measure, in an RDP
session and on a VM without a GPU, the time until the window appears and whether the software fallback (WARP) works.

### Table to fill in

| Token / case | Windows | Middleware (version) | `list` without PIN | Provider shown | `allow` | `prefer` | `only` | PKCS#11 | PINs per session | Dialog in front (terminal / browser) |
|---|---|---|---|---|---|---|---|---|---|---|
| SafeSign | | | | | | | | | | |
| SafeNet | | | | | | | | | | |
| ePass2003 | | | | | | | | | | |
| Watchdata | | | | | | | | | | |
| A1 (wizard) | | — | | | | | | — | | |
| A1 (strong protection) | | — | | | | | | — | | |

| MSIX | Windows 10 | Windows 11 |
|---|---|---|
| `test-msix.ps1` without FAIL | | |
| Chrome starts the host and exchanges a message | | |
| Edge starts the host and exchanges a message | | |
| Firefox starts the host and exchanges a message | | |
| Alias off: host still opens | | |
| Leftovers after uninstall | | |

## Known risks

- **Third-party `PROV_RSA_FULL` CSP without SHA-2:** CAPI does not sign; `PREFER` only bridges to CNG
  the Microsoft providers. Way out: PKCS#11 of the same token (deduplication already prefers the OS and falls back to
  PKCS#11 as an alternative).
- **`CRYPT_ACQUIRE_COMPARE_KEY_FLAG`** requires reading the container's public key; an old CSP that does not
  export the public key would fail to open. If this shows up, remove the flag for that case.
- **Cached handle and removed card:** handled by discarding the handle after a native error; confirm
  with the tokens that the error is native (and not, for example, an "insert the card" dialog).
- **Dialog focus:** even with an owner, Windows may deny the foreground to a process that has not
  received input (the dialog flashes in the taskbar). The host is a child of the browser, which has focus,
  but the owner it can get is the hidden console (`--parent-window` comes as `0` from an MV3 extension);
  measure in step 7. In the app, the owner will be the Confirmation window, which is already in front.
- **Uninstalling the MSIX leaves the HKCU keys and the manifests behind** (without virtualization Windows does not
  clean them). The host disappears, the browser reports "host not found", and the extension shows "app missing".
  The app recreates everything at each start.
- **Alias collision:** another package with `websign-probe.exe`/`websign.exe` takes the root alias;
  that is why the manifest uses the family-folder copy when it exists.
- **`unvirtualizedResources` is a restricted capability:** Microsoft may reject it in certification.
  It needs a justification in Partner Center (native messaging integration, precedents of apps
  that register hosts). This is the point most likely to bring MSIX down, and it can only be proven by submitting.
- **Visual C++ runtime:** a regular Rust MSVC binary depends on `vcruntime140.dll`, which a clean machine
  may not have. The kit already builds with `+crt-static` (`kit/.cargo/config.toml`) and does not depend on it;
  the app must keep this (or declare `Microsoft.VCLibs.140.00.UWPDesktop` in the MSIX).
- **Corporate environments** that block store apps/aliases (AppLocker, Store policies) will not
  get the MSIX; the MSI remains necessary for them in the long run, even with the MSIX approved.
- **The runner is Windows Server:** any CI success must be repeated on client Windows 10/11.

## Proposed decision

**MSIX approved** if, and only if:

1. `test-msix.ps1` without FAIL in CI **and** on Windows 10 22H2 and Windows 11;
2. Chrome, Edge, and Firefox start the host through the alias and exchange a message (step 7);
3. the test submission in Partner Center accepts `runFullTrust` + `unvirtualizedResources` with our
   justification.

If any of these fails → **MSI signed through SignPath** (writes the same HKCU keys, host
in `%LOCALAPPDATA%\Programs`), keeping the same binary.

Regardless of packaging, the app's signing path is: `--ncrypt allow` + AES reopening
for A1 on `PROV_RSA_FULL`, **provided** the four tokens sign SHA-256/384/512 through this path
with the PIN dialog in front. If a token only works with `prefer`, the app switches to
`prefer` for that provider; if no mode works, the token is left to PKCS#11.
