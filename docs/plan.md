# Implementation plan

**Status:** waves A and B are implemented: every crate and package works on
Linux, Windows and macOS, the CI gates run on every push, and `v*` tags publish
unsigned prereleases. The risk proofs that preceded them are in
[`docs/prototypes/`](prototypes/). Contracts are in
[`docs/architecture/`](architecture/); the UX is [`docs/ux.md`](ux.md).

What remains:

- **Maintainer actions** (§5): enable GitHub Pages for `site/`
  (`.github/workflows/pages.yml` is ready), the security contact in
  [`SECURITY.md`](../SECURITY.md), final name and domain, store and npm
  accounts.
- **Wave C** (§3): code signing and notarization, store channels, the macOS
  PKCS#11 Add-on.
- **Hardware verification**: real tokens and cards on each OS, following the
  checklist in [`docs/compatibility.md`](compatibility.md).
- **Screenshots** of Windows, macOS and the Linux distributions in
  [`docs/screenshots/`](screenshots/) from a green `e2e.yml` run.

## 1. Decisions

Final unless a new fact contradicts them. D1–D10 were approved by the
maintainer; D11–D20 were taken while defining the contracts.

| # | Decision | Why |
|---|---|---|
| D1 | `sign({ hash, algorithm?, prepare(certificate) ⇒ digest })`: one confirmation window per signature; the certificate is chosen in the window; the digest is requested after the choice | PAdES puts the certificate in the signed attributes; one window instead of two for people who sign 30 reports a day |
| D2 | `certificates()` never returns the machine's list: choose-mode window, or the remembered caller's certificates | shared clinic computers: the list would leak other doctors' names and CPFs |
| D3 | English everywhere: code, comments, docs, commits (`<type>(<scope>): <imperative>`, header ≤ 100, DCO sign-off) | public international project |
| D4 | No batch signing in v1 (the queue exists) | one confirmation per signature is the consent model |
| D5 | PKCS#11 PIN cached per session (until token removal or idle close), except `CKA_ALWAYS_AUTHENTICATE` keys | same behavior as CNG/CryptoTokenKit middlewares |
| D6 | Provisional names and IDs in `project.toml`, generated into Rust (`websign-project`), TypeScript and manifests | renaming = editing one file |
| D7 | Commits carry the maintainer as author with `Signed-off-by`; the environment's SSH key is not his | re-sign on integration if needed |
| D8 | Own tolerant DER reader (promoted `probe-core`): summarizes, never validates chains; fuzzed in CI | the `x509-cert` stack dropped real certificates |
| D9 | Windows acquires legacy CAPI keys with `PREFER` (CNG bridge → PSS works), falling back to `ALLOW` per key when a third-party CSP refuses | proven for Microsoft CSPs in proof 1 |
| D10 | Ship **direct** unsigned builds on every OS now; macOS store build and PKCS#11 Add-on later. The direct macOS build is not sandboxed, so PKCS#11 works there | proofs 2–3: the store sandbox blocks external PKCS#11 modules |
| D11 | **Certificate release needs consent**: `sign.need_digest` (which discloses the certificate) is sent at once only for a remembered caller; otherwise after the person clicks **Continue** in the window | without it any site could call `sign()` and cancel to harvest names/CPFs, defeating D2. Costs one click for new sites only. **Needs UX sign-off** (adds `action.continue`, `code.continue_hint`) |
| D12 | Protocol version = one integer; strict parsing (unknown fields refused); any catalog change bumps it; ranges negotiated in `hello` | strictness closes smuggling; ranges keep old peers working |
| D13 | `websign-protocol`, `websign-project`, `websign-client`, `@websign/sdk`, `@websign/desktop` are Apache-2.0 and depend on nothing GPL; the verification-code algorithm lives in the protocol crate | the GPL must not reach programs that only talk to the app |
| D14 | Pure view logic in `crates/websign-ui-model` (confirm state machine, list rules, diagnostics report); the app only renders | blind-TDD without egui; the host shares the list rules |
| D15 | Renderer: wgpu everywhere (WARP/llvmpipe/Metal cover RDP and VMs); glow only as a re-exec fallback on macOS/Linux | proof 5 on 7 systems; glow fails on Windows |
| D16 | Desktop caller = the parent process (path, product name, verified code signer); consent key `app:<signer>` or `path:<exe>`; unsigned callers flagged | the program cannot lie about itself |
| D17 | `websign sign|choose` print the final protocol message (no envelope) as one JSON object; exit codes from `ErrorCode::exit_code` | one schema for every interface |
| D18 | CLI output and error messages are English only; windows, popup and SDK texts are localized (7 locales) | developer surfaces vs. end-user surfaces |
| D19 | App, extension, SDK and desktop client versions move in lockstep with the `v*` tag | one compatibility story |
| D20 | Brainpool P256r1/P384r1 verified with RustCrypto `bp256`/`bp384` 0.14; P512r1 signed but not self-verified (no maintained verifier) | European cards use brainpool; no unmaintained crypto |

