# Minimal self-test for install.ps1 (no Pester, so it runs on a bare runner):
#   powershell -NoProfile -ExecutionPolicy Bypass -File scripts\install\tests\install_ps1_selftest.ps1
# Works in Windows PowerShell 5.1 and PowerShell 7. It never touches the user
# PATH, the Start menu or a real install: the installer runs with -NoPath
# -NoShortcut -NoRegister -InstallDir <temp>.
$ErrorActionPreference = 'Stop'
$installer = Join-Path $PSScriptRoot '..\install.ps1'
. $installer
$Script:Failed = 0

function Test-Case([string]$Name, [scriptblock]$Body) {
    try { & $Body; Write-Host "ok   $Name" }
    catch { Write-Host "FAIL $Name : $($_.Exception.Message)"; $Script:Failed = 1 }
}
function Assert-Equal($Expected, $Actual) { if ($Expected -ne $Actual) { throw "expected '$Expected' but got '$Actual'" } }
function Assert-Throws([scriptblock]$Body) {
    $threw = $false
    try { & $Body } catch { $threw = $true }
    if (-not $threw) { throw 'expected an error' }
}

$root = Join-Path ([IO.Path]::GetTempPath()) ("websign-ps1-test-" + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $root | Out-Null

Test-Case 'architecture mapping' {
    Assert-Equal 'x64' (Get-WebsignArch 'AMD64' '')
    Assert-Equal 'arm64' (Get-WebsignArch 'ARM64' '')
    Assert-Equal 'arm64' (Get-WebsignArch 'x86' 'ARM64')
    Assert-Throws { Get-WebsignArch 'x86' '' }
}

Test-Case 'PATH entries are added once and removed cleanly' {
    Assert-Equal 'C:\a;C:\w' (Add-PathEntry 'C:\a' 'C:\w')
    Assert-Equal 'C:\a;C:\W\' (Add-PathEntry 'C:\a;C:\W\' 'C:\w')   # already there: unchanged
    Assert-Equal 'C:\a' (Remove-PathEntry 'C:\a;C:\w' 'C:\W')
    Assert-Equal 'C:\w' (Add-PathEntry '' 'C:\w')
}

# A fake release: a zip with a placeholder websign.exe and its SHA256SUMS.
$version = '9.9.9'
$arch = Get-WebsignArch $env:PROCESSOR_ARCHITECTURE $env:PROCESSOR_ARCHITEW6432
$zipName = "websign-$version-windows-$arch.zip"
$release = Join-Path $root 'release'
$content = Join-Path $root 'content'
New-Item -ItemType Directory -Path $release, $content | Out-Null
Set-Content -Path (Join-Path $content 'websign.exe') -Value 'placeholder'
Set-Content -Path (Join-Path $content 'README.txt') -Value 'readme'
Compress-Archive -Path (Join-Path $content '*') -DestinationPath (Join-Path $release $zipName)
function Write-Sums([string]$Folder) {
    $lines = Get-ChildItem -LiteralPath $Folder -File | Where-Object { $_.Name -ne 'SHA256SUMS' } | ForEach-Object {
        '{0}  {1}' -f (Get-FileHash -Algorithm SHA256 -LiteralPath $_.FullName).Hash.ToLowerInvariant(), $_.Name
    }
    Set-Content -Path (Join-Path $Folder 'SHA256SUMS') -Value $lines
}
Write-Sums $release

function Invoke-Installer([string]$Folder, [string]$Target, [string[]]$Extra) {
    $ErrorActionPreference = 'Continue'   # 5.1 turns redirected native stderr into errors
    $arguments = @('-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', $installer, '-Version', $version,
        '-ReleaseDir', $Folder, '-InstallDir', $Target, '-NoPath', '-NoShortcut', '-NoRegister') + $Extra
    # The same PowerShell that runs this test, so 5.1 and 7 are each covered.
    $shell = (Get-Process -Id $PID).Path
    & $shell @arguments *> $null
    return $LASTEXITCODE
}

Test-Case 'install from a verified release' {
    $target = Join-Path $root 'ok'
    Assert-Equal 0 (Invoke-Installer $release $target @())
    if (-not (Test-Path (Join-Path $target 'websign.exe'))) { throw 'websign.exe missing' }
    Assert-Equal 0 (Invoke-Installer $release $target @())   # second run upgrades in place
}

Test-Case 'checksum mismatch installs nothing' {
    $bad = Join-Path $root 'bad'
    Copy-Item -Recurse $release $bad
    Add-Content -Path (Join-Path $bad $zipName) -Value 'tampered'
    $target = Join-Path $root 'bad-target'
    if ((Invoke-Installer $bad $target @()) -eq 0) { throw 'the installer accepted a tampered zip' }
    if (Test-Path (Join-Path $target 'websign.exe')) { throw 'files were installed' }
}

Test-Case 'a file missing from SHA256SUMS is refused' {
    $unlisted = Join-Path $root 'unlisted'
    Copy-Item -Recurse $release $unlisted
    Set-Content -Path (Join-Path $unlisted 'SHA256SUMS') -Value ''
    if ((Invoke-Installer $unlisted (Join-Path $root 'unlisted-target') @()) -eq 0) { throw 'accepted' }
}

Test-Case 'a foreign folder is never emptied' {
    $foreign = Join-Path $root 'foreign'
    New-Item -ItemType Directory -Path $foreign | Out-Null
    Set-Content -Path (Join-Path $foreign 'keep.txt') -Value 'user data'
    if ((Invoke-Installer $release $foreign @()) -eq 0) { throw 'installed over a foreign folder' }
    Invoke-Installer $release $foreign @('-Uninstall') | Out-Null
    if (-not (Test-Path (Join-Path $foreign 'keep.txt'))) { throw 'user data deleted' }
}

Test-Case 'uninstall removes the folder' {
    $target = Join-Path $root 'ok'
    Assert-Equal 0 (Invoke-Installer $release $target @('-Uninstall'))
    if (Test-Path $target) { throw 'folder still there' }
    Assert-Equal 0 (Invoke-Installer $release $target @('-Uninstall'))   # idempotent
}

Remove-Item -LiteralPath $root -Recurse -Force -ErrorAction SilentlyContinue
exit $Script:Failed
