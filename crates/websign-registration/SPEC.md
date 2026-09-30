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
