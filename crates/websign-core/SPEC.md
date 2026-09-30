# websign-core — specification

Pure Rust library (no operating-system calls) with the pieces every key source
and both windows share. It is the source of truth for the tests (`tests/`, one
file per section, over OpenSSL fixtures) and for the implementation (`src/`,
whose unit tests cover what OpenSSL cannot generate). Every behavior change
starts here. If the spec is silent while writing a test or code, record the
assumption as a `// SPEC:` comment for the reviewer; once reconciled, the rule
moves into this file and the comment goes away.

**Do not change public signatures.** Types, variants and public fields are the
contract. Private modules are free.

Sections §1–§8 are **promoted from the Phase-0 kit's `probe-core`**, reviewed
through blind TDD (391 tests, 460 k mutated inputs without panic). Sections
marked **NEW** are to be implemented.

General conventions:

- Errors are `enum`/`struct` with `thiserror`, `Debug + Clone + PartialEq + Eq`.
- Nothing panics on external input (certificate bytes, signatures, strings):
  every failure is an `Err`.
- OIDs are dotted text (`"2.16.76.1.2.3.4"`).
- Times are Unix seconds (`i64`, UTC).

---

## 1. `hash` — `HashAlgorithm`

`enum HashAlgorithm { Sha256, Sha384, Sha512 }`

| Method | Behavior |
|---|---|
| `ALL` | `[Sha256, Sha384, Sha512]`, in this order |
| `digest_len()` | 32, 48, 64 |
| `name()` | `"SHA-256"`, `"SHA-384"`, `"SHA-512"` (WebCrypto names) |
| `digest(data)` | the matching SHA-2 over `data` |
| `check_digest(d)` | `Ok(())` if `d.len() == digest_len()`, else `DigestLengthError { algorithm, expected, actual }` |
| `Display` | same as `name()` |
| `FromStr` | accepts, ASCII-case-insensitively, `SHA-256`/`SHA256`, `SHA-384`/`SHA384`, `SHA-512`/`SHA512`. Anything else (surrounding spaces, `SHA-1`, `SHA-224`, `SHA3-256`, empty, and letters that only become ASCII through Unicode case folding, such as `ſ` → `S`) → `UnknownAlgorithmError { name }` with the original string |

`DigestLengthError` displays: `digest for SHA-256 must be 32 bytes, got 31`.

## 2. `algorithm` — `SignatureAlgorithm`

`enum SignatureAlgorithm { Ecdsa, RsaPkcs1v15, RsaPss }`

| Method | Behavior |
|---|---|
| `ALL` | `[Ecdsa, RsaPkcs1v15, RsaPss]` |
| `name()` | `"ECDSA"`, `"RSASSA-PKCS1-v1_5"`, `"RSASSA-PSS"` (WebCrypto names) |
| `is_rsa()` | `false`, `true`, `true` |
| `Display` | same as `name()` |
| `FromStr` | exactly the three names, ASCII-case-insensitive. Others → `UnknownAlgorithmError { name }` |

`UnknownAlgorithmError` displays: `unknown algorithm: <name>`.

## 3. `pkcs1` — `digest_info`

`digest_info(hash, digest) -> Result<Vec<u8>, DigestLengthError>`

Returns the DER `DigestInfo` (RFC 8017 §9.2, note 1) = fixed prefix ‖ digest.
This is what `CKM_RSA_PKCS` and raw RSA signers expect when the hash is
already computed. Validates the digest length first (same error as
`check_digest`).

Prefixes (hex):

```
SHA-256: 30 31 30 0d 06 09 60 86 48 01 65 03 04 02 01 05 00 04 20
SHA-384: 30 41 30 0d 06 09 60 86 48 01 65 03 04 02 02 05 00 04 30
SHA-512: 30 51 30 0d 06 09 60 86 48 01 65 03 04 02 03 05 00 04 40
```

## 4. `ecdsa` — curves and signature encoding

`enum Curve { P256, P384, P521, BrainpoolP256r1, BrainpoolP384r1, BrainpoolP512r1 }`

