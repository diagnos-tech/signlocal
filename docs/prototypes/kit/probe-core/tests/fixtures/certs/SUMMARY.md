# probe-core/tests/fixtures/certs

DER certificates, one per scenario. Files are grouped below by name pattern; the generator in `../gen/` defines each one.

- `bare.der`, `tiny.der`, `v1.der` — minimal, truncated, and X.509 v1 certificates
- `ca*.der` — CA certificates: plain, with `digitalSignature`, with path length 0
- `rsa*.der`, `p256*.der`, `p384.der`, `p521.der`, `brainpoolP256r1.der`, `secp224r1.der`, `secp256k1.der`, `dsa1024.der`, `ed25519.der`, `ed448.der` — one certificate per public key kind, including unsupported ones
- `ku-*.der` — one certificate per KeyUsage bit
- `eku-*.der` — extended key usage scenarios
- `policies-*.der` — certificate policies with several entries and qualifiers
- `bad-*.der` — known extensions that are malformed
- `serial-*.der` — serial number encodings
- `time-*.der`, `expired.der` — validity encodings and ranges
- `dn-*.der` — subject name string encodings and shapes
- `icp-*.der`, `non-icp-upn.der` — ICP-Brasil: levels, person and company data, name conventions, look-alikes
- `qc-*.der`, `bad-qc-statement-element.der` — eIDAS qcStatements scenarios
- `unknown-extension.der` — an unrecognized extension
