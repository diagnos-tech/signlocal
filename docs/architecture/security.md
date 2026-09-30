# Security

What WebeSign protects, from whom, and where each defense lives. Every row
names the file or crate that must enforce it and the test that proves it.

## Assets

1. **The private key** — never leaves the OS key store or the token; the app
   only asks for signatures.
2. **The PIN** — typed in the OS dialog or our window, never in the browser.
3. **The person's intent** — a signature happens only for a digest the person
   saw (verification code), from a caller the person saw, after a deliberate
   click.
4. **Personal data in certificates** — name, CPF/CNPJ, national IDs: released
   to a caller only for the certificate the person chose.
5. **The machine** — no open port, no background process, no network use.

## Trust boundaries

| Boundary | Trusted side | Untrusted side |
|---|---|---|
| page ↔ content script | content script (but it shares a process with the page) | the page and everything it posts |
| content script ↔ background | background | the content script's messages (re-validated) |
| extension ↔ app | the browser's launch arguments and manifest checks | message contents |
| desktop program ↔ app | the OS's view of the parent process | everything the program sends |
| app ↔ PKCS#11 module | nothing: a module is foreign code in our process | module outputs (verified) |

## Threats and mitigations

| # | Threat | Mitigation | Where | Proof |
|---|---|---|---|---|
| T1 | A page impersonates another site | Origin from the browser's `MessageSender`, never the payload; the window shows the registrable domain emphasized, punycode for IDN, alerts for IP/localhost; `http:` (non-local), `file:`, `data:`, extension origins refused | extension `background/origin.ts`; `websign-core::present::origin`; host `caller.rs` | origin vectors (`docs/ux.md` §16.2); e2e insecure-origin test |
| T2 | Another extension or program talks to the app via native messaging | Manifests list only our extension IDs (`allowed_origins`/`allowed_extensions`); the app re-checks the launch origin against `project.toml`. Weak spot: the development ID is derived from a published key (see "The development key") | `websign-registration::manifest`, `websign-host::launch` | manifest unit tests; launch tests |
| T3 | Click-jacking / accidental confirmation | Our own window (the page cannot draw in it); primary button arms after 600 ms visible and focused, re-arms on any change; press **and** release after arming; input discarded while unarmed; initial focus never on Sign; Enter in a list row never signs | `websign-ui-model::confirm::arming`, `machine` | arming unit tests; egui_kittest tests |
| T4 | Digest swapped or truncated | Exact length per hash; verification code on both sides (window and page via `fingerprint()`); digest bound to the certificate by `seq` | protocol, host `flow/sign.rs`, SDK | protocol vectors; flow scenario tests |
| T5 | Site harvests the holder's identity (including another person's, on a shared computer) | `certificates()` returns only the chosen certificate (D2); `sign.need_digest` releases a certificate without "Continue" only when the caller's consent record covers **that certificate** (per caller and fingerprint, D11): a remembered site still needs "Continue" for any other one, and moving the selection never discloses a row; remembering is opt-in, stores only the chosen fingerprint, is disabled for IP/IDN and for interpreters and shells, and is revocable; using a certificate without "Remember" never extends a consent | host `flow/sign`, `engine/persist.rs`, `store/documents.rs`, ui-model `confirm` | flow scenario tests "new site cancels before Continue → caller got nothing"; `engine_consent_per_certificate.rs` (someone else's token, arrowing, revoke) |
| T6 | PIN leak | PIN never in browser or extension; our field only for PKCS#11 keys; `SecretString` + zeroize after `C_Login`; hidden from AccessKit; no clipboard; macOS `EnableSecureEventInput` while focused; never logged | app `ui/confirm`, keystores `pkcs11/login.rs` | review checklist; kittest checks the AccessKit value is hidden |
| T7 | Malformed input crashes or confuses the app | Strict parsing (unknown fields refused), size limits, tolerant-but-bounded DER reader, fuzzing of the DER reader, framing and message parser, no `unwrap`/`panic` on external input | protocol, core, host | `cargo-fuzz` targets in CI; clippy lints |
| T8 | A PKCS#11 driver returns a wrong signature | Every signature verified against the certificate before replying (except brainpoolP512r1, logged) | host `flow/sign.rs`, `websign-core::verify` | scenario test with a fake store returning garbage |
| T9 | Protocol downgrade / confusion | Explicit version, negotiation by range, one schema per version, `hello` required first | protocol, host session | negotiation vectors |
| T10 | Personal data in logs, diagnostics or issues | Logs carry steps, codes, sizes only; diagnostics report built from types that cannot hold names, IDs, sites, serials, fingerprints or digests; token labels and USB serials never read into reports | `app/src/logging.rs`, `websign-ui-model::diagnostics::report`, devices | golden report test; log-audit test greps a run's log for fixture names |
| T11 | Local program spoofing a desktop caller | Identity from the parent process (path, verified code signature); unsigned programs flagged "Unverified program"; consent per signer/path; interpreters, shells and terminal hosts (`node`, `python`, `bash`, `powershell`, `cmd`…) are never remembered and are shown as "a script run by {program}", since every script they run shares their identity | `app/src/platform/caller.rs`, `websign-core::present::caller`, host `caller.rs` | platform contract tests; `script_hosts` unit tests; `engine_consent_per_certificate.rs` |
| T12 | Tampered downloads | SHA256SUMS on every release; `install.sh`/`install.ps1` verify it before installing; later: code signing (Authenticode, Developer ID, store signing) | `scripts/install/`, release workflow | install script tests |
| T13 | Replay of a signature request | Each signature needs a fresh confirmation; no batch signing (D4); requests expire (300 s) | host | scenario tests |
| T14 | Denial of service by a page | Queue limit (`Busy`), per-connection limit, frame size limits, timeouts | protocol limits, host queue | scenario tests |
| T15 | Sandbox escapes; a stranger driving the Safari relay | Safari delivers `sendNativeMessage` only from our extension to our appex; the appex is sandboxed and its host copy inherits that sandbox (entitlements: smart card, USB, read-only PKCS#11 locations); relay messages are validated strictly (exact keys, sizes, client message types) and sessions are per Safari profile; the host accepts the Safari launch shape only from inside an `.appex` and only with the appex bundle ID from `project.toml`; the web origin still comes only from Safari's `MessageSender`, never from the relay or the page. Store channel (MSIX, Mac App Store): entitlements limited to `NativeMessagingHosts` folders | `packaging/macos`, `safari/`, `websign-host::launch`, extension `background/safari-*.ts` | relay tests (`swift test`); launch tests; extension Safari transport tests; review |
| T16 | One tab reads, feeds or cancels another tab's request over the shared native port | Native ids are `<tab>.<frame>.<pageId>` with tab and frame from `MessageSender`; app messages go only to the exact id's entry and back to that tab and frame; continuations (`sign.digest`, `cancel`) resolve only within the sender's own ids; replies carry the requesting document's token, so a reply racing a navigation is dropped instead of reaching the next page | extension `background/requests.ts`, `router.ts`, `content/relay.ts` | router and relay tests ("one shared port, isolated tabs", "drops a reply meant for another document") |
| T17 | The extension itself becomes an attack surface | Permission `nativeMessaging` only (no host permissions, no web-accessible resources, no `externally_connectable`); extension pages under `default-src 'none'; script-src 'self'` CSP, no remote code, no `eval`; the content script posts only to `location.origin` and runs only on secure contexts; no logging | `extension/build/manifest.ts`, `entrypoints/content.ts` | manifest tests |

## The development key

Direct (unpacked) Chromium builds carry `project.toml`'s `dev_key` as the
manifest `key`. Chromium derives an unpacked extension's ID from that key, so
every copy gets `dev_id` on every machine, and the native messaging manifests
allow `dev_id` next to the store IDs. The key is public (it is in this
repository and in every release zip), so the ID is not proof of origin: any
unpacked extension that sets the same `key` gets `dev_id` and may start the
app and talk to it like ours. Loading it takes Developer mode and a manual
**Load unpacked**, which is the bar this risk rests on today.

Such an extension fills `web.origin` itself, so it can claim any site,
including one the person remembered, whose certificate is then released
without **Continue** (D11). It still cannot sign without the person's
confirmation in our window (T3, T13).

Mitigations:
- Store builds (`WEBSIGN_CHANNEL=store`) never carry the key; the stores
  assign and sign their IDs (`extension/build/manifest.ts`, tested).
- Once the store IDs exist, direct builds either move to a key whose private
  and public halves stay in a CI secret (a new `dev_id`), or stop being
  published, and `dev_id` leaves the release manifests.
- `TODO(gustavo)`: decide whether to rotate or withdraw the development key
  when the store listing exists.

## Out of scope

Building or validating signature formats, time-stamping, certificate chain
validation (the site's job); malware running as the user (it can drive the
UI like the user; the PIN and the OS's own protections are what remains);
cloud-only certificates.

## Logs

Allowed in any `log::` call: step names, API names, `CKR_*`/`HRESULT`/
`OSStatus` codes, error codes, counts, sizes, module file names, browser
family. Forbidden: names, document numbers, e-mail, certificate contents or
fingerprints, token labels, serial numbers, digests, signatures, PINs, sites.
Reviewers reject violations; the log-audit e2e test enforces it for fixture
data.

## Reporting vulnerabilities

Privately to the maintainer (`SECURITY.md`, `TODO(gustavo)`: contact address),
never in public issues.
