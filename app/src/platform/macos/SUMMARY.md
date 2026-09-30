# app/src/platform/macos

- `bundle.rs` — The product name from the enclosing `.app`'s `Info.plist`.
- `caller.rs` — The parent process (`proc_pidpath`) with its name and code signature, checked by audit token when possible and against PID reuse.
- `channel.rs` — Mac App Store or not, from our own App Sandbox entitlement.
- `code_signature.rs` — Team ID and identifier of a signature that chains to Apple.
- `file_picker.rs` — `NSOpenPanel` for one file of the requested types, modal on the main thread.
- `focus.rs` — Activating the app and making the window key.
- `mod.rs` — MacOS implementations of the [`crate::platform`] functions.
- `objc.rs` — Typed `objc_msgSend` calls, run-time class definition and an autorelease pool guard, without a binding crate.
- `peer.rs` — The audit token of the process at the other end of stdin, when stdin is a Unix socket.
- `secure_input.rs` — Carbon's `EnableSecureEventInput`/`DisableSecureEventInput`, balanced by an RAII guard.
- `settings.rs` — Reduce motion (`NSWorkspace`) and dark mode (`AppleInterfaceStyle`).
- `system_ui.rs` — `SFCertificatePanel`, `.pfx` import through Keychain Access (with `file_picker`'s panel), URLs.
- `url_events.rs` — The `kAEGetURL` Apple Event handler, installed when AppKit posts "will finish launching".
