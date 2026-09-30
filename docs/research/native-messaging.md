# Native messaging: where each browser looks for the host

Research for the risk proofs and for the app's `register`. It covers paths and registry keys by
operating system × browser, launch arguments, limits, the Firefox Snap and Flatpak case, Safari (which
uses no manifest), and extension pre-registration.

**How to read the "Basis" column:**

| Code | Meaning |
|---|---|
| **D** | official vendor documentation (links at the end) |
| **C** | Chromium source code, read at `raw.githubusercontent.com/chromium/chromium` |
| **B** | Bitwarden, `native-messaging.main.ts` (only to compare paths) |
| **K** | KeePassXC, `NativeMessageInstaller.cpp` (same) |
| **W** | web-eid-app (`CMakeLists.txt`, `web-eid.wxs`, manifest templates) |
| **?** | not confirmed in a primary source: treat as a hypothesis until proven |

Nothing here was copied: the three codebases were read only to check paths.

---

## 1. Summary of what decides the project

1. **The manifest is per browser, and the format differs by family.** Chromium: `allowed_origins`
   (`chrome-extension://<id>/`, no wildcard). Firefox: `allowed_extensions` (Gecko ID). That is two files.
2. **Windows has no folder: it has a registry key.** `HKCU` is enough (Chrome consults `HKCU` before `HKLM`) and
   the manifest can be in any folder. **Edge, Brave, Vivaldi, and Opera fall back to Chrome's key** when they
   have none of their own, so one Chrome key already covers several (§3.1).
3. **macOS and Linux use `<browser data folder>/NativeMessagingHosts/<host>.json`.** That is why
   Chromium's `--user-data-dir` allows testing without touching the real profile (§3.2, §3.3).
4. **In MV3 the host never receives `--parent-window`**: Chrome passes `0` when the caller is a service
   worker (§2). System PIN dialogs cannot depend on that handle: the app's window must find the
   browser window on its own or bring itself to the foreground.
5. **Firefox Snap does not read the manifest: it asks the portal** (`org.freedesktop.portal.WebExtensions`, today;
   `org.freedesktop.NativeMessagingProxy`, next). The host runs **outside** the Snap, started by the portal,
   with the portal's environment, not Firefox's (§3.4).
6. **Safari has no manifest, registry, or `allowed_origins`.** The extension lives inside the app and talks to an
   app extension (appex) through `runtime.sendNativeMessage` (§3.5).
7. **Extension pre-registration:** Chromium has a file/registry channel (Linux without confirmation; Windows and
   macOS ask the user to enable it). Firefox only through system policy (§4).

---

## 2. How the browser launches the host

### Arguments

| Browser | Arguments, in order | Basis |
|---|---|---|
| Chrome, Edge, Brave, Vivaldi, Opera (Linux, macOS) | `chrome-extension://<id>/` | D, C |
| Chrome, Edge, Brave, Vivaldi, Opera (Windows) | `chrome-extension://<id>/` `--parent-window=<decimal HWND>` | D |
| Firefox (all OSes) | full path of the manifest, the extension's Gecko ID (since Firefox 55) | D |
| Safari | none: there is no child process, see §3.5 | D |

- Chromium's first argument is the **origin that the browser itself validated** against `allowed_origins`.
  It is the only extension identity the host can trust; the `origin` that the test extension sends in the
  JSON is informational.
- **`--parent-window` is `0` when the calling context is a service worker** (Chrome and Edge, D). Every
  MV3 extension is a service worker, so the handle does not arrive. The `probe` treats `0` as absent
  (`nm/launch.rs`).
- The host's working directory is the executable's folder (C, `LaunchContext`).
- The environment is the browser's (confirmed in the e2e: `WEBSIGN_PROBE_PIN` and `SOFTHSM2_CONF` exported before
  Chromium reach the host). Exception: hosts started by the Firefox Snap portal (§3.4).

### Protocol (the same in both directions)

