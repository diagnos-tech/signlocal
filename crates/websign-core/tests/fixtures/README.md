# websign-core test fixtures

DER certificates and reference vectors generated with **OpenSSL 3** by
`generate.sh`. The tests do **not** run OpenSSL: they read the files committed
here. Every expected value in the tests comes from the generator's
configuration (listed below) or from the manifests in `vectors/`, which
OpenSSL itself fills in (`openssl x509 -fingerprint`, `-serial`, `-startdate`,
`asn1parse`, `pkeyutl`); none comes from the implementation's output.
Promoted from the Phase-0 kit (`probe-core`); the fixture message keeps its
original text, because the digests were computed over it.

## How to regenerate

```
tests/fixtures/generate.sh            # needs bash >= 4, openssl >= 3.0, od, GNU date
KEEP_WORKDIR=1 tests/fixtures/generate.sh   # keeps keys and CSRs in a temporary folder
```

`ADDITIVE=1 tests/fixtures/generate.sh` adds only the Brainpool and personal-name
fixtures next to the committed ones (the committed Brainpool and `dn-pii*`
files were made that way, so their issuer key differs from `ca.der`'s; nothing
checks certificate signatures).

It takes 1 to 3 minutes. **Keys are random** and not committed (tests never
need a private key), so each run produces different bytes. This is safe
because:

- no test embeds a fingerprint, signature or key: all of them come from
  `vectors/*.txt`, regenerated together with the certificates;
- the other values (RSA bits, curve, names, dates, serial numbers, OIDs,
  KeyUsage bits) come from the configuration, not from key material.

Layout: `certs/*.der` (143 certificates), `vectors/*.txt` (manifests),
`gen/*.sh` (the generator, split by subject).

## Authority and default certificate

`ca.der` is the self-signed root (P-256, serial `01`, critical `CA:TRUE`, KeyUsage
`keyCertSign,cRLSign`): `C=BR, O=SignLocal Test Authority, OU=Fixtures Root,
CN=SignLocal Test Root CA`.

