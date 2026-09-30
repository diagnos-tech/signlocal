# Contributing

## License and DCO (required)

Every commit needs a `Signed-off-by: Name <email>` line (`git commit -s`). With
it you agree to the [Developer Certificate of Origin 1.1](https://developercertificate.org/)
and that your contribution is licensed under the license of the part you
changed:

| Part | License |
|---|---|
| `app/`, `crates/` (except the two below), `extension/`, `safari/`, `packaging/`, `scripts/`, `e2e/`, `xtask/`, `i18n/`, `docs/` | GPL-3.0-or-later **with the app-store additional permission** (section 7, in [`LICENSE`](LICENSE)) |
| `sdk/`, `clients/`, `crates/websign-protocol/`, `crates/websign-project/` | Apache-2.0 |
| `devices.json`, `devices.schema.json` | CC0-1.0 |

The additional permission lets the app ship through the Mac App Store and the
Microsoft Store. Contributions that do not accept it cannot be merged.

New dependencies must be GPL-3.0-compatible (MIT, Apache-2.0, BSD, ISC, Zlib,
MPL-2.0; `cargo deny check` enforces it). Apache-2.0 parts must not depend on
GPL parts. The SDK and the client libraries have **zero runtime dependencies**.

## Commits

```
<type>(<scope>): imperative description in English
```

- Header at most 100 characters.
- Types: `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `build`, `ci`, `chore`.
- Scopes: `app`, `core`, `protocol`, `keystores`, `devices`, `host`,
  `registration`, `i18n`, `ui`, `extension`, `sdk`, `clients`, `safari`,
  `packaging`, `e2e`, `xtask`, `devices-db`, `docs`, `kit`.
- One commit, one subject.

Example: `feat(keystores): keep PKCS#11 sessions logged in until the token leaves`

## Development flow (TDD in three roles)

1. **Specification** — the public interface (types and signatures) and the
   expected behavior, including errors, in a `SPEC.md` next to the code.
2. **Blind tests and implementation, in parallel** — whoever writes the tests
   does not see the implementation and vice versa; both start from the spec.
   Divergences reveal ambiguities in the spec, not only bugs.
3. **Critical review** — someone experienced runs everything, decides each
   divergence against the spec and the standards, fixes it and records the
   decision in the spec.

Platform code is proven by contract test suites instead
([`docs/architecture/testing.md`](docs/architecture/testing.md)).

## Style

- Small files (ideally < 200 lines), one concept per file, folders by concept.
- Names that explain; comments explain **why**, never the obvious.
- Everything in English: code, comments, docs, commit messages.
- Every folder has a `SUMMARY.md`; every component root a `README.md`
  ([format](docs/architecture/repository-layout.md#summarymd)).
- Pending maintainer decisions as `TODO(gustavo)`; no references to plans or
  phases in code.
- Rust: `cargo fmt`, `cargo clippy --all-targets -- -D warnings` clean on
  every OS. TypeScript: strict `tsc`, Biome, Vitest.

## Security

Never log a PIN, a certificate, a name, a CPF/CNPJ, a serial number or a
digest. Vulnerabilities: do not open a public issue — write to the
maintainer. `TODO(gustavo)`: create `SECURITY.md` with the contact address.
