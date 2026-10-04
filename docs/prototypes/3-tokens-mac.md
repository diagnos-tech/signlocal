# Proof 3: Tokens on Mac: CryptoTokenKit or PKCS#11?

**Status:** real token missing · **Partial result:** PKCS#11 does **not** load inside the sandbox (CI) → a PKCS#11-only token requires the complement ·
**Decision:** _depends on the matrix below_; the criterion is in [§1](#1-criterion-does-the-dmg-complement-exist)

## Goal

1. Which middlewares expose the token to **CryptoTokenKit** (CTK), that is, make the keys appear
   in the keychain for any app, including the Mac App Store one, without PKCS#11?
2. Does a **PKCS#11** module load and sign **inside the Mac App Store sandbox**?
3. With that: does the **`.dmg` complement** (Developer ID + notarization + Sparkle, outside the sandbox,
   called by the store app) need to exist?

On the app side, both paths are already in the kit: `macos:ctk` (query with
`kSecAttrAccessGroupToken`, see [proof 2](2-mac.md#1-how-the-kit-proves-it)) and `pkcs11:<module>` (list
of known paths, p11-kit, and `--module`). The same certificate seen through both paths is
deduplicated by thumbprint, preferring the system one.

## 1. Criterion: does the `.dmg` complement exist?

The complement exists **if and only if** there is at least one **relevant** middleware for which:

1. the token does **not** appear in `macos:ctk` (token inserted, current middleware installed, macOS
   supported by the vendor); **and**
2. PKCS#11 does **not** work inside the store app, because it fails in the sandbox (`dlopen`,
   `C_Initialize`, `C_Login`, or `C_Sign`; see §3) **or** because App Review rejects loading an
   external module (guideline 2.5.2: the app may not execute code that did not ship in the bundle).

"Relevant" = used by Diagnos customers (ICP-Brasil A3) or by the European cards on the list.
`TODO(gustavo)`: close the list using each token's market share among physicians.

Consequences:

- All relevant ones go through CTK → **no complement**; PKCS#11 on Mac remains only for
  diagnostics ("add module").
- Some only work through PKCS#11 **and** PKCS#11 works in the sandbox **and** Review accepts it →
  **no complement**; the store app loads the module.
- Otherwise → **complement**, and `devices.json` marks `macos.cryptotokenkit: false` for
  that device (this is what triggers the suggestion in the interface, see `docs/ux.md` §7).

Even with CTK working, it is worth measuring whether the CTK PIN experience is acceptable to the physician (system dialog
on every signature or per-session cache, depending on the driver).

## 2. Middleware matrix

Legend: **YES** = documented by the vendor; **probable** = public indications; **?** = no
information; "Proof" = result of script §4 on a real Mac (to be filled in).

| Middleware (tokens) | Exposes via CTK? | PKCS#11 module on macOS | Source | Proof |
|---|---|---|---|---|
| **SafeSign IC** (A.E.T. Europe): JCOP and G+D tokens and cards used in ICP-Brasil | **YES** since 4.0: *Smart Card Extension* `aetsce.appex` inside `tokenadmin.app/Contents/PlugIns` ("used for Apple (native) applications, such as Safari and Mail"; tested with Chrome 111 on macOS 13.2) | `/Applications/tokenadmin.app/Contents/Frameworks/libaetpkss.dylib` (4.0); older versions: `/usr/local/lib/libaetpkss.dylib` (to be confirmed) | *SafeSign IC Standard Version 4.0 for macOS Release Document* (Mar 2023, KPN mirror); notes 3.5/3.6/3.7/4.1/4.2 on UZI-register | |
| **SafeNet Authentication Client** (Thales): eToken 5110/5110+/5110 CC, IDPrime | **probable** (reports of the token's certificates showing up in the keychain on recent macOS; I found no Thales document citing CTK) | `/usr/local/lib/libeTPkcs11.dylib` | DigiCert KB (module for Acrobat); SAC 10.9 for Mac announcement (Jan 2025, macOS 15) | |
| **OpenSC**: ePass2003 (`epass2003` driver), DNIe, PIV, many cards | **YES** in the official package: `OpenSCToken.app` (extension `org.opensc-project.mac.opensctoken.OpenSCTokenApp.OpenSCToken`). The Homebrew formula does not ship the CTK token (to be confirmed) | `/Library/OpenSC/lib/opensc-pkcs11.so` (and a link at `/usr/local/lib/opensc-pkcs11.so`) | `MacOSX/build` and `MacOSX/opensc-uninstall` in the OpenSC repository | |
| **Feitian ePass2003** with its own middleware (distributed by CAs) | ? | ? (to be confirmed in the installer) | n/a | |
| **Watchdata** (WD tokens used by Brazilian CAs) | ? | Indian variant (ProxKey): `/usr/local/lib/wdProxKeyUsbKeyTool/libwdpkcs_Proxkey.dylib`; Brazilian variant: ? | Guides from Indian resellers (Acrobat on Mac) | |
| **G+D StarSign** (StarSign Crypto USB Token S) | **YES via SafeSign** (the token is on SafeSign IC 4.0 for macOS's supported list); G+D's own middleware: ? | via SafeSign: `libaetpkss.dylib` | SafeSign 4.0 Release Document (section 7) | |
| **Cartão de Cidadão PT** (Autenticação.gov) | **YES** since 3.11.0: `PteidToken` module ("implements the CryptoTokenKit framework") | `/usr/local/lib/libpteidpkcs11.dylib` | Autenticação.gov User Manual. Cards issued since Jun 2024 use **ECDSA** | |
| **DNIe ES** (Policía Nacional, `libpkcs11-dnie` 1.6.8) | **NO** in the official package: PKCS#11 only (the `.pkg` ships no `.appex`); alternative: OpenSC, which has a `dnie` driver and the `OpenSCToken` | `/Library/Libpkcs11-dnie/lib/libpkcs11-dnie.so` | Payload of `libpkcs11-dnie-1.6.8_arm.pkg` inspected (install-location `/Library`; ships `DialogSign.app`) | |
| **Apple PIV** (`com.apple.pivtoken`, built in): YubiKey and PIV cards | **YES** (native) | not needed | macOS | |

Notes that weigh on the decision:

- **SafeSign requires the host app's CTK entitlement even through PKCS#11.** The 4.0 document
  says: *"If an application (based on PKCS #11) does not have CTK entitlement, the SafeSign PKCS #11
  Library that is loaded by that application does not have this entitlement either"*; there is a
  PC/SC workaround (`EnableMacOSXPCSCLayerFallback`, on by default, in
  `~/Library/Application Support/safesign/registry`). The store app declares
  `com.apple.security.smartcard`, so the library's own CTK path should work; the
  workaround reads a file in `$HOME`, which in the sandbox is the container (to be confirmed).
- **DNIe opens a helper app** (`DialogSign.app`) for the PIN/confirmation. Inside the sandbox, starting
  another executable inherits the sandbox and may be denied: a specific risk to measure.
- **Brazil:** CAs distribute their own (and sometimes old) versions of SafeSign and SAC. What
  matters is the version the physician receives from the CA, not the vendor's latest; note the origin of the
  installer in every proof.
- **Cheap test of the CTK path without middleware:** a YubiKey 5 with a PIV certificate
  (`ykman piv keys generate` + `ykman piv certificates generate`) shows up in `macos:ctk` through Apple's
  native driver. It proves the kit's code and the sandbox before the Brazilian tokens are at hand.

## 3. PKCS#11 inside the sandbox

`kit/macos/sandbox/sandbox-test.sh` (CI) measures with Homebrew's SoftHSM2, with the token **inside the
container** (`~/Library/Containers/dev.websign.app/Data/websign-softhsm`, via `SOFTHSM2_CONF`):

| ID | What it measures | Why |
|---|---|---|
| `pkcs11-setup` | Preparing the token in the container from the outside | On macOS 14+ the system protects other apps' containers; if it fails, the script redoes it outside the container (then only `dlopen` is conclusive) |
| `pkcs11-load:store` | `list --module libsofthsm2.so` in the sandbox, signed like the store build (without *hardened runtime*) | `dlopen` of a library outside the bundle (`/opt/homebrew`) and `C_Initialize` reading the configuration |
| `pkcs11-sign:store` | `sign` with `C_Login` + `C_Sign` | The full flow with the PIN through the app |
| `pkcs11-load:hardened` / `pkcs11-sign:hardened` | The same with `--options runtime` | The Developer ID complement needs *hardened runtime*; without `com.apple.security.cs.disable-library-validation`, library validation should refuse modules from another Team ID: expected **NO**, confirming the complement needs that entitlement |

SoftHSM proves the sandbox rules for files and code, but not PC/SC: a real token module
still talks to the reader (through `com.apple.security.smartcard`), writes logs, and reads configuration in
its own places. Only script §4 with the real token settles the question.

Result in CI (run [36629289998](https://github.com/diagnos-tech/signlocal/actions/runs/36629289998),
macOS 26.6.2 arm64):

| ID | Result | Evidence |
|---|---|---|
| `pkcs11-setup` | YES | SoftHSM2 token created inside the container |
| `pkcs11-load:store` | **NO** | `dlopen(/opt/homebrew/opt/softhsm/lib/softhsm/libsofthsm2.so)`: `file system sandbox blocked open()` |
| `pkcs11-sign:store` | **NO** | consequence of the previous one |
| `pkcs11-load:hardened` / `pkcs11-sign:hardened` | **NO** | same denial |

Kernel log: `Sandbox: websign-probe(…) deny(1) file-read-data /opt/homebrew/Cellar/softhsm/2.7.0/lib/softhsm/libsofthsm2.so`.

**Reading:** the store app **cannot even open** a PKCS#11 module installed outside its own
bundle. The only way out inside the store would be a `temporary-exception.files.absolute-path.read-only`
exception for each vendor folder (`/usr/local/lib`, `/Library/…`), which App Review tends to reject under
guideline 2.5.2 (executing code that did not ship in the bundle). So **every token that only works through PKCS#11
on Mac needs the complement** (§1). With what the matrix already shows (§2), that includes the **DNIe**.
The question still to be answered with real tokens is which middlewares do **not** go through CryptoTokenKit.

## 4. Test script for Gustavo

One block per token. Note: token model, ATR, middleware + version + **where it came from** (vendor's
or the CA's site), macOS version and chip.

Preparation (once): `cd docs/prototypes/kit && cargo build --release -p websign-probe &&
export PROBE_EXE=$PWD/target/release/websign-probe`.

1. **System, without the probe** (token inserted):
   ```sh
   system_profiler SPSmartCardsDataType      # readers, tokens, available CTK drivers
   pluginkit -mAvvv -p com.apple.ctk-tokens   # installed CTK extensions
   security list-smartcards                   # token IDs present (part before ":" = driver)
   sc_auth identities                         # identities the system sees on the card
   ```
2. **Device:** `"$PROBE_EXE" devices` (VID:PID, reader, ATR) → goes into `devices.json`.
3. **All paths:** `"$PROBE_EXE" list --every-path` (if the module is not in the known
   list: `--module <path to the dylib>`). Expected with CTK: line `macos:ctk (<driver>,
   hardware; PIN by OS)`; with PKCS#11: `also via pkcs11:<module>`.
4. **Sign via CTK:** `"$PROBE_EXE" sign --cert <16 hex> --hash all --pss`. The PIN is asked by the
   system/driver. Also test **Cancel** (expected: "cancelled by the user"). Do **not** test a wrong
   PIN more than once: the token locks.
5. **Sign via PKCS#11:** `read -rs WEBSIGN_PIN && export WEBSIGN_PIN` and
   `"$PROBE_EXE" sign --cert <16 hex> --every-path --pin-env WEBSIGN_PIN --hash sha256`.
6. **Inside the sandbox** (what the store app would manage):
   ```sh
   bash macos/sandbox/run-sandboxed.sh list --every-path
   bash macos/sandbox/run-sandboxed.sh sign --cert <16 hex> --hash sha256
   bash macos/sandbox/run-sandboxed.sh list --module <dylib> --no-known-modules --no-p11-kit
   bash macos/sandbox/run-sandboxed.sh sign --cert <16 hex> --every-path --pin-env WEBSIGN_PIN --module <dylib>
   HARDENED=1 bash macos/sandbox/run-sandboxed.sh list --module <dylib>   # like the complement
   ```
   The script prints the sandbox denials at the end; copy all of them.
7. **Through the browser:** step 6 of [proof 2](2-mac.md#7-script-for-gustavo-real-mac) with this
   token (Chrome starting the sandboxed host).
8. **Report:** `"$PROBE_EXE" report --run-signatures --cert <fp> --every-path --hash all --pss --out token-<model>.md`
   (no names, CPF, or serial numbers) and fill in the row of the §2 matrix.

## 5. References

- SafeSign IC Standard 4.0 for macOS, Release Document (A.E.T. Europe, Mar 2023), mirrored at
  `certificaat.kpn.com/files/drivers/SafeSign/`; versions 3.5–4.2 at `uziregister.nl`.
- Thales: SAC 10.8 R2 and 10.9 for Mac announcements at `data-protection-updates.gemalto.com`;
  DigiCert KB "SafeNet hardware token not detected in Adobe Reader on Mac OS".
- OpenSC: `MacOSX/build`, `MacOSX/opensc-uninstall` (github.com/OpenSC/OpenSC).
- Autenticação.gov: User Manual (amagovpt.github.io/docs.autenticacao.gov).
- DNIe: dnielectronico.es, downloads area → "Software para Sistemas MacOS" (1.6.8).
- Apple: `com.apple.security.smartcard` (required for `TKSmartCardSlotManager` and for PC/SC in the
  sandbox); `kSecAttrAccessGroupToken` (granted by default to every app).
