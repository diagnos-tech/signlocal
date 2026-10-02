# probe/src/keystores/pkcs11

PKCS#11 key sources, one keystore per loaded module.

- `always_authenticate.rs` — keys with `CKA_ALWAYS_AUTHENTICATE`: context login and raw `C_Sign`
- `ckr.rs` — raw `CKR_*` return codes and names
- `discovery.rs` — which module files to try: requested, p11-kit registered, and known vendor paths
- `errors.rs` — translation of `cryptoki` errors into typed keystore errors
- `file_id.rs` — file identity, so one library reached through several paths loads once
- `finder.rs` — finds again, at signing time, the certificate and key that `list` reported
- `keystore.rs` — one loaded module as a `Keystore`
- `known_paths/` — per-OS lists of vendor install paths
- `listing.rs` — lists signing certificates on every token without a PIN
- `locator.rs` — the opaque handle (`slot=3;id=0a1b`) that lets a listed key be found again
- `login.rs` — `C_Login`: the only place a PIN leaves its `SecretString`
- `mechanism.rs` — which mechanism signs an already-computed digest, and how to normalize the answer
- `mod.rs` — module overview and the PKCS#11 source
- `module.rs` — loads a module once per process
- `objects.rs` — reads certificates and private key identifiers from a token session
- `p11kit/` — parsing of p11-kit `.module` registrations
- `path_patterns.rs` — expansion of Windows environment variables and one wildcard directory in path templates
- `provider.rs` — how a token is described to people, and whether it counts as hardware (never uses the label or serial)
- `signing.rs` — one signature with a PKCS#11 private key
