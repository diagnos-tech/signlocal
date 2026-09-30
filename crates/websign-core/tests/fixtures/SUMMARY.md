# crates/websign-core/tests/fixtures

- `certs/` — test certificates, DER
- `gen/` — the fixture generator, split by subject
- `vectors/` — reference values computed by OpenSSL
- `README.md` — how the fixtures are generated and what each certificate exercises
- `generate.sh` — regenerates every fixture (bash >= 4, OpenSSL 3)
