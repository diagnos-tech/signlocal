# Proof 4: Linux: signing through p11-kit and native messaging

**Status:** draft · **Result (software keys):** **YES**: PKCS#11 lists, deduplicates, and signs RSA
(PKCS#1 v1.5 and PSS) and ECDSA (P-256, P-384, P-521) with SHA-256/384/512, through the module directly and through
`p11-kit-proxy.so`, and the page → extension → host → SoftHSM2 exchange passed in a real Chromium ·
**Result (real token):** _to be filled in_ · **Decision:** _depends on the matrix in §6_

## Goal

1. Signing through p11-kit with a real token (SafeNet, SafeSign, ePass2003, Watchdata…).
2. Native messaging in Ubuntu's Firefox Snap (§7).

Without hardware: SoftHSM2, and what remains unproven is recorded in §5.

## 1. How the kit proves it

```sh
cd docs/prototypes/kit
sudo apt-get install -y softhsm2 opensc p11-kit pcscd libpcsclite-dev
cargo build --release -p websign-probe
(cd nm-e2e && npm ci && npx playwright-core install --with-deps chromium)
PROBE_EXE=$PWD/target/release/websign-probe REPORT_PATH=/tmp/report-linux.md bash linux/ci-linux.sh
```

`CHROMIUM=/path/to/chrome` uses an already installed Chromium instead of the one Playwright downloads. CI does the
same in the `linux` job of the [workflow](../../.github/workflows/prototypes.yml) and publishes the report as an artifact.

| File | What it does |
|---|---|
| [`linux/softhsm-setup.sh`](kit/linux/softhsm-setup.sh) | creates an isolated SoftHSM2 token (its own directory and `SOFTHSM2_CONF`) with RSA-2048, EC P-256, P-384, and P-521; key and certificate with the same `CKA_ID`; idempotent. `SOFTHSM_ALWAYS_AUTH=1` adds a `CKA_ALWAYS_AUTHENTICATE` key |
| [`linux/ci-linux.sh`](kit/linux/ci-linux.sh) | runs `list`, `sign`, p11-kit, deduplication, `devices`, `report`, and the end-to-end test; fails if any expectation fails; cleans everything up on exit |
| [`probe/src/keystores/pkcs11/`](kit/probe/src/keystores/pkcs11/) | discovery, single load, `list` without PIN, `sign` (small files, one concept per file) |
| [`probe/src/devices/`](kit/probe/src/devices/) | USB (`nusb`) and readers with ATR (`pcsc`) |

Module sources and risks: [pkcs11-modules.md](../research/pkcs11-modules.md). Tokens and ATRs:
[tokens.md](../research/tokens.md).

## 2. Result

Environment: Ubuntu 24.04.4 (kernel 6.18, x86_64), SoftHSM2 2.6.1, OpenSC 0.25.0~rc1, p11-kit 0.25.3,
libccid 1.5.5, pcscd 2.0.3 (no readers), Chromium 141 (Playwright 1194), Node 22.22.2, rustc 1.98.1,
`cryptoki` 0.12.1, `websign-probe` 0.1.0. The whole script takes ~5 s.

| # | Question | Result | Evidence |
|---|---|---|---|
| 1 | Does `list` show the 4 certificates, marked as software, with PIN by the app, without the token label? | **YES** | §3.1 |
| 2 | Does `sign --all --hash all --pss` sign everything and does each signature verify? | **YES**: 15 of 15 (RSA 3 hashes × 2 algorithms, 3 EC keys × 3 hashes) | §3.2 |
| 3 | Does a wrong PIN become "wrong PIN" and does the token warn that attempts are running out? | **YES** | §3.3 |
| 4 | Is a module registered in p11-kit (user directory) discovered, and is a broken registration only a warning? | **YES** | §3.4 |
| 5 | Does the same certificate through two modules appear once, with "+1 other path"? | **YES**, in both load orders | §3.5 |
| 6 | Does `p11-kit-proxy.so` sign on its own? | **YES**: 15 of 15 | §3.5 |
| 7 | Does a key with `CKA_ALWAYS_AUTHENTICATE` sign? | **YES**: 6 of 6: `C_SignInit`, `C_Login(CKU_CONTEXT_SPECIFIC)`, and a raw `C_Sign` through the module's function table (`cryptoki-sys`) | §3.6 |
| 8 | Does `devices` run without hardware, without `pcscd`, and without `/sys/bus/usb`? | **YES**: clear messages, exit code 0; with `pcscd` running and no reader: "none found" | §3.7 |
| 9 | Does `report` leak neither the token label nor the PIN? | **YES** (the script checks) | §3.8 |
| 10 | Page → extension → host → SoftHSM2 in a real Chromium? | **YES**: `NM-E2E: PASS`, signature verified | §3.9 |

## 3. Evidence

Output of `linux/ci-linux.sh` (the provider text was shortened with `…` on some lines; the rest is literal;
there is no PIN or person's name: the test certificate is `SignLocal Test …`).

### 3.1 `list`

```text
> websign-probe list --no-known-modules --no-p11-kit --module /usr/lib/softhsm/libsofthsm2.so
 1. SignLocal Test ec-p384 | certificate | EC P-384 | until 2036-09-26 | pkcs11:libsofthsm2.so (SoftHSM v2; …; key hidden until login, software; PIN by app) | 256adb9aabc7534b
 2. SignLocal Test ec-p521 | certificate | EC P-521 | until 2036-09-26 | pkcs11:libsofthsm2.so (SoftHSM v2; …; key hidden until login, software; PIN by app) | 3e1a37449c5cdd7d
 3. SignLocal Test rsa-2048 | certificate | RSA-2048 | until 2036-09-26 | pkcs11:libsofthsm2.so (SoftHSM v2; …; key hidden until login, software; PIN by app) | 6f0071f15c22c1de
 4. SignLocal Test ec-p256 | certificate | EC P-256 | until 2036-09-26 | pkcs11:libsofthsm2.so (SoftHSM v2; …; key hidden until login, software; PIN by app) | d2e5ac9b7e671211
```

The full provider text is `SoftHSM v2 (SoftHSM project); slot "SoftHSM slot ID 0x…"; libsofthsm2.so; key hidden
until login`: token model and manufacturer, slot description, module file, and whether the key is visible without a PIN.
**Never the token label**, which on Brazilian and European cards usually carries the holder's name.

### 3.2 `sign --all --hash all --pss` (the module directly)

```text
SignLocal Test rsa-2048 | certificate | RSA-2048 | … | 6f0071f15c22c1de
   OK    SHA-256 RSASSA-PKCS1-v1_5 via C_Sign in 3 ms
   OK    SHA-256 RSASSA-PSS via C_Sign in 7 ms
   OK    SHA-384 RSASSA-PKCS1-v1_5 via C_Sign in 3 ms
   OK    SHA-384 RSASSA-PSS via C_Sign in 3 ms
   OK    SHA-512 RSASSA-PKCS1-v1_5 via C_Sign in 3 ms
   OK    SHA-512 RSASSA-PSS via C_Sign in 3 ms
SignLocal Test ec-p256 | … | d2e5ac9b7e671211
   OK    SHA-256 ECDSA via C_Sign in 2 ms
   OK    SHA-384 ECDSA via C_Sign in 2 ms
   OK    SHA-512 ECDSA via C_Sign in 2 ms
(… P-384 and P-521 identical: 3 OK each; total 15 OK, 0 FAIL, exit code 0)
```

Each signature is verified against the certificate by `probe-core` (RSA: PKCS#1 v1.5 over the `DigestInfo`;
PSS with MGF1 of the same hash and a salt of the digest length; ECDSA as `r ‖ s`). `CKM_RSA_PKCS` receives the
`DigestInfo` already assembled: the `CKM_SHA256_RSA_PKCS` mechanisms and similar are never used, because they would hash the
digest again.

### 3.3 Wrong PIN

```text
> websign-probe sign --cert 6f0071f15c22c1de --pin-env WRONG_PIN --no-known-modules --no-p11-kit --module …
   FAIL  SHA-256 RSASSA-PKCS1-v1_5: wrong PIN
(exit code 1)
```

Right afterwards, the token's `list` shows `…key hidden until login; PIN attempts running low…`: SoftHSM2 raised
`CKF_USER_PIN_COUNT_LOW`, which the interface uses for "few attempts left"
([ux.md](../ux.md), `pin.incorrect_low`). The next signature with the right PIN resets the counter.

### 3.4 User-registered p11-kit

A `~/.config/pkcs11/modules/websign-test.module` points to a copy of SoftHSM2; a second one points to a
file that does not exist:

```text
> websign-probe list --every-path --no-known-modules
warning: pkcs11:libsofthsm2-missing.so: module file not found: /tmp/websign-ci-linux.…/lib/libsofthsm2-missing.so
 1. SignLocal Test ec-p384 | … | pkcs11:libsofthsm2.so (SoftHSM v2; …) | 256adb9aabc7534b
    also via pkcs11:libsofthsm2-registered.so (SoftHSM v2; …)
```

### 3.5 The same certificate through two modules

```text
> websign-probe list --no-known-modules --no-p11-kit --module /usr/lib/softhsm/libsofthsm2.so --module /usr/lib/x86_64-linux-gnu/p11-kit-proxy.so
 1. SignLocal Test ec-p384 | certificate | EC P-384 | … | pkcs11:libsofthsm2.so (SoftHSM v2; …) | 256adb9aabc7534b
    (+1 other path(s); --every-path to show)
```

With `--every-path`: `also via pkcs11:p11-kit-proxy.so (…)`. Four certificates, not eight.
`p11-kit-proxy.so` alone (`--module …/p11-kit-proxy.so`) signs the same 15 cases. SoftHSM2 loaded
directly **and** through the proxy in the same process works: the second `C_Initialize` returns
`CKR_CRYPTOKI_ALREADY_INITIALIZED`, which is treated as success.

### 3.6 Key with `CKA_ALWAYS_AUTHENTICATE`

```text
> websign-probe sign --cert ee8c71007793ea12 --hash all --pss --pin-env WEBSIGN_PROBE_PIN …
   OK    SHA-256 RSASSA-PKCS1-v1_5 via C_Sign in 5 ms
   OK    SHA-256 RSASSA-PSS via C_Sign in 5 ms
   (… 6 of 6, one line per hash and algorithm)
```

`cryptoki` 0.12 only offers `C_Sign` together with its own `C_SignInit`, and PKCS#11 requires
`C_Login(CKU_CONTEXT_SPECIFIC)` **between** the two. That is why the kit does `C_SignInit` through `cryptoki`, the
context login, and a raw `C_Sign` obtained from the module's `C_GetFunctionList` (the only symbol the
specification requires a module to export). A single call, with a buffer for any key: a size query
beforehand would be a second `C_Sign` after the context login, which some modules count as the single use.
The script requires all 6 combinations.

### 3.7 `devices`

```text
> websign-probe devices
USB smart card readers and tokens:
  USB enumeration failed: /sys/bus/usb/devices/ not found (errno 2)

PC/SC readers:
  the PC/SC service is not running (start pcscd on Linux, or the Smart Card service on Windows)
(exit code 0)
```

With `pcscd` running and no reader, the second section says `none found`. `--json` carries the same fields
(`usb.devices`, `usb.hidden`, `readers.readers[].atr`, `problem`); the USB serial number is never read, and what
pcsc-lite appends to the reader name (`[interface] (serial number)`) is trimmed off.

### 3.8 `report` (excerpts)

```text
### Key sources
- `pkcs11:libsofthsm2.so`: opened
### Certificates
| # | Holder | Type | Key | Valid until | Source | Other paths |
| 1 | _hidden_ | certificate | EC P-384 | 2036-09-26 | pkcs11:libsofthsm2.so (SoftHSM v2 (SoftHSM project); slot "SoftHSM slot ID 0x…"; …) | — |
### Devices
_USB enumeration failed: /sys/bus/usb/devices/ not found (errno 2)_
_the PC/SC service is not running (start pcscd on Linux, or the Smart Card service on Windows)_
### Signatures
| 3 | `OK    SHA-256 RSASSA-PKCS1-v1_5 via C_Sign in 3 ms` |
```

### 3.9 End-to-end native messaging (Chromium, SoftHSM2)

```text
> node nm-e2e/run.mjs --probe …/release/websign-probe --expect-sign --chromium /opt/pw-browsers/chromium-1194/chrome-linux/chrome
  NM-E2E: PASS
  … event=request type=list … event=certificates count=4 warnings=0
  … event=sign hash=SHA-256 algorithm=RSASSA-PKCS1-v1_5 digest_bytes=32
  … event=signed api=C_Sign verified=true
```

The host found SoftHSM2 without any `--module`: through the known-paths list and the p11-kit registration
(`SOFTHSM2_CONF` and `WEBSIGN_PROBE_PIN` reach the host through the browser's environment). Since the host never
receives the PIN from the browser, this test uses the variable only because the kit has no PIN window.

## 4. What the proof fixed in the code

| Decision | Why |
|---|---|
| **`list` never asks for a PIN**; "has a key" = a `CKO_PRIVATE_KEY` with the same `CKA_ID` exists and is visible without login; if the token lists **no** key at all and requires login (`CKF_LOGIN_REQUIRED`), every certificate counts; if it does not require login, there is no key | SoftHSM2 and many cards hide the private key until the PIN. The two-step rule avoids listing CA certificates when the token shows its keys; `provider` says which rule applied (`key visible` / `key hidden until login`) |
| `provider` = token model and manufacturer + slot description + module file; **no label** and no USB serial number that pcsc-lite puts in parentheses in the reader name (the slot description is usually that name; `devices` trims the same part) | Public repository; the label often carries the holder's name, and the serial number identifies the device |
| `hardware` = `Some(false)` only if the name (model, manufacturer, slot description) says "softhsm"/"software"; otherwise `Some(true)` | `CKF_HW_SLOT` does not decide: SoftHSM2 leaves it off, but so do real card modules |
| One read-only session per signature, `C_Login(CKU_USER)` with the PIN straight from the `SecretString` (no copy), logout (also when signing fails), and close at the end | No PIN cached in the probe. The app, under [decision D5](../plan.md), will keep the session open: this changes `Keystore`, not the `list`/`sign` here |
| Slot changes? Find the certificate again by its **DER** (the locator is only the first guess) and the key by `CKA_ID` | Slot numbering changes with the order of the readers |
| Mechanism: RSA v1.5 = `CKM_RSA_PKCS` over `DigestInfo`; PSS = `CKM_RSA_PKCS_PSS` with hash, MGF1, and `sLen` = digest length; ECDSA = `CKM_ECDSA` (accepts `r ‖ s`; converts DER; rejects any other size). `C_GetMechanismInfo` first: `Unsupported` if the token cannot sign with it | Correct signature over the digest, never over the hash of the hash |
| Errors: `CKR_PIN_INCORRECT`/`PIN_INVALID`/`PIN_LEN_RANGE` → `WrongPin`; `PIN_LOCKED` → `PinLocked`; `FUNCTION_CANCELED`/`CANCEL`/`FUNCTION_REJECTED` → `Cancelled`; `USER_NOT_LOGGED_IN` without a PIN → `PinRequired`; the rest → `Native { api, code, message }` with the `CKR_*` name | The interface reacts to the types; support needs the code |
| Single load per file (inode) and never `C_Finalize`; a load failure = `SourceFailure`, never aborts | `C_Initialize` can take 1 s; unloading a library with threads brings the process down |
| `provider` warns `PIN attempts running low` / `last PIN attempt` / `PIN locked` when the token reports it | The remaining-attempts UX uses these flags. A field is missing in `KeystoreError::WrongPin` to carry them to the window (`model.rs` belongs to the orchestrator) |

## 5. What remains unproven

| Item | Why it is missing | How to prove it |
|---|---|---|
| **Real token** through p11-kit or a known path (SafeNet 5110, SafeSign, ePass2003, Watchdata, DXToken) | no hardware | script in §6 |
| **`CKA_ALWAYS_AUTHENTICATE` with a real card** (Cartão de Cidadão, DNIe, Estonian card) | proven only on SoftHSM2 (§3.6); real cards may open their own PIN dialog on the context login | Real Cartão de Cidadão/DNIe, script in §6 |
| **PIN pad** (`CKF_PROTECTED_AUTHENTICATION_PATH`) | SoftHSM2 has none; the code passes `NULL` to `C_Login` and `pin: App { protected_path: true }` to the caller | reader with a keypad (GemPCPinpad `08e6:3478`) |
| Token with `CKF_CLOCK_ON_TOKEN` and an invalid time | `cryptoki` rejects `C_GetTokenInfo` and the slot vanishes from the list | real token; if it happens, read the raw `CK_TOKEN_INFO` |
| `pcscd` with a reader and card (real ATR, `devices` with USB) | the container has no `/sys/bus/usb`; I tested `pcscd` without a reader | `websign-probe devices --all-usb` on a machine with a reader |
| Modules that open their own PIN dialog during `C_Sign` | no token | Real Cartão de Cidadão/DNIe |
| Fedora/Arch and `aarch64` | only Ubuntu 24.04 x86_64; the `lib64` paths are convention | CI in a Fedora container |
| `libpcsclite.so.1` is a dynamic dependency of the binary (`ldd`) | `devices` uses `pcsc`; without pcsc-lite the host does not even start, not even for PKCS#11 | `Depends: libpcsclite1` in the `.deb` and `pcsc-lite-libs` in the `.rpm`, or load the library at runtime |

## 6. Script with a real token (Linux)

1. `sudo apt install pcscd opensc p11-kit` and the vendor's middleware ([tokens.md](../research/tokens.md)); `sudo systemctl start pcscd`.
2. Download `websign-probe` from the CI `report-linux` artifact (or build it).
3. `websign-probe devices --all-usb` → note `VID:PID` and ATR in the matrix.
4. `websign-probe list` → does the token appear? through which module? (`--every-path` shows all of them).
5. `PIN=… websign-probe sign --cert <thumbprint> --hash all --pss --pin-env PIN` (the PIN comes from a
   variable **only in this test**; without `--pin-env` the probe asks in the terminal). Careful: each wrong PIN uses up
   an attempt.
6. `websign-probe report --run-signatures --all --hash all --pss --pin-env PIN --out report.md` and paste it here
   (the report contains no name, CPF/CNPJ, or serial number).

| Token / card | Module (file) | Registered in p11-kit? | `list` | RSA v1.5 | RSA-PSS | ECDSA | Wrong PIN | Notes |
|---|---|---|---|---|---|---|---|---|
| _SafeNet eToken 5110_ | | | | | | | | |
| _SafeSign / StarSign_ | | | | | | | | |
| _ePass2003_ | | | | | | | | |
| _Watchdata ProxKey_ | | | | | | | | |
| _DXToken_ | | | | | | | | |
| _Cartão de Cidadão_ | | | | | | | | `CKA_ALWAYS_AUTHENTICATE` |

## 7. Native messaging

| Browser | Result | Evidence | Missing |
|---|---|---|---|
| Chromium (deb/Playwright) | **YES**: ping, digest validation, and a SoftHSM2 signature verified by the host (`verified=true`) | `NM-E2E: PASS` in `ci-linux.sh` (in this container); host log in §3.9 | n/a |
| Google Chrome, Edge, Brave, Vivaldi (deb) | **Probably YES**: same mechanism as Chromium; `register` writes into each `~/.config/<browser>/NativeMessagingHosts/` | [research/native-messaging.md](../research/native-messaging.md) §3 | Run the script below with the browser installed |
| Chromium Snap | **Probably YES**: the Snap reads `~/snap/chromium/common/chromium/NativeMessagingHosts/` (`register` writes there) | same, §3.3 | Ubuntu with Snap |
| Firefox (deb/rpm) | **Probably YES**: manifest in `~/.mozilla/native-messaging-hosts/`, `allowed_extensions` with the fixed ID | same, §3.1 | Run the script below (Playwright does not load extensions in Firefox) |
| **Firefox Snap (Ubuntu)** | **No proof yet.** Confined Firefox does not read manifests: it asks the `org.freedesktop.portal.WebExtensions` portal (Ubuntu patch), which starts the host **outside** the Snap with the portal's environment | same, §3.4 | Ubuntu 24.04 with a graphical interface: does the egui window open with the portal's environment? Does the portal's "allow" dialog appear once? |

**Consequences for the product** (already in the [plan](../plan.md)):

- The `.deb`/`.rpm` package installs the **system** manifests (`/etc/opt/chrome/native-messaging-hosts/`,
  `/etc/chromium/native-messaging-hosts/`, `/usr/lib/mozilla/native-messaging-hosts/` …), with an absolute `path`.
  This is what the Firefox Snap portal consults.
- The Confirmation window cannot depend on variables inherited from the browser. When the host is started
  by the portal, the graphical environment comes from the session.
- The portal's replacement (`org.freedesktop.NativeMessagingProxy`, Firefox 157+) requires no change in the host;
  follow up when Ubuntu enables it in the stable Snap.

**Script (Ubuntu 24.04 with a graphical interface, default Firefox Snap):**

1. `./websign-probe register --browser firefox` and, for the portal, also the system manifest:
   `sudo install -Dm644 ~/.mozilla/native-messaging-hosts/dev.websign.host.json /usr/lib/mozilla/native-messaging-hosts/dev.websign.host.json`.
2. `about:debugging` → "Load Temporary Add-on" → `kit/extension/manifest.json`.
3. Open `kit/nm-e2e/page.html` served at `http://localhost:8000` (`python3 -m http.server` in the folder).
4. Note: did the portal dialog appear? Did the page show `pong`? Does the `/tmp/websign-probe-host.log` log
   have `family=firefox`? With the token, does `sign` return `verified=true`?

## 8. Decision

- **PKCS#11 through p11-kit and known paths: yes**, with software keys. Discovery, deduplication by file and by
  certificate, and RSA/PSS/ECDSA signatures are proven; what remains is a real
  token (§6) and `CKA_ALWAYS_AUTHENTICATE` with a real card (§5).
- **Dependency adopted in the kit:** `cryptoki-sys = "0.5"` (the same version `cryptoki` 0.12 uses) for the
  raw `C_Sign` of qualified-signature keys (§3.6). It goes away when `cryptoki` offers `C_Sign` without
  `C_SignInit`.
- **Linux package:** `libpcsclite1` required (§5) and `p11-kit` recommended.