Every "ordinary" leaf (the generator's `leaf`/`variant` functions) has:

| Field | Value |
|---|---|
| subject | `C=BR, O=SignLocal Test Fixtures, OU=Unit A, OU=Unit B, CN=Fixture <file name>` (all UTF8String, except C) |
| issuer | the root above |
| validity | 2020-01-01T00:00:00Z = **1577836800** to 2040-01-01T00:00:00Z = **2208988800** |
| BasicConstraints | `CA:FALSE` (empty SEQUENCE) |
| KeyUsage (critical) | `digitalSignature`, `nonRepudiation` |
| EKU | `1.3.6.1.5.5.7.3.4`, `1.3.6.1.5.5.7.3.2` (in this order) |
| serial | sequential from 0x1000 (not tested, except in `serial-*`) |

OpenSSL adds SubjectKeyIdentifier/AuthorityKeyIdentifier; the SPEC says to ignore them.
Every certificate that says "p256 key" shares **the same EC P-256 key** as
`p256.der`; that is why the `p256` signature also verifies against them.

## Key types

| File | Key | Expected in `CertInfo.key` |
|---|---|---|
| `rsa2048`, `rsa2048b` | RSA 2048 (two different keys) | `Rsa { bits: 2048 }` |
| `rsa3072` | RSA 3072 | `Rsa { bits: 3072 }` |
| `rsa4096` | RSA 4096 | `Rsa { bits: 4096 }` |
| `rsa2047` | 2047-bit modulus | `Rsa { bits: 2047 }` |
| `rsapss2048` | OID RSASSA-PSS `1.2.840.113549.1.1.10`, no parameters | `Rsa { bits: 2048 }` |
| `p256`, `p256b` | EC P-256 (two keys) | `Ec { curve: P256 }` |
| `p384`, `p521` | EC P-384 / P-521 | `Ec { curve: P384 / P521 }` |
| `ed25519` | Ed25519 | `Unsupported { oid: "1.3.101.112" }` |
| `ed448` | Ed448 | `Unsupported { oid: "1.3.101.113" }` |
| `dsa1024` | DSA | `Unsupported { oid: "1.2.840.10040.4.1" }` |
| `secp256k1` | EC, unknown curve | `Unsupported { oid: "1.3.132.0.10" }` |
| `secp224r1` | EC, unknown curve | `Unsupported { oid: "1.3.132.0.33" }` |
| `brainpoolP256r1`, `brainpoolP384r1`, `brainpoolP512r1` | EC on the Brainpool `r1` curves (SPEC §4.1) | `Ec { curve: BrainpoolP256r1 / BrainpoolP384r1 / BrainpoolP512r1 }` |
| `brainpoolP256t1`, `brainpoolP384t1`, `brainpoolP512t1` | EC on the twisted curves, which stay unknown | `Unsupported { oid: "1.3.36.3.3.2.8.1.1.8" / "…1.12" / "…1.14" }` |

## Extensions (p256 key)

| File | Content | Expected |
|---|---|---|
| `ca-pathlen0` | `CA:TRUE,pathlen:0`, no KeyUsage | `is_ca`, `key_usage None`, `can_sign false` |
| `ca-digital-signature` | `CA:TRUE`, KeyUsage `digitalSignature,keyCertSign,cRLSign` | `is_ca`, `can_sign false` |
| `ku-<bit>` | a single KeyUsage bit: `digital-signature`, `non-repudiation`, `key-encipherment`, `data-encipherment`, `key-agreement`, `key-cert-sign`, `crl-sign` | exactly that bit `true`; `can_sign` only for the first two |
| `ku-decipher-only` | only `decipherOnly` (bit 8, outside the struct) | `Some(KeyUsage::default())`, `can_sign false` |
| `ku-all` | the 7 bits + `encipherOnly` + `decipherOnly`, `CA:FALSE` | the 7 `true`, `is_ca false`, `can_sign true` |
| `eku-multi` | EKU: `1.3.6.1.5.5.7.3.4`, `1.3.6.1.4.1.311.10.3.12`, `1.3.6.1.5.5.7.3.2`, `2.5.29.37.0`, `1.3.6.1.5.5.7.3.1` | same order |
| `eku-server-only` | EKU only `serverAuth` + `digitalSignature` | `can_sign true` (EKU does not count) |
| `policies-multi` | `1.2.3.4`, `2.23.140.1.2.2`, `1.3.6.1.4.1.99999.1.2` | same order |
| `policies-qualifiers` | `1.3.6.1.4.1.99999.2.1` (with CPS and UserNotice), `1.2.3.4.5` | only the OIDs, in order |
| `bare` | only `nsComment` (v3, no KeyUsage/BC/EKU/policies) | everything empty/`None`, `can_sign true` |
| `v1` | version 1, no extensions | same |
| `tiny` | self-signed v1, empty names, about 233 bytes, under 256 (outer SEQUENCE `30 81 xx`) | empty subject/issuer, serial `01` |
| `unknown-extension` | KeyUsage `digitalSignature` + unknown critical extension `1.3.6.1.4.1.99999.9` with garbage | ignored; `key_usage` only `digital_signature` |

## Serial, dates and names (p256 key)

| File | Expected |
|---|---|
| `serial-01`, `serial-7f`, `serial-1234`, `serial-0102` | `"01"`, `"7f"`, `"1234"`, `"0102"` |
| `serial-80` | content `00 80` → `"80"` |
| `serial-ff20` | twenty `ff` bytes (content with a leading `00`) → `"ff"` × 20 |
| `time-window` | 2024-03-15T10:20:30Z (**1710498030**) to 2024-09-15T18:40:50Z (**1726425650**) |
| `time-pre-1970` | UTCTime `600101000000Z` = 1960-01-01 (**-315619200**) to 2049-12-31T23:59:59Z (**2524607999**, still UTCTime) |
| `time-generalized` | GeneralizedTime: 2050-01-01 (**2524608000**) to 2051-01-01 (**2556144000**) |
| `time-y2k` | 1999-12-31T23:59:59Z (**946684799**) to 2000-01-01T00:00:00Z (**946684800**) |
| `time-no-expiry` | `notAfter` = 9999-12-31T23:59:59Z (**253402300799**) |
| `expired` | 2010-01-01 (**1262304000**) to 2011-01-01 (**1293840000**) |
| `dn-duplicates` | `C=BR,C=US,O=First,O=Second,OU=A,OU=B,OU=C,CN=One,CN=Two` → the first C/O/CN wins; OU `[A,B,C]` |
| `dn-multivalue-rdn` | multi-valued RDN `OU=Unit+CN=Multi` |
| `dn-empty`, `dn-country-only`, `dn-org-only` | empty subject; only `C=BR`; only `O=Only Org Name` |
| `dn-utf8` | UTF8String: CN `JOSÉ AÇAÍ DA SILVA`, O `Açougue & Cia Ltda`, OU `Divisão São João` |
| `dn-printable` | PrintableString: CN `PRINTABLE NAME 123`, O `Printable Org`, OU `Printable Unit` |
| `dn-teletex` | TeletexString (Latin-1): CN `JOSÉ TELETEX`, O `Ação Ltda`, OU `Divisão` |
| `dn-bmp` | BMPString (UTF-16BE): CN `JOSÉ Ω BMP`, O `Êxito Ltda`, OU `Divisão` |
| `dn-ia5` | IA5String: CN `ia5.name.example.com`, O `ia5.org.example`, OU `ia5.unit` |
| `dn-numeric` | O `Numeric Org`; OU `First`, **`222` (NumericString)**, `Third`; **CN `12345` (NumericString)** → CN and the middle OU ignored |
| `dn-universal` | CN in UniversalString (ignored); O `Universal Org`, OU `Unit` |

`dn-ia5`, `dn-numeric` and `dn-universal` are made in two steps: OpenSSL does
not put IA5/Numeric/Universal strings in CN/O/OU, so the certificate is born
with PrintableString and the generator swaps the tag (`patch_cert`/
`retag_string`) without changing the length (for Universal, the content
becomes UCS-4BE). **The signature of these three no longer verifies**, which
does not matter: `CertInfo` does not check the certificate's signature.

## Personal name attributes (SPEC §6.1a)

Invented people. All use the `p256` key in a full regeneration.

| File | Subject | Expected (`given_name`, `surname`, `serial_number`) |
|---|---|---|
| `dn-pii` | `C=PT, O=SignLocal Test Fixtures, GN=José Ângelo, SN=Conceição, serialNumber=IDCPT-12345123, CN=JOSÉ ÂNGELO CONCEIÇÃO` (UTF8String, serialNumber PrintableString) | `José Ângelo`, `Conceição`, `IDCPT-12345123` |
| `dn-pii-only` | `C=PT, GN=MARIA, SN=SILVA, serialNumber=IDCPT-12345123` (no CN or O) | `MARIA`, `SILVA`, `IDCPT-12345123` |
| `dn-pii-duplicates` | two of each: `First`/`Second`, `One`/`Two`, `IDCPT-1111111`/`IDCPT-2222222` | the first of each |
| `dn-pii-unreadable-first` | as above with `First`, `Alpha` and `111` as NumericString (retagged) before `Second`, `Beta`, `IDCPT-2222222` | `Second`, `Beta`, `IDCPT-2222222` |
| `dn-pii-bmp` | BMPString `GN=JOSÉ Ω`, `SN=ÊXITO`, no serialNumber | `JOSÉ Ω`, `ÊXITO`, `None` |

## Malformed known extensions

Each carries an INTEGER `02 01 05` where the extension type requires a BIT
STRING or SEQUENCE. All must give `CertError::Malformed`:
`bad-ext-key-usage`, `bad-ext-extended-key-usage`, `bad-ext-policies`,
`bad-ext-basic-constraints`, `bad-ext-san`, `bad-ext-qc-statements`, and
`bad-qc-statement-element` (qcStatements = `SEQUENCE { INTEGER 5 }`).

## ICP-Brasil

Person otherName value (`2.16.76.1.3.1`) = birth date (8) + CPF (11) + NIS
(11) + RG (15) + issuing body (6): `15051990` `12345678901` `98765432109`
`000000123456789` `SSP-SP`. Company officer (`.3.4`): `10101980` + CPF
`98765432100` + zeros + `SSP-SP`. CNPJ (`.3.3`): `12345678000195`. A second CPF,
`22222222222`, tells which otherName the CPF came from. Subject:
`C=BR, O=ICP-Brasil, OU=Autoridade Certificadora Teste, OU=Certificado <name>, CN=...`.

| File | CN | Policy / SAN | Expected (`IcpBrasil`) |
|---|---|---|---|
| `icp-pf-a3` | `ANA BEATRIZ SOUZA:12345678901` | `2.16.76.1.2.3.1`; e-mail, `.3.1` (OCTET STRING), `.3.5`, UPN | `A3`, `ANA BEATRIZ SOUZA`, cpf `12345678901`, cnpj `None` |
| `icp-pf-printable` / `-utf8` / `-ia5` | same | `.3.1` as PrintableString / UTF8String / IA5String | same as `icp-pf-a3` |
| `icp-pf-bmp` | same | `.3.1` as BMPString | cpf `None` (type not accepted), rest the same |
| `icp-pf-len-19` / `-len-18` / `-len-8` | same | value with 19 / 18 / 8 characters | cpf `12345678901` / `None` / `None` |
| `icp-pf-cpf-letter` | same | CPF `1234567890X` | cpf `None` |
| `icp-pf-cpf-zeros` | same | CPF `00000000000` | cpf `None` |
| `icp-pf-odd-birth-tail` | same | `ABCDEFGH` + CPF + `tail-with-letters` | cpf `12345678901` |
| `icp-pf-priority` | same | `.3.4` (CPF `22222222222`) before `.3.1` (CPF `12345678901`) | cpf `12345678901` |
| `icp-pf-invalid-primary` | same | `.3.1` with the birth date only; valid `.3.4` (`22222222222`) | cpf `None` (does not fall back to `.3.4`; SPEC §6.3) |
| `icp-only-34` | same | only `.3.4` (`22222222222`) | cpf `22222222222` |
| `icp-pj-a1` | `EMPRESA TESTE LTDA:12345678000195` | `2.16.76.1.2.1.1`; `.3.2`, `.3.3`, `.3.4` | `A1`, `EMPRESA TESTE LTDA`, cpf `98765432100`, cnpj `12345678000195` |
| `icp-pj-printable` / `-utf8` / `-ia5` | same | `.3.3` and `.3.4` as the named type | same as `icp-pj-a1` |
| `icp-pj-cnpj-13` / `-15` / `-zeros` / `-letter` | same | CNPJ with 13 digits / 15 / zeros / a letter | cnpj `None`, cpf `98765432100` |
| `icp-pj-no-responsible` | same | only `.3.3` | cnpj `12345678000195`, cpf `None` |
| `icp-cn-plain`, `-suffix`, `-alpha-suffix`, `-mixed-suffix`, `-space-suffix`, `-nested-colon`, `-accented` | `MARIA SILVA`; `MARIA SILVA:12345678901`; `…:ABC`; `…:123ABC`; `…: 123`; `R2:D2:12345678901`; `JOSÉ D'ÁVILA:12345678901` | `2.16.76.1.2.1.1` | holder: `MARIA SILVA`; `MARIA SILVA`; `MARIA SILVA:ABC`; `MARIA SILVA:123ABC`; `MARIA SILVA: 123`; `R2:D2`; `JOSÉ D'ÁVILA` |
| `icp-no-cn` | (no CN) | `2.16.76.1.2.1.1` | holder `None`, level `A1` |
| `icp-policy-only` | `POLICY ONLY:11111111111` | only the `A1` policy | `Some`, no CPF/CNPJ |
| `icp-san-only` | `SAN ONLY:11111111111` | only a valid `.3.1`, no policy | `Some`, level `None`, cpf `12345678901` |
| `icp-san-other-arc` | `OTHER ARC:11111111111` | only `.3.2` | `Some`, level `None`, no CPF/CNPJ |
| `icp-lookalike` | same | policies `2.16.76.1.20.3`, `2.16.760.1.2.3.1`; otherName `2.16.76.1.30.1` | `icp_brasil None` |
| `non-icp-upn` | `PLAIN NAME:12345678901` | e-mail, DNS, UPN `1.3.6.1.4.1.311.20.2.3` | `icp_brasil None` |
| `icp-multi-policy` | `ANA BEATRIZ SOUZA:…` | `1.2.3.4`, `2.16.76.1.2.3.4`, `2.16.76.1.2.1.2` | level `A3` (first ICP policy) |
| `icp-multi-policy-other-first` | same | `2.16.76.1.2.999.1`, `2.16.76.1.2.3.1` | level `Other(999)` |
| `icp-level-<n>` | `LEVEL <n>:11111111111` | `2.16.76.1.2.<n>.1` | see the table below |
| `icp-level-no-subarc` | same | `2.16.76.1.2.3` (no more arcs) | `A3` |
| `icp-level-huge` | same | `2.16.76.1.2.4294967296.1` | policy listed as is, level `None` (does not fit `u32`) |

`icp-level-<n>`, with n = 0, 1, 2, 3, 4, 5, 100, 101, 102, 103, 104, 105, 302,
303, 304, 305, 999, 4294967295:

| n | 0 | 1 | 2 | 3 | 4 | 5 | 100 | 101 | 102 | 103 | 104 | 105 | 302 | 303 | 304 | 305 | 999 | 4294967295 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| level | `Other(0)` | `A1` | `A2` | `A3` | `A4` | `Other(5)` | `Other(100)` | `S1` | `S2` | `S3` | `S4` | `Other(105)` | `Other(302)` | `T3` | `T4` | `Other(305)` | `Other(999)` | `Other(4294967295)` |

## eIDAS (qcStatements)

Subject `C=PT, O=Test Qualified Provider, CN=QC fixture <name>`, KeyUsage
`nonRepudiation`. Statements: `0.4.0.1862.1.<n>` (1 QcCompliance, 2 QcLimitValue,
3 QcRetentionPeriod, 4 QcSSCD, 5 QcPDS, 6 QcType; 6.1 esign, 6.2 eseal, 6.3 web).

| File | Statements, in order | Expected (`Qualified`) |
|---|---|---|
| `qc-esign-sscd` | 1.2 (EUR 1000), 1.1, 1.3 (15), 1.4, 1.5 (URL+`en`), 1.6 `{6.1}`, `1.2.3.4.5` | compliance, sscd, `[ESign]` |
| `qc-eseal` | 1.1, 1.6 `{6.2}` | compliance, no sscd, `[ESeal]` |
| `qc-web` | 1.6 `{6.3}` | no compliance, no sscd, `[Web]` |
| `qc-type-order` | 1.6 `{6.3, 6.9, 6.1, 6.2}` | `[Web, ESign, ESeal]` (6.9 ignored) |
| `qc-sscd-only` | 1.4 | sscd only |
| `qc-unknown-only` | `1.2.3.4.5` | `Some(default)` |
| `qc-empty` | `SEQUENCE {}` | `Some(default)` |
| `qc-type-empty-info` | 1.6 with `SEQUENCE {}` | `Some(default)` |

## Manifests (`vectors/`)

Text, `#` starts a comment, one record per line, fields separated by spaces.

| File | Record |
|---|---|
| `digests.txt` | `hash hex(digest)` of the message `SignLocal probe-core fixture message` (`openssl dgst`) |
| `digestinfo.txt` | `hash hex(DigestInfo)`, built by OpenSSL's ASN.1 generator from the hash OIDs and checked against a "raw" RSA signature (`rsautl -pkcs`) equal to what `pkeyutl -pkeyopt digest:` produces |
| `signatures.txt` | `key algorithm hash hex(signature)` over the digest above; `key` is the certificate name. `pkcs1`, `pss` (salt = digest length, MGF1 with the same hash), `ecdsa` (raw `r‖s`). Keys: `rsa2048`, `rsa2048b`, `rsa3072`, `rsa4096`, `rsa2047` (pkcs1 and pss × 3 hashes), `rsapss2048` (pss × 3), `p256`, `p256b`, `p384`, `p521` (ecdsa × 3) = 45 valid. Plus 9 **deliberately invalid**, on `rsa2048`: `pss-salt-0`, `pss-salt-20`, `pss-mgf1-sha1` × 3 hashes |
| `ecdsa.txt` | `curve shape hash hex(raw) hex(DER)`: the same signature in both formats; the raw form is read back by OpenSSL's `asn1parse`. `shape`: `plain` (no leading zero, no high bit), `high-r`, `high-s`, `high-both` (high bit: the DER gets a `00`), `short-r`, `short-s` (first byte zero: the DER is shorter than the field). One line per shape and curve; P-521 always uses the long length `81 xx` |
| `brainpool-signatures.txt` | `key ecdsa hash hex(raw r‖s)` over the digest above, for `brainpoolP256r1`, `brainpoolP384r1` and `brainpoolP512r1` × 3 hashes (the last has no verifier: `UnsupportedKey`) |
| `brainpool-ecdsa.txt` | same format as `ecdsa.txt`, for the three Brainpool curves (kept apart so tests that walk `ecdsa.txt` keep their NIST-only curve lists) |
| `fingerprints.txt` | `name sha256(DER)` of every certificate (`openssl x509 -fingerprint`) |
| `validity.txt` | `name serial notBefore notAfter` of every certificate, as OpenSSL reports them (Unix seconds, `date -u -d`) |
