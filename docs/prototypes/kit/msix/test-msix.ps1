#Requires -Version 7.2
<#
.SYNOPSIS
    Proves the Microsoft Store path with the package build-msix.ps1 installed:
    started through its app execution alias, the packaged probe registers the
    native messaging host where browsers really look, and what it registers
    starts.

.DESCRIPTION
    Required checks:
      alias     the alias exists in %LOCALAPPDATA%\Microsoft\WindowsApps and
                runs (`--version`), with its output reaching our stdout
      (a) hkcu  `register` run through the alias writes the keys to the REAL
                HKCU, read here by reg.exe from outside the package
      (b) file  each key points to a manifest file that exists at that real
                path (not in the package's private copy) and was just written
      (c) path  the manifest `path` is the alias, and that alias runs
    Observed: the per-package alias copy, the other browsers' keys, and
    (d) the native messaging end-to-end test in nm-e2e/ through the alias,
    when that test exists.

    Every key this script checks is deleted first, so only the packaged run
    can have written them.

    Only the packaged copy runs here, so $env:PROBE_EXE is not used.
#>
[CmdletBinding()]
param(
    [int] $E2eTimeoutSeconds = 300,
    # Time limit for each run of the packaged probe.
    [int] $ProbeTimeoutSeconds = 60
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
. "$PSScriptRoot/common.ps1"

$package = Get-ProbePackage
$installed = Get-InstalledProbePackage $package.Name
if (-not $installed) { throw "$($package.Name) is not installed; run build-msix.ps1 first." }
$installed | Format-List | Out-String | Write-Host

$results = [Collections.Generic.List[object]]::new()
function Add-Result([string] $Check, [bool] $Passed, [string] $Detail, [switch] $Observed) {
    $status = if ($Passed) { 'PASS' } elseif ($Observed) { 'INFO' } else { 'FAIL' }
    $results.Add([pscustomobject]@{ Status = $status; Check = $Check; Detail = $Detail })
    Write-Host "[$status] ${Check}: $Detail"
}

# Browser vendor keys under HKCU\Software; $true = required.
$VendorKeys = [ordered]@{
    'Google\Chrome'               = $true
    'Microsoft\Edge'              = $true
    'Mozilla'                     = $true
    'Chromium'                    = $false
    'BraveSoftware\Brave-Browser' = $false
    'Vivaldi'                     = $false
}
function Get-HostKey([string] $Vendor) {
    "HKCU\Software\$Vendor\NativeMessagingHosts\$($package.NativeHost)"
}

# Runs a program with a time limit and returns its exit code ($null when it
# was killed) and output lines.
function Invoke-Program([string] $Path, [string[]] $Arguments) {
    $run = Invoke-Bounded -FilePath $Path -Arguments $Arguments -TimeoutSeconds $ProbeTimeoutSeconds
    [pscustomobject]@{ ExitCode = $run.ExitCode; Lines = $run.Lines }
}

# The default value of a key, read by reg.exe: a process outside the package
# sees only the real registry.
function Get-RegistryDefault([string] $Key) {
    $lines = @(reg.exe query $Key /ve 2>$null)
    if ($LASTEXITCODE -ne 0) { return $null }
    # With /ve the only value line is the default one, whatever its localized name.
    foreach ($line in $lines) {
        if ($line -match '\sREG_(?:EXPAND_)?SZ\s+(.*)$') { return $Matches[1].Trim() }
    }
    $null
}

function Test-Alias {
    $apps = Join-Path $env:LOCALAPPDATA 'Microsoft\WindowsApps'
    $shared = Join-Path $apps $package.Alias
    $own = Join-Path (Join-Path $apps $installed.PackageFamilyName) $package.Alias
    foreach ($entry in @(@{ Path = $shared; Required = $true }, @{ Path = $own; Required = $false })) {
        # File.Exists and GetAttributes do not follow the alias's reparse point.
        $exists = [IO.File]::Exists($entry.Path)
        $detail = if ($exists) { "$($entry.Path) [$([IO.File]::GetAttributes($entry.Path))]" } else { "$($entry.Path) missing" }
        Add-Result 'alias exists' $exists $detail -Observed:(-not $entry.Required)
    }
    $run = Invoke-Program $shared @('--version')
    Add-Result 'alias runs' ($run.ExitCode -eq 0 -and ($run.Lines -join ' ') -match 'websign-probe') `
        "exit $($run.ExitCode): $($run.Lines -join ' ')"
    $shared
}

# Deletes the keys and the manifest files they point to (only files named
# after our host), so whatever exists afterwards was written by this run.
function Clear-Registration {
    foreach ($vendor in $VendorKeys.Keys) {
        $key = Get-HostKey $vendor
        $manifest = Get-RegistryDefault $key
        if ($manifest -and (Split-Path -Leaf $manifest).StartsWith($package.NativeHost)) {
            Remove-Item -LiteralPath $manifest -ErrorAction SilentlyContinue
        }
        reg.exe delete $key /f 2>$null | Out-Null
    }
    $global:LASTEXITCODE = 0
}

# Where a write to $Path lands when file system virtualization is on.
function Get-VirtualizedPath([string] $Path) {
    $local = $env:LOCALAPPDATA.TrimEnd('\')
    if (-not $Path.StartsWith("$local\", [StringComparison]::OrdinalIgnoreCase)) { return $null }
    Join-Path "$local\Packages\$($installed.PackageFamilyName)\LocalCache\Local" $Path.Substring($local.Length + 1)
}

function Test-Registration([string] $Alias, [datetime] $Since) {
    $run = Invoke-Program $Alias @('register')
    Add-Result 'register via alias' ($run.ExitCode -eq 0) "exit $($run.ExitCode)"

    $manifests = @{}
    foreach ($vendor in $VendorKeys.Keys) {
        $key = Get-HostKey $vendor
        $value = Get-RegistryDefault $key
        Add-Result '(a) hkcu' ($null -ne $value) "$key -> $value" -Observed:(-not $VendorKeys[$vendor])
        if ($value) { $manifests[$value] = $VendorKeys[$vendor] -or $manifests[$value] }
    }
    foreach ($path in $manifests.Keys) {
        $required = [bool] $manifests[$path]
        $file = Get-Item -LiteralPath $path -ErrorAction SilentlyContinue
        $real = $file -and $path -notmatch '\\Packages\\'
        $fresh = $file -and $file.LastWriteTime -ge $Since
        Add-Result '(b) file' ($real -and $fresh) `
            "$path (exists: $([bool] $file), written by this run: $([bool] $fresh))" -Observed:(-not $required)
        if ($file) {
            Test-Manifest $path $required
        } else {
            $virtual = Get-VirtualizedPath $path
            $found = $virtual -and (Test-Path -LiteralPath $virtual)
            Add-Result '(b) virtualized copy' $found "$virtual (a copy here means the write was virtualized)" -Observed
        }
    }
}

function Test-Manifest([string] $Path, [bool] $Required) {
    $manifest = Get-Content -LiteralPath $Path -Raw | ConvertFrom-Json
    $pathProperty = $manifest.PSObject.Properties['path']
    $hostPath = if ($pathProperty) { [string] $pathProperty.Value } else { '' }
    $isAlias = $hostPath -match '\\Microsoft\\WindowsApps\\' -and
        $hostPath.EndsWith("\$($package.Alias)", [StringComparison]::OrdinalIgnoreCase)
    Add-Result '(c) path is the alias' $isAlias $hostPath -Observed:(-not $Required)
    if ($hostPath -and (Test-Path -LiteralPath $hostPath)) {
        $run = Invoke-Program $hostPath @('--version')
        Add-Result '(c) path runs' ($run.ExitCode -eq 0) "exit $($run.ExitCode)" -Observed:(-not $Required)
    } else {
        Add-Result '(c) path runs' $false "$hostPath does not exist" -Observed:(-not $Required)
    }
    $allowed = @($manifest.PSObject.Properties |
            Where-Object Name -In 'allowed_origins', 'allowed_extensions' |
            ForEach-Object { $_.Value })
    $expected = @("chrome-extension://$($package.ExtensionDevId)/", $package.FirefoxId)
    $matching = @($allowed | Where-Object { $_ -in $expected })
    Add-Result 'allowed extensions' ($matching.Count -gt 0) ($allowed -join ', ') -Observed
}

# Optional: the end-to-end test drives Chromium with the test extension. The
# host is already registered (through the alias), hence --no-register.
function Test-EndToEnd([string] $Alias) {
    $runner = Join-Path $PSScriptRoot '..\nm-e2e\run.mjs'
    if (-not (Test-Path -LiteralPath $runner) -or -not (Get-Command node -ErrorAction SilentlyContinue)) {
        Add-Result '(d) native messaging e2e' $false 'skipped: nm-e2e/run.mjs or node not available' -Observed
        return
    }
    $arguments = @($runner, '--probe', $Alias, '--no-register') | ForEach-Object { "`"$_`"" }
    Write-Host "> node $($arguments -join ' ')"
    $process = Start-Process -FilePath 'node' -ArgumentList $arguments -NoNewWindow -PassThru
    # Caching the handle now is what makes ExitCode available after the wait.
    $null = $process.Handle
    if (-not $process.WaitForExit($E2eTimeoutSeconds * 1000)) {
        $process.Kill($true)
        Add-Result '(d) native messaging e2e' $false "timed out after $E2eTimeoutSeconds s" -Observed
        return
    }
    Add-Result '(d) native messaging e2e' ($process.ExitCode -eq 0) "exit $($process.ExitCode)" -Observed
}

$alias = Test-Alias
Clear-Registration
$since = (Get-Date).AddSeconds(-2)
Test-Registration $alias $since
Test-EndToEnd $alias

Write-Host "`nSummary"
$results | Format-Table -AutoSize -Wrap | Out-String -Width 220 | Write-Host
$failed = @($results | Where-Object Status -EQ 'FAIL')
if ($failed.Count -gt 0) {
    Write-Host "$($failed.Count) required check(s) failed."
    exit 1
}
Write-Host 'All required MSIX checks passed.'
exit 0
