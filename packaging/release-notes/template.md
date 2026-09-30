> **Prerelease, not code-signed yet.** Windows shows SmartScreen and macOS shows Gatekeeper once; the steps
> below get past them. Verify every download against `SHA256SUMS` first. Full guide:
> [docs/install.md](https://github.com/{{repo}}/blob/{{tag}}/docs/install.md).

WebeSign has two parts: the **app** (`websign`) and the **browser extension**. Install both. This is a
prerelease, so GitHub's "latest release" skips it: every command names the tag `{{tag}}`.

## Download

The repository is **private for now**, so downloads need an authenticated GitHub CLI (`gh auth login`). Once it is
public, the same files are at `https://github.com/{{repo}}/releases/download/{{tag}}/<file>` and `gh` is no longer
needed (the public commands are shown below each private one).

## macOS (universal) and Linux (amd64, arm64)

```sh
# Private repository (now)
gh release download {{tag}} --repo {{repo}} --pattern install.sh --pattern SHA256SUMS
sha256sum --ignore-missing -c SHA256SUMS && sh install.sh --version {{version}}   # macOS: shasum -a 256 --ignore-missing -c SHA256SUMS

# Once the repository is public: same steps, downloading with curl
curl -fsSLO https://github.com/{{repo}}/releases/download/{{tag}}/install.sh
curl -fsSLO https://github.com/{{repo}}/releases/download/{{tag}}/SHA256SUMS
sha256sum --ignore-missing -c SHA256SUMS && sh install.sh --version {{version}}   # macOS: shasum -a 256 --ignore-missing -c SHA256SUMS
```

The script verifies what it downloads, never pipes anything into a shell and never needs `sudo` on macOS. On Linux it
installs per user from the tarball; `--system` installs the verified deb or rpm instead.

**Linux packages** (system-wide):

```sh
# Ubuntu 22.04 / 24.04, Debian 12 (arm64: websign_{{version}}_arm64.deb)
gh release download {{tag}} --repo {{repo}} --pattern 'websign_{{version}}_amd64.deb' --pattern SHA256SUMS
sha256sum --ignore-missing -c SHA256SUMS && sudo apt install ./websign_{{version}}_amd64.deb

# Fedora 42, Rocky Linux 9 (arm64: websign-{{version}}-1.aarch64.rpm)
gh release download {{tag}} --repo {{repo}} --pattern 'websign-{{version}}-1.x86_64.rpm' --pattern SHA256SUMS
sha256sum --ignore-missing -c SHA256SUMS && sudo dnf install ./websign-{{version}}-1.x86_64.rpm
```

Smart cards and tokens need the PC/SC daemon: `sudo apt install pcscd` or `sudo dnf install pcsc-lite`, then
`sudo systemctl enable --now pcscd.socket`.

## Windows (x64, arm64)

```powershell
# Private repository (now)
gh release download {{tag}} --repo {{repo}} --pattern install.ps1 --pattern SHA256SUMS

# Once the repository is public
[Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
$base = 'https://github.com/{{repo}}/releases/download/{{tag}}'
Invoke-WebRequest -UseBasicParsing -Uri "$base/install.ps1" -OutFile install.ps1
Invoke-WebRequest -UseBasicParsing -Uri "$base/SHA256SUMS" -OutFile SHA256SUMS
```

Verify `install.ps1` (PowerShell 5.1 or 7), then run it. No administrator rights are needed.

```powershell
$file = 'install.ps1'
$m = Select-String -Path .\SHA256SUMS -Pattern ('^([0-9a-fA-F]{64}) [ *]' + [regex]::Escape($file) + '$')
if (-not $m) { throw "${file} is not listed in SHA256SUMS" }
if ((Get-FileHash -Algorithm SHA256 -LiteralPath $file).Hash -ne $m.Matches[0].Groups[1].Value) {
  throw "Checksum mismatch for ${file}: delete it and do not run it"
}
"${file}: OK"
powershell -NoProfile -ExecutionPolicy Bypass -File .\install.ps1 -Version {{version}}
```

## Browser extension

Download `websign-extension-{{version}}-chromium.zip` or `websign-extension-{{version}}-firefox.zip`, verify it
against `SHA256SUMS`, and unzip it.

- **Chrome, Edge, Brave:** `chrome://extensions` (`edge://extensions`, `brave://extensions`) → **Developer mode** →
  **Load unpacked** → the unzipped `websign-extension-{{version}}-chromium` folder. Keep the folder.
- **Firefox:** `about:debugging` → **This Firefox** → **Load Temporary Add-on** → `manifest.json` in the unzipped
  folder (lasts until Firefox restarts). Firefox Developer Edition, Nightly or ESR with
  `xpinstall.signatures.required` set to `false` can keep it: `about:addons` → **Install Add-on From File**.
- **Safari (macOS 13+, Safari 17+):** nothing to download: the extension is inside `WebeSign.app`. Open the app
  once, then in Safari **Settings → Advanced** → **Show features for web developers**, **Settings → Developer** →
  **Allow unsigned extensions** (again after every Safari restart while builds are unsigned), and tick WebeSign in
  **Settings → Extensions**. Keychain and CryptoTokenKit certificates work; a token reachable only
  through a PKCS#11 driver may not, because the host runs inside the extension's sandbox: use Chrome, Edge,
  Brave, Opera or Firefox for it.

## Unsigned builds: what to expect

| Where | What happens | What to do |
|---|---|---|
| Windows | SmartScreen "Windows protected your PC" when running a downloaded `websign.exe` by hand | The install script avoids it. Manual download: **More info → Run anyway**. |
| macOS | Gatekeeper "cannot be opened because the developer cannot be verified" | The install script removes the quarantine flag from `WebeSign.app` only. Manual: System Settings → Privacy & Security → **Open Anyway** (macOS 14 and earlier: right-click the app → **Open**), or `xattr -dr com.apple.quarantine /Applications/WebeSign.app`. |
| Linux | nothing (packages are unsigned) | — |
| Chrome / Edge / Brave | not in the stores yet | **Load unpacked** (above). |
| Firefox | release Firefox refuses unsigned add-ons | **Load Temporary Add-on** (above). |
| Safari | unsigned app extensions are off until allowed | **Allow Unsigned Extensions** (above), after each Safari restart. |

A checksum mismatch means the download is corrupt or altered: delete it and do not run it.

## Files

{{artifacts}}