## 2. How we work

Blind TDD for pure logic, contract tests for platform code, a senior review
for everything ([`testing.md`](architecture/testing.md)).

- **Pure tracks** (`blind`): two Sonnet agents in separate git worktrees from
  the same commit — one writes tests from `SPEC.md`, one writes the code —
  then an Opus reviewer reconciles, fixes the spec, reviews security and
  quality.
- **Platform tracks** (`contract`): one Opus agent implements against the
  contract suite and the kit's proven code; an Opus reviewer checks unsafe
  code, handles, error mapping and the no-PIN-on-list rule.
- **UI tracks**: Opus implements rendering on top of `websign-ui-model`;
  `egui_kittest` tests and snapshots; UX review against `docs/ux.md`.
- Every track stays inside its **file boundary**, keeps files < ~200 lines,
  runs `cargo fmt`, `cargo clippy -D warnings` (all targets), tests, and for
  TypeScript `bun run typecheck`, Biome and Vitest. No commits: the
  orchestrator commits.
- A spec gap is recorded as `// SPEC:` and resolved by the reviewer; a
  maintainer decision as `TODO(gustavo)`.

## 3. Tracks

Legend — **Kind**: blind / contract / ui / tooling. **Agents**: S = Sonnet,
O = Opus. Tracks of the same wave run in parallel; boundaries never overlap.

### Wave A — contracts to code (starts now)

| Track | Files (boundary) | Kind | Agents | Depends on | Acceptance |
|---|---|---|---|---|---|
| A1 Core | `crates/websign-core/**` | blind | S+S, O review | — | `SPEC.md` §4.1 (brainpool), §6.1a (DN attributes), §9–§13 implemented; all vectors of `docs/ux.md` §16.1–16.4 pass; the 391 promoted tests still pass; coverage ≥ 90 %; mutants killed ≥ 85 % |
| A2 Protocol | `crates/websign-protocol/**`, `sdk/src/generated/**`, `extension/src/generated/**`, `clients/node/src/generated/**` | blind | S+S, O review | — | `SPEC.md` vectors pass (negotiation, ids, strict parsing, verification code §16.1); round-trip of every message; generated TS committed and fresh |
| A3 i18n engine | `crates/websign-i18n/**` | blind | S+S, O review | — | plural rules for 7 locales, fallback chain, placeholder rendering, date formats, `check_locale` vectors |
| A4 Translations | `i18n/*.toml` except `en.toml` | tooling | S (one per locale), O review | A3 checker | every key translated; `cargo xtask check i18n` green; pt-BR texts taken from `docs/ux.md` §13/§15 |
| A5 UI model | `crates/websign-ui-model/**` | blind | S+S, O review | A1 API | `docs/ux.md` §4.7, §4.8, §5, §6, §8.1–8.2, §8.7, §16.5–16.6 as tests; confirm machine scenario table passes |
| A6 PKCS#11 + hub + contract | `crates/websign-keystores/src/{lib.rs,model.rs,hub.rs,contract.rs,inventory.rs,pkcs11/**}`, `crates/websign-keystores/tests/**` | contract | O, O review | A1 | contract suite green with SoftHSM2 on Linux/macOS/Windows; D5 sessions; `pin_state`; device links; list never logs in |
| A7 Windows key store | `crates/websign-keystores/src/windows/**` | contract | O, O review | A6 trait | contract suite green on `windows-latest` (CNG, CAPI, A1 in `PROV_RSA_FULL`); D9 fallback; reader link; chain via `CertGetCertificateChain` |
| A8 macOS key store | `crates/websign-keystores/src/macos/**` | contract | O, O review | A6 trait | contract suite green on `macos-latest` (temporary keychain); CTK token link; chain via `SecTrust`; calls serialized |
| A9 Devices | `crates/websign-devices/**`, `devices.json`, `devices.schema.json` | blind (hints, possible) + contract (monitor) | S+S, O | — | `devices.json` seeded from `docs/research/tokens.md` and schema-valid; monitor events with `pcscd`; no serial numbers anywhere |
| A10 Registration | `crates/websign-registration/**` | contract | O, O review | — | detect/status/url_scheme/preregister/system per `SPEC.md`; tests in throwaway HOME/HKCU on the three OSes |
| A11 Host engine | `crates/websign-host/**` | blind (session, queue, flows, engine with fakes) + contract (runtime) | S+S, O | A1, A2, A5, A6 API | every scenario of `SPEC.md` §8 passes with fake ports; runtime serves stdio with the real key worker on Linux (SoftHSM2) |
| A12 Web SDK | `sdk/**` except `src/generated` | blind | S+S, O review | A2 | `SPEC.md` behaviors; zero dependencies; < 5 KB gzip; typed errors |
| A13 Extension | `extension/**` except `src/generated` | blind (pure modules) + ui (popup) | S+S, O review | A2, A4 | page relay and background validation per `SPEC.md`; popup's six states; < 15 KB popup; builds for chrome, edge, firefox, safari |
| A14 Node client | `clients/node/**` except `src/generated` | blind | S, O review | A2 | framing, flow, locate; tests against a fake `websign connect` |
| A15 Rust client | `clients/rust/**` | blind | S, O review | A2 | same, in Rust |
| A16 xtask | `xtask/**` | tooling | S, O review | A2, A3 | `gen`, `check summaries|i18n|generated|release`, `screenshots` working |
| A17 CI | `.github/workflows/**` (except `prototypes.yml`, `ui-spike.yml`) | tooling | O | A16 | all gates of `testing.md` §6 wired; e2e jobs stubbed until B6 |

