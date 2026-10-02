# crates/websign-keystores/tests/support

- `mod.rs` — the shared module every integration test binary includes
- `softhsm-fixture.sh` — creates an isolated SoftHSM2 token: RSA, NIST and Brainpool EC keys, a `CKA_ALWAYS_AUTHENTICATE` key and a root/intermediate CA chain
- `softhsm.rs` — reads the fixture and re-runs the test binary with `SOFTHSM2_CONF` set (SoftHSM reads it once per process)
