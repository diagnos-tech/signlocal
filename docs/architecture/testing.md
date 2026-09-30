# Testing

How every part is proven, and what CI requires before anything merges.

## 1. Blind TDD for pure logic

For every pure crate or module family (`websign-core`, `websign-protocol`,
`websign-i18n`, `websign-ui-model`, the host engine, the SDK/extension/client
pure modules):

1. **Specification** (Opus): public API in code with `todo!()` bodies (this
   skeleton) plus a `SPEC.md` with behavior, errors, edge cases and vectors.
2. **Tests and implementation in parallel, blind** (Sonnet, separate git
   worktrees from the same commit): one agent writes tests from `SPEC.md`,
   another writes the code; neither reads the other's work. Assumptions the
   spec does not cover are marked `// SPEC:` for the reviewer.
3. **Critical review** (Opus): merges, runs, decides every divergence by the
   spec and the standards, writes the decision into `SPEC.md`, reviews
   security and quality.

Promoted kit code (`probe-core`, keystore adapters, registration, framing,
launch detection) already went through this; its tests came along and must
stay green.

Rules for tests:

- Vectors come from standards and `docs/ux.md` §16, or from OpenSSL-generated
  fixtures (`crates/websign-core/tests/fixtures`), never from the
  implementation's output.
- One test file per spec section; names describe behavior
  (`rejects_a_digest_of_the_wrong_length`).
- Time is injected (manual clocks), never slept.

## 2. Contract tests for platform code

Adapters that talk to an OS or a driver cannot be blind-TDD'd; they share a
**contract suite** written once against the trait:

| Contract | Suite | Runs against |
|---|---|---|
| `Keystore` | `websign_keystores::contract::run` | Linux: SoftHSM2 directly and via p11-kit; Windows: CNG software KSP, legacy CSP keys (A1-like), SoftHSM2 for Windows; macOS: temporary keychain, SoftHSM2 (Homebrew) |
| Registration | `crates/websign-registration/tests` | real user folders in a throwaway `HOME`/`HKCU` sandbox per OS |
| Caller identity | `app/tests/caller.rs` | spawns itself through signed and unsigned helper binaries |
| Device monitor | `crates/websign-devices/tests` | `pcscd` without readers (Linux); reader emulation is manual |