### Wave B — the app (after A2, A5, A6, A11 APIs are stable)

| Track | Files | Kind | Agents | Depends on | Acceptance |
|---|---|---|---|---|---|
| B1 App shell & CLI | `app/src/{main.rs,launch.rs,host_process.rs,logging.rs,cli/**}` | contract | O, O review | A10, A11 | every command of `desktop-api.md` with exact flags, JSON and exit codes; host process threads per `overview.md`; log privacy audit |
| B2 Platform | `app/src/platform/**` | contract | O (per OS), O review | B1 | caller identity (Authenticode, code signature, `/proc`), channel detection, focus, reduce motion, certificate viewer, `.pfx` import |
| B3 UI foundation | `app/src/ui/{theme,fonts,icons,widgets,renderer.rs,i18n.rs}`, `app/assets/**`, `design/tokens.css` | ui | O, UX review | A3 | tokens of `docs/ux.md` §11 (test against `tokens.css`); Inter/JetBrains Mono embedded; Phosphor; light/dark; renderer per D15 |
| B4 Confirmation window | `app/src/ui/{confirm/**,bridge.rs}` | ui | O, UX review | A5, B3 | every state of §4.8 rendered; kittest AccessKit checks (arming, focus, PIN hidden); opens < 300 ms |
| B5 Diagnostics window | `app/src/ui/diagnostics/**` | ui | O, UX review | A5, A9, A10, B3 | four tabs of §8; "Copy diagnostics" golden test; add/remove driver; revoke sites |
| B6 E2E | `e2e/**`, `app/src/e2e.rs` | tooling | S, O review | B1–B4, A13 | scenarios of `testing.md` §5 green on the matrix; screenshots collected |
| B7 Packaging & installers | `packaging/**`, `scripts/install/**` | tooling | S, O review | B1 | artifacts of `packaging-and-release.md`; install/uninstall scripts with checksum verification, tested in CI on the three OSes |
| B8 Release | `.github/workflows/release.yml`, `docs/screenshots/**` | tooling | O | B6, B7 | tag → prerelease with every artifact, SHA256SUMS, install commands and the unsigned notice |
| B9 Docs | `docs/ux.md` (→ English), `docs/compatibility.md`, `docs/research/**`, `site/**` | tooling | S, O review | — | English docs; download and privacy pages |

### Wave C — later

| Track | Files | Trigger |
|---|---|---|
| C1 Store channel (MSIX, Mac App Store, Safari appex) | `packaging/windows/msix/**`, `packaging/macos/store/**`, `safari/**`, `app/src/platform/macos/safari_socket.rs` | accounts (D10) |
| C2 macOS PKCS#11 Add-on | `packaging/macos/addon/**` | store build exists and the token matrix requires it (proof 3 criterion) |
| C3 Code signing | release workflow | certificates bought |

## 4. Reviews and gates

Every track ends with an Opus review that: runs everything; checks the file
boundary; reads every `unsafe`, every `// SPEC:` and every `TODO(gustavo)`;
checks the privacy rules (logs, diagnostics); updates the `SPEC.md` with
decisions. Merging needs the CI gates of `testing.md` §6.

## 5. What the maintainer must decide

- D11 (Continue before the certificate reaches a new site) — UX sign-off.
- AMO unlisted signing for the Firefox direct build (free) — see
  `packaging-and-release.md`.
- Security contact for [`SECURITY.md`](../SECURITY.md).
- Enable GitHub Pages ("Source: GitHub Actions") so `pages.yml` can publish
  `site/`.
- Confirm the license change: `project.toml` and `i18n/*.toml` are
  dual-licensed GPL-3.0-or-later or Apache-2.0 so the Apache-2.0 SDK and
  clients can ship what is generated from them ([`LICENSE`](../LICENSE)).
- Everything already marked `TODO(gustavo)` in `docs/ux.md` §17 and
  `project.toml`.
