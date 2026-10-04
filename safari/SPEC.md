# Safari bridge — specification

Role: the native side of the Safari web extension. Safari starts no native
messaging host and gives the extension no stdio connection; it delivers each
`browser.runtime.sendNativeMessage()` of the extension's background to the
app extension (`.appex`) inside `SignLocal.app`, one message in, one reply
out, and never lets the appex speak first. The relay here turns that into
what every other browser gets: one `websign` host process per connection,
speaking the unchanged protocol ([`protocol.md`](../docs/architecture/protocol.md))
on its stdio.

```
page ─SDK─▶ content script ─▶ background (Safari) ──sendNativeMessage──▶ appex (sandboxed)
                                   ▲    open / send / poll / close          │ spawns, per session
                                   └──────── replies (host output) ─────────┤
                                                                            ▼
                               <appex>/Contents/MacOS/websign  (native messaging on stdio)
```

## 1. Relay messages (background → appex)

Every message is a JSON object with exactly the keys listed; anything else
(unknown key, wrong type, fraction, boolean for a number) gets
`{"relay":1,"error":"BadRequest"}` and reaches no host.

| `op` | Keys besides `relay: 1`, `op` | Effect |
|---|---|---|
| `open` | — | Starts a host process; the reply names the new session |
| `send` | `session`, `message` | Writes `message` (a protocol `ClientEnvelope`) to the host's stdin |
| `poll` | `session`, `wait` (integer ms, 0–10000) | Waits until the host says something, the host ends, or `wait` passes |
| `close` | `session` | Closes the host's stdin; the host is terminated after 3 s and killed after 6 s if still alive |

`session` is the canonical upper-case UUID the `open` reply gave. `message`
must be an object with a string `id` of 1–64 bytes and a `type` among the
protocol's client types (`hello`, `status`, `choose`, `sign.begin`,
`sign.digest`, `cancel`, `diagnostics.open`; a test keeps the list equal to
`extension/src/generated/ClientMessage.ts`), at most 1 MiB encoded. The
content is validated by the host, as for every browser.

## 2. Replies (appex → background)

- Session reply: `{"relay":1,"session":S,"messages":[AppEnvelope…],"open":bool}`.
  `messages` holds the host output not yet collected, in order (at most
  1 MiB of it unless one message alone is larger; poll again for the rest).
  Every reply to `open`, `send`, `poll` and `close` has this shape. `open:
  false` means the host is gone and the session with it: the equivalent of
  a port's `onDisconnect`, after the messages it carries.
- Error reply: `{"relay":1,"error":CODE}`.

| `CODE` | Meaning | The background reports |
|---|---|---|
| `BadRequest` | not a relay message of this version | `Internal` (a bug on our side) |
| `NoSession` | unknown, ended, or another Safari profile's session | the port closed (`closedCode(heard)`, as elsewhere) |
| `TooManySessions` | 4 hosts already run in this appex | `Internal` |
| `HostMissing` | the bundled `websign` is missing or would not start | `AppMissing` |

A newer request on a session answers a parked `poll` at once with no
messages, so a background never holds two polls.

## 3. Background transport (extension side)

The Safari build of the background replaces `runtime.connectNative` with a
port-shaped object over these messages; everything above the port
(`hello` first, the shared connection, idle close, ids) stays as in
[`extension/SPEC.md`](../extension/SPEC.md) §2.1:

1. `connectNative` → `open`; `HostMissing` or a rejected
   `sendNativeMessage` → `onDisconnect` with nothing heard.
2. `postMessage(m)` → `send` with `message: m`; the reply's `messages` go
   to `onMessage`, in order.
3. While the session is open, keep exactly one `poll` outstanding
   (`wait: 5000`), sending the next as soon as one answers; its `messages`
   go to `onMessage`.
4. `open: false` or `NoSession` → `onDisconnect`; `disconnect()` → `close`.

The page's origin still comes from Safari's `MessageSender`, never from
the payload (§2.3 there); the appex adds nothing and trusts nothing.

## 4. Host launch

The appex starts `<appex>/Contents/MacOS/<SignLocalHostExecutable>` with the
arguments `[--safari-web-extension, <SignLocalHostExtensionID>]`, both keys
read from the appex's `Info.plist` (rendered from `project.toml` by `cargo
xtask package`; the ID is `[ids] safari_extension_bundle_id`). This is the
host's Safari launch shape (`websign-host` `launch`, `BrowserFamily::Safari`):
the host accepts it only when its own executable is
`<name>.appex/Contents/MacOS/<binary>` on macOS, and only for the bundle ID in
`project.toml`; anywhere else the same words are a command line (usage
error). It then serves native messaging exactly as for the other browsers,
logs "started by safari", and shows "Safari" from `hello.browser`. The web
origin still comes only from the extension, which reads it from Safari's
`MessageSender`.

**Why a copy of the binary inside the appex:** Safari loads only sandboxed
app extensions, and a sandboxed process may always read and execute its own
bundle; reaching into the containing app's `Contents/MacOS` is not
something the sandbox promises. The copy is signed with
`com.apple.security.inherit`, so it runs in the appex's sandbox:

- its home is the appex's container, so settings, remembered sites and logs
  of Safari connections are kept apart from the other browsers';
- Keychain and CryptoTokenKit work (proof 2); PC/SC and USB through the
  appex's `smartcard` and `device.usb` entitlements; PKCS#11 modules load
  from the read-only locations the appex's entitlements list
  (`packaging/macos/safari-extension.entitlements`).

## 5. Lifecycle and limits

- One session per background connection; at most 4 per appex process.
- A session nobody touched for 120 s is ended (the background closes idle
  ports after 60 s and polls every few seconds while open, so only an
  abandoned session gets there).
- Host output nobody collects: past 64 messages or 4 MiB the session ends.
- A host that breaks framing (length over 1 MiB, a frame that is not a JSON
  object with string `id` and `type`) ends its session at once.
- If Safari ends the appex process, every host's stdin closes and each host
  exits on its own, as when a browser closes a port.
- Nothing is logged but fixed event texts (`session opened`, `session
  ended: idle`…); the host's stderr is discarded.