The contract checks (`crates/websign-keystores/SPEC.md` §7) include: `list`
never prompts (Windows runs with `silent`, macOS with a keychain set to
never ask, PKCS#11 without a PIN), every listed key signs every
supported hash × algorithm and verifies, wrong PIN → `WrongPin` and the
count-low flag, sessions stay logged in until `end_sessions` (D5),
`CKA_ALWAYS_AUTHENTICATE` asks every time.

Real tokens are covered by the manual matrix ([compatibility.md](compatibility.md))
using `websign doctor` and the e2e build.

## 3. UI tests

- **State machines** (`websign-ui-model`): blind TDD, no egui.
- **Rendering** (`app/src/ui`): `egui_kittest` tests query the AccessKit tree
  ("Sign button disabled during the first 600 ms", "PIN field value not
  exposed", "focus starts on the selected row") and take snapshots of every
  window state in light and dark.
- Snapshot baselines are **per platform** (Linux, macOS, Windows) with a
  tolerance: fonts rasterize slightly differently (proof 5 measured PNG size
  differences of ~0.1 %). Baselines are reviewed by the UI reviewer, never
  auto-accepted in CI.

## 4. Renderer

Evidence: [`docs/prototypes/5-ui-screenshots.md`](../prototypes/5-ui-screenshots.md)
(CI on 7 systems).

- wgpu renders everywhere a window opens: Windows DX12 WARP ("Microsoft
  Basic Render Driver", which also covers RDP and GPU-less VMs), macOS Metal,
  Linux Vulkan llvmpipe.
- glow fails on Windows (`egui_glow requires opengl 2.0+`) and works on macOS
  and Linux.
- Contract (`app/src/ui/renderer.rs`): the app uses **wgpu**; if wgpu cannot
  start on macOS or Linux, the process **re-executes itself** with
  `WEBSIGN_RENDERER=glow` (a second winit event loop in one process is only
  proven on X11). No glow fallback on Windows.
- `egui_kittest` headless rendering (wgpu, software adapters) works on all 7
  systems, including bare containers without X libraries.

## 5. E2E

`e2e/` (Playwright) drives: fixture page → SDK → extension (unpacked, loaded
in Chromium) → app built with `--features e2e` → software keys. The `e2e`
feature (never in release builds; `cargo xtask check release` fails if the
release binary contains the marker `WEBSIGN_E2E_BUILD`) makes the
confirmation window:

- confirm by itself once armed — after the real 600 ms, through the same code
  path as a click (`WEBSIGN_E2E_CONFIRM=sign|choose|cancel`);
- type `WEBSIGN_E2E_PIN` into our PIN field for PKCS#11 keys;
- save a PNG of every state it shows (`WEBSIGN_E2E_SCREENSHOTS=<dir>`), via
  `ViewportCommand::Screenshot`.

Scenarios (each asserts the signature verifies against the certificate with
an independent verifier):

1. sign SHA-256/384/512 with every software key (RSA PKCS#1 v1.5, RSA-PSS,
   ECDSA P-256/P-384/P-521);
2. `certificates()` on a new site (choose mode) and on a remembered site (no
   window);
3. D11: new site cancels before Continue → the page received no certificate;
4. switch certificate → new `need_digest`, stale digest ignored;
5. cancel → `UserCancelled`; tab closed → window shows "site cancelled";
6. `ExtensionMissing` (extension not loaded), `AppMissing` (manifest removed),
   `AppOutdated` (fake old app);
7. desktop: `websign sign` and `@websign/desktop` against the same keys;
8. log audit: the run's log contains none of the fixture holder names.

### Matrix

| OS | Runner | Browser | Keys | Screenshots |
|---|---|---|---|---|
| Windows 11/Server 2025 | `windows-latest` | Chromium, Edge | CNG software KSP, legacy CSP, SoftHSM2 | real window + kittest |
| macOS (arm64) | `macos-latest` | Chromium | temporary keychain, SoftHSM2 | real window + kittest |
| Ubuntu 24.04, 22.04 | runners, Xvfb + mesa | Chromium | SoftHSM2 direct and via p11-kit | real window + kittest |
| Debian 12 | container, Xvfb | distro Chromium | SoftHSM2 | kittest (+ real window once X11 client libs are installed) |
| Fedora 42 | container, Xvfb | distro Chromium | SoftHSM2 | kittest (+ real window) |
| Rocky Linux 9 (RHEL family) | container, Xvfb | Chromium (EPEL) | SoftHSM2 | kittest (+ real window) |
| Arch Linux (best effort) | container, Xvfb | Chromium | SoftHSM2 | kittest (+ real window) |

Containers need, for real windows, the X11 client libraries winit loads with
`dlopen`: Debian `libx11-6 libxcursor1 libxi6 libxrandr2`; Fedora/Rocky
`libX11 libXcursor libXi libXrandr`; Arch `libx11 libxcursor libxi libxrandr`
(plus the packages listed in proof 5). Firefox is tested manually (Playwright
cannot load extensions in Firefox).

### Screenshots

Every window state of `docs/ux.md` §4.8 and each diagnostics tab, light and
dark, per OS, land in `docs/screenshots/<os>/<window>-<state>-<theme>.png`
(`cargo xtask screenshots --from <artifact dir> --os <name>`, committed from CI
artifacts by the orchestrator). The folder's `SUMMARY.md` and an index page
are regenerated by the same command.

## 6. CI gates (all required)

| Gate | Command |
|---|---|
| Rust format | `cargo fmt --all --check` |
| Rust lints, every OS | `cargo clippy --workspace --all-targets -- -D warnings` (Linux, Windows, macOS runners) |
| Rust tests | `cargo test --workspace` on the three OSes |
| Keystore contract | per-OS jobs with software keys |
| Licenses/advisories | `cargo deny check` |
| Coverage | `cargo llvm-cov` ≥ 90 % lines for `websign-core`, `websign-protocol`, `websign-host`, `websign-ui-model` |
| Mutation (weekly) | `cargo mutants` on `websign-core` and `websign-ui-model`, ≥ 85 % killed |
| Fuzzing (nightly, 10 min each) | `cargo fuzz` on the DER reader, framing, `parse_client_message` |
| TypeScript | `bun run typecheck`, `bunx biome check .`, `bun run test` |
| Sizes | SDK < 5 KB gzip; popup (HTML+CSS+JS+SVG) < 15 KB |
| Generated files | `cargo xtask check generated` |
| i18n | `cargo xtask check i18n` |
| SUMMARY.md | `cargo xtask check summaries` |
| Release hygiene | `cargo xtask check release --binary <release websign>` |
| E2E | the matrix above on every PR touching app/extension/sdk/protocol (macOS and containers nightly) |
