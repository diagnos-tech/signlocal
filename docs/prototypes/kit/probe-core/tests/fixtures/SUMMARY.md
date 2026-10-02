# probe-core/tests/fixtures

Test data made by OpenSSL 3 and committed, so tests never run OpenSSL and no expected value comes from the code under test.

- `README.md` — how to regenerate the fixtures and what each certificate and vector is (currently in Portuguese)
- `certs/` — the DER certificates
- `gen/` — the generator, split by topic
- `generate.sh` — regenerates every certificate and vector
- `vectors/` — text manifests of reference values
