# websign-protocol — specification

The wire contract. Design and JSON examples:
[`docs/architecture/protocol.md`](../../docs/architecture/protocol.md). This
file is the testable part. Types in `src/` are the contract: do not change
public signatures, variants or serde names.

`framing` and `base64` are **promoted from the Phase-0 kit** (`probe/src/nm`),
reviewed, with their tests. Everything else is NEW.

## 1. Serialization rules (all types)

- JSON field names are camelCase (`topOrigin`, `notBefore`, `colorIndex`).
- `type` values are the dotted names of `protocol.md` §4; enum values as the
  serde attributes say (`"SHA-256"`, `"RSASSA-PKCS1-v1_5"`, `"chrome"`,
  `"UserCancelled"`).
- Optional fields are omitted when `None` (never `null`) and accepted when
  absent.
- `Base64Bytes`: canonical padded standard Base64 (§7) both ways.
- `i64` times are JSON numbers (TypeScript `number`).
- Every struct carries `deny_unknown_fields`; unknown fields fail parsing.

## 2. `version::negotiate(app, client)`

| app | client | result |
|---|---|---|
| {1,1} | {1,1} | `Ok(1)` |
| {1,3} | {2,5} | `Ok(3)` |
| {2,3} | {1,1} | `Err(ClientOutdated)` |
| {1,1} | {2,2} | `Err(AppOutdated)` |
| {1,1} | {0,1} | `Err(InvalidRequest)` (min 0) |
| {1,1} | {2,1} | `Err(InvalidRequest)` (min > max) |

## 3. `RequestId::new`

Valid: 1–64 bytes, each in `[A-Za-z0-9._:-]`. Examples valid: `"1"`, `"h"`,
`"p1.3.abc"`, `"a:b-c_d"` (64 × `"a"`). Invalid: `""`, 65 × `"a"`, `"a b"`,
`"é"`, `"a\n"`, `"<x>"`. Serde uses the same check (`try_from`).

## 4. `FingerprintHex::new`

Exactly 64 characters from `[0-9a-f]`. Uppercase, separators, 63 or 65
characters → `InvalidFingerprint`.

## 5. Envelope parsing

`parse_client_message(frame, negotiated)` and `parse_app_message(frame,
negotiated)`; `to_json(envelope)` serializes compactly (no whitespace).

Order of checks, first failure wins:

1. Not UTF-8 JSON, or not an object → `ParseError { id: None, code: InvalidRequest }`.
2. `id`: absent, not a string, or invalid (§3) → `id: None`, `InvalidRequest`.
   From here on the error carries the id.
3. `v`: absent or not a positive integer → `InvalidRequest`.
4. `type`: absent, not a string, or unknown for the direction → `InvalidRequest`
   (message names the type only if it is ≤ 32 printable ASCII characters).
5. Version:
   - `negotiated == None`: only `hello` is accepted (anything else →
     `InvalidRequest` "hello must be the first message"); `v` must be within
     `hello.protocols` and at least 1 — the app does not reject `hello` for
     `v` alone, negotiation decides.
   - `negotiated == Some(n)`: `v` must equal `n`, else `InvalidRequest`
     "message version v does not match the negotiated version n".
6. Body: every other field deserialized strictly into the type's struct;
   unknown field, wrong type, invalid enum value, invalid Base64, invalid
   fingerprint → `InvalidRequest` naming the field (never echoing its value).

The app side additionally rejects frames larger than
`limits::MAX_INCOMING_FRAME` before parsing (framing).

Round trip: for every message type, `parse(to_json(x)) == x` (property test
over generated values).

## 6. Framing (promoted)

`read_frame` / `write_frame`, `u32` native-endian length + body;
`MAX_INCOMING = MAX_OUTGOING = 1 MiB`. `Ok(None)` = clean end between frames;
`Truncated` = EOF inside a header or body; `TooLarge` for either direction.
The promoted tests in `src/framing/tests.rs` are the vectors.

## 7. Base64 (promoted)

RFC 4648 §4, padded, strict: no whitespace, no URL-safe alphabet, padding
required, unused trailing bits must be zero. Promoted tests in
`src/base64.rs`.

## 8. Verification code

`verification_code(digest)`:

```
b          = digest[0..8]                         (None if len < 8)
text       = hex_upper(b) in groups of 4 separated by one space
colorIndex = b[0] >> 5                           → 0..7
bits       = (b[1] << 8) | b[2]                  → the low 15 bits are used
cell(row r ∈ 0..5, column c ∈ 0..3) lit ⇔ bit (r*3 + c) of bits is 1
column 3 = column 1; column 4 = column 0         (mirror)
cells[r*5 + c], row-major, 25 entries
```

| Digest (first bytes) | text | colorIndex | rows (1 = lit) |
|---|---|---|---|
| `7F3A9C21E0B455D8…` | `7F3A 9C21 E0B4 55D8` | 3 | `00100 11011 01010 10101 11011` |
| SHA-256("") = `E3B0C44298FC1C14…` | `E3B0 C442 98FC 1C14` | 7 | `00100 00000 11011 00000 11011` |
| SHA-256("abc") = `BA7816BF8F01CFEA…` | `BA78 16BF 8F01 CFEA` | 5 | `01110 01010 00000 00100 11111` |
| 32 zero bytes | `0000 0000 0000 0000` | 0 | all off |
| 32 × `FF` | `FFFF FFFF FFFF FFFF` | 7 | all on |
| SHA-384("abc") = `CB00753F45A35E8B…` | `CB00 753F 45A3 5E8B` | 6 | `10101 01110 10001 00000 00000` |
| 7 bytes | `None` | | |

The SDK's `fingerprint()` and both client libraries implement the same table.

## 9. Page types

`PageToExtension`/`ExtensionToPage` are tagged by `kind`
(`"discover"`, `"request"`, `"announce"`, `"message"`); `PageRequest`/`PageReply`
by `type`. `source` is `PAGE_SOURCE` or `EXTENSION_SOURCE`; a message with any
other `source` is not ours (the receiver ignores it; parsing it is not an
error of this crate).

## 10. TypeScript generation

With feature `typescript`, every public wire type derives `ts_rs::TS` and
exports one file per type. `cargo xtask gen` runs `cargo test -p
websign-protocol --features typescript export_bindings` with
`TS_RS_EXPORT_DIR`, adds an `index.ts` barrel, and copies the folder to
`sdk/src/generated`, `extension/src/generated` and
`clients/node/src/generated`. ts-rs prints "failed to parse serde attribute"
notes for `try_from`/`into`; they are harmless (the `ts(type = "string")`
override is what applies).