| Method | Behavior |
|---|---|
| `field_len()` | 32, 48, 66, 32, 48, 64 |
| `signature_len()` | `2 * field_len()` |
| `name()` | `"P-256"`, `"P-384"`, `"P-521"`, `"brainpoolP256r1"`, `"brainpoolP384r1"`, `"brainpoolP512r1"` |
| `has_verifier()` | `false` only for `BrainpoolP512r1` |
| `from_oid(&str)` | `1.2.840.10045.3.1.7` → P256, `1.3.132.0.34` → P384, `1.3.132.0.35` → P521; brainpool per §4.1; other → `None` |

ECDSA signatures travel in two formats: **raw** (`r ‖ s`, each `field_len`
bytes big-endian, left-padded with zeros — IEEE P1363; what the SDK returns,
what CNG and PKCS#11 produce) and **DER** (`ECDSA-Sig-Value ::= SEQUENCE { r
INTEGER, s INTEGER }` — what macOS returns).

`der_to_raw(der, curve) -> Result<Vec<u8>, EcdsaEncodingError>`

- Required structure: `30 len 02 len r 02 len s`, nothing after `s` inside the
  SEQUENCE nor after the end of the SEQUENCE.
- Lengths in short form or 1-byte long form (`81 xx`, needed for P-521;
  `81 xx` with `xx < 0x80`, non-minimal, is also accepted). Indefinite form
  (`80`) or longer long forms → `Malformed`.
- Negative INTEGER (first content byte ≥ `0x80`) or empty → `Malformed`.
- Extra leading zeros are **tolerated** (some tokens emit non-minimal DER):
  strip them.
- `r` or `s` equal to zero → `Malformed`.
- A value that, without leading zeros, does not fit `field_len` bytes →
  `IntegerTooLarge`.
- Order: the whole structure (including sign and zero of `r` **and** `s`) is
  checked before sizes, so a structural defect gives `Malformed` even if the
  other integer is too large.
- Does not compare `r`/`s` with the curve order: this is format conversion,
  not validation (`verify` validates).
- Output: `r` and `s` left-padded to `field_len`, concatenated.

`raw_to_der(raw, curve) -> Result<Vec<u8>, EcdsaEncodingError>`

- `raw.len() != signature_len()` → `WrongLength { expected, actual }`.
- Each half becomes a **minimal** DER INTEGER: strip leading zeros (keep at
  least one byte), prepend `00` if the first remaining byte is ≥ `0x80`.
- Long form `81 xx` when the SEQUENCE content exceeds 127 bytes.
- Does not validate values: an all-zero half becomes `02 01 00`, values ≥ the
  curve order pass.
- `der_to_raw(raw_to_der(x)) == x` for every `x` with non-zero `r` and `s`.

### 4.1 Brainpool curves — NEW

- `from_oid`: `1.3.36.3.3.2.8.1.1.7` → `BrainpoolP256r1`,
  `1.3.36.3.3.2.8.1.1.11` → `BrainpoolP384r1`, `1.3.36.3.3.2.8.1.1.13` →
  `BrainpoolP512r1`. The `t1` twisted variants (`…1.1.8`, `…1.1.12`,
  `…1.1.14`) stay unknown (`None`).
- Consequence for §6.2: certificates on these curves summarize as
  `Ec { curve }` instead of `Unsupported`. **Update** the promoted tests that
  list `brainpoolP256r1` as unsupported (`tests/cert_key.rs`,
  `tests/ecdsa.rs`, `tests/verify.rs`, `tests/fixtures/README.md`).
- `der_to_raw`/`raw_to_der` work unchanged with `field_len` 32/48/64.
- `verify` (§7): BrainpoolP256r1 and BrainpoolP384r1 verify with the `bp256`/
  `bp384` crates (`r1::ecdsa::VerifyingKey`, prehash, FIPS 186-5 digest
  truncation as for NIST curves). BrainpoolP512r1 → `UnsupportedKey` (no
  maintained verifier); callers check `has_verifier()` first.
- New fixtures: keys and certificates for the three curves, signature
  vectors for P256r1/P384r1 from OpenSSL (`tests/fixtures/gen`).

## 5. `fingerprint` — `Fingerprint`

SHA-256 of the certificate DER. The identity of a certificate across the
project (dedup, "last used", selection on the command line, wire
`FingerprintHex`).

| Method | Behavior |
|---|---|
| `of(der)` | SHA-256 of `der` |
| `from_bytes([u8; 32])` / `as_bytes()` | direct conversions |
| `to_hex()` | 64 lowercase hex characters, no separator |
| `Display` | **uppercase** hex pairs separated by `:` (95 characters) |
| `Debug` | `Fingerprint(<to_hex()>)` |
| `FromStr` | ignores `:` and ASCII space (U+0020) anywhere; the rest must be exactly 64 ASCII hex digits (any case). Any other character, including tab, newline and other Unicode spaces → `ParseFingerprintError` |

Derives `Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord`.

## 6. `cert` — X.509 certificate summary

`CertInfo::from_der(der) -> Result<CertInfo, CertError>`

Bytes without the structure of an X.509 `Certificate` → `CertError::Malformed(<what
broke>: <detail>)`. A known extension that is malformed is also `Malformed`.
Unknown extensions are ignored without looking at their content, even when
critical (this is a summary for display, not path validation). The
certificate's signature is not checked.

### 6.0 Tolerant reading

No real certificate may disappear from the list over an encoding detail that
does not change the summary. The reader accepts these deviations from DER /
RFC 5280, all seen in issued certificates:

- long-form lengths that are not minimal (1 to 4 length octets);
- non-minimal INTEGER; serial numbers of any size (including > 20 octets),
  zero or negative; RSA modulus without the sign byte (read as unsigned);
- BOOLEAN with any non-zero octet = TRUE (BER); DEFAULT values written out
  (explicit v1 version, `critical FALSE`, `cA FALSE`); absent or any version;
- multi-valued RDN out of DER order; `issuerUniqueID`/`subjectUniqueID`;
- GeneralizedTime before 2050;
- repeated extension (forbidden by RFC 5280): the first wins;
- OIDs with arcs up to 128 bits (the `2.25` UUIDs are the largest in use). An
  OID that cannot be written (invalid encoding or larger arc) is ignored where
  it need not be printed (extension type, name attribute type, otherName
  type, statement type, QcType) and is `Malformed` where it must be (EKU,
  policy, key algorithm).

Still `Malformed`: indefinite form (`80`) or lengths over 4 octets, multi-octet
tags, truncated values, leftover bytes (at the end or inside a read
structure), unknown or out-of-order TBSCertificate fields, empty serial, times
outside the forms below (`not_before`), RSA/EC key BIT STRING with unused
bits, unreadable `RSAPublicKey`.

`CertInfo` fields:

| Field | Content |
|---|---|
| `fingerprint` | `Fingerprint::of(der)` |
| `subject`, `issuer` | `DistinguishedName` (below) |
| `serial_hex` | lowercase hex of the serial's content octets without leading `00`, keeping at least one octet (`00 80` → `"80"`; `12 34` → `"1234"`; `01 02` → `"0102"`; `00` → `"00"`; `00 00 05` → `"05"`). A negative serial stays in two's complement, as Windows and macOS show it (`ff 7f` → `"ff7f"`) |
| `not_before`, `not_after` | Unix seconds, negative before 1970. Only RFC 5280 §4.1.2.5 forms: UTCTime `YYMMDDHHMMSSZ` (YY 50–99 → 19YY, 00–49 → 20YY) and GeneralizedTime `YYYYMMDDHHMMSSZ` in any year. Without seconds, with fractions, with a zone, or with an impossible date/time (30 Feb, 24h, 60 s) → `Malformed` |
| `key` | `PublicKeyKind` |
| `key_usage` | `Some(KeyUsage)` if extension 2.5.29.15 exists (even with no bit set, or only bits outside the struct), else `None` |
| `extended_key_usage` | OIDs of extension 2.5.29.37, in certificate order; empty if absent or empty |
| `policies` | policy OIDs of extension 2.5.29.32, in order; empty if absent |
| `is_ca` | `cA` of BasicConstraints (2.5.29.19); `false` if absent |
| `icp_brasil` | `Some` for ICP-Brasil certificates (§6.3) |
| `qualified` | `Some` if the qcStatements extension (1.3.6.1.5.5.7.1.3) exists (§6.4) |

### 6.1 `DistinguishedName`

`{ common_name, organization, organizational_units: Vec<String>, country,
given_name, surname, serial_number }` (all `Option<String>` but the units).

- CN (2.5.4.3), O (2.5.4.10), OU (2.5.4.11, all, in order), C (2.5.4.6).
- Strings: UTF8String, PrintableString and IA5String as UTF-8; BMPString as
  UTF-16BE; TeletexString as Latin-1. Other types (NumericString,
  UniversalString, VisibleString…) and bytes invalid for their type (broken
  UTF-8, odd-length BMPString or lone surrogate): only that attribute is
  ignored, as if absent.
- A single-valued attribute appearing more than once: the first **readable**
  value wins (a CN ignored by the rule above does not hide the next CN).

### 6.1a New attributes — NEW

- `given_name` (2.5.4.42), `surname` (2.5.4.4), `serial_number` (2.5.4.5),
  same string rules and first-readable-wins as CN. Absent → `None`.
- `serial_number` is personal data: it may be read here, but only masked
  forms (§11) leave the crate's presentation layer.

### 6.2 `PublicKeyKind` and `KeyUsage`

`enum PublicKeyKind { Rsa { bits: u32 }, Ec { curve: Curve }, Unsupported { oid: String } }`

- RSA (`1.2.840.113549.1.1.1` and RSA-PSS `1.2.840.113549.1.1.10`): `bits` =
  modulus size in bits (2048, 3072, 4096; a 2047-bit modulus → 2047). RSA-PSS
  key parameters are ignored, and such a key also accepts `RsaPkcs1v15` in
  `supports` (the API cannot tell them apart; RFC 4055 restricts it to PSS,
  but tokens with such keys are rare).
- EC (`1.2.840.10045.2.1`) with a known named curve → `Ec`. Unknown named
  curve → `Unsupported { oid: <curve OID> }`. No named curve (explicit
  parameters, `NULL` or absent) → `Unsupported { oid: "1.2.840.10045.2.1" }`.
- Any other algorithm → `Unsupported { oid: <algorithm OID> }`.
- The EC point and the RSA pair are not validated here (that needs curve
  arithmetic); `verify` rejects them (§7).
- `supports(alg)`: RSA accepts `RsaPkcs1v15` and `RsaPss`; EC accepts
  `Ecdsa`; `Unsupported` accepts nothing.

`KeyUsage` has one public `bool` per bit: `digital_signature`,
`non_repudiation`, `key_encipherment`, `data_encipherment`, `key_agreement`,
`key_cert_sign`, `crl_sign`.

### 6.3 ICP-Brasil — `IcpBrasil`

`{ level: Option<IcpLevel>, holder_name: Option<String>, cpf: Option<String>, cnpj: Option<String> }`

A certificate is ICP-Brasil (`icp_brasil = Some`) if a policy starts with
`2.16.76.1.2.` **or** a SubjectAltName (2.5.29.17) `otherName` has an OID
starting with `2.16.76.1.3.`.

- `level`: from the first policy starting with `2.16.76.1.2.`, reading the
  arc `n` right after the prefix (`2.16.76.1.2.3.1` and also `2.16.76.1.2.3`
  with no more arcs give n = 3): n = 1..4 → `A1..A4`; 101..104 → `S1..S4`;
  303 → `T3`; 304 → `T4`; other n → `Other(n)`. An `n` that does not fit in
  `u32` → `None` (the certificate is still ICP-Brasil; later policies are not
  consulted). No ICP policy → `None`.
- `holder_name`: the subject CN without its **last** `:` when followed by one
  or more ASCII digits and nothing else (`"ANA BEATRIZ SOUZA:12345678901"` →
  `"ANA BEATRIZ SOUZA"`; `"R2:D2:123"` → `"R2:D2"`; `"A:1:2"` → `"A:1"`;
  `"MARIA:"`, `"MARIA:12A"` and `"MARIA: 12"` stay as they are). No CN → `None`.
- `otherName` value: accepts OCTET STRING, PrintableString, UTF8String or
  IA5String with ASCII-only content. Other types (BMPString, INTEGER…) or a
  non-ASCII byte: the otherName exists but its value is unreadable.
- `cpf`: from `otherName` 2.16.76.1.3.1 (natural person) or, **only if it does
  not exist**, from 2.16.76.1.3.4 (person responsible for a company). The
  value starts with the birth date (8 characters, not checked) followed by the
  CPF (11): `cpf` = characters 8..19. `None` if unreadable, too short, if the
  11 are not all digits or are all `0`. A present but invalid 2.16.76.1.3.1
  gives `None` and does **not** fall back to 2.16.76.1.3.4: DOC-ICP-04 fills
  missing data with zeros, and showing the company officer's CPF as the
  holder's would attribute it to the wrong person.
- `cnpj`: `otherName` 2.16.76.1.3.3, whole value; must be 14 digits and not
  all `0`, else `None`.
- `masked_cpf()`: `"***.456.789-**"` for CPF `12345678901` (gov.br pattern).
  `None` without CPF or if the (public, anyone can set it) field is not
  exactly 11 ASCII digits.
- `formatted_cnpj()`: `"12.345.678/0001-95"` for `12345678000195`. `None`
  without CNPJ or if not exactly 14 ASCII digits.

`enum IcpLevel { A1, A2, A3, A4, S1, S2, S3, S4, T3, T4, Other(u32) }`, with
`Display` = `"A1"` … `"T4"` and `Other(n)` → `"ICP-Brasil (n)"`.

### 6.4 eIDAS qualified — `Qualified`

`{ compliance: bool, sscd: bool, types: Vec<QcType> }` from qcStatements
(RFC 3739 / ETSI EN 319 412-5): `SEQUENCE OF SEQUENCE { statementId OID,
statementInfo ANY OPTIONAL }`.

- `compliance`: statement `0.4.0.1862.1.1` (QcCompliance) exists.
- `sscd`: `0.4.0.1862.1.4` (QcSSCD — key in a qualified device) exists.
- `types`: from statements `0.4.0.1862.1.6` (QcType), whose info is `SEQUENCE
  OF OID`: `…1.6.1` → `ESign`, `…1.6.2` → `ESeal`, `…1.6.3` → `Web`; other
  OIDs ignored; certificate order. With several QcType statements, all types
  are added, in order, without removing repeats. QcType without info adds
  nothing; info that is not `SEQUENCE OF OID` → `Malformed`.
- Unknown statements are ignored, and the info of other statements is not read.
- The extension's presence suffices for `Some`, even empty or with only
  unknown statements (`Some(Qualified::default())`).

### 6.5 `CertInfo` methods

- `is_valid_at(t)`: `not_before <= t <= not_after`.
- `can_sign()`: `!is_ca` and (`key_usage` absent, or with
  `digital_signature` or `non_repudiation`). KeyUsage present without either →
  `false`. EKU is not considered: the site decides.
- `display_name()`: `icp_brasil.holder_name`, else `subject.common_name`,
  else `subject.organization`, else the first 16 characters of
  `fingerprint.to_hex()`. Empty or whitespace-only candidates are skipped (a
  CN `":123"` gives holder `""`, also skipped).

## 7. `verify` — check a raw signature

`verify(cert_der, hash, algorithm, digest, signature) -> Result<(), VerifyError>`

Proves that what a key store returned is a valid signature in the format the
SDK promises. The host runs it on every signature before replying. Check
order and errors:

1. Invalid certificate → `VerifyError::Certificate(CertError)` (same as
   `CertInfo::from_der`).
2. Wrong digest length → `VerifyError::Digest(DigestLengthError)`.
3. `Unsupported` key → `UnsupportedKey`.
4. Algorithm incompatible with the key (`!key.supports(algorithm)`) → `KeyMismatch`.
5. Recognized but unusable key → `UnsupportedKey`: EC point off the curve,
   identity or wrong size; RSA with zero or even modulus, larger than 8192
   bits, or even exponent, smaller than 3, larger than 2³³−1 or not smaller
   than the modulus. Curves without a verifier (`BrainpoolP512r1`) →
   `UnsupportedKey`.
6. Signature does not match, or has the wrong format/size → `InvalidSignature`.

Formats: RSA = a block of exactly the modulus size in bytes and, read as an
integer, **smaller than the modulus** (RFC 8017 §5.2.2: with a 2047-bit
modulus, `s + n` still fits the block and must not pass as a second
signature). ECDSA = raw `r ‖ s` of exactly `signature_len()` bytes, `r` and
`s` in `[1, n−1]`; high `s` is accepted (CMS/PAdES do not require low `s`).
RSASSA-PSS: MGF1 with the same hash and salt = digest length, required (no
salt detection). ECDSA with a hash longer or shorter than the curve follows
FIPS 186-5 (the digest is truncated / used as is), so P-256 + SHA-384 and
P-521 + SHA-256 are valid.

`verify` only checks the math: not validity, KeyUsage, EKU nor the
certificate's own signature.

## 8. `dedup` — the same certificate through two paths

`enum SourceKind { System, Pkcs11 }` — `System` is the OS store (CNG/CAPI,
Keychain/CryptoTokenKit); it wins.

`dedup_by_fingerprint(items: Vec<T>, key: impl Fn(&T) -> (Fingerprint, SourceKind)) -> Vec<Deduped<T>>`

`Deduped<T> { primary: T, alternates: Vec<T> }`

- Groups by fingerprint.
- Output order: order of **first appearance** of each fingerprint.
- `primary`: the first `System` item of the group (input order); if none, the
  group's first item.
- `alternates`: the other items of the group, in input order.
- Identical items are not dropped (they become alternates).
- Empty input → empty output.

---

## 9. `present::origin` — NEW (`docs/ux.md` §4.3, vectors §16.2)

`format_origin(origin: &str) -> Result<FormattedOrigin, OriginError>`

Input: a serialized origin as browsers produce it (`scheme://host[:port]`,
no path, no trailing slash). Steps:

