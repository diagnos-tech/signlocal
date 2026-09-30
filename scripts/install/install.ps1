<#
.SYNOPSIS
  WebeSign installer for Windows (PowerShell 5.1 and 7).
.DESCRIPTION
  Downloads the zip and SHA256SUMS of release v<Version>, verifies the hash
  (a mismatch aborts before anything is installed), installs to
  %LOCALAPPDATA%\Programs\WebeSign, adds the folder to the user PATH, creates
  the Start menu shortcut and runs `websign install`. No administrator rights.
  Contract: docs/architecture/packaging-and-release.md, scripts/install/README.md.
.EXAMPLE
  powershell -NoProfile -ExecutionPolicy Bypass -File .\install.ps1 -Version 0.1.0
.EXAMPLE
  powershell -NoProfile -ExecutionPolicy Bypass -File .\install.ps1 -Uninstall
#>
[CmdletBinding()]
param(
    [string]$Version = $env:WEBSIGN_VERSION,
    [switch]$Uninstall,
    [switch]$NoRegister,
    [switch]$DryRun,
    # Where to install; default %LOCALAPPDATA%\Programs\WebeSign.
    [string]$InstallDir,
    # A folder that already holds the release files (tests, offline installs).
    [string]$ReleaseDir = $env:WEBSIGN_RELEASE_DIR,
    # Leave the user PATH / the Start menu alone (tests, managed machines).
    [switch]$NoPath,
    [switch]$NoShortcut
)

$ErrorActionPreference = 'Stop'
# Windows PowerShell 5.1 draws a progress bar per downloaded chunk, which makes
# Invoke-WebRequest many times slower.
$ProgressPreference = 'SilentlyContinue'

# Identifiers come from project.toml; `cargo xtask check release` keeps them equal.
$Script:Slug = 'websign'
$Script:AppName = 'WebeSign'
$Script:Repo = if ($env:WEBSIGN_REPO) { $env:WEBSIGN_REPO } else { 'diagnos-tech/web-esign' }

function Say([string]$Message) { [Console]::Error.WriteLine($Message) }

# Runs the action, or only prints it under -DryRun.
function Invoke-Step([string]$Description, [scriptblock]$Action) {
    if ($DryRun) { Say "+ $Description" } else { & $Action }
}

# ---- detection ---------------------------------------------------------------

# 'x64' or 'arm64' from PROCESSOR_ARCHITECTURE-style values. A 32-bit
# PowerShell on 64-bit Windows reports the real CPU in PROCESSOR_ARCHITEW6432.
function Get-WebsignArch([string]$Native, [string]$Emulated) {
    $value = if ($Emulated) { $Emulated } else { $Native }
    switch ($value.ToUpperInvariant()) {
        'AMD64' { return 'x64' }
        'ARM64' { return 'arm64' }
        default { throw "unsupported CPU architecture: $value" }
    }
}

# ---- download and verification -----------------------------------------------

# Whether gh is installed and logged in. The try/catch is for Windows
# PowerShell 5.1, which raises an error for native stderr output when the
# error preference is Stop.
function Test-GhLoggedIn {
    if (-not (Get-Command gh -ErrorAction SilentlyContinue)) { return $false }
    try {
        & gh auth status *> $null
        return ($LASTEXITCODE -eq 0)
    } catch {
        return $false
    }
}

