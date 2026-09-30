# app/src/cli

- `activate.rs` — `websign:activate` from the website's `/activate` page: register with every browser (stores run no install scripts), then open diagnostics with "Getting started".
- `choose.rs` — `websign choose`: the certificate the person picks, as JSON.
- `connect.rs` — `websign connect`: the framed protocol on stdin/stdout, identical to native messaging except that the caller is the parent process.
- `diagnostics.rs` — `websign diagnostics` (and `websign` with no command): the diagnostics window, in this process.
- `doctor.rs` — `websign doctor`: the diagnostics report on stdout, for support and CI.
- `install.rs` — `websign install` / `websign uninstall`: what an installer runs once, and what the app repeats silently on every start (`docs/architecture/desktop-api.md`).
- `mod.rs` — The command line: the desktop API for scripts and programs, and the installer's entry points (`docs/architecture/desktop-api.md`).
- `options.rs` — Value types shared by several commands.
- `register.rs` — `websign register`: native messaging manifests only; the kit's `websign-probe register`, kept for tests and support.
- `sign.rs` — `websign sign`: one signature from the command line.
- `version.rs` — `websign version`.