1. Parse: scheme `https` or `http` (ASCII-case-insensitive); host non-empty;
   optional port 1–65535. A path, query, fragment, userinfo, `null`, a length
   over 512, or anything else → `Malformed`. A trailing `/` alone is accepted
   and ignored.
2. Hosts: lowercase; a Unicode host is converted to ASCII (IDNA/UTS 46,
   `idna` crate); IPv6 literals in brackets are kept with brackets.
3. Secure context: `https` always; `http` only for `localhost`,
   `*.localhost`, `127.0.0.0/8` and `[::1]` → else `Insecure`. Any other
   scheme → `Insecure`.
4. `canonical` = `scheme://ascii-host[:port]`, default ports (443 for https,
   80 for http) omitted.
5. Split: for IP literals, `localhost` and `*.localhost`, `registrable` = the
   whole host, `prefix` = `scheme://`. Otherwise `registrable` = eTLD+1 from
   the Public Suffix List (`psl` crate, ICANN and private sections); when the
   host is itself a public suffix, `registrable` = the whole host.
   `prefix` = `scheme://` + the labels before `registrable` + `.`.
6. `port` = the non-default port.
7. `unicode` = the Unicode form when any label is `xn--` (after step 2).
8. `warning` (first that applies): IDN → `Idn`; loopback or `localhost` →
   `Localhost`; private IPv4 (10/8, 172.16/12, 192.168/16), link-local
   (169.254/16, fe80::/10), unique-local (fc00::/7) → `LocalIp`; any other IP
   literal → `PublicIp`; else `None`.
