# safari — Safari web extension bridge

The app extension (`.appex`) Apple requires for a Safari web extension,
embedded in `WebeSign.app` by `cargo xtask package --target
universal-apple-darwin`. Safari hands it one message at a time; the relay
turns those into sessions with a `websign` host process bundled inside the
appex, which then works exactly as it does for Chrome or Firefox (window,
signing, PIN).

- Contract (relay messages, host launch, limits): [`SPEC.md`](SPEC.md).
- `Relay/` is platform-neutral Swift (Foundation only); `Extension/` is the
  SafariServices entry point. Both compile into one module for the appex.
- Test: `swift test --package-path safari` (macOS or Linux; fake hosts are
  `cat` and `sh`). With `WEBSIGN_TEST_APPEX=<path to the packaged .appex>`
  one more test starts the real bundled host and exchanges `hello`.
- CI: `.github/workflows/safari.yml` builds and inspects the packaged app on
  macOS. Safari itself cannot be driven in CI (safaridriver does not install
  extensions); the manual check is in
  [`docs/compatibility.md`](../docs/compatibility.md) §Safari.
- Unsigned builds need Safari's **Allow Unsigned Extensions**
  ([`docs/install.md`](../docs/install.md) §Safari).
  TODO(gustavo): Developer ID signing and notarization, then the Mac App Store.
- License: GPL-3.0-or-later.
