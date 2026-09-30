# websign-keystores — specification

Every place a signing key can live, behind [`Keystore`](src/lib.rs). The
adapters (`pkcs11/`, `windows/`, `macos/`), `model.rs` and `inventory.rs` are
**promoted from the Phase-0 kit** (`probe/src/keystores`), where CI proved them
with software keys on all three systems (proofs 1–4 in
`docs/prototypes/`). Sections marked NEW are to be implemented; the contract
suite (§7) is the acceptance test of every adapter.

## 1. Rules for every adapter

1. `list` never prompts, never logs in, never touches a card beyond reading
   public objects and metadata.
2. `list` reports each certificate with a private key once per source, with
   `provider` text that never contains a token label, key container name or
   serial number (they often carry the holder's name).
3. `sign` receives a digest of exactly `hash.digest_len()` bytes and returns
   the final format: RSA block, or ECDSA raw `r‖s` (DER converted with
   `websign_core::ecdsa::der_to_raw` and the curve from the certificate).
4. Never ask a store to hash: PKCS#1 v1.5 over `DigestInfo` (`CKM_RSA_PKCS`),
   CNG/CAPI/SecKey "digest" algorithms.
5. PSS: MGF1 with the same hash, salt = digest length.
6. Errors map to `KeystoreError`: cancel → `Cancelled`; wrong PIN →
   `WrongPin`; blocked → `PinLocked`; missing PIN → `PinRequired`; card gone
   → `TokenRemoved`; unsupported mechanism → `Unsupported`; key vanished →
   `NotFound`; anything else → `Native { api, code, message }` with the
   symbolic name (`CKR_DEVICE_ERROR`, `NTE_BAD_KEYSET`, `errSecAuthFailed`).
7. `unsafe` only here, with `// SAFETY:`; every handle in an RAII type.
8. Nothing is `Send`; the host confines the hub to one thread.

## 2. Model (promoted, extended)

`FoundKey` gains `device: Option<DeviceLink>` (NEW). `PinState` (NEW) is
what `pin_state` reports. `KeystoreError::TokenRemoved` (NEW).
`NcryptPreference` default is `Prefer` (D9).

## 3. PKCS#11 (`pkcs11/`)

Promoted behavior (proof 4): discovery from p11-kit registrations (user and
system `modules/*.module`, `enable-in`/`disable-in` honored), known vendor
paths per OS, and `extra_modules`; one load per file (inode), never
`C_Finalize`; `CKR_CRYPTOKI_ALREADY_INITIALIZED` is success; "has key" rule
(visible private key with the same `CKA_ID`, or none visible and
`CKF_LOGIN_REQUIRED`); locator = slot + `CKA_ID`, certificate found again by
DER when slots move; mechanisms checked with `C_GetMechanismInfo`;
`CKA_ALWAYS_AUTHENTICATE` handled with a raw `C_Sign` after
`C_Login(CKU_CONTEXT_SPECIFIC)`.

### 3.1 Sessions (NEW, D5)

- The first successful `C_Login(CKU_USER)` on a token keeps that session
  open and logged in; later signatures on any key of the token reuse it and
  need no PIN (`pin_state().unlocked = true`, the UI shows "Token unlocked for
  this session").
- The session ends on `end_sessions()`, on `CKR_SESSION_HANDLE_INVALID`/
  `CKR_DEVICE_REMOVED`/`CKR_TOKEN_NOT_PRESENT` (then `TokenRemoved`), and when
  the process exits.
- `CKA_ALWAYS_AUTHENTICATE` keys ask for the PIN on every signature even in
  an unlocked session (`PinState.always_authenticate`).
- The PIN arrives as `SecretString`; it is passed to `C_Login` without
  copies and dropped (zeroized) right after.

### 3.2 `pin_state` (NEW)

From `C_GetTokenInfo` without login: `length = (ulMinPinLen, ulMaxPinLen)`
when both are non-zero and min ≤ max; the `CKF_USER_PIN_*` flags;
`unlocked` from §3.1; `always_authenticate` from the key object when
readable without login, else `false`.

### 3.3 Device link (NEW)

`DeviceLink::Pkcs11Token { model, manufacturer }` from `CK_TOKEN_INFO`
(trimmed, blank → no link); when the slot description is a PC/SC reader name,
also usable as `Reader { name: anonymous_reader_name(desc) }` — the adapter
prefers `Reader` when the slot has `CKF_REMOVABLE_DEVICE` and the description
matches a reader seen by PC/SC.

### 3.4 `chain` (NEW)

CA certificates stored on the same token (`CKO_CERTIFICATE`, `CKA_CERTIFICATE_CATEGORY`
authority or `basicConstraints cA`), ordered issuer-first by matching
subject/issuer DN, leaf excluded, at most 8.

## 4. Windows (`windows/`)

Promoted behavior (proof 1): `CurrentUser\MY` read-only with auto-resync;
provider metadata without opening keys; hardware decided by asking the
provider (`NCRYPT_IMPL_TYPE_PROPERTY`, `PP_IMPTYPE`), name heuristics as
fallback; key acquired once and cached per locator; A1 keys in
`PROV_RSA_FULL` reopened in the AES CSP; PIN dialog owner via
`CRYPT_ACQUIRE_WINDOW_HANDLE_FLAG`, `NCRYPT_WINDOW_HANDLE_PROPERTY`,
`PP_CLIENT_HWND`; one sign call with a 16384-bit buffer.

### 4.1 Owner window

`SignRequest.parent_window` is the confirmation window's HWND (the MV3
browser passes `--parent-window=0`). Without it, the console window (hidden
for browser-started hosts).

### 4.2 D9 fallback (NEW)

Default `Prefer`. When acquiring or signing with `Prefer` fails for a key
whose provider is a CSP (`dwProvType != 0`) with `NTE_BAD_PROVIDER`,
`NTE_PROV_TYPE_NOT_DEF`, `NTE_NOT_SUPPORTED` or `SCARD_E_UNSUPPORTED_FEATURE`,
retry once with `Allow` and remember `Allow` for that locator. PSS through
`Allow` on a CSP → `Unsupported`.

### 4.3 Device link (NEW)

`NCRYPT_READER_PROPERTY` (CNG) or `PP_SMARTCARD_READER` (CAPI) read without
opening the key when possible → `Reader { name: anonymous_reader_name }`;
fully qualified container names `\\.\<reader>\` are also parsed. Unknown →
`None`.

### 4.4 `chain` (NEW)

`CertGetCertificateChain` with `CERT_CHAIN_CACHE_ONLY_URL_RETRIEVAL` (no
network), leaf excluded, at most 8.

## 5. macOS (`macos/`)

Promoted behavior (proofs 2–3): two sources, `macos:keychain` and
`macos:ctk` (query with `kSecAttrAccessGroupToken`); `SecKeyCreateSignature`
with the "Digest" algorithms; DER → raw; errors mapped by OSStatus and by
the `CryptoTokenKit` error domain; home from `getpwuid_r` when sandboxed.

### 5.1 Serialization (NEW)

All Security.framework calls happen on the hub's thread only (the host
guarantees it); no internal locking needed.

### 5.2 Device link (NEW)

`kSecAttrTokenID` driver part → `CryptoTokenKit { driver }` (never the
instance part, often a serial number).

### 5.3 `chain` (NEW)

`SecTrustCreateWithCertificates` + `SecTrustCopyCertificateChain` without
network evaluation, leaf excluded, at most 8.

### 5.4 PIN blocked (NEW)

`TKErrorCodeAuthenticationFailed` with zero remaining attempts in `userInfo`
→ `PinLocked` (`TODO(gustavo)`: confirm with a real token).

## 6. `KeystoreHub` (NEW)

- `inventory()`: on first call `open_all(options)` (OS first, then PKCS#11);
  then `Inventory::list`; cached until `invalidate()`.
- `KeyRef { fingerprint, path }`: path 0 = the dedup group's primary
  (`Inventory::groups`), n = the n-th alternate. Unknown → `NotFound`.
- `sign(key, request)`: routes to the owning keystore. A `Native` error on
  the primary path is returned as is; the host decides whether to offer an
  alternate (`docs/ux.md` §5.11).
- `chain`, `pin_state`: route likewise; unknown key → empty/`None`.
- `end_sessions()`: calls every keystore's `end_sessions`.

## 7. Contract suite (`contract::run`)

Checks, each with a stable name:

| Check | Expectation |
|---|---|
| `lists-expected` | every fingerprint of `fixture.expected` is listed exactly once by this source |
| `list-is-quiet` | listing twice returns the same set; no `WrongPin`/`PinRequired` from `list` |
| `provider-is-anonymous` | `provider` contains no token label, serial or holder name of the fixture |
| `signs-every-combination` | every listed key signs SHA-256/384/512 × each supported algorithm and `websign_core::verify` accepts |
| `rejects-wrong-length` | a 31-byte digest for SHA-256 → error, never a signature |
| `wrong-pin` (if `wrong_pin`) | `WrongPin`; then `pin_state().count_low` or not, per token; then the right PIN signs |
| `session-reuse` (PKCS#11) | second signature without PIN succeeds; after `end_sessions` → `PinRequired` |
| `always-authenticate` (if the fixture has one) | every signature needs the PIN |
| `chain-best-effort` | `chain` never panics; leaf never included |

Fixtures per OS job: Linux/macOS/Windows SoftHSM2 (`kit/linux/softhsm-setup.sh`
promoted by the CI track), Windows software KSP + legacy CSP keys
(`kit/windows/make-test-certs.ps1`), macOS temporary keychain
(`kit/macos/ci-macos.sh`).
