# probe-core/tests/fixtures/gen

The fixture generator, split by subject and sourced by `../generate.sh`.

- `certs-basic.sh` — root CA, one certificate per key type, one per extension scenario
- `certs-fields.sh` — serial numbers, validity and name encodings, malformed known extensions
- `certs-icp.sh` — ICP-Brasil policies, `otherName`s, and CN conventions
- `certs-qualified.sh` — eIDAS qualified certificates
- `keys.sh` — key pairs, kept only in the scratch directory
- `lib.sh` — helpers shared by the generators
- `oracles.sh` — per-certificate values (fingerprint, serial, validity) as OpenSSL reports them
- `vectors.sh` — reference vectors: digests, `DigestInfo`, signatures, ECDSA DER/raw pairs
