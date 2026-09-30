# safari — Safari web extension bridge

The app extension Apple requires for a Safari web extension. It relays every
native message, unchanged, to the WebeSign app over a Unix socket in the app
group container (framed like native messaging) and starts the app when
nobody listens. The app owns the window, the signing and the PIN.

- Design and review: [`docs/prototypes/2-mac.md`](../docs/prototypes/2-mac.md) §5.
- Built only for the Mac App Store channel (decision D10); direct macOS builds
  have no Safari support. `project.yml` (XcodeGen) describes the app and the
  appex; the `App/` sources, plists and entitlements are added by the store
  track (`packaging/macos`).
- License: GPL-3.0-or-later.
