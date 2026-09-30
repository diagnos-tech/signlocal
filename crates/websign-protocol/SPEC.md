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
- `Base64Bytes`: canonical padded standard Base64 (§7) both ways.
- `i64` times are JSON numbers (TypeScript `number`).
- Every struct and every tagged enum (page enums included) refuses unknown
  fields (D12).

**One spelling per message.** A relay that validates one spelling could be
walked around with another, so every alternative spelling is refused, by the
types themselves (`serde_json::from_*` on any wire type) as well as by the
parsers:

| Spelling | Rule |
|---|---|
| Optional field | Omitted when `None`, accepted when absent; an explicit `null` is **refused**. No field of the protocol accepts `null`. |
| Struct or tagged enum | A JSON object only; the array form serde would accept (`[1, 1]` for a `ProtocolRange`, `["RSA", 2048]` for a key) is **refused**. |
| Integer (`v`, `seq`, `bits`, …) | A JSON integer only: `1.0`, `1e0` and `"1"` are refused. |
| Object keys | Unique per object after unescaping (`"id"` = `"i\u0064"`); a repeated key makes the whole frame unreadable (§5 step 1). The types alone reject repeated *known* fields; the parsers reject every repeat. |

Implementation (`src/strict.rs`): wire types derive with
`#[serde(remote = "Self")]` and get their trait impls from `object_serde!`,
which deserializes through an object-only wrapper; optional fields use
`deserialize_with = "crate::strict::present"`. The derived inherent
`X::deserialize` functions this leaves behind do not refuse arrays: always go
through the `Deserialize` trait.

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

1. Not UTF-8 JSON, trailing data, not an object, or a repeated object key at
   any depth → `ParseError { id: None, code: InvalidRequest }`.
2. `id`: absent, not a string, or invalid (§3) → `id: None`, `InvalidRequest`.
   From here on the error carries the id.
3. `v`: absent or not a positive JSON integer that fits `u32` → `InvalidRequest`.
4. `type`: absent, not a string, or unknown for the direction → `InvalidRequest`
   (message names the type only if it is ≤ 32 printable ASCII characters).
5. Version:
   - `negotiated == None`: only `hello` is accepted (anything else →
     `InvalidRequest` "hello must be the first message").
     Client `hello`: `v` must be within `hello.protocols` (the app does not
     reject `hello` for `v` alone, negotiation decides). App `hello`: `v` must
     equal `hello.protocol`, the version the app picked. A malformed
     `protocols`/`protocol` is left to step 6.
   - `negotiated == Some(n)`: `v` must equal `n`, else `InvalidRequest`
     "message version v does not match the negotiated version n". This holds
     for `hello` too: a second `hello` with `v == n` **parses**, and the
     session refuses it (`websign-host` SPEC §2).
6. Body: every other field deserialized strictly into the type's struct (§1);
   unknown field, `null`, wrong type, invalid enum value, invalid Base64,
   invalid fingerprint → `InvalidRequest` naming the field by its path
   (`web.sneaky`, `filter.algorithms`, `certificates[0].key`), never echoing
   a value. A path segment is a key taken from the frame, so it is echoed only
   if it is ≤ 32 printable ASCII characters ("a field of web is unknown …"
   otherwise). A missing required field is named as-is ("required field hash
   is missing"). Exact wording is free; tests check only the field name and
   the absence of values.
7. Limits the types cannot express (`src/envelope/bounds.rs`), each failure
   `InvalidRequest` naming the field:

   | Direction | Rule |
   |---|---|
   | client | `web.origin`, `web.topOrigin` ≤ `MAX_ORIGIN_LEN` (512) bytes |
   | client | `hello`: `client.name`, `client.version`, `browser.version` ≤ `MAX_SHORT_TEXT_LEN` (64) bytes |
   | client | `sign.begin.algorithms` and `choose.filter.algorithms`: absent or non-empty (an empty list would disable every certificate) |
   | app | `choose.result.certificates`: at least one |
   | app | every `certificate.chain` ≤ `MAX_CHAIN_LEN` (8) |

   Not checked here: the digest's length (it depends on the `hash` of the
   open request; the app refuses a wrong length, `websign-host` SPEC §4) and
   whether `web`/`browser` are required or refused (that depends on the
   transport, `websign-host` SPEC §2.2).

The app side additionally rejects frames larger than
`limits::MAX_INCOMING_FRAME` before parsing (framing). Parsing never panics
and its work is linear in the frame size (nesting is capped at 128 by
`serde_json`; locating the offending field descends at most 8 levels).

`AppMessage::is_final` is true for every message but `sign.need_digest`
(`hello` included). `Base64Bytes`' `Debug` prints the length only.

Round trip: for every message type, `parse(to_json(x)) == x` (property test
over generated values).

### 5.1 Refusing `hello`

An `error` that answers a connection's first frame (a refused `hello`: failed
negotiation, a transport rule, a malformed body; or a first message that is
not `hello`) carries **the `v` of that frame** — `refusal_version(frame)`, or
`PROTOCOL_VERSION` when the frame has no positive `u32` `v`. A client sends
`hello` at a version it speaks, so the refusal is readable by a client of any
version, including one whose range does not overlap the app's
(`ClientOutdated`, `AppOutdated`). This works because the `error` message
(`code`, `message`, `details {installed, required, native}`) and the codes that
can refuse `hello` (`InvalidRequest`, `ClientOutdated`, `AppOutdated`,
`Internal`) are frozen: no protocol version changes them.

`parse_hello_reply(frame, hello_v)` parses the app's answer to a `hello` sent
at `hello_v`, with the §5 order of checks except step 5:

| Frame | Result |
|---|---|
| `hello` with `v == protocol` | `Ok` (`v` need not equal `hello_v`) |
| `hello` with `v != protocol` | `InvalidRequest` |
| `error` with `v == hello_v` | `Ok`, body strict as always |
| `error` with `v != hello_v` | `InvalidRequest` naming the version |
| any other `type` | `InvalidRequest` "hello must be answered by hello or error" |

`parse_app_message(frame, None)` is unchanged: it accepts only `hello`.

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
by `type`. They follow §1 like every wire type: unknown fields are refused on
every variant, so a page cannot smuggle `web` (or anything else) into a
request. `source` is `PAGE_SOURCE` or `EXTENSION_SOURCE`; a message with any
other `source` is not ours (the receiver ignores it; parsing it is not an
error of this crate).

## 10. TypeScript generation

With feature `typescript`, every public wire type derives `ts_rs::TS` and
exports one file per type. `cargo xtask gen` runs `cargo test -p
websign-protocol --features typescript export_bindings` with
`TS_RS_EXPORT_DIR`, adds an `index.ts` barrel, and copies the folder to
`sdk/src/generated`, `extension/src/generated` and
`clients/node/src/generated`. ts-rs prints "failed to parse serde attribute"
notes for `try_from`/`into`, `remote` and `deserialize_with`; they are
harmless (the `ts(type = "string")` overrides and `ts(optional)` are what
apply, and the generated TypeScript is unchanged by them).
