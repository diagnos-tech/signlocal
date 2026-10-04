# Proof 2: Mac: Mac App Store sandbox, native messaging, CryptoTokenKit, and Safari

**Status:** proven in CI with software keys; a real Mac with Chrome, Safari, and a token is still missing · **Partial result:** YES for sandbox + manifests + host + Keychain; Safari only designed ·
**Proposed decision:** see [§8](#8-proposed-decision)

## Goal

Answer with evidence, before building the app:

1. Does the app **inside the Mac App Store sandbox** write the native messaging manifests into the browsers'
   folders (Chrome, Edge, Brave, Firefox…)?
2. Does Chrome start that sandboxed binary as the host, and does it sign through Keychain/CryptoTokenKit?
3. Does the Safari extension sign (directly in the appex or by opening the app)?
4. Does `SFSafariExtensionManager` report the extension's state?

If the sandbox blocks PKCS#11, the answer comes from [proof 3](3-tokens-mac.md) (`.dmg` complement).

## Short answer (so far)

| # | Question | Answer | Evidence | Missing |
|---|---|---|---|---|
| 1 | Does the sandbox write the manifests? | **YES (CI)**, with `temporary-exception.files.home-relative-path.read-write` only on the `NativeMessagingHosts` folders and the real home via `getpwuid_r` | §4: Chrome, Edge, Brave, and Firefox written from inside the sandbox | App Review accepting the exceptions (precedent: Bitwarden) |
| 2a | Does the browser start the sandboxed host? | **YES (simulated in CI)**: started with Chrome's origin and stdio, the sandboxed host answered `ping` and `list` | §4; Bitwarden's `desktop_proxy` precedent | Real Chrome (script §7) |
| 2b | Does it sign through Keychain/CTK inside the sandbox? | Keychain: **YES (CI)**, PKCS#1 v1.5, PSS, and ECDSA. CTK token: **probably YES**, the query runs in the sandbox without an extra entitlement | §4 | Real token (script §7, proof 3) |
| 3 | Does Safari sign? | **Proposed architecture** (§5): the appex only relays; the app shows the confirmation and signs | web-eid-app (`src/mac/`), Bitwarden (socket in the app group) | Everything: needs an Xcode project and a real Mac |
| 4 | `SFSafariExtensionManager`? | API available to the app that contains the extension; from Rust through `objc2-safari-services` (0.3.2) | web-eid-app `main.mm` uses `getStateOfSafariExtension` and `showPreferencesForExtension` | Proof on a real Mac |

## 1. How the kit proves it

### macOS keystore (`probe/src/keystores/macos/`)

| File | Role |
|---|---|
| `mod.rs` | Two origins: `macos:keychain` (keys in keychain files: imported A1) and `macos:ctk` (keys in CryptoTokenKit tokens: A3). Each reports its own failures. `locator` = SHA-256 of the certificate |
| `identities.rs` | `SecItemCopyMatching(kSecClassIdentity, kSecMatchLimitAll, kSecReturnRef)`; the token query adds `kSecAttrAccessGroup = kSecAttrAccessGroupToken`. An unreadable item does not bring the list down |
| `token.rs` | `kSecAttrTokenID` of the private key (`SecKeyCopyAttributes`). `provider` = only the driver part (`com.apple.pivtoken`, `…OpenSCToken`), without the instance, which is usually the card's serial number |
| `sign.rs` | Checks the digest size → `SecKeyIsAlgorithmSupported` → `SecKeyCreateSignature` → ECDSA DER → `r‖s` (`probe_core::ecdsa::der_to_raw`, curve read from the certificate) |
| `algorithm.rs` | Only the "Digest" algorithms: `RSASignatureDigestPKCS1v15SHA*`, `RSASignatureDigestPSSSHA*`, `ECDSASignatureDigestX962SHA*` |
| `errors.rs` | `errSecUserCanceled` and `TKErrorCodeCanceledByUser` → `Cancelled`; `errSecAuthFailed` and `TKErrorCodeAuthenticationFailed` → `WrongPin`; everything else → `Native { api, code, message }` with `SecCopyErrorMessageString` (or the `CFError` domain + description) |

Decisions and reasons:

- **Two queries, not one.** Current Chromium makes a single identity query
  (`net/ssl/client_cert_store_mac.cc`, with `kSecAttrCanSign`). Whether it includes tokens depends on
  Security.framework's internal routing (file keychain × "iOS" keychain); Apple's
  documentation says to query tokens through the access group: *"Use this access group identifier as the value
  for the `kSecAttrAccessGroup` attribute in a keychain query to access external tokens such as smart
  cards. Access to this group is granted by default and does not require an explicit entry in your
  app's `keychain-access-groups`."* Token identities that show up in the plain query
  are discarded there and counted only in `macos:ctk`, so nothing appears twice.
- **`list` does not ask for a PIN.** Only references and attributes; the PIN dialog (from the token driver) or the
  password/permission dialog (from the keychain) appears inside `SecKeyCreateSignature`. File-keychain keys
  are `hardware: Some(false)`; token keys, `Some(true)`.
- **Token errors arrive in the `CryptoTokenKit` domain**, not as `OSStatus`: Apple's `SecCTKKey.m`
  passes the TK `NSError` through without converting it. That is why the mapping looks at the domain.
  `TODO(gustavo)`: a locked PIN arrives as `AuthenticationFailed` with zero attempts in
  `userInfo`; map it to `PinLocked` once seen on a real token.
- **Calls are not serialized.** Chromium puts every Security.framework call behind a lock
  (the legacy keychain code is not thread-safe); the app must do the same if it uses several threads.

### RSASSA-PSS: Apple's salt is the digest length

Evidence, in three layers:

1. `SecKey.h` (Apple open source, `keychain/headers/SecKey.h`):
   *"`kSecKeyAlgorithmRSASignatureDigestPSSSHA256` — RSA signature with RSASSA-PSS padding according
   to PKCS#1 v2.1, input data must be SHA-256 generated digest. PSS padding is calculated using MGF1
   with SHA256 and saltLength parameter is set to 32 (SHA-256 output size)."* Same for 48 and 64.
2. Implementation (`OSX/sec/Security/SecKeyAdaptors.m`): the IDs are
   `algid:sign:RSA:digest-PSS:SHA256:SHA256:32` (hash, MGF1 hash, salt) and the encoder calls
   `ccrsa_emsa_pss_encode(di, di, di->output_size, salt, …)`: MGF1 with the same hash, salt =
   the digest's `output_size`.
3. Execution: the `algorithm.rs` test checks, on the CI Mac, each algorithm's ID at runtime; and
   `ci-macos.sh` signs PSS and `probe_core::verify` checks it with salt = digest length.

### Scripts (`kit/macos/`)

| File | What it does |
|---|---|
| `ci-macos.sh` | Uses OpenSSL to generate RSA-2048, P-256, and P-384 identities (`.p12` with legacy algorithms, which `security import` accepts), creates a temporary keychain in `~/Library/Keychains`, imports with `-A` and `set-key-partition-list` (Apple, `unsigned:` and the probe's `cdhash`, so no dialog hangs the runner), adds it to the search list, runs `list` and `sign --hash all --pss` and checks each case; generates the report; restores the search list and deletes the keychain |
| `sandbox/entitlements.mas.plist` | Store app entitlements (table below) |
| `sandbox/Info.plist` | Minimal bundle; the `CFBundleIdentifier` (= `project.toml`) defines the container |
| `sandbox/sandbox-test.sh` | The experiment (§3). Prints `RESULT <id> YES/NO/N/A` and a table in the job summary |
| `sandbox/run-sandboxed.sh` | Runs **any** probe command inside the sandbox and shows the denials from the log; for the real Mac |
| `lib/common.sh`, `lib/bundle.sh` | Shared functions (macOS bash 3.2: runs on any Mac) |

`sign --all` is deliberately not used in CI: every Mac's System keychain has identities
(`com.apple.systemdefault`, `com.apple.kerberos.kdc`) whose keys only system services use;
signing with them opens a password dialog that nobody answers on the runner. CI selects the
test certificates with `--cert`.

## 2. Mac App Store app entitlements

| Entitlement | Why | Precedent |
|---|---|---|
| `com.apple.security.app-sandbox` | Mandatory in the store. **Its own, without `com.apple.security.inherit`**: the host is started by the browser, not by a sandboxed parent | Bitwarden `entitlements.desktop_proxy.plist` (app-sandbox + app group, no inherit) |
| `com.apple.security.smartcard` | `TKSmartCardSlotManager` and PC/SC (readers and ATR in diagnostics) and PKCS#11 modules that talk to the card from inside our process. Token items in the keychain do **not** need it | web-eid-app `web-eid-safari.entitlements`; Apple docs: *"requires this entitlement for sandboxed applications that access smart cards using legacy PCSC framework APIs"*; SafeSign (proof 3) requires it of the host app |
| `com.apple.security.device.usb` | USB listing (VID:PID) in diagnostics (`nusb`) | Bitwarden MAS |
| `com.apple.security.application-groups` = `TEAMID.dev.websign` | Socket of the Safari bridge (§5) and of the complement, if one exists | Bitwarden (IPC socket in the group container); web-eid (shared app↔appex group) |
| `…temporary-exception.files.home-relative-path.read-write` | Only the `NativeMessagingHosts` folders of Chrome (+Beta/Dev/Canary), Chromium, Edge (+Beta/Dev/Canary), Brave (+Beta/Nightly), Vivaldi, Opera (+Developer), and `Mozilla/` | Bitwarden MAS (same technique; it does not list Brave or Opera); Brave and Vivaldi checked in KeePassXC; Opera by Chromium's rule (`<user data dir>/NativeMessagingHosts`), **to be confirmed** |
| `com.apple.application-identifier`, `com.apple.developer.team-identifier` | Required by the store; `TEAMID` is `TODO(gustavo)` | Bitwarden MAS |

No `network.client` (the app does not use the network), no `cs.allow-jit` (it is not Electron), no
`files.user-selected` (the `.pfx` is imported by macOS itself). **Arc** is left out until the folder is
confirmed on a real Mac (`~/Library/Application Support/Arc/User Data/NativeMessagingHosts/`,
not verified).

**Finding that changes the `register` code:** inside the sandbox, `$HOME` points to
`~/Library/Containers/<bundle id>/Data`. Bitwarden solves this in Electron with
`os.userInfo().homedir` and documents it in Rust (`desktop_native/core/src/ipc/mod.rs`): *"While running
sandboxed, it's different: /Users/<user>/Library/Containers/com.bitwarden.desktop/Data"*.
`std::env::home_dir()` reads `$HOME` → it would write **inside the container**, where no browser looks.
That is why the kit's `nm/register`, on macOS, takes the home from the user database
(`getpwuid_r(getuid())`, in `probe/src/nm/register/home.rs`), which the sandbox does not redirect. The
`sandbox-test.sh` script detects this and explicitly flags the case where the manifest lands inside the container.

## 3. The sandbox experiment (`sandbox-test.sh`)

It builds `SignLocal.app` with `websign-probe` inside, signs it **ad hoc** with the entitlements above
(except those that require a provisioning profile) and, from inside the sandbox:

| ID | Experiment | YES means |
|---|---|---|
| `sandbox` | Runs `--version` | macOS created `~/Library/Containers/dev.websign.app`: the sandbox was applied |
| `register:<browser>` | `register --browser chrome --browser edge --browser brave --browser firefox` (browser folders created beforehand, to simulate an installation) | The manifest appeared in the **real** folder and points to the sandboxed binary |
| `host-stdio` | Starts the binary the way Chrome does (`chrome-extension://<dev_id>/` + messages with a 4-byte prefix) and sends `ping` and `list` | The sandboxed host answered both |
| `keychain-list`, `keychain-sign` | Test keychain in `~/Library/Keychains`, in the search list; `list` and `sign` (PKCS#1, PSS, ECDSA) | The sandbox sees keychains in the search list and uses the keys (the result also shows the same test outside the sandbox, to isolate the cause) |
| `ctk-query` | The query with `kSecAttrAccessGroupToken` | Ran without error (the runner has no token) |
| `pkcs11-*` | See [proof 3](3-tokens-mac.md#3-pkcs11-inside-the-sandbox) | n/a |

At the end, the script prints the denials the sandbox logged (`sender == "Sandbox"`) and
undoes everything: manifests (restoring the ones that existed), created folders, keychain and search list, and the
container if it created it.

### What an ad hoc binary proves, and what it does not

| Proves | Does not prove (only Apple signing / TestFlight / the store) |
|---|---|
| The sandbox profile rules for **these** entitlements: files outside the container, keychain, `dlopen`, PC/SC | That **App Review accepts** the `temporary-exception` entries (they need a written justification) and the loading of external PKCS#11 modules (guideline 2.5.2) |
| The container and the redirected `$HOME` | The **app group** with a Team ID (removed in the test: it requires a profile) and the macOS 15 warning for groups not authorized by a profile |
| That a sandboxed binary started by a non-sandboxed process works as a stdio host | The behavior of an app installed from the store/TestFlight (quarantine, Gatekeeper, path in `/Applications`) |
| Keychain partitions for ad hoc code (`cdhash:`) | The keychain access dialog for the app with a Team ID (`teamid:`), which is what the user will see |
| n/a | Anything about Safari: the extension only runs signed (or with "Allow Unsigned Extensions" in Safari) |

## 4. What was proven in CI

Run [36629289998](https://github.com/diagnos-tech/signlocal/actions/runs/36629289998), job `macos`,
on 2026-09-29: **macOS 26.6.2 (25G83), arm64**, ad hoc signed binary.

**Keychain** (`ci-macos.sh`, disposable keychain with 3 test identities; all signatures
verified with `probe-core::verify`):

```
RSA-2048   SHA-256/384/512 × RSASSA-PKCS1-v1_5 and RSASSA-PSS   6 × OK via SecKeyCreateSignature (9–14 ms)
EC P-256   SHA-256/384/512 × ECDSA (DER → r‖s)                   3 × OK (5–7 ms)
EC P-384   SHA-256/384/512 × ECDSA (DER → r‖s)                   3 × OK (8–14 ms)
All required checks passed.
```

**Sandbox** (`sandbox/sandbox-test.sh`: `.app` with the store entitlements, signed ad hoc):

| Experiment | Result | Detail |
|---|---|---|
| sandbox applied | **YES** | macOS created the container `~/Library/Containers/dev.websign.app/Data` |
| `register` writes to the real home (Chrome, Edge, Brave, Firefox) | **YES** | the four manifests in `~/Library/Application Support/<browser>/NativeMessagingHosts/` point to the sandboxed binary; browsers without a folder are skipped |
| sandboxed host started the way Chrome starts it (origin + stdio) | **YES** | answered `pong` and `certificates`; host log inside the container |
| keychain listed inside the sandbox | **YES** | both test identities |
| signing inside the sandbox (PKCS#1 v1.5, PSS, ECDSA) | **YES** | three OK, verified |
| CryptoTokenKit query (`kSecAttrAccessGroupToken`) inside the sandbox | **YES** (no token on the runner) | the query runs without an extra entitlement |
| PKCS#11 (`dlopen` of SoftHSM2 in `/opt/homebrew`) | **NO** | `file system sandbox blocked open()`; the kernel logs `deny(1) file-read-data /opt/homebrew/Cellar/softhsm/…` |
| same with *hardened runtime* | **NO** | same denial (the sandbox blocks before library validation) |

| Item | Result | macOS version / runner |
|---|---|---|
| Keychain: RSA-2048 PKCS#1 v1.5 + PSS × SHA-256/384/512 | ✅ | 26.6.2 arm64 |
| Keychain: P-256 and P-384 ECDSA × SHA-256/384/512 (DER → `r‖s`) | ✅ | 26.6.2 arm64 |
| Sandbox applied | ✅ | 26.6.2 arm64 |
| Manifests written by the sandbox (Chrome, Edge, Brave, Firefox) | ✅ | 26.6.2 arm64 |
| Sandboxed host answers over stdio | ✅ | 26.6.2 arm64 |
| Keychain read and used inside the sandbox | ✅ | 26.6.2 arm64 |
| PKCS#11 module outside the bundle loaded inside the sandbox | ❌ | 26.6.2 arm64 |

## 5. Safari: bridge architecture

Apple requires the Safari extension to ship inside an app, with an *app extension*
(`.appex`, extension point `com.apple.Safari.web-extension`) whose class
`SafariWebExtensionHandler` receives every `browser.runtime.sendNativeMessage()`. The appex is a
separate, sandboxed process with **no interface**, which Safari starts and stops whenever it wants.

```
page ──SDK──▶ content script ──▶ extension service worker (Safari)
                                   │ browser.runtime.sendNativeMessage({...,"origin": sender.origin})
                                   ▼
                   SafariWebExtensionHandler (.appex, sandboxed, no UI)
                                   │ Unix socket in ~/Library/Group Containers/TEAMID.dev.websign/
                                   │ (same framing as native messaging: 4 bytes + JSON)
                                   ▼
                   SignLocal.app (Rust) — same handler as the Chrome host
                     confirmation window (egui) → SecKeyCreateSignature → system PIN
```

**Sign directly in the appex?** No. The appex could call `SecKeyCreateSignature` (the PIN dialog
belongs to the system), but decision 5 requires the app's confirmation window (site, certificate,
thumbprint, Sign button) and the appex cannot show a window. It would also duplicate in Swift what
the app does in Rust. The appex stays dumb: it relays bytes and does not know the protocol, so it does not change
when the protocol changes.

**How does the window appear?** The appex starts the app (`NSWorkspace.openApplication`, `activates = false`)
if it is not listening; the app, already holding the request, brings the confirmation window to the front
(`NSApp.activate`), as it does when the request comes from Chrome. The app stays open with an idle
shutdown (decision 6), so from the second request on there is no cold start.

**Why a socket in the app group, and not what web-eid does?** web-eid-app uses
`NSDistributedNotificationCenter` (broadcast to any process of the user) + the group's `UserDefaults`, with busy waiting
(`sleepForTimeInterval` in a loop) and quits the app at every request. It works,
but it is slow, and the notification name is public. XPC with a *Mach service* would require registering the service
with launchd (login item/`SMAppService`) and XPC/Objective-C bindings in Rust. The Unix socket in the
group container is what Bitwarden ships in the store (`~/Library/Group Containers/<group>/s.<name>`),
is trivial on both sides, and reuses the native messaging framing, so the app's handler is the
same for Chrome, Firefox, and Safari.

**Bridge security.** Any non-sandboxed process of the user can try to connect to the socket.
The app must check the peer: `getsockopt(LOCAL_PEERTOKEN)` → *audit token* →
`SecCodeCopyGuestWithAttributes` → `SecCodeCheckValidity` with the requirement
`anchor apple generic and certificate leaf[subject.OU] = "TEAMID" and identifier "dev.websign.app.extension"`.
The site's origin comes from Safari (`sender.origin`/`sender.url` in the service worker), never from the
page's payload, the same rule as Chrome.

**Extension state.** `SFSafariExtensionManager.getStateOfSafariExtension(withIdentifier:)` and
`SFSafariApplication.showPreferencesForExtension(withIdentifier:)` only work in the app that contains the
extension. In the Rust app, through `objc2-safari-services`; they feed the Browsers tab of diagnostics and the
"enable in Safari" button. A disabled extension does not send a ping, so this query is the only source
for "installed but turned off".

### Proposed appex code (~110 lines without comments)

```swift
// SafariWebExtensionHandler.swift — relay between the Safari web extension and the app.
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Safari hands every browser.runtime.sendNativeMessage() to this extension. It forwards the
// message unchanged to the app over a Unix socket in the shared app group container, framed
// like native messaging, and returns the app's reply. The app owns everything else: the
// confirmation window, the signing, the OS PIN dialog. This file never parses the protocol.

import AppKit
import Foundation
import SafariServices
import os

private let appBundleID = "dev.websign.app"
private let appGroupID = "TEAMID.dev.websign"  // TODO(gustavo): real Team ID
private let socketName = "s.host"  // short: sun_path holds 104 bytes
private let startTimeout: TimeInterval = 10
private let maxReplyBytes = 1 << 20  // native messaging's host-to-browser limit
private let log = Logger(subsystem: "dev.websign.app.extension", category: "relay")

final class SafariWebExtensionHandler: NSObject, NSExtensionRequestHandling {
    /// One request at a time: the app shows one confirmation window at a time anyway.
    private static let queue = DispatchQueue(label: "dev.websign.relay")

    func beginRequest(with context: NSExtensionContext) {
        let item = context.inputItems.first as? NSExtensionItem
        let message = item?.userInfo?[SFExtensionMessageKey]
        Self.queue.async {
            let reply: Any
            do {
                reply = try Relay.exchange(message)
            } catch {
                log.error("relay failed: \(String(describing: error), privacy: .public)")
                // Same envelope as the host's own errors (nm/protocol.rs), so the SDK sees one shape.
                let id = (message as? [String: Any])?["id"] ?? NSNull()
                reply = ["v": 1, "id": id, "ok": false,
                         "error": ["code": "app_unavailable", "message": "\(error)"]] as [String: Any]
            }
            let response = NSExtensionItem()
            response.userInfo = [SFExtensionMessageKey: reply]
            context.completeRequest(returningItems: [response], completionHandler: nil)
        }
    }
}

enum RelayError: Error { case notJSON, noAppGroup, appNotFound, tooLarge, closed, posix(Int32) }

enum Relay {
    static func exchange(_ message: Any?) throws -> Any {
        guard let message, JSONSerialization.isValidJSONObject(message) else { throw RelayError.notJSON }
        let fd = try connectToApp()
        defer { close(fd) }
        let body = try JSONSerialization.data(withJSONObject: message)
        var length = UInt32(body.count)  // native byte order, like native messaging
        try writeAll(Data(bytes: &length, count: 4) + body, to: fd)
        let header = try readExactly(4, from: fd)
        let replyLength = Int(header.withUnsafeBytes { $0.loadUnaligned(as: UInt32.self) })
        guard replyLength <= maxReplyBytes else { throw RelayError.tooLarge }
        return try JSONSerialization.jsonObject(with: readExactly(replyLength, from: fd))
    }

    /// Connects to the app's socket, starting the app once if nobody is listening yet.
    private static func connectToApp() throws -> Int32 {
        guard let group = FileManager.default.containerURL(
            forSecurityApplicationGroupIdentifier: appGroupID)
        else { throw RelayError.noAppGroup }
        let path = group.appendingPathComponent(socketName).path
        let deadline = Date().addingTimeInterval(startTimeout)
        var launched = false
        while true {
            let fd = socket(AF_UNIX, SOCK_STREAM, 0)
            guard fd >= 0 else { throw RelayError.posix(errno) }
            if connectUnix(fd, path) == 0 { return fd }
            let failure = errno
            close(fd)
            guard failure == ENOENT || failure == ECONNREFUSED, Date() < deadline else {
                throw RelayError.posix(failure)
            }
            if !launched {
                try launchApp()
                launched = true
            }
            Thread.sleep(forTimeInterval: 0.1)
        }
    }

    private static func launchApp() throws {
        guard let url = NSWorkspace.shared.urlForApplication(withBundleIdentifier: appBundleID)
        else { throw RelayError.appNotFound }
        let configuration = NSWorkspace.OpenConfiguration()
        configuration.activates = false  // the app raises its own confirmation window
        configuration.addsToRecentItems = false
        NSWorkspace.shared.openApplication(at: url, configuration: configuration)
    }

    private static func connectUnix(_ fd: Int32, _ path: String) -> Int32 {
        var address = sockaddr_un()
        address.sun_family = sa_family_t(AF_UNIX)
        let bytes = Array(path.utf8)
        guard bytes.count < MemoryLayout.size(ofValue: address.sun_path) else {
            errno = ENAMETOOLONG
            return -1
        }
        withUnsafeMutableBytes(of: &address.sun_path) { $0.copyBytes(from: bytes) }
        return withUnsafePointer(to: &address) {
            $0.withMemoryRebound(to: sockaddr.self, capacity: 1) {
                connect(fd, $0, socklen_t(MemoryLayout<sockaddr_un>.size))
            }
        }
    }

    private static func writeAll(_ data: Data, to fd: Int32) throws {
        try data.withUnsafeBytes { raw in
            var offset = 0
            while offset < raw.count {
                let written = write(fd, raw.baseAddress! + offset, raw.count - offset)
                guard written > 0 else { throw RelayError.posix(errno) }
                offset += written
            }
        }
    }

    private static func readExactly(_ count: Int, from fd: Int32) throws -> Data {
        var data = Data(count: count)
        try data.withUnsafeMutableBytes { raw in
            var offset = 0
            while offset < count {
                let got = read(fd, raw.baseAddress! + offset, count - offset)
                guard got > 0 else { throw got == 0 ? RelayError.closed : RelayError.posix(errno) }
                offset += got
            }
        }
        return data
    }
}
```

App side (Rust), outside the scope of this proof: a `UnixListener` at
`~/Library/Group Containers/TEAMID.dev.websign/s.host` (the sandbox's `$HOME` does not work; same computation as
Bitwarden), the peer verification described above, and the **same** message loop as the stdio host.

Risks to measure on the real Mac: Safari may terminate the appex if the request takes long (user typing
a PIN for a minute?); measure with a long wait in the confirmation window. Also, the app's first launch
by the appex with `activates = false` must still bring the window to the front.

## 6. What is missing

- [ ] CI log (§4): keychain and ad hoc sandbox.
- [x] `register` uses the real home on macOS (`getpwuid_r`, finding from §2). CI still needs to be rerun.
- [ ] Real Chrome starting the sandboxed host and signing with a keychain A1 and with a CTK token.
- [ ] Real Firefox (reads `~/Library/Application Support/Mozilla/NativeMessagingHosts/`).
- [ ] Safari: Xcode project with the appex above, extension loaded, end-to-end signing,
      `getStateOfSafariExtension` with the extension on and off.
- [ ] TestFlight: the same app signed by Apple (Team ID, app group, the keychain's `teamid:` partition).
- [ ] App Review: submit a build with the exceptions and see whether it passes (the only proof of acceptance).

## 7. Script for Gustavo (real Mac)

Prerequisites: macOS 14 or 15 (note the exact version and the chip), Xcode Command Line Tools, Rust,
Homebrew, Chrome and Firefox installed **and opened once** (so they create their folders).

1. **Build:** `cd docs/prototypes/kit && cargo build --release -p websign-probe && export PROBE_EXE=$PWD/target/release/websign-probe`.
2. **Keychain with test keys:** `bash macos/ci-macos.sh` → it must end in "All required
   checks passed." (it uses a temporary keychain; it does not touch the login keychain).
3. **Sandbox:** `bash macos/sandbox/sandbox-test.sh` → copy the `RESULT` table and the denials.
4. **Real A1 (optional):** import a `.pfx` with Keychain Access; `"$PROBE_EXE" list` → the line must
   say `macos:keychain (keychain, software; PIN by OS)`. `"$PROBE_EXE" sign --cert <16 hex> --hash all --pss`
   → macOS asks whether to allow use of the key: test **Deny** (it must print "cancelled by the user"),
   then **Allow**.
5. **Same test inside the sandbox:** `bash macos/sandbox/run-sandboxed.sh list` and
   `bash macos/sandbox/run-sandboxed.sh sign --cert <16 hex> --hash sha256`.
   Note whether the permission dialog appears and what it says.
6. **Chrome starting the sandboxed host:**
   `bash macos/sandbox/run-sandboxed.sh register --browser chrome` (the bundle lives in
   `~/Library/Caches/dev.websign.sandbox-test/SignLocal.app`); check
   `cat ~/Library/Application\ Support/Google/Chrome/NativeMessagingHosts/dev.websign.host.json`.
   Load the development extension (`kit/extension/`, fixed ID `nhnkdpljdgjflbflkhnkmfmcmodboeii`)
   in `chrome://extensions` (developer mode → "Load unpacked") and trigger `ping` and
   `list`. Evidence: response in the extension, the host log (in the sandbox `TMPDIR` also goes to the
   container: `find ~/Library/Containers/dev.websign.app -name '*-probe-host.log'`) and
   `log show --last 5m --predicate 'sender == "Sandbox"' --style compact | grep websign`.
7. **Token via CryptoTokenKit:** see the script in [proof 3](3-tokens-mac.md#4-test-script-for-gustavo)
   and run `list`/`sign` outside and inside the sandbox (`run-sandboxed.sh`), and through Chrome (step 6).
8. **Safari:** once `safari/` exists (Xcode project with the appex from §5): Safari → Settings →
   Advanced → "Show features for web developers"; Develop → "Allow Unsigned Extensions"; enable
   the extension; sign from the test page; turn the extension off and check the
   state in diagnostics.
9. **Report:** `"$PROBE_EXE" report --run-signatures --cert <fp> --hash all --pss --out mac-real.md`
   and attach it here (it contains no names, CPF, or serial numbers).

## 8. Proposed decision

- **A single binary in the Mac App Store** = native messaging host (Chrome, Edge, Brave, Vivaldi,
  Opera, Firefox) + windows + signer, with the entitlements from `entitlements.mas.plist`. `register`
  computes the real home (`getpwuid_r`), writes only into existing browser folders, and runs on every
  app launch (and through the `websign://` scheme, since the store does not run an installer).
- **Keychain + CryptoTokenKit first**, through this kit's two queries. PKCS#11 inside the store app
  only if [proof 3](3-tokens-mac.md) shows it works **and** App Review accepts it; otherwise, a
  `.dmg` complement (Developer ID, notarized, Sparkle), called through the same app-group socket.
- **Safari:** ~100-line appex that only relays, Unix socket in the app group, confirmation and signing
  in the app; extension state through `SFSafariExtensionManager` from the app.

Revert if: the sandbox denies writing to the folders even with the exceptions (→ the app cannot register
itself; the alternative would be for the complement to register), Chrome cannot start the sandboxed
binary, or Safari terminates the appex before the user confirms (→ the appex answers "pending" and the
extension asks the app again).

## References

- Bitwarden (GPL-3.0): `apps/desktop/resources/entitlements.mas.plist`,
  `entitlements.desktop_proxy.plist`, `src/main/native-messaging.main.ts` (`userInfo().homedir` on
  macOS), `desktop_native/core/src/ipc/mod.rs` (socket in the app group).
- web-eid-app (MIT): `src/mac/safari-extension.mm`, `main.mm`, `shared.hpp`,
  `web-eid-safari*.entitlements`.
- Chromium: `net/ssl/client_cert_store_mac.cc`.
- Apple: documentation of `kSecAttrAccessGroupToken`, `com.apple.security.smartcard`, and "Using
  cryptographic assets stored on a smart card"; `Security` open source: `keychain/headers/SecKey.h`,
  `OSX/sec/Security/SecKeyAdaptors.m`, `OSX/sec/Security/SecCTKKey.m`; `TKError.h`
  (`TKErrorCodeCanceledByUser = -4`, `TKErrorCodeAuthenticationFailed = -5`).
- KeePassXC: `src/browser/NativeMessageInstaller.cpp` (Brave, Vivaldi, Edge folders on macOS).
