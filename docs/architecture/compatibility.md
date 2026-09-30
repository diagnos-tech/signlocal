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
| RSASSA-PSS (MGF1 same hash, salt = digest length) | RSA | block | `BCRYPT_PAD_PSS` | only via the CNG bridge (`PREFER`, D9) | `RSASignatureDigestPSSSHA*` (salt = digest length, verified in Apple's sources) | `CKM_RSA_PKCS_PSS` |
| ECDSA P-256 / P-384 / P-521 | EC | raw `r‖s` | native `r‖s` | — (CAPI has no ECC) | DER → converted | `CKM_ECDSA` (raw; DER converted) |
| ECDSA brainpoolP256r1 / P384r1 / P512r1 | EC | raw `r‖s` | if the KSP supports the curve | — | if the token driver does | if the module does |

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
| Safari | — | store channel: appex relay (later) | — |

## Operating systems

Windows 10 22H2+ and 11 (x64, arm64); macOS 13+ (universal); Linux with
glibc ≥ 2.31: Ubuntu 22.04/24.04, Debian 12, Fedora 42, Rocky/RHEL 9, Arch
(best effort), amd64 and arm64. Requirements: PC/SC (`pcscd`, `libpcsclite`)
on Linux; any GPU or a software renderer (WARP, llvmpipe, Apple software).
