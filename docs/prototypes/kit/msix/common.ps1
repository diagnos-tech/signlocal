# Shared by build-msix.ps1 and test-msix.ps1 (dot-source it): the probe
# package's names, all derived from project.toml, and small Windows helpers.

Set-StrictMode -Version Latest

# Flat "table.key" -> value map of the string entries in project.toml. Enough
# for that file; not a general TOML parser.
function Read-ProjectToml {
    $path = Join-Path $PSScriptRoot '..\..\..\..\project.toml'
    $values = @{}
    $table = ''
    foreach ($line in Get-Content -LiteralPath $path) {
        if ($line -match '^\s*\[([^\]]+)\]\s*$') {
            $table = $Matches[1]
        } elseif ($line -match '^\s*([A-Za-z0-9_]+)\s*=\s*"([^"]*)"') {
            $values["$table.$($Matches[1])"] = $Matches[2]
        }
    }
    $values
}

# Names of the probe package. It is a separate identity from the future app
# ("<package name>.Probe") so both can be installed side by side, and it uses
# a test publisher: the Store assigns the real one.
function Get-ProbePackage {
    $toml = Read-ProjectToml
    $slug = $toml['product.slug']
    $product = $toml['product.name']
    [pscustomobject]@{
        Name                 = "$($toml['ids.windows_package_name']).Probe"
        DisplayName          = "$product Probe"
        Publisher            = "CN=$product Probe Test"
        PublisherDisplayName = "$product Probe Test"
        Executable           = "$slug-probe.exe"
        # Must match platform::windows::alias_file_name in the probe.
        Alias                = "$slug-probe.exe"
        UrlScheme            = $toml['ids.url_scheme']
        NativeHost           = $toml['ids.native_host']
        FirefoxId            = $toml['extension.firefox_id']
        ExtensionDevId       = $toml['extension.dev_id']
    }
}

# Newest x64 copy of a Windows SDK tool (makeappx.exe, signtool.exe).
function Find-SdkTool([string] $Name) {
    $root = Join-Path ${env:ProgramFiles(x86)} 'Windows Kits\10\bin'
    $tool = Get-ChildItem -LiteralPath $root -Directory -Filter '10.*' -ErrorAction SilentlyContinue |
        Sort-Object { [version] $_.Name } -Descending |
        ForEach-Object { Join-Path $_.FullName "x64\$Name" } |
        Where-Object { Test-Path -LiteralPath $_ } |
        Select-Object -First 1
    if (-not $tool) { throw "$Name not found under $root; install the Windows 10/11 SDK." }
    $tool
}

# Runs a script in Windows PowerShell 5.1 and returns its output. The Appx
# cmdlets do not load in every PowerShell 7 build; Windows PowerShell always
# has them.
function Invoke-WindowsPowerShell([string] $Script) {
    $prelude = "`$ErrorActionPreference = 'Stop'; `$ProgressPreference = 'SilentlyContinue'; "
    $encoded = [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes($prelude + $Script))
    $output = & powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -EncodedCommand $encoded 2>&1
    if ($LASTEXITCODE -ne 0) {
        throw "Windows PowerShell failed ($LASTEXITCODE): $Script`n$($output -join "`n")"
    }
    $output
}

# The installed probe package (Name, PackageFamilyName, PackageFullName,
# InstallLocation, Version), or $null.
function Get-InstalledProbePackage([string] $Name) {
    $json = Invoke-WindowsPowerShell @"
Get-AppxPackage -Name '$Name' |
    Select-Object Name, PackageFamilyName, PackageFullName, InstallLocation, @{ n = 'Version'; e = { `$_.Version.ToString() } } |
    ConvertTo-Json -Compress
"@
    $text = ($json -join '').Trim()
    if (-not $text) { return $null }
    $text | ConvertFrom-Json
}

function Test-Administrator {
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    ([Security.Principal.WindowsPrincipal] $identity).IsInRole(
        [Security.Principal.WindowsBuiltInRole]::Administrator)
}
