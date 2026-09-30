# crates/websign-registration/src

- `destination/` — registry writes and destination tests
- `linux/` — tests of the Linux manifest locations
- `browsers.rs` — The browsers the host can be registered with.
- `destination.rs` — One place a browser looks for the host, and how to write or remove it.
- `detect.rs` — Which browsers are installed, for the diagnostics Browsers tab.
- `home.rs` — The home directory browsers keep their folders in.
- `lib.rs` — Making browsers find the app: native messaging host manifests, the `websign:` URL scheme, extension pre-registration, and read-back status for diagnostics.
- `linux.rs` — Where Linux browsers look for user-level native messaging manifests.
- `macos.rs` — Where macOS browsers look for user-level native messaging manifests: `NativeMessagingHosts/` inside each browser's Application Support folder.
- `manifest.rs` — The native messaging host manifest: the small JSON file a browser reads to learn which program to start and which extensions may talk to it.
- `msix.rs` — Windows packaging facts.
- `preregister.rs` — Extension pre-registration on Windows: `HKCU\Software\Google\Chrome\ Extensions\<id>` (and the Edge/Brave equivalents) with the store's `update_url`, so the browser offers "New extension added — Enable" on its next start.
- `status.rs` — Reads registrations back, for diagnostics ("Chrome can't find the app") and for the repair button.
- `system.rs` — System-wide manifest locations on Linux, written by the deb/rpm post-install step (`websign register --scope system`, as root).
- `url_scheme.rs` — The `websign:` URL scheme, so the website's `/activate` page can start the app once to register it with browsers (stores run no install scripts).
- `windows.rs` — How Windows browsers find hosts: a `HKCU` key per browser whose default value is the path of a manifest file that can live anywhere.
