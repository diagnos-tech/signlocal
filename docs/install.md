# Install SignLocal

Use your smart card, USB token, or OS certificate to sign on any website, in Chrome, Edge, Firefox, Opera,
Brave or Safari on Windows, macOS or Linux. SignLocal has two parts: the **app** (`websign`, on your
computer) and the **browser extension**. Install both, in three steps:

1. Install the app for your system: [Windows](#windows-x64-arm64), [macOS](#macos-universal-apple-silicon-and-intel)
   or [Linux](#linux-amd64-arm64).
2. Add the [browser extension](#browser-extension).
3. Sign a harmless sample on the website's `/test` page to check that everything works.

The [download page](https://diagnos-tech.github.io/signlocal/download.html) shows the same steps for your
system and browser. Prerelease builds are **not code-signed yet** (see [Why unsigned?](#why-unsigned)), so each OS shows a
warning once. File names below are those of [`docs/architecture/packaging-and-release.md`](architecture/packaging-and-release.md);
replace `<v>` with the release version (for example `0.1.0`).

Releases are published as GitHub **prereleases** while they are unsigned. GitHub's "latest release" skips
prereleases, so every command below names the tag (`v<v>`) instead of relying on "latest".

## Downloading

The repository is **private for now**, so downloads need an authenticated GitHub CLI (`gh auth login`):

```sh
gh release download v<v> --repo diagnos-tech/signlocal --pattern install.sh --pattern SHA256SUMS
```

Once the repository is public, the same files are at
`https://github.com/diagnos-tech/signlocal/releases/download/v<v>/<file>` and `gh` is no longer needed.

### Verify every download

`SHA256SUMS` lists every release file. The install scripts verify what they download; verify the script itself
and anything you download by hand. Each command below exits with an error on a mismatch:

```sh
# Linux: checks every downloaded file listed in SHA256SUMS
sha256sum --ignore-missing -c SHA256SUMS
# macOS
shasum -a 256 --ignore-missing -c SHA256SUMS
```

```powershell
# Windows (PowerShell 5.1 or 7): set $file, then run the rest as is
$file = 'websign-<v>-windows-x64.zip'
$m = Select-String -Path .\SHA256SUMS -Pattern ('^([0-9a-fA-F]{64}) [ *]' + [regex]::Escape($file) + '$')
if (-not $m) { throw "${file} is not listed in SHA256SUMS" }
if ((Get-FileHash -Algorithm SHA256 -LiteralPath $file).Hash -ne $m.Matches[0].Groups[1].Value) {
  throw "Checksum mismatch for ${file}: delete it and do not run it"
}
"${file}: OK"
```

A mismatch means the download is corrupt or altered: delete it and do not run it.

## Windows (x64, arm64)

**Installer script** (recommended). It downloads the zip, verifies the hash, installs to
`%LOCALAPPDATA%\Programs\SignLocal`, adds it to your user `PATH` and registers the app with your browsers. No
administrator rights are needed.

```powershell
# Private repository (now)
gh release download v<v> --repo diagnos-tech/signlocal --pattern install.ps1 --pattern SHA256SUMS
# verify install.ps1 with the checksum block above ($file = 'install.ps1'), optionally read it, then:
powershell -NoProfile -ExecutionPolicy Bypass -File .\install.ps1 -Version <v>
```

`-ExecutionPolicy Bypass` applies to this one run only; Windows blocks scripts by default.

```powershell
# Once the repository is public: download, verify as above, then run
[Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
$base = 'https://github.com/diagnos-tech/signlocal/releases/download/v<v>'
Invoke-WebRequest -UseBasicParsing -Uri "$base/install.ps1" -OutFile install.ps1
Invoke-WebRequest -UseBasicParsing -Uri "$base/SHA256SUMS" -OutFile SHA256SUMS
```

The first line enables TLS 1.2, which Windows PowerShell 5.1 may not use by default.

**Manual.** Download `websign-<v>-windows-x64.zip` (or `websign-<v>-windows-arm64.zip`) and `SHA256SUMS`, verify
the hash, unzip to `%LOCALAPPDATA%\Programs\SignLocal`, then run `.\websign.exe install` in that folder.

> **SmartScreen.** Running a `websign.exe` unzipped from a browser download shows "Windows protected your PC".
> Click **More info**, then **Run anyway**. The install script avoids this prompt.

**Uninstall.** `powershell -NoProfile -ExecutionPolicy Bypass -File .\install.ps1 -Uninstall` runs
`websign uninstall` (removes the browser registrations, the `websign:` link handler and the Start menu entry) and
deletes the files and the `PATH` entry. By hand: run `websign uninstall` (add `--purge` to also delete settings and
remembered sites), delete `%LOCALAPPDATA%\Programs\SignLocal`, and remove that folder from your user `PATH`
(Settings, "Edit environment variables for your account").

## macOS (universal: Apple silicon and Intel)

```sh
# Private repository (now)
gh release download v<v> --repo diagnos-tech/signlocal --pattern install.sh --pattern SHA256SUMS
shasum -a 256 --ignore-missing -c SHA256SUMS && sh install.sh --version <v>

# Once the repository is public: same steps, downloading with curl
curl -fsSLO https://github.com/diagnos-tech/signlocal/releases/download/v<v>/install.sh
curl -fsSLO https://github.com/diagnos-tech/signlocal/releases/download/v<v>/SHA256SUMS
shasum -a 256 --ignore-missing -c SHA256SUMS && sh install.sh --version <v>
```

Read `install.sh` before running it if you like; it never needs `sudo`. It installs `SignLocal.app` (from
`websign-<v>-macos-universal.zip`) to `/Applications`, or `~/Applications` without admin rights, links
`~/.local/bin/websign` and registers the app with your browsers.

> **Gatekeeper.** The app is ad-hoc signed and **not notarized yet**. The script removes the quarantine flag from
> `SignLocal.app` only. If you installed by hand and macOS says the app cannot be opened or the developer cannot
> be verified, open System Settings, Privacy & Security, and click **Open Anyway** (on macOS 14 and earlier,
> right-clicking the app and choosing **Open** also works), or remove the flag from our app only:
> `xattr -dr com.apple.quarantine /Applications/SignLocal.app`.

**Uninstall.** `sh install.sh --uninstall` runs `websign uninstall` (removes the browser registrations) and deletes
the app and the link. By hand: `websign uninstall` (add `--purge` to also delete settings and remembered sites), then
`rm ~/.local/bin/websign` and move `SignLocal.app` to the Trash.

## Linux (amd64, arm64)

**Installer script** (any distribution; per user by default, no root):

```sh
# Private repository (now)
gh release download v<v> --repo diagnos-tech/signlocal --pattern install.sh --pattern SHA256SUMS
sha256sum --ignore-missing -c SHA256SUMS && sh install.sh --version <v>

# Once the repository is public: same steps, downloading with curl
curl -fsSLO https://github.com/diagnos-tech/signlocal/releases/download/v<v>/install.sh
curl -fsSLO https://github.com/diagnos-tech/signlocal/releases/download/v<v>/SHA256SUMS
sha256sum --ignore-missing -c SHA256SUMS && sh install.sh --version <v>
```

By default it installs `~/.local/bin/websign` from the tarball and registers the app for your user, without root.
On Debian, Ubuntu, Fedora and RHEL-family systems it asks whether to install the system package instead (through
`sudo`; the answer defaults to **No**, and `--yes` or a non-interactive run keeps that default). `--system` installs
the verified deb or rpm without asking. After a per-user install it prints the
`sudo ~/.local/bin/websign install --system` command in case you want other users registered too. Other options:
`--prefix DIR` (default `~/.local`), `--no-register`, `--dry-run`, `--uninstall`.

**Packages** (system-wide; registration for all users runs automatically). Download the file for your
architecture and `SHA256SUMS`, verify, then install:

```sh
# Ubuntu 22.04 / 24.04, Debian 12 (arm64: websign_<v>_arm64.deb)
gh release download v<v> --repo diagnos-tech/signlocal --pattern 'websign_<v>_amd64.deb' --pattern SHA256SUMS
sha256sum --ignore-missing -c SHA256SUMS && sudo apt install ./websign_<v>_amd64.deb

# Fedora 42, Rocky Linux 9 (arm64: websign-<v>-1.aarch64.rpm)
gh release download v<v> --repo diagnos-tech/signlocal --pattern 'websign-<v>-1.x86_64.rpm' --pattern SHA256SUMS
sha256sum --ignore-missing -c SHA256SUMS && sudo dnf install ./websign-<v>-1.x86_64.rpm
```

| Distribution | Files | Install |
|---|---|---|
| Ubuntu 22.04 / 24.04, Debian 12 | `websign_<v>_amd64.deb`, `websign_<v>_arm64.deb` | `sudo apt install ./websign_<v>_amd64.deb` |
| Fedora 42, Rocky Linux 9 | `websign-<v>-1.x86_64.rpm`, `websign-<v>-1.aarch64.rpm` | `sudo dnf install ./websign-<v>-1.x86_64.rpm` |
| Arch and others | `websign-<v>-linux-x64.tar.gz`, `websign-<v>-linux-arm64.tar.gz` | the installer script above (it uses the tarball) |

Smart cards and tokens need the PC/SC daemon: `sudo apt install pcscd` (Debian, Ubuntu) or
`sudo dnf install pcsc-lite` (Fedora, Rocky), then `sudo systemctl enable --now pcscd.socket`. The packages
depend on `libpcsclite1` / `pcsc-lite-libs` but leave the daemon's service to its own package; the
diagnostics window shows the command when the service is stopped.

**Uninstall (packages).** As each user who used the app, run `websign uninstall` (add `--purge` to also delete
settings and remembered sites; this also removes the per-user Firefox registrations). Then remove the
package; its pre-remove script removes the system registrations (`websign uninstall --system`):

```sh
sudo apt remove websign     # Debian, Ubuntu
sudo dnf remove websign     # Fedora, Rocky
```

**Uninstall (script or tarball).** `sh install.sh --uninstall` runs `websign uninstall` and deletes the files. If
you accepted `sudo websign install --system`, first run `sudo websign uninstall --system`.

## Browser extension

The extension is not in the stores yet. Download `websign-extension-<v>-chromium.zip` or
`websign-extension-<v>-firefox.zip` from the same release, verify it against `SHA256SUMS`, and unzip it.

| Browser | Steps |
|---|---|
| Chrome, Edge, Brave, Vivaldi | Open `chrome://extensions` (`edge://extensions`, `brave://extensions`, `vivaldi://extensions`), turn on **Developer mode**, click **Load unpacked** and pick the unzipped `websign-extension-<v>-chromium` folder. Keep the folder: the browser loads it from there. Its fixed development key gives the ID the app allows. |
| Opera, Opera GX | Open `opera://extensions`, turn on **Developer mode** (top right), click **Load unpacked** and pick the same unzipped `websign-extension-<v>-chromium` folder. Once the extension is in the Chrome Web Store, Opera installs it from there after you add Opera's **Install Chrome Extensions** add-on (from `addons.opera.com`). |
| Firefox | Open `about:debugging`, **This Firefox**, **Load Temporary Add-on**, and pick `manifest.json` in the unzipped folder. It lasts until Firefox restarts. To keep it, use Firefox Developer Edition, Nightly or ESR: set `xpinstall.signatures.required` to `false` in `about:config`, then in `about:addons` choose **Install Add-on From File** and pick the zip. |
| Safari | Comes inside `SignLocal.app`; nothing to download. See [Safari](#safari) below. |

**About the development key.** The Chromium zip pins its ID with a public development key from `project.toml`, so the
app can allow it before the extension is in a store. Anyone can reuse that key: an unpacked extension built with it gets
the same ID and can start the app like ours. Only load the zip from our release after checking `SHA256SUMS`, and do not
keep other unpacked extensions you do not trust. Your confirmation in the app window is still needed for every
signature. Store builds never carry this key.

The app registers itself with every browser it finds each time it starts (`websign register` does it by hand).
Opera reads Google Chrome's registration, so on Linux and macOS the app writes Chrome's folder for it even when
Chrome is not installed. Which browser and OS combinations are tested, and how, is in
[compatibility.md](compatibility.md#browsers-native-messaging).

**Uninstall:** remove it from the browser's extensions page (`about:addons` in Firefox), then delete the folder.

## Safari

The Safari extension ships inside `SignLocal.app` (macOS 13 or later, Safari 17 or later), so installing the app
on macOS is the whole download. The build is not signed by Apple yet, so Safari only loads it with
**Allow Unsigned Extensions**, a developer setting Safari turns off again every time it quits.

1. Install the app (see [macOS](#macos-universal-apple-silicon-and-intel)) and open **SignLocal** once from
   Applications, so macOS registers the extension it contains. Close the window.
2. In Safari, open **Settings → Advanced** and turn on **Show features for web developers**.
3. Open **Settings → Developer** and turn on **Allow unsigned extensions**. Safari asks for your Mac password.
4. Open **Settings → Extensions**, tick **SignLocal**, and allow it on the websites where you sign (**Edit
   Websites…**, or **Always Allow on This Website** the first time a site asks).
5. Check it: open the test page of the website (`/test/`) and sign once. The SignLocal window says
   "via Safari".

**After every Safari restart**, repeat step 3; the extension stays ticked. If SignLocal is missing from
**Settings → Extensions**, open the app once more, or run
`pluginkit -a /Applications/SignLocal.app/Contents/PlugIns/SignLocalExtension.appex` and restart Safari.

**What is different in Safari.** Safari runs the extension's native part in a sandbox, and the SignLocal window it
opens runs in that sandbox too. Certificates in the Keychain and tokens that work with macOS (CryptoTokenKit)
behave as in other browsers. Sites you choose to remember, and the diagnostics of Safari connections, are kept
separately from the other browsers'. Token drivers that are only PKCS#11 modules may not work in Safari: the sandbox limits which driver files can be
loaded (`/Applications`, `/Library`, `/usr/local/lib` and `/opt/homebrew/lib` are tried), and this has not been
verified with a real token yet. If your token does not show up in Safari, use another browser.

**Uninstall.** Untick SignLocal in **Settings → Extensions**; deleting the app removes the extension.

`TODO(gustavo)`: Apple Developer ID signing and notarization (Safari then loads the extension without the developer
setting), and later the Mac App Store.

## Why unsigned?

Signing certificates cost money and need company accounts that are still being set up. Windows shows SmartScreen and
macOS shows Gatekeeper until then; Linux packages are unsigned too. Checking `SHA256SUMS` is how you confirm what you
downloaded. `TODO(gustavo)`: Windows code signing (SignPath or Azure Trusted Signing), Apple Developer ID and
notarization, and an unlisted AMO signature for Firefox.
