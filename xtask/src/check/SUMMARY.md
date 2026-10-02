# xtask/src/check

- `summaries/` — unit tests of the SUMMARY.md rules
- `versions/` — unit tests of the version lockstep check
- `generated.rs` — generated files equal a fresh `gen`
- `glob.rs` — the `*` and `?` wildcards of SUMMARY.md rows
- `i18n.rs` — every locale against en.toml with the websign-i18n checker
- `installers.rs` — install.sh and install.ps1 identifiers equal project.toml
- `listing.rs` — repository files as git sees them, grouped by folder
- `mod.rs` — `cargo xtask check`: runs the selected checks and reports every problem
- `release.rs` — release inputs: versions, license texts, unsigned-build notice, installer identifiers, e2e marker in the binary
- `summaries.rs` — every folder has a SUMMARY.md covering each entry exactly once
- `summary_file.rs` — parses the rows of one SUMMARY.md
- `versions.rs` — Cargo, package.json, project.toml and tag versions agree
