# websign-registration — specification

Browser manifests per OS are **promoted from the Phase-0 kit**
(`probe/src/nm/register`) with their tests: target lists for Linux (native,
Snap, Flatpak with a host copy), macOS (real home from `getpwuid_r` inside
the sandbox) and Windows (`HKCU` keys pointing to manifest files); writes
only where the browser's folder exists; manifest JSON with our extension
origins (`allowed_origins`) or the Gecko ID (`allowed_extensions`).
`run`/`plan`/`host_binary` are promoted too. The rest is NEW.

## 1. `run` / `plan`

As promoted, plus `Scope::System` → `system::targets` (Linux only; other
OSes → `Unsupported`).

## 2. `detect::installed_browsers`

- Windows: `HKLM`/`HKCU\Software\Clients\StartMenuInternet\*` and `App Paths`
  (`chrome.exe`, `msedge.exe`, `firefox.exe`, `brave.exe`, `vivaldi.exe`,
  `opera.exe`); version from the executable's version resource.
- macOS: LaunchServices by bundle ID (`com.google.Chrome`,
  `com.microsoft.edgemac`, `org.mozilla.firefox`, `com.brave.Browser`,
  `com.vivaldi.Vivaldi`, `com.operasoftware.Opera`, `org.chromium.Chromium`,
  `com.apple.Safari`); version from `CFBundleShortVersionString`.
- Linux: executables on `PATH` and `/opt` (`google-chrome`, `chromium`,
  `microsoft-edge`, `firefox`, `brave-browser`, `vivaldi`, `opera`), Snap
  (`/snap/bin/<name>` → `Snap`), Flatpak (`flatpak list --app` IDs →
  `Flatpak`).
- Never reads profile folders.

## 3. `status::registration_state`

Reads the manifest(s) the browser would read (per-user first; on Linux also
system): missing → `Missing`; unreadable JSON, wrong `name`, or none of our
IDs → `Broken`; `path` ≠ `host_binary()` → `PointsElsewhere`; else
`Registered`. Windows: the registry value must point to an existing file.

## 4. `url_scheme`

- Windows: `HKCU\Software\Classes\websign` with `URL Protocol` = `""` and
  `shell\open\command` = `"<exe>" "%1"`.
- Linux: `~/.local/share/applications/websign-url.desktop` with
  `MimeType=x-scheme-handler/websign;` and `Exec=<exe> %u`, then
  `xdg-mime default websign-url.desktop x-scheme-handler/websign` (skipped
  when `xdg-mime` is missing → `Skipped`).
- macOS: `Skipped("declared in Info.plist")`.
- `unregister` removes what `register` wrote; absent → `NotPresent`.

## 5. `preregister` (Windows)

For each non-empty store ID: Chrome/Brave `HKCU\Software\Google\Chrome\
Extensions\<id>` and Edge `HKCU\Software\Microsoft\Edge\Extensions\<id>`
with `update_url` (`https://clients2.google.com/service/update2/crx`,
`https://edge.microsoft.com/extensionwebstorebase/v1/crx`). No store IDs yet
→ empty result. Other OSes → empty result.

## 6. `system::targets` (Linux)

| Browser | System folder |
|---|---|
| Chrome | `/etc/opt/chrome/native-messaging-hosts` |
| Chromium | `/etc/chromium/native-messaging-hosts` |
| Edge | `/etc/opt/edge/native-messaging-hosts` |
| Brave | `/etc/opt/chrome/native-messaging-hosts` (Brave reads Chrome's) |
| Vivaldi | `/etc/opt/vivaldi/native-messaging-hosts` |
| Opera | `/etc/opt/chrome/native-messaging-hosts` |
| Firefox | `<lib>/mozilla/native-messaging-hosts` for each `lib_dirs` entry |

System targets are written unconditionally (no `requires`), deduplicated by
path.

## 7. Additions beyond the sections above

- `registry::{Registry, Hive, MemoryRegistry, system}`: the registry trait
  used by §1 (Windows keys), §2 (Windows detection), §4 and §5; writes are
  `HKCU`-only by construction. `destination::apply_with` is `apply` against a
  given registry.
- Linux user targets also cover Chrome Canary (`google-chrome-canary`) and
  Snap Firefox (`~/.mozilla/native-messaging-hosts`, written when
  `~/snap/firefox` exists, because the portal reads the host's folder).
  macOS user targets add Arc (`Arc/User Data`, selected with Chrome).
  Flatpak host copies are named `<slug>`.
- §3 on Windows reads keys in the browser's documented lookup order (Edge:
  Edge, Chromium, Chrome; every other browser: its own key only) and the
  first key found decides. A browser with several channels is `Registered` when any
  channel is; otherwise the first problem in reading order is reported.
- §3 reads `HKCU` only, although browsers fall back to `HKLM`: the app
  writes only `HKCU`, which browsers read first, so `Missing` leads the
  repair button to write the key that wins. A machine-wide key from another
  installer is not ours to judge.
- §4 Linux honours an absolute `$XDG_DATA_HOME`. When `xdg-mime` is missing
  the entry is still written and the outcome is `Skipped` with a reason that
  says so: the file exists (`unregister` removes it), but nothing made it the
  default handler, so the caller must not treat the scheme as working.
  `Written` means both steps ran; an `xdg-mime` that runs and fails is
  `Failed`.
  `url_scheme::info_plist_url_types()` gives the macOS `CFBundleURLTypes`
  fragment for packaging.
- §5 `preregister::linux_system_files()` gives the external-extension files
  (`/usr/share/{google-chrome,chromium,microsoft-edge}/extensions/<id>.json`
  with `external_update_url`) for the Linux packages; empty until a store ID
  exists.
- §2 does not report Safari or Arc: `Browser` means "a browser with native
  messaging manifests", and every per-OS table (`family`, Windows keys,
  system folders) is exhaustive over it. Safari has no manifest (direct
  builds do not support it; the store channel's appex handles it later), and
  Arc is registered as a Chrome channel on macOS
  (`Arc/User Data/NativeMessagingHosts`). TODO(gustavo): report Safari with
  the store channel, as its own detection result rather than a `Browser`.
- Registration outcomes and status reasons name the paths involved, which
  may contain the user's login name (home folder); callers that log them
  treat them as personal data.