9. `can_remember` = `false` for `Idn`, `LocalIp` and `PublicIp`; `true`
   otherwise (localhost may be remembered: developers).

Vectors (`docs/ux.md` §16.2, extended):

| Input | canonical | prefix | registrable | port | warning | can_remember |
|---|---|---|---|---|---|---|
| `https://app.diagnos.health` | same | `https://app.` | `diagnos.health` | — | — | yes |
| `https://app.diagnos.health:443` | `https://app.diagnos.health` | `https://app.` | `diagnos.health` | — | — | yes |
| `https://laudos.clinicasaolucas.med.br` | same | `https://laudos.` | `clinicasaolucas.med.br` | — | — | yes |
| `https://diagnos.health.cadastro-medico.com` | same | `https://diagnos.health.` | `cadastro-medico.com` | — | — | yes |
| `https://xn--dignos-4nf.health` | same | `https://` | `xn--dignos-4nf.health` | — | `Idn` (unicode `diаgnos.health`) | no |
| `https://192.168.0.20:8443` | same | `https://` | `192.168.0.20` | 8443 | `LocalIp` | no |
| `https://203.0.113.7` | same | `https://` | `203.0.113.7` | — | `PublicIp` | no |
| `http://localhost:5173` | same | `http://` | `localhost` | 5173 | `Localhost` | yes |
| `http://127.0.0.1:8000` | same | `http://` | `127.0.0.1` | 8000 | `Localhost` | yes |
| `http://[::1]:3000` | same | `http://` | `[::1]` | 3000 | `Localhost` | yes |
| `HTTPS://App.Example.COM` | `https://app.example.com` | `https://app.` | `example.com` | — | — | yes |
| `http://laudos.exemplo.com` | — | — | — | — | `Err(Insecure)` | — |
| `file://`, `chrome-extension://abc`, `data:` | — | — | — | — | `Err(Insecure)` or `Err(Malformed)` (no host) | — |
| `https://a.example/path` | — | — | — | — | `Err(Malformed)` | — |

