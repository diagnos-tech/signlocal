# linux

Scripts of the Linux proof with software keys.

- `ci-linux.sh` — runs `list`, `sign`, p11-kit, deduplication, `devices`, `report`, and the end-to-end test; fails on any unmet expectation and cleans up on exit
- `softhsm-setup.sh` — creates an isolated SoftHSM2 token with RSA-2048 and EC P-256/P-384/P-521 keys (optionally a `CKA_ALWAYS_AUTHENTICATE` key)
