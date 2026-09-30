# Compatibility

What the app can sign with, through which key store, and why some things are
out. The manual matrix of real tokens and browsers is
[`docs/compatibility.md`](../compatibility.md).

## Algorithms

Hashes: **SHA-256, SHA-384, SHA-512** only (SHA-1 is broken; SHA-3 is not
offered by CNG, CryptoTokenKit or common tokens).

| Signature | Key | Encoding returned | Windows CNG | Windows CAPI (legacy CSP) | macOS Keychain/CTK | PKCS#11 |
|---|---|---|---|---|---|---|
| RSASSA-PKCS1-v1_5 | RSA 1024–16384 | modulus-length block | `NCryptSignHash` + `BCRYPT_PAD_PKCS1` | `CryptSignHash` (bytes reversed); `PROV_RSA_FULL` keys reopened in the AES CSP | `RSASignatureDigestPKCS1v15SHA*` | `CKM_RSA_PKCS` over the `DigestInfo` |
| RSASSA-PSS (MGF1 same hash, salt = digest length) | RSA | block | `BCRYPT_PAD_PSS` | only via the CNG bridge (`PREFER`, D9) for Microsoft's CSPs; not offered for third-party CSPs or after the fallback to `ALLOW` | `RSASignatureDigestPSSSHA*` (salt = digest length, verified in Apple's sources) when `SecKeyIsAlgorithmSupported` says so | `CKM_RSA_PKCS_PSS`, offered only when the token lists it with `CKF_SIGN` |
| ECDSA P-256 / P-384 / P-521 | EC | raw `r‖s` | native `r‖s` | — (CAPI has no ECC) | DER → converted | `CKM_ECDSA` (raw; DER converted) |
| ECDSA brainpoolP256r1 / P384r1 / P512r1 | EC | raw `r‖s` | if the KSP supports the curve | — | — (the Security framework has no brainpool curves; use the token's PKCS#11 module) | if the module does |

The app never asks a key store to hash (`CKM_SHA256_RSA_PKCS` would hash the
digest again). Every result is verified before it is returned (brainpoolP512r1
excepted: no maintained pure-Rust verifier; the app logs that it skipped).

**Brainpool.** European national cards (German eID-based QES cards, some
Spanish and Austrian cards) use brainpool curves. The `bp256`/`bp384` 0.14
crates (RustCrypto, same generation as `p256` 0.14) verify P256r1/P384r1;
nothing maintained verifies P512r1. Summaries recognize all three
(`websign-core` SPEC §4.1).

**Why EdDSA is out.** Pure Ed25519/Ed448 sign the *message*, not a hash
(RFC 8032): the key store must see the whole signed attributes, which breaks
the hash-only design (decision 1) and the verification-code UX. HashEdDSA
(Ed25519ph) exists but no OS key store or common token offers it, and PAdES
profiles do not use it. EdDSA certificates are listed as hidden
(`UnsupportedKey`) in diagnostics.

## Key stores

| OS | Store | What it reaches | PIN |
|---|---|---|---|
| Windows | `CurrentUser\MY` via CNG (`NCrypt*`) | minidrivers, software KSP, TPM ("Platform Crypto Provider"), virtual smart cards, imported A1 | OS or middleware dialog, owned by our window (`NCRYPT_WINDOW_HANDLE_PROPERTY`) |
| Windows | same store via CAPI (`CryptAcquireCertificatePrivateKey`) | legacy CSPs (older tokens), A1 imported into `PROV_RSA_FULL` | CSP dialog (`PP_CLIENT_HWND`) |
| macOS | Keychain (`macos:keychain`) | imported A1 (`.p12`) | keychain access dialog |
| macOS | CryptoTokenKit (`macos:ctk`) | tokens whose middleware ships a CTK extension (SafeSign ≥ 4, OpenSC, Autenticação.gov, Apple PIV) | driver dialog |
| all | PKCS#11 modules | p11-kit registrations, known vendor paths (`crates/websign-keystores/src/pkcs11/known_paths`), user-added drivers | our PIN field (or the reader keypad) |

The same certificate seen by the OS and by a PKCS#11 module is shown once,
through the OS (decision 2); the module path remains as an alternate ("Try
through the token driver"). PKCS#11 sessions stay logged in until the token
leaves or the connection idles (D5), except `CKA_ALWAYS_AUTHENTICATE` keys.

## Devices and detection

The app never talks to a chip. Hardware detection only helps:

- USB: VID:PID of CCID-class devices and of models in `devices.json`
  (`nusb`); serial numbers are never read.
- PC/SC: readers and ATRs (`pcsc`), with live insert/remove events that
  invalidate the certificate cache.
- `devices.json` (CC0) maps VID:PID/ATR → model → driver download and
  PKCS#11 path per OS, and flags macOS models without CryptoTokenKit.
- A detected device without certificates becomes a "possible certificate"
  hint with the right driver link (`docs/ux.md` §6); a missing entry falls
  back to generic advice. Hints never block anything.

## Browsers

| Browser | Windows | macOS | Linux |
|---|---|---|---|
| Chrome, Edge, Brave, Chromium, Vivaldi, Opera | HKCU keys (proof 1) | per-user manifests (proof 2) | per-user and system manifests (proof 4); Chromium Snap/Flatpak folders |
| Firefox ≥ 121 | HKCU key | per-user manifest | per-user and system manifests; Snap via the WebExtensions portal (system manifest) |
| Safari 17+ | — | macOS 13+: the app extension inside `WebeSign.app` relays to a bundled host (`safari/SPEC.md`); nothing to register. Unsigned builds need **Allow Unsigned Extensions**; PKCS#11-only tokens may not work (below). TODO(gustavo): Developer ID signing, notarization, Mac App Store | — |

**Safari and PKCS#11.** Safari starts no native host: the host runs as the
app extension's copy, inside its sandbox (`safari/SPEC.md` §4). Keychain
identities (imported certificates) sign inside that sandbox (proven in CI,
[proof 2](../prototypes/2-mac.md) §5), and the CryptoTokenKit query runs there
without an extra entitlement (proven; a real CTK token is still to be tried).
Loading a PKCS#11 module inside a sandbox was refused in the same proof
(`deny file-read-data` on SoftHSM2 under `/opt/homebrew/Cellar`). The appex
grants read-only access to `/Applications`, `/Library`, `/usr/local/lib` and
`/opt/homebrew/lib`, but no real vendor module has been tried through it, and
modules resolved elsewhere (Homebrew's `lib/` links into `Cellar/`) stay
blocked. So a token reachable only through PKCS#11 may not work from Safari;
the same token works from Chrome, Edge, Brave, Opera or Firefox on the same Mac,
whose host is not sandboxed.

## Operating systems

Windows 10 22H2+ and 11 (x64, arm64); macOS 13+ (universal;
`LSMinimumSystemVersion` in `packaging/macos/Info.plist.in`); Linux with
glibc ≥ 2.34 (release builds run in Rocky Linux 9): Ubuntu 22.04/24.04,
Debian 12, Fedora 42, Rocky/RHEL 9, Arch (best effort), amd64 and arm64. Requirements: PC/SC (`pcscd`, `libpcsclite`)
on Linux; any GPU or a software renderer (WARP, llvmpipe, Apple software).