A `u32` with the length in **native byte order** + UTF-8 JSON. On the platforms we care about (x86-64,
ARM64) that is little-endian, but the code uses `from_ne_bytes`/`to_ne_bytes` as the documentation says.

| Direction | Limit | Basis |
|---|---|---|
| host → browser | **1 MB** (the browser drops the connection if exceeded) | D (Chrome, Edge, Firefox) |
| browser → host | 64 MiB (Chrome, current doc); 4 GB (Edge, Firefox) | D |

The `probe` rejects replies above 1 MiB and accepts at most 1 MiB of input (the messages are small).

### Lifecycle

- `runtime.connectNative()` keeps the process until the port closes; `sendNativeMessage()` starts **one process
  per message** and only the first reply counts (D).
- **Chrome 105+: a `connectNative` port keeps the service worker alive.** If the host dies, the port closes and the
  worker ends after the timers; the doc recommends calling `connectNative()` again in `onDisconnect`
  (D, service worker lifecycle). The test extension closes the port when idle (60 s) and reconnects
  on the next request.
- The host's `stderr`: Firefox redirects it to the Browser Console (D). Chrome does not show it in any
  interface (on POSIX systems the host inherits the browser process's `stderr`, visible only if it was opened
  from a terminal; C, `launch_context_posix.cc`). That is why the `probe` writes `<temp>/websign-probe-host.log`.

### Typical error messages (for diagnostics)

| Browser | Text | Probable cause |
|---|---|---|
| Chrome/Edge | `Specified native messaging host not found.` | manifest or key missing; different name |
| Chrome/Edge | `Access to the specified native messaging host is forbidden.` | origin not in `allowed_origins` |
| Chrome/Edge | `Native host has exited.` | the process ended (seen when the host crashed on `todo!()`) |
| Firefox | `No such native application <name>` | manifest/key not found |
| Firefox | `This extension does not have permission to use native application <name>` | ID not in `allowed_extensions` |
| Firefox | `File at path <path> does not exist, or is not executable` | manifest found, wrong `path` |

---

## 3. Registration matrix

`<host>` = `dev.websign.host` (`project.toml`, `[ids].native_host`; Chrome only accepts `[a-z0-9_.]`).

### 3.1 Windows

The manifest can live in **any folder**; the key's default value is its path. The manifest's `path`
can be relative to the manifest's folder (D).

