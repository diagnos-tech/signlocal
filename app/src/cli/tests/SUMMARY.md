# app/src/cli/tests

- `golden.rs` — The JSON documents (`sign`, `choose`, `version`, reports, `doctor --json`) and exit codes callers depend on, byte for byte.
- `mod.rs` — Command-line contract tests.
- `parse.rs` — Every command and flag parses; mistakes are usage errors (exit 2); `--help` shows the examples.
