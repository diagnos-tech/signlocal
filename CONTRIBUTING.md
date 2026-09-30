# Contributing

> Every commit must be signed off (`git commit -s`, DCO 1.1). By signing off you also
> accept that your contribution is distributed under the license of the part you
> change, including the GPL section 7 additional permission for app stores in
> [`LICENSE`](LICENSE). New behavior starts with a spec and tests.

## License and DCO (required)

Every commit needs a `Signed-off-by: Name <email>` line (use `git commit -s`).
With it you state that you agree to the [Developer Certificate of Origin 1.1](https://developercertificate.org/)
and that your contribution is licensed under the license of the part you changed:

| Part | License |
|---|---|
| `app/`, `extension/`, `safari/`, `packaging/`, `site/`, `docs/` | GPL-3.0-or-later **with the app-store additional permission** (section 7, in [`LICENSE`](LICENSE)) |
| `sdk/` | Apache-2.0 |
| `devices.json` | CC0-1.0 |

The additional permission exists so the app can be distributed through the Mac App Store
and the Microsoft Store. A contribution that does not accept this permission cannot be merged.

New dependencies must be compatible with GPL-3.0 (MIT, Apache-2.0, BSD, ISC, Zlib, MPL-2.0).
In `sdk/` the rule is stricter: **zero runtime dependencies**.

## Commits

```
<type>(<scope>): <imperative description in English>
```

- Header of at most 100 characters.
- Every commit carries a DCO sign-off (`git commit -s`).
- Types: `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `build`, `ci`, `chore`.
- Scopes: `app`, `extension`, `sdk`, `safari`, `devices`, `packaging`, `site`, `docs`, `kit`.
- One commit, one subject. `git add` only the files that belong to your work.

Example: `feat(app): list Windows certificate store entries through CNG`

## Development workflow (TDD in three roles)

All new behavior goes through three roles, which can be people or agents:

1. **Specification**: the public interface (types and signatures) and the expected
   behavior, including errors, in a `SPEC.md` next to the code or in the PR description.
2. **Blind tests and implementation, in parallel**: whoever writes the tests does not see
   the implementation and vice versa; both start from the specification only.
   Divergences reveal ambiguities in the specification, not just bugs.
3. **Critical review**: an experienced person runs everything, decides who is right in
   each divergence (in light of the specification and the standards), fixes it, and
   documents the decision.

## Style

- Small files (ideally < 200 lines), one concept per file, subfolders by topic.
- Names that need no comment; comments explain the **why**, never the obvious.
- Code, comments, and documentation are written in English.
- Pending decisions as `TODO(name)`; no references to plans or phases in code.
- Rust: `cargo fmt` and `cargo clippy -- -D warnings` must be clean.

## Security

Never log a PIN, a full certificate, a name, a CPF/CNPJ, or a digest.
Vulnerabilities: do not open a public issue; write to the maintainer.
<!-- TODO(gustavo): create SECURITY.md with the contact email for vulnerabilities. -->