## 10. `present::holder` — NEW (`docs/ux.md` §5.2, vectors §16.3)

`display_name(info)`: take `info.display_name()` (§6.5); if no candidate was
found (fingerprint fallback), first try `given_name + " " + surname` when both
exist; then apply `title_case` **only** when the result has at least one
letter and all its letters are uppercase.

`title_case(name)`:

- Words are maximal runs of non-space characters; separators are kept exactly.
- Each word: first character uppercase, rest lowercase (Unicode-aware:
  `ÇÃO` → `Ção`), except:
  - particles `da das de di do dos du e del la van von y` → all lowercase,
    unless it is the first word;
  - company suffixes kept as written in the input: `ME`, `EPP`, `EIRELI`,
    `S.A.`, `S/A`; `LTDA` → `Ltda`.
- Hyphenated parts are capitalized each (`MARIA-CLARA` → `Maria-Clara`), and
  so is the letter after an apostrophe (`D'ÁVILA` → `D'Ávila`).

| CN | Result |
|---|---|
| `ANA BEATRIZ SOUZA:12345678909` | `Ana Beatriz Souza` |
| `JOAO DA SILVA DOS SANTOS:98765432100` | `Joao da Silva dos Santos` |
| `CLINICA SOUZA IMAGEM LTDA:12345678000190` | `Clinica Souza Imagem Ltda` |
| `E SOUZA COMERCIO ME` | `E Souza Comercio ME` |
| `Marta Sofia Carvalho` | `Marta Sofia Carvalho` |
| `JOSÉ D'ÁVILA` | `José D'Ávila` |

