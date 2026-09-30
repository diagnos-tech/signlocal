# crates/websign-keystores/src/pkcs11/p11kit

- `mod.rs` — Modules registered with p11-kit: the `*.module` files that packages drop in a well-known directory so every PKCS#11 client finds the same drivers.
- `module_file.rs` — The `key: value` lines of one p11-kit `.module` file.
