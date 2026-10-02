#Requires -Version 7.2
<#
.SYNOPSIS
    Packages websign-probe as a signed MSIX and installs it, the way the
    Microsoft Store would deliver the app.

.DESCRIPTION
    1. Lays out the package: the probe binary, AppxManifest.xml filled from
       project.toml, and minimal generated PNG logos.
    2. makeappx pack (validates the manifest against the SDK schemas).
    3. Signs with a throwaway self-signed certificate whose subject equals the
       manifest Publisher, and trusts it in LocalMachine\TrustedPeople.
    4. Replaces any installed copy and runs Add-AppxPackage.

    Needs an elevated prompt (trusting the certificate) and the Windows SDK.
    The signing certificate never stays in CurrentUser\My, so it does not show
    up in `websign-probe list`. Remove the package afterwards with:
        Get-AppxPackage WebeSign.Probe | Remove-AppxPackage

.PARAMETER ProbeExe
    The websign-probe.exe to package. Default: $env:PROBE_EXE.
#>
[CmdletBinding()]
param(
    [string] $ProbeExe = $env:PROBE_EXE,
    [string] $OutDir = (Join-Path ([IO.Path]::GetTempPath()) 'websign-msix'),
    [string] $Version = '0.1.0.0'
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
. "$PSScriptRoot/common.ps1"

if (-not $ProbeExe -or -not (Test-Path -LiteralPath $ProbeExe)) {
    throw 'Set PROBE_EXE (or -ProbeExe) to the websign-probe binary.'
}
if (-not (Test-Administrator)) {
    throw 'Run elevated: the signing certificate must be trusted in LocalMachine\TrustedPeople.'
}

$package = Get-ProbePackage
$layout = Join-Path $OutDir 'layout'
$msix = Join-Path $OutDir "$($package.Name).msix"

function New-Logo([string] $Path, [int] $Size) {
    Add-Type -AssemblyName System.Drawing
    $bitmap = [Drawing.Bitmap]::new($Size, $Size)
    $graphics = [Drawing.Graphics]::FromImage($bitmap)
    try {
        $graphics.Clear([Drawing.Color]::FromArgb(255, 32, 87, 166))
        $inset = [int] [Math]::Round($Size * 0.3)
        $graphics.FillRectangle([Drawing.Brushes]::White, $inset, $inset, $Size - 2 * $inset, $Size - 2 * $inset)
        $bitmap.Save($Path, [Drawing.Imaging.ImageFormat]::Png)
    } finally {
        $graphics.Dispose()
        $bitmap.Dispose()
    }
}

function New-Layout {
    Remove-Item -LiteralPath $OutDir -Recurse -Force -ErrorAction SilentlyContinue
    New-Item -ItemType Directory -Path (Join-Path $layout 'Assets') -Force | Out-Null
    Copy-Item -LiteralPath $ProbeExe -Destination (Join-Path $layout $package.Executable)
    New-Logo (Join-Path $layout 'Assets\StoreLogo.png') 50
    New-Logo (Join-Path $layout 'Assets\Square150x150Logo.png') 150
    New-Logo (Join-Path $layout 'Assets\Square44x44Logo.png') 44

    $tokens = @{
        PACKAGE_NAME           = $package.Name
        PUBLISHER              = $package.Publisher
        VERSION                = $Version
        DISPLAY_NAME           = $package.DisplayName
        PUBLISHER_DISPLAY_NAME = $package.PublisherDisplayName
        EXECUTABLE             = $package.Executable
        ALIAS                  = $package.Alias
        URL_SCHEME             = $package.UrlScheme
    }
    $manifest = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'AppxManifest.xml') -Raw
    foreach ($token in $tokens.Keys) {
        $manifest = $manifest.Replace("@@$token@@", [Security.SecurityElement]::Escape($tokens[$token]))
    }
    if ($manifest -match '@@[A-Z_]+@@') { throw "Unfilled token in AppxManifest.xml: $($Matches[0])" }
    Set-Content -LiteralPath (Join-Path $layout 'AppxManifest.xml') -Value $manifest -Encoding utf8NoBOM
}

function Invoke-Tool([string] $Tool, [string[]] $Arguments, [int] $TimeoutSeconds = 300) {
    $shown = for ($i = 0; $i -lt $Arguments.Count; $i++) {
        if ($i -gt 0 -and $Arguments[$i - 1] -eq '/p' -and $Arguments[0] -eq 'sign') { '***' } else { $Arguments[$i] }
    }
    $name = Split-Path -Leaf $Tool
    $run = Invoke-Bounded -FilePath $Tool -Arguments $Arguments -TimeoutSeconds $TimeoutSeconds `
        -Label "$name $($shown -join ' ')"
    if ($run.TimedOut) { throw "TIMEOUT in $name after $TimeoutSeconds s" }
    if ($run.ExitCode -ne 0) { throw "$name failed with exit code $($run.ExitCode)" }
}

# Signs the package with a throwaway certificate and trusts that certificate.
function Protect-Package {
    $certificate = New-SelfSignedCertificate -Type Custom -Subject $package.Publisher `
        -FriendlyName 'websign-probe MSIX test signing' -CertStoreLocation 'Cert:\CurrentUser\My' `
        -KeyUsage DigitalSignature -KeyAlgorithm RSA -KeyLength 2048 -HashAlgorithm SHA256 `
        -KeyExportPolicy Exportable -NotAfter (Get-Date).AddDays(30) `
        -TextExtension @('2.5.29.37={text}1.3.6.1.5.5.7.3.3', '2.5.29.19={text}')
    $pfx = Join-Path $OutDir 'signing.pfx'
    $cer = Join-Path $OutDir 'signing.cer'
    $password = [Guid]::NewGuid().ToString('N')
    try {
        $secure = ConvertTo-SecureString -String $password -AsPlainText -Force
        Export-PfxCertificate -Cert $certificate -FilePath $pfx -Password $secure | Out-Null
        Export-Certificate -Cert $certificate -FilePath $cer | Out-Null
    } finally {
        Remove-Item -LiteralPath "Cert:\CurrentUser\My\$($certificate.Thumbprint)" -DeleteKey
    }
    try {
        Invoke-Tool (Find-SdkTool 'signtool.exe') @('sign', '/fd', 'SHA256', '/f', $pfx, '/p', $password, $msix)
    } finally {
        Remove-Item -LiteralPath $pfx -ErrorAction SilentlyContinue
    }
    Import-Certificate -FilePath $cer -CertStoreLocation 'Cert:\LocalMachine\TrustedPeople' | Out-Null
    Write-Host "Trusted $($package.Publisher) ($($certificate.Thumbprint)) in LocalMachine\TrustedPeople"
}

New-Layout
Invoke-Tool (Find-SdkTool 'makeappx.exe') @('pack', '/o', '/d', $layout, '/p', $msix)
Protect-Package

if (Get-InstalledProbePackage $package.Name) {
    Write-Host "Removing the installed $($package.Name)"
    Invoke-WindowsPowerShell "Get-AppxPackage -Name '$($package.Name)' | Remove-AppxPackage" | Out-Null
}
Write-Host "Installing $msix"
Invoke-WindowsPowerShell "Add-AppxPackage -Path '$($msix.Replace("'", "''"))' -ForceApplicationShutdown" | Out-Null

$installed = Get-InstalledProbePackage $package.Name
if (-not $installed) { throw "$($package.Name) is not installed after Add-AppxPackage." }
$installed | Format-List | Out-String | Write-Host
exit 0