## 11. `present::document` — NEW (`docs/ux.md` §5.5, vectors §16.4)

`display_document(info) -> Option<DocumentLabel>`, first match:

1. `icp_brasil.cpf` = `12345678909` → `Cpf { masked: "•••.456.789-••",
   visible: "456 789" }` (digits 4–9; `•` is U+2022).
2. `icp_brasil.cnpj` = `12345678000190` → `Cnpj { formatted:
   "12.345.678/0001-90" }`.
3. `subject.serial_number` with an ETSI prefix (`^[A-Z]{3}[A-Z]{2}-` such as
   `IDCPT-`, `PNOPT-`, `IDCES-`, `TINIT-`) and at least 3 characters after
   the dash → `National { masked: "•••••" + last 3 }`.
4. Otherwise `None`.

The full CPF never appears in any output of this crate.

## 12. `present::caller` — NEW (`docs/architecture/desktop-api.md` §6)

`caller_label(caller) -> CallerLabel`:

- `name` = `product_name` when non-blank (trimmed, at most 64 characters with
  `…`), else the executable's file name.
- Signed: `detail` = the signer (`Authenticode.subject`, or
  `Apple.identifier` + ` (` + `team_id` + `)`), `verified = true`.
- Unsigned: `detail` = the executable path, `verified = false`.

`consent_key(caller) -> String`:

- Authenticode: `"app:authenticode:" + subject + ":" + file name (lowercase)`.
- Apple: `"app:apple:" + team_id + ":" + identifier`.
- Unsigned: `"path:" + executable path` (as given, not canonicalized here).

## 13. `present::wire::certificate_profile` — NEW

`certificate_profile(info, hardware) -> CertificateProfile`:

- `icp_brasil`: `Some(level.to_string())` for `A1…T4`, `Some("ICP-Brasil")`
  for `Other(_)` or `None` level on an ICP-Brasil certificate, `None` otherwise.
- `eidas`: `Some { qualified: compliance, qscd: sscd, types }` when
  `qualified` is `Some`, mapping `ESign/ESeal/Web` → `esign/eseal/web`.
- `key_storage`: `Some(true)` → `Hardware`, `Some(false)` → `Software`,
  `None` → `Unknown`.

The other functions of `present::wire` are total mappings (already
implemented).