# Downloads asset $Name of release v$Version into $Dest. Source order: a local
# release folder, gh when logged in, a bearer token for the private
# repository, anonymous HTTPS.
function Get-ReleaseFile([string]$Name, [string]$Dest) {
    $target = Join-Path $Dest $Name
    if ($ReleaseDir) {
        Copy-Item -LiteralPath (Join-Path $ReleaseDir $Name) -Destination $target -Force
        return
    }
    if (Test-GhLoggedIn) {
        & gh release download "v$Version" --repo $Script:Repo --pattern $Name --dir $Dest --clobber
        if ($LASTEXITCODE -ne 0) { throw "gh could not download $Name of v$Version" }
        return
    }
    # Windows PowerShell 5.1 may not offer TLS 1.2 by default.
    [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
    if ($env:WEBSIGN_GITHUB_TOKEN) {
        $headers = @{ Authorization = "Bearer $($env:WEBSIGN_GITHUB_TOKEN)"; Accept = 'application/vnd.github+json' }
        $api = "https://api.github.com/repos/$($Script:Repo)/releases"
        $release = Invoke-RestMethod -Headers $headers -Uri "$api/tags/v$Version"
        $asset = $release.assets | Where-Object { $_.name -eq $Name } | Select-Object -First 1
        if (-not $asset) { throw "release v$Version has no file named $Name" }
        $headers.Accept = 'application/octet-stream'
        Invoke-WebRequest -UseBasicParsing -Headers $headers -Uri "$api/assets/$($asset.id)" -OutFile $target
        return
    }
    $url = "https://github.com/$($Script:Repo)/releases/download/v$Version/$Name"
    try {
        Invoke-WebRequest -UseBasicParsing -Uri $url -OutFile $target
    } catch {
        throw "cannot download $Name (private repository? run 'gh auth login' or set WEBSIGN_GITHUB_TOKEN): $($_.Exception.Message)"
    }
}

# Throws unless $File matches its line in $SumsFile. Fails closed: a missing
# line is an error, not a pass.
function Assert-Checksum([string]$File, [string]$SumsFile) {
    $name = Split-Path -Leaf $File
    $pattern = '^([0-9a-fA-F]{64}) [ *]' + [regex]::Escape($name) + '$'
    $line = Select-String -Path $SumsFile -Pattern $pattern | Select-Object -First 1
    if (-not $line) { throw "$name is not listed in SHA256SUMS" }
    $want = $line.Matches[0].Groups[1].Value
    $got = (Get-FileHash -Algorithm SHA256 -LiteralPath $File).Hash
    if ($got -ne $want) { throw "checksum mismatch for ${name}: refusing to install (delete the download)" }
    Say "verified $name"
}

# ---- PATH --------------------------------------------------------------------

function Split-PathList([string]$List) {
    @($List -split ';' | Where-Object { $_ -ne '' })
}

function Test-SameDir([string]$A, [string]$B) {
    $A.TrimEnd('\') -ieq $B.TrimEnd('\')
}

# The PATH text with $Dir added once (case-insensitive).
function Add-PathEntry([string]$List, [string]$Dir) {
    $entries = @(Split-PathList $List)
    if ($entries | Where-Object { Test-SameDir $_ $Dir }) { return ($entries -join ';') }
    return (($entries + $Dir) -join ';')
}

# The PATH text without $Dir.
function Remove-PathEntry([string]$List, [string]$Dir) {
    (Split-PathList $List | Where-Object { -not (Test-SameDir $_ $Dir) }) -join ';'
}

function Set-UserPath([string]$Dir, [bool]$Add) {
    $current = [Environment]::GetEnvironmentVariable('Path', 'User')
    $updated = if ($Add) { Add-PathEntry $current $Dir } else { Remove-PathEntry $current $Dir }
    if ($updated -ne $current) { [Environment]::SetEnvironmentVariable('Path', $updated, 'User') }
}

# ---- install -----------------------------------------------------------------

function Get-StartMenuLink {
    Join-Path ([Environment]::GetFolderPath('Programs')) "$($Script:AppName).lnk"
}

function New-StartMenuLink([string]$Exe) {
    $shell = New-Object -ComObject WScript.Shell
    $link = $shell.CreateShortcut((Get-StartMenuLink))
    $link.TargetPath = $Exe
    $link.WorkingDirectory = Split-Path -Parent $Exe
    $link.Save()
}

function Get-InstallDir {
    if ($InstallDir) { return $InstallDir }
    Join-Path $env:LOCALAPPDATA "Programs\$($Script:AppName)"
}

# Whether $Dir may be emptied: it does not exist, is empty, or holds our
# executable. Guards against -InstallDir pointing at a folder with other data.
function Test-OwnedDir([string]$Dir) {
    if (-not (Test-Path -LiteralPath $Dir)) { return $true }
    if (Test-Path -LiteralPath (Join-Path $Dir "$($Script:Slug).exe")) { return $true }
    return -not (Get-ChildItem -LiteralPath $Dir -Force | Select-Object -First 1)
}

# Removes the Mark of the Web from the installed files only. Unblock-File
# does not support Linux, where the self-test also runs ($IsLinux is unset,
# so false, in Windows PowerShell 5.1).
function Unblock-Tree([string]$Dir) {
    if (-not $IsLinux) { Get-ChildItem -LiteralPath $Dir -Recurse -File | Unblock-File }
}

function Install-Websign {
    if (-not $Version) { throw '-Version is required: prereleases are not "latest" (see docs/install.md)' }
    $Script:Version = $Version.TrimStart('v')
    $arch = Get-WebsignArch $env:PROCESSOR_ARCHITECTURE $env:PROCESSOR_ARCHITEW6432
    $zipName = "$($Script:Slug)-$($Script:Version)-windows-$arch.zip"
    $dir = Get-InstallDir
    if (-not (Test-OwnedDir $dir)) { throw "$dir is not empty and has no $($Script:Slug).exe: refusing to replace it" }
    $work = Join-Path ([IO.Path]::GetTempPath()) ("websign-install-" + [Guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path $work | Out-Null
    try {
        Get-ReleaseFile 'SHA256SUMS' $work
        Get-ReleaseFile $zipName $work
        Assert-Checksum (Join-Path $work $zipName) (Join-Path $work 'SHA256SUMS')
        $staging = Join-Path $work 'files'
        Expand-Archive -LiteralPath (Join-Path $work $zipName) -DestinationPath $staging -Force
        $exe = Join-Path $dir "$($Script:Slug).exe"
        Invoke-Step "replace the contents of $dir" {
            if (Test-Path -LiteralPath $dir) { Get-ChildItem -LiteralPath $dir | Remove-Item -Recurse -Force }
            New-Item -ItemType Directory -Path $dir -Force | Out-Null
            Copy-Item -Path (Join-Path $staging '*') -Destination $dir -Recurse -Force
            Unblock-Tree $dir
        }
        if (-not $NoPath) { Invoke-Step "add $dir to the user PATH" { Set-UserPath $dir $true } }
        if (-not $NoShortcut) { Invoke-Step 'create the Start menu shortcut' { New-StartMenuLink $exe } }
        if ($NoRegister) {
            Say "skipping browser registration (-NoRegister); run '$exe install' later"
        } else {
            Invoke-Step "$exe install" {
                try { & $exe install } catch { $global:LASTEXITCODE = 1 }
                if ($LASTEXITCODE -ne 0) { Say "registration reported a problem; run '$exe doctor'" }
            }
        }
    } finally {
        Remove-Item -LiteralPath $work -Recurse -Force -ErrorAction SilentlyContinue
    }
    Say ''
    Say "$($Script:AppName) is installed. Last step: install the browser extension."
    Say "Steps per browser: https://github.com/$($Script:Repo)/blob/main/docs/install.md#browser-extension"
    Say "(download websign-extension-$($Script:Version)-chromium.zip or -firefox.zip from the same release)."
}

function Uninstall-Websign {
    $dir = Get-InstallDir
    $exe = Join-Path $dir "$($Script:Slug).exe"
    if (Test-Path -LiteralPath $exe) {
        Invoke-Step "$exe uninstall" {
            try { & $exe uninstall } catch { Say "websign uninstall failed: $($_.Exception.Message)" }
        }
    }
    if (-not $NoShortcut) { Invoke-Step 'remove the Start menu shortcut' { Remove-Item -LiteralPath (Get-StartMenuLink) -Force -ErrorAction SilentlyContinue } }
    if (-not $NoPath) { Invoke-Step "remove $dir from the user PATH" { Set-UserPath $dir $false } }
    if (Test-OwnedDir $dir) {
        Invoke-Step "delete $dir" { Remove-Item -LiteralPath $dir -Recurse -Force -ErrorAction SilentlyContinue }
    } else {
        Say "left $dir in place: it has no $($Script:Slug).exe"
    }
    Say "$($Script:AppName) removed. Remove the extension from your browser's extensions page."
}

# Dot-sourcing (the self-test) loads the functions without running anything.
if ($MyInvocation.InvocationName -ne '.') {
    if ($Uninstall) { Uninstall-Websign } else { Install-Websign }
}
