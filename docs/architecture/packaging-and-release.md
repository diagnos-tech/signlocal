# Packaging and release

## Channels

| Channel | When | Windows | macOS | Linux | Extension |
|---|---|---|---|---|---|
| **direct** | now | zip + `install.ps1` (x64, arm64) | universal `.app` zip + `install.sh` (not sandboxed, so PKCS#11 works: D10), with the Safari appex inside | `.deb`, `.rpm` (amd64, arm64), tar.gz + `install.sh` | unpacked zips (Chromium, Firefox); Safari inside the `.app` (unsigned: Allow Unsigned Extensions) |
| **store** | later (D10) | MSIX (Microsoft Store; `unvirtualizedResources`, proven in proof 1) | Mac App Store (sandbox, entitlements of proof 2) + the same Safari appex; PKCS#11-only tokens need the Add-on | APT/YUM repositories on GitHub Pages | Chrome Web Store, Edge Add-ons, AMO, Safari (inside the app) |

Same code on both channels. Differences are detected at run time
(`app/src/platform/channel.rs`: MSIX package identity, macOS sandbox
container) and reported as `AppInfo.channel`; there are no build forks.

## Artifacts (every `v*` tag)

| File | Contents |
|---|---|
| `websign-<v>-windows-x64.zip`, `websign-<v>-windows-arm64.zip` | `websign.exe` (MSVC, `+crt-static`), `README.txt`, licenses |
| `websign-<v>-macos-universal.zip` | `WebeSign.app` (arm64 + x86_64 `lipo`) with the Safari appex in `Contents/PlugIns` (§Safari appex), ad-hoc signed inside out |
| `websign_<v>_amd64.deb`, `websign_<v>_arm64.deb` | `/usr/bin/websign`, desktop entry, icons, system manifests via postinst |
| `websign-<v>-1.x86_64.rpm`, `websign-<v>-1.aarch64.rpm` | same layout |
| `websign-<v>-linux-x64.tar.gz`, `websign-<v>-linux-arm64.tar.gz` | binary + desktop entry + icons, for `install.sh` and other distros |
| `websign-extension-<v>-chromium.zip`, `websign-extension-<v>-firefox.zip` | built extension (development key for Chromium, so its ID matches the manifests) |
| `install.sh`, `install.ps1` | one-line installers |
| `SHA256SUMS` | hashes of every file above |

Built by `cargo xtask package --target <triple>` (nfpm for deb/rpm), on the
native runner of each OS; the release job assembles and publishes.

### Safari appex

`cargo xtask package --target universal-apple-darwin` (`xtask/src/package/safari.rs`)
adds the app extension Safari requires ([`safari/SPEC.md`](../../safari/SPEC.md)):

```
WebeSign.app/Contents/
  MacOS/websign                                  the app (Chrome, Edge, Brave, Firefox, CLI)
  PlugIns/WebeSignExtension.appex/Contents/
    Info.plist                                   packaging/macos/Extension-Info.plist.in
    MacOS/WebeSignExtension                      safari/Relay + safari/Extension, swiftc per slice, lipo
    MacOS/websign                                copy of the app binary, started per Safari session
    Resources/                                   extension/.output/safari-mv3 (the WXT safari build)
```

- The WXT safari build must exist first (`WEBSIGN_CHANNEL=direct bunx wxt build
  -b safari` in `extension/`); packaging fails with that command otherwise.
- Signed inside out, never `--deep` (which would drop the appex's
  entitlements): the host copy with `packaging/macos/safari-host.entitlements`
  (sandbox + inherit), the appex with `safari-extension.entitlements`
  (sandbox, smart cards, USB, read-only PKCS#11 locations), then the app
  without entitlements. Safari loads only sandboxed app extensions; the app
  itself stays unsandboxed for the other browsers (D10).
- The appex needs no app group and no Team ID: it starts its own host
  instead of talking to a running app, so the ad-hoc build works as is.
- CI: `.github/workflows/safari.yml` packages the app and inspects all of the
  above; loading it into Safari is the manual script of
  [`docs/compatibility.md`](../compatibility.md) §Safari.

## Install locations

| OS / package | Binary | Registration |
|---|---|---|
| Windows direct | `%LOCALAPPDATA%\Programs\WebeSign\websign.exe`, folder added to the user `PATH` | `websign install`: `HKCU\Software\<vendor>\NativeMessagingHosts\dev.websign.host` for every browser, manifests in `%LOCALAPPDATA%\websign\NativeMessagingHosts`, `HKCU\Software\Classes\websign`, Start menu shortcut |
| Windows store | package folder; alias `websign.exe` in `%LOCALAPPDATA%\Microsoft\WindowsApps` | same keys, written by the app on first start (manifests point at the alias) |
| macOS direct | `/Applications/WebeSign.app` (or `~/Applications` without admin rights); `~/.local/bin/websign` symlink | `websign install`: manifests in each browser's `NativeMessagingHosts` under `~/Library/Application Support`; URL scheme from `Info.plist` |
| Linux deb/rpm | `/usr/bin/websign`, `/usr/share/applications/websign.desktop` | postinst runs `websign install --system` (system manifests; the Firefox Snap portal also reads `~/.mozilla`, which `websign install` writes per user); it leaves `pcscd.socket` to the pcscd package (diagnostics shows the enable command); `Depends: libpcsclite1` / `Requires: pcsc-lite-libs`; `Recommends: pcscd, p11-kit` |
| Linux tar.gz | `~/.local/bin/websign`, `~/.local/share/applications/websign.desktop` | `websign install` (per user); per-user manifests also cover the Firefox Snap; the script offers `sudo websign install --system` for other users |

`websign-client` and `@websign/desktop` search these locations in this order
after `WEBSIGN_EXECUTABLE` and `PATH`.

## Install commands

Releases stay GitHub **prereleases** while the builds are unsigned, and both
`releases/latest` and a bare `gh release download` skip prereleases, so every
command names the tag. Each one downloads, verifies `SHA256SUMS`, then runs;
nothing is piped into a shell. [`docs/install.md`](../install.md) has the full
per-OS steps; the shape is:

```sh
# macOS and Linux (private repository: authenticated with gh)
gh release download v<version> --repo diagnos-tech/web-esign --pattern install.sh --pattern SHA256SUMS
sha256sum --ignore-missing -c SHA256SUMS && sh install.sh --version <version>   # macOS: shasum -a 256
```

```powershell
# Windows (private repository: authenticated with gh)
gh release download v<version> --repo diagnos-tech/web-esign --pattern install.ps1 --pattern SHA256SUMS
# verify install.ps1 against SHA256SUMS (docs/install.md), then:
powershell -NoProfile -ExecutionPolicy Bypass -File .\install.ps1 -Version <version>
```

Once the repository is public, the same files come from
`https://github.com/diagnos-tech/web-esign/releases/download/v<version>/`.

The scripts use `gh` when it is installed and logged in, else
`WEBSIGN_GITHUB_TOKEN` as a bearer token, else anonymous HTTPS. They:
detect OS and architecture; download the artifact and `SHA256SUMS`; **verify
the hash**; place the app (table above); remove the quarantine/Mark of the
Web; run `websign install`; print the extension step. Flags: `--version`,
`--prefix`, `--no-register`, `--uninstall`, `--dry-run` (`-Version`, … in
PowerShell). Contract: `scripts/install/README.md`.

## Unsigned builds: what users must do (for now)

Direct builds are not code-signed yet (costs and accounts pending, brief
§12). Every release page and README shows this notice:

| Where | What happens | What to do |
|---|---|---|
| Windows | SmartScreen "Windows protected your PC" when running a downloaded `websign.exe` by hand | The install script avoids it (it downloads with PowerShell and unblocks the file). Manual download: **More info → Run anyway**. |
| macOS | Gatekeeper "cannot be opened because the developer cannot be verified" | The install script removes the quarantine attribute from `WebeSign.app` only (`xattr -dr com.apple.quarantine`). Manual: System Settings → Privacy & Security → **Open Anyway** (macOS 14 and earlier: right-click the app → **Open**). |
| Linux | nothing (packages are unsigned; `apt install ./websign_<v>_amd64.deb`, `dnf install ./websign-<v>-1.x86_64.rpm`) | — |
| Chrome / Edge / Brave | the extension is not in the stores yet | `chrome://extensions` → Developer mode → **Load unpacked** → the unzipped `websign-extension-<v>-chromium` folder. The development key pins the ID the app allows. |
| Firefox | release Firefox refuses unsigned add-ons | `about:debugging` → This Firefox → **Load Temporary Add-on** (lasts until restart), or Firefox Developer/Nightly/ESR with `xpinstall.signatures.required = false`. `TODO(gustavo)`: sign on AMO as *unlisted* (free, automatic) to remove this step. |
| Safari | Safari loads unsigned extensions only with **Settings → Developer → Allow unsigned extensions**, which it turns off at every quit | [`install.md`](../install.md) §Safari. `TODO(gustavo)`: Developer ID signing and notarization remove this step. |

## Release workflow

`.github/workflows/release.yml`:

1. Versions move in lockstep: workspace `Cargo.toml`, `sdk/`, `clients/node/`,
   `extension/`. The **gate** job runs `cargo xtask check release`, which
   fails unless they equal the tag without its `v`.
2. Push tag `vX.Y.Z` (or run the workflow by hand with an existing tag to
   rebuild and replace a release). `cargo xtask package` builds every
   artifact without caches: Linux in a `rockylinux:9` container on
   `ubuntu-24.04` and `ubuntu-24.04-arm` (Rocky 9's glibc 2.34 is the oldest
   of the supported distributions, so it is the Linux floor; deb and rpm with the pinned, hash-checked nfpm;
   tar.gz), Windows x64 and arm64 on
   `windows-latest` (arm64 cross-compiled by MSVC), the universal `.app` on
   `macos-latest` (lipo, the Safari appex from `swiftc` and the WXT safari
   build, which the job builds first with `WEBSIGN_CHANNEL=direct bunx wxt
   build -b safari`; ad-hoc signature; `LSMinimumSystemVersion` 13.0, the
   macOS floor of [`compatibility.md`](compatibility.md)). Every release binary is checked
   for the `e2e` marker. The extension zips are the `direct` channel
   (`WEBSIGN_CHANNEL=direct bun run zip`, development key). The e2e suite
   runs in `e2e.yml`, not against the packaged files.
3. The **publish** job, the only one allowed to write, checks that every file
   of §Artifacts is present, runs `cargo xtask package --sums-only` (adds
   `install.sh`, `install.ps1` and `SHA256SUMS`) and creates a GitHub
   **prerelease** (draft until every file is uploaded, `--verify-tag`). Its
   page is `packaging/release-notes/template.md` rendered by `render.sh`:
   the install commands of [`docs/install.md`](../install.md) for the tag, the
   unsigned-build notice, a file table with SHA-256, then GitHub's generated
   changes.
4. npm packages (`@websign/sdk`, `@websign/desktop`) and the `websign-client`
   crate are published by hand by the maintainer until the names are final
   (D6). The workflow attaches `npm pack` tarballs, the Safari and Edge zips
   and the Firefox sources zip to the run (`maintainer-npm` artifact), never
   to the release.

## Later: signing and stores

Authenticode (SignPath or Azure Trusted Signing), Apple Developer ID +
notarization, MSIX and Mac App Store submissions, AMO/Chrome/Edge listings.
The code paths already exist (proofs 1 and 2); only packaging and accounts
remain.