| Browser | `HKCU` key (default value = manifest path) | Basis |
|---|---|---|
| Chrome (and Beta/Dev/Canary) | `Software\Google\Chrome\NativeMessagingHosts\<host>` | D, C |
| Chromium | `Software\Chromium\NativeMessagingHosts\<host>` (read first; then Chrome's) | C |
| Edge | `Software\Microsoft\Edge\NativeMessagingHosts\<host>`; fallback: Chromium, then Chrome | D |
| Brave | `Software\BraveSoftware\Brave-Browser\NativeMessagingHosts\<host>`; KeePassXC uses Chrome's | B, K |
| Vivaldi | `Software\Vivaldi\NativeMessagingHosts\<host>`; KeePassXC uses Chrome's | B, K |
| Opera | Chrome's (Opera forum; no known key of its own) | ? |
| Firefox | `Software\Mozilla\NativeMessagingHosts\<host>` | D |

Chromium details (C, `launch_context_win.cc`):

- Search order: `HKCU` (if the `NativeMessagingUserLevelHosts` policy does not forbid it) and then `HKLM`; in each
  root, the 32-bit view before the 64-bit one. The **first key found wins**: Edge documents that, if the extension
  is on both Edge Add-ons and the Chrome Web Store, **both IDs** must be in the same `allowed_origins`.
- Only builds with `CHROMIUM_BRANDING` read `Software\Chromium` first; all others read only
  `Software\Google\Chrome`. That is why the `probe` writes its own keys **and** Chrome's, and Opera/Chrome
  share one.
- The host is started by `cmd.exe /d /s /c "<command>" < pipe > pipe`, unless the
  `NativeHostsExecutablesLaunchDirectly` policy (or the `LaunchWindowsNativeHostsDirectly` feature) is on.
  In both modes `start_hidden = true`, **except** when the executable is of the GUI subsystem and the launch is
  direct. **Risk to prove:** a console host started with `SW_HIDE` may have its first window created
  hidden. The confirmation window must call `ShowWindow(SW_SHOW)` explicitly or the binary must be of the Windows
  subsystem.
- **MSIX:** Microsoft's doc says that, on Windows 10 1903+, **new** files created in
  `AppData\Local`, `Local\Microsoft`, `Roaming`, and `Roaming\Microsoft` go to a private package location, and
  every write to `HKCU` is private copy-on-write. Without `desktop6:FileSystemWriteVirtualization` and
  `desktop6:RegistryWriteVirtualization` turned off (with `rescap:unvirtualizedResources`), neither the manifest nor
  the key reaches the browser. `register` has `--manifest-dir` to point at another folder. See
  `docs/prototypes/1-windows.md`.
- `register` in MSIX writes the **execution alias** as `path` (`platform::windows::msix_alias_path`), because
  files in `WindowsApps` are not executable by other processes.

### 3.2 macOS

User: `~/Library/Application Support/<folder>/NativeMessagingHosts/<host>.json`.

| Browser | `<folder>` | Basis |
|---|---|---|
| Chrome | `Google/Chrome` (Beta: `Google/Chrome Beta`; Dev: `Google/Chrome Dev`; Canary: `Google/Chrome Canary`) | D, B |
| Chrome for Testing (146+) | `Google/ChromeForTesting` (before 146 it used Chrome's folder) | D |
| Chromium | `Chromium` | D, K |
| Edge | `Microsoft Edge` (+ ` Beta`, ` Dev`, ` Canary`) | D, B |
| Brave | `BraveSoftware/Brave-Browser` (+ `-Beta`, `-Nightly`) | K; Beta/Nightly by analogy (?) |
| Vivaldi | `Vivaldi` | B, K |
| Opera | `com.operasoftware.Opera` | ? |
| Firefox | `Mozilla/NativeMessagingHosts` (note: `Mozilla`, no profile subfolder) | D |

System (all users): `/Library/Google/Chrome/NativeMessagingHosts/`,
`/Library/Microsoft/Edge/NativeMessagingHosts/`, `/Library/Application Support/Chromium/NativeMessagingHosts/`,
`/Library/Application Support/Mozilla/NativeMessagingHosts/` (D). web-eid installs the Chrome and Firefox ones
at those paths (W).

**Sandbox (Mac App Store).** Only the `NativeMessagingHosts/` folders need write access, through
`com.apple.security.temporary-exception.files.home-relative-path.read-write` (Bitwarden lists exactly
these folders in the MAS build; B, `entitlements.mas.plist`). The path is relative to the **real home**, but a sandboxed
process sees `$HOME` inside the container (`<home>/Library/Containers/<bundle>/Data`); `register` rebuilds
the real home by cutting at `/Library/Containers/` (Bitwarden uses `os.userInfo().homedir`, which queries the user
database). The binary the browser calls runs with `com.apple.security.inherit` in Bitwarden (B,
`entitlements.desktop_proxy*.plist`).

### 3.3 Linux

User: `<config folder>/NativeMessagingHosts/<host>.json` (Firefox: `~/.mozilla/native-messaging-hosts/`).
In Chromium this is `DIR_USER_DATA/NativeMessagingHosts`, that is, it **applies to any `--user-data-dir`**
(C, `chrome_paths.cc`: `DIR_USER_NATIVE_MESSAGING`; only compiled for Linux, ChromeOS, macOS, and Android, not
for Windows).

| Browser | Folder under `~/.config/` (user) | System | Basis |
|---|---|---|---|
| Chrome | `google-chrome`, `google-chrome-beta`, `google-chrome-unstable` | `/etc/opt/chrome/native-messaging-hosts/` | D, B |
| Chrome for Testing (146+) | `google-chrome-for-testing` | `/etc/opt/chrome_for_testing/native-messaging-hosts/` | D |
| Chromium | `chromium` | `/etc/chromium/native-messaging-hosts/` | D, C |
| Edge | `microsoft-edge`, `-beta`, `-dev` | `/etc/opt/edge/native-messaging-hosts/` | D |
| Brave | `BraveSoftware/Brave-Browser` (+ `-Beta`, `-Nightly`) | `/etc/chromium/native-messaging-hosts/`? | B, K; system ? |
| Vivaldi | `vivaldi`, `vivaldi-snapshot` | `/etc/chromium/native-messaging-hosts/` (forum) | B, K |
| Opera | `opera`, `opera-beta`, `opera-developer` | ? | ? (2014 forum) |
| Firefox | `~/.mozilla/native-messaging-hosts/` | `/usr/lib/mozilla/native-messaging-hosts/` and `/usr/lib64/...` | D |

web-eid (system package) installs into `/usr/lib/mozilla/native-messaging-hosts/` (Debian; `${LIBDIR}` on
the others), `/etc/chromium/native-messaging-hosts/`, and `/etc/opt/chrome/native-messaging-hosts/`, with the
absolute `path` `/usr/bin/web-eid` (W). Our app's `.deb` will do the same; the per-user `register` is the route for the
"complement" and for tests.

**Only register where the browser exists.** `register` only writes when the browser's config folder
already exists (same rule as Bitwarden, which warns "not found, skipping"). Creating `~/.config/vivaldi` for someone who
does not have Vivaldi clutters the home and misleads tools that treat the folder as proof of installation. The price: a
browser that is installed and never opened is ignored until the next registration; the final app registers at every start and
closes the gap. On Windows there is no folder to test, so the `HKCU` keys are always written.

**Snap (Chromium).** The Snap's Chromium reads `~/snap/chromium/common/chromium/NativeMessagingHosts/` (`register`
writes here) and runs with a private `/tmp` (the host log ends up in the Snap's `/tmp`). Whether the confinement
lets the host execute is not proven: **proof 4**.

**Flatpak.** Each Flatpak browser only sees its own `~/.var/app/<id>/`: manifest in
`~/.var/app/<id>/config/<folder>/NativeMessagingHosts/` (Firefox: `.../.mozilla/native-messaging-hosts/`) and the
binary **copied there**, since the sandbox's `/usr` is the runtime's (same technique as Bitwarden, which makes a
hard link). IDs: `com.google.Chrome`, `org.chromium.Chromium`, `com.microsoft.Edge`, `com.brave.Browser`,
`com.vivaldi.Vivaldi`, `com.opera.Opera`, `org.mozilla.firefox`. **Known limit:** inside the sandbox the host
sees neither `pcscd` nor the system's PKCS#11 modules, so it answers `ping` but does not sign. Solving this requires
the portal (§3.4) or `flatpak-spawn --host`; out of the spike's scope.

### 3.4 Firefox Snap and Flatpak: the portal

Confined Firefox (Ubuntu Snap; Flatpak) **cannot** read manifests or execute hosts. Instead,
it delegates to a D-Bus service that, outside the sandbox, finds the manifest, validates the extension ID against
`allowed_extensions`, **asks the user once per extension × application pair**, and starts the process,
returning `stdin`/`stdout`/`stderr` descriptors.

| | WebExtensions portal | Native Messaging Proxy |
|---|---|---|
| D-Bus name | `org.freedesktop.portal.WebExtensions` | `org.freedesktop.NativeMessagingProxy` |
| Methods | `CreateSession`, `GetManifest`, `Start` (+ `Response` signal), `GetPipes`, `Close` | `GetManifest`, `Start`, `Close` |
| Firefox preference | `widget.use-xdg-desktop-portal.native-messaging` | `widget.use-xdg-desktop-portal.native-messaging-proxy` |
| Values | 0 off, 1 on, 2 autodetect | same |
| Status (September 2026) | distribution patch in Ubuntu since 22.04 (did not land in upstream `xdg-desktop-portal`); it is what the **stable** Firefox Snap uses today | replacement, planned for Ubuntu 26.04. Bugzilla 1955255: `RESOLVED FIXED` in Firefox 157 (landed in mozilla-central on 2026-09-10), default preference 0; on 2026-09-29 Canonical had just enabled it only in the `nightly` Snap |
| User is asked | yes | no |

Sources: Firefox design doc and Bugzilla 1955255 (links at the end). Consequences for the app:

1. **The manifest must be on the host's disk**, in `~/.mozilla/native-messaging-hosts/` or
   `/usr/lib/mozilla/native-messaging-hosts/`, and the `path` must be a host path (never inside
   `~/snap/`). `register` already writes that file; nothing Snap-specific is needed.
2. **web-eid has no Snap-specific handling** in the installers I read (`CMakeLists.txt`, `web-eid.wxs`):
   it installs the system manifest with an absolute `path` and lets the portal do the rest. It is the same bet
   we are making.
3. **The portal starts the host, not Firefox**: the environment (variables, `DISPLAY`/`WAYLAND_DISPLAY`,
   `XDG_*`) is that of the session's `xdg-desktop-portal` service. `WEBSIGN_PROBE_PIN` does not arrive by this route, and an
   egui window depends on the portal having the graphical environment. Both are **proof 4** points.
4. On the first connection the user sees a portal dialog ("allow <extension> to start <application>"):
   it goes into the popup's help text.
5. Firefox Snap PKCS#11 modules do **not** come into play here: the portal only covers native messaging.

### 3.5 Safari

There is no manifest, registry key, `allowed_origins`, or child process. Apple (D):

- The web extension has three parts that run isolated: the **app** (macOS/iOS), the extension's **JavaScript**, and an
  **app extension (appex)** that mediates. The sandboxes are separate; what ties them together are *app groups*.
- The background script (or an extension page) calls `browser.runtime.sendNativeMessage(app, message)`;
  **the first parameter is ignored** and the message always goes to the appex of the app that contains the extension,
  in `beginRequest(with:)` (`NSExtensionRequestHandling`). **Content scripts cannot** talk to the appex,
  so the site's bridge goes through the background, as in Chrome.
- The reverse path (app → JS) uses `SFSafariApplication.dispatchMessage` and a `runtime.connectNative`
  whose port only connects to the app that contains the extension.
- web-eid uses the appex only as a bridge: it **opens the app** (`NSWorkspace.launchApplication`), stores the
  message in a `UserDefaults` shared through an app group, and notifies through `NSDistributedNotificationCenter`; the
  reply comes back the same way (W, `src/mac/`). This is the design of proof 2 (the appex signs directly or opens the
  app).
- Extension state: `SFSafariExtensionManager.getStateOfSafariExtension(withIdentifier:)`, from the app.

---

## 4. Extension pre-registration

"Pre-registering" makes the browser offer/install the extension without the user going to the store. It only applies to the
production extension; the test one is loaded with `--load-extension`.

| Browser | OS | Where | How the user sees it | Basis |
|---|---|---|---|---|
| Chrome | Windows | `HKLM\Software\Wow6432Node\Google\Chrome\Extensions\<id>` (32-bit: without `Wow6432Node`), `update_url` value = `https://clients2.google.com/service/update2/crx`. The code **also** reads `HKCU\Software\Google\Chrome\Extensions` (uses `HKCU` only when the `HKLM` key does not open) | must be enabled in a dialog | D, C |
| Chrome | macOS | `~/Library/Application Support/Google/Chrome/External Extensions/<id>.json` (user) or `/Library/Application Support/Google/Chrome/External Extensions/` (all users: only read if the tree's owners and permissions are root/admin) | dialog | D, W |
| Chrome | Linux | `/opt/google/chrome/extensions/<id>.json` or `/usr/share/google-chrome/extensions/<id>.json` | **installs by itself** | D |
| Chromium | Linux | `/usr/share/chromium/extensions/<id>.json` | same | W |
| Edge | Windows | `HKLM\Software\Microsoft\Edge\Extensions\<id>` (`update_url` = `https://edge.microsoft.com/extensionwebstorebase/v1/crx`) | dialog | D, W |
| Edge | macOS | `~/Library/Application Support/Microsoft Edge/External Extensions/` or `/Library/Application Support/Microsoft/Edge/External Extensions/` | dialog | D |
| Edge | Linux | `~/.config/microsoft-edge/External Extensions/` or `/usr/share/microsoft-edge/extensions/` | automatic | D |
| Firefox | all | **there is no per-user file.** `ExtensionSettings` policy (`installation_mode: force_installed` + `install_url`) in `policies.json` (`distribution/`, `/etc/firefox/policies/`) or in the registry `HKLM\Software\Policies\Mozilla\Firefox`. The doc only mentions `HKLM` | installs without asking | D |
| Firefox | Linux (package) | `/usr/share/mozilla/extensions/{ec8030f7-c20a-464f-9b0e-13a3a9e97384}/<id>.xpi` (web-eid packages it this way; the GUID is the Firefox application ID) | Firefox offers the installation | W |
| Safari | macOS | the extension comes **inside the app**; the user turns it on in Safari > Settings > Extensions | turn on manually | D |

Chromium external file: `{"external_update_url": "https://clients2.google.com/service/update2/crx"}`.
On Windows and macOS, Chrome only accepts external installation coming from the Chrome Web Store (since Chrome 33 and
44, respectively).

**Consequence for MSIX (HKCU-only):** Chrome pre-registration through `HKCU` is plausible per the code, but the
documentation only talks about `HKLM`; Firefox's requires a system policy, impossible without elevation. **Proof 1**
must say whether `HKCU\Software\Google\Chrome\Extensions\<id>` really works and, if not, the extension popup's
"Download" and the store page are the only path for Firefox.

Policies that can block all of this and that diagnostics must be able to explain (Chrome/Edge, D):
`NativeMessagingAllowlist`, `NativeMessagingBlocklist`, and `NativeMessagingUserLevelHosts` (when `false`,
Chrome ignores `HKCU` and the user folders, and only reads the system location).

---

## 5. What the kit implements

- `websign-probe register [--browser <all|chrome|chromium|edge|brave|vivaldi|opera|firefox>]... [--uninstall]
  [--user-data-dir DIR] [--dry-run] [--extension-id ID]... [--manifest-dir DIR]` writes the manifests from the
  tables above (`probe/src/nm/register/`). `allowed_origins` = development ID + non-empty store IDs +
  `--extension-id`. Firefox: `allowed_extensions` = `firefox_id`.
- `--user-data-dir DIR` writes only `DIR/NativeMessagingHosts/<host>.json`, which Chromium reads with
  `--user-data-dir=DIR` on Linux and macOS; this is what `nm-e2e` uses.
- The host (`probe/src/nm/`) recognizes the launch by the arguments in §2, speaks the `"v": 1` protocol, and logs
  to `<temp>/websign-probe-host.log` only: time, pid, family, extension ID, message type and size,
  result. Never a PIN, digest, signature, name, certificate, thumbprint, or site.
- Test-only variables: `WEBSIGN_PROBE_PIN` (PKCS#11 PIN) and `WEBSIGN_PROBE_MODULES` (extra modules, a list
  in `PATH` format), because the browser does not pass arguments.

### What remains unproven

| Item | Why | Proof |
|---|---|---|
| Firefox (all OSes) and Safari | there is no Firefox or Mac in this environment; `web-ext lint` approved the manifest (only the expected `service_worker` warning) | 2, 4 |
| Edge, Brave, Vivaldi, Opera on Windows/Linux/macOS | paths come from documentation and third parties (B, K); only Chromium was executed | 1, 2 |
| Opera (all), Brave Beta/Nightly | no primary source | 1, 2 |
| Chromium Snap, Flatpak, Firefox Snap | no such environments here | 4 |
| Real `--parent-window` on Windows and a window hidden by `SW_HIDE` | only exists on Windows | 1 |
| Pre-registration through `HKCU` (Chrome/Edge) | the doc only mentions `HKLM` | 1 |

---

## 6. Sources

**Documentation**

- Chrome, Native messaging: <https://developer.chrome.com/docs/extensions/develop/concepts/native-messaging>
- Chrome, service worker lifecycle (`connectNative` keeps it alive): <https://developer.chrome.com/docs/extensions/develop/concepts/service-workers/lifecycle>
- Chrome, installing extensions through file/registry: <https://developer.chrome.com/docs/extensions/how-to/distribute/install-extensions>
- Edge, Native messaging: <https://learn.microsoft.com/en-us/microsoft-edge/extensions/developer-guide/native-messaging> (source: <https://raw.githubusercontent.com/MicrosoftDocs/edge-developer/main/microsoft-edge/extensions/developer-guide/native-messaging.md>)
- Edge, alternate distribution: <https://raw.githubusercontent.com/MicrosoftDocs/edge-developer/main/microsoft-edge/extensions/developer-guide/alternate-distribution-options.md>
- MDN, Native messaging: <https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/Native_messaging>
- MDN, Native manifests: <https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/Native_manifests>
- Firefox, native messaging in a confined browser: <https://firefox-source-docs.mozilla.org/toolkit/components/extensions/webextensions/native-messaging-portal-design.html>
- Bugzilla 1955255 (native messaging proxy, Snap/Flatpak; status read through the REST API on 2026-09-29): <https://bugzilla.mozilla.org/show_bug.cgi?id=1955255>
- Ubuntu, call for testing native messaging in the Firefox Snap: <https://discourse.ubuntu.com/t/call-for-testing-native-messaging-support-in-the-firefox-snap/29759>
- Firefox, `ExtensionSettings` policy: <https://mozilla.github.io/policy-templates/>
- Apple, messaging between the app and the JavaScript of a Safari web extension: <https://developer.apple.com/documentation/safariservices/messaging-between-the-app-and-javascript-in-a-safari-web-extension>
- Microsoft, how packaged desktop apps (MSIX) run, with file and registry virtualization: <https://learn.microsoft.com/en-us/windows/msix/desktop/desktop-to-uwp-behind-the-scenes>

**Code read (not copied)**

- Chromium, manifest lookup on Windows and launch: <https://raw.githubusercontent.com/chromium/chromium/main/chrome/browser/extensions/api/messaging/launch_context_win.cc>
- Chromium, POSIX lookup: <https://raw.githubusercontent.com/chromium/chromium/main/chrome/browser/extensions/api/messaging/launch_context_posix.cc>
- Chromium, `DIR_USER_NATIVE_MESSAGING`: <https://raw.githubusercontent.com/chromium/chromium/main/chrome/common/chrome_paths.cc>
- Chromium, pre-registration through the registry: <https://raw.githubusercontent.com/chromium/chromium/main/chrome/browser/extensions/external_registry_loader_win.cc>
- Bitwarden, paths per OS: <https://raw.githubusercontent.com/bitwarden/clients/main/apps/desktop/src/main/native-messaging.main.ts>
- web-eid-app, installation: <https://raw.githubusercontent.com/web-eid/web-eid-app/main/src/app/CMakeLists.txt> and <https://raw.githubusercontent.com/web-eid/web-eid-app/main/install/web-eid.wxs>
- web-eid-app, Safari bridge: `src/mac/` (same repository)
- KeePassXC, manifest installer: `src/browser/NativeMessageInstaller.cpp`
