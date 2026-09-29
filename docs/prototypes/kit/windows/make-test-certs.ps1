#Requires -Version 7.2
<#
.SYNOPSIS
    Creates (or removes) software test certificates that exercise every
    Windows signing path of websign-probe.

.DESCRIPTION
    All certificates go to Cert:\CurrentUser\My with a subject that starts
    with "CN=websign-probe test", so -Remove deletes exactly these (with their
    private keys) and nothing else.

      cng-rsa2048        RSA-2048 in the Microsoft Software KSP (CNG)
      cng-p256/-p384     ECDSA in the Microsoft Software KSP (CNG)
      capi-aes-rsa2048   RSA-2048 in the Enhanced RSA and AES CSP (CAPI, PROV_RSA_AES)
      a1-pfx-rsa2048     an "A1": PFX imported into the Enhanced CSP v1.0
                         (CAPI, PROV_RSA_FULL, AT_KEYEXCHANGE), as the import
                         wizard does with certificates bought as a file
      capi-base-rsa2048  RSA-2048 in the Base CSP v1.0 (PROV_RSA_FULL), when
                         the cmdlet accepts that provider

    Writes one object per certificate: Name, Thumbprint, Sha256 (the probe's
    fingerprint), Api (CNG or CAPI), Provider, Key (RSA or EC) and AllowApi
    (the native call expected with --ncrypt allow).

.EXAMPLE
    $certs = ./make-test-certs.ps1
    ./make-test-certs.ps1 -Remove
#>
[CmdletBinding()]
param(
    # Delete the test certificates and their keys instead of creating them.
    [switch] $Remove
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$SubjectPrefix = 'CN=websign-probe test'
$Store = 'Cert:\CurrentUser\My'
$Ksp = 'Microsoft Software Key Storage Provider'
$AesCsp = 'Microsoft Enhanced RSA and AES Cryptographic Provider'
$EnhancedCsp = 'Microsoft Enhanced Cryptographic Provider v1.0'
$BaseCsp = 'Microsoft Base Cryptographic Provider v1.0'

function Remove-TestCertificate {
    Get-ChildItem -Path $Store |
        Where-Object { $_.Subject -like "$SubjectPrefix*" } |
        ForEach-Object {
            Write-Host "Removing $($_.Subject) ($($_.Thumbprint))"
            Remove-Item -LiteralPath $_.PSPath -DeleteKey
        }
}

function New-TestCertificate([string] $Name, [hashtable] $KeyParameters) {
    $parameters = @{
        Subject           = "$SubjectPrefix $Name"
        CertStoreLocation = $Store
        Type              = 'Custom'
        KeyUsage          = 'DigitalSignature', 'NonRepudiation'
        KeyExportPolicy   = 'NonExportable'
        HashAlgorithm     = 'SHA256'
        NotAfter          = (Get-Date).AddDays(7)
    }
    foreach ($key in $KeyParameters.Keys) { $parameters[$key] = $KeyParameters[$key] }
    New-SelfSignedCertificate @parameters
}

# The way certificate files usually end up in Windows: a PFX imported into
# the default legacy CSP, which is PROV_RSA_FULL and predates SHA-2.
function New-PfxImportedCertificate([string] $Name) {
    $source = New-TestCertificate $Name @{
        Provider = $Ksp; KeyAlgorithm = 'RSA'; KeyLength = 2048; KeyExportPolicy = 'Exportable'
    }
    $pfx = Join-Path ([IO.Path]::GetTempPath()) "websign-probe-$Name.pfx"
    $password = [Guid]::NewGuid().ToString('N')
    try {
        $secure = ConvertTo-SecureString -String $password -AsPlainText -Force
        Export-PfxCertificate -Cert $source -FilePath $pfx -Password $secure | Out-Null
        Remove-Item -LiteralPath "$Store\$($source.Thumbprint)" -DeleteKey
        $output = certutil -f -user -p $password -csp $EnhancedCsp -importpfx My $pfx AT_KEYEXCHANGE
        if ($LASTEXITCODE -ne 0) { throw "certutil -importpfx failed:`n$($output -join "`n")" }
    } finally {
        Remove-Item -LiteralPath $pfx -ErrorAction SilentlyContinue
    }
    Get-Item -LiteralPath "$Store\$($source.Thumbprint)"
}

function ConvertTo-Result($Certificate, [string] $Name, [string] $Api, [string] $Provider,
    [string] $Key, [string] $AllowApi) {
    [pscustomobject]@{
        Name       = $Name
        Thumbprint = $Certificate.Thumbprint
        Sha256     = [Convert]::ToHexString(
            [Security.Cryptography.SHA256]::HashData($Certificate.RawData)).ToLowerInvariant()
        Api        = $Api
        Provider   = $Provider
        Key        = $Key
        AllowApi   = $AllowApi
    }
}

if ($Remove) {
    Remove-TestCertificate
    return
}

$cng = @{ Provider = $Ksp }
$rsa = @{ KeyAlgorithm = 'RSA'; KeyLength = 2048 }

ConvertTo-Result (New-TestCertificate 'cng-rsa2048' ($cng + $rsa)) `
    'cng-rsa2048' 'CNG' $Ksp 'RSA' 'NCryptSignHash'
ConvertTo-Result (New-TestCertificate 'cng-p256' ($cng + @{ KeyAlgorithm = 'ECDSA_nistP256'; CurveExport = 'CurveName' })) `
    'cng-p256' 'CNG' $Ksp 'EC' 'NCryptSignHash'
ConvertTo-Result (New-TestCertificate 'cng-p384' ($cng + @{ KeyAlgorithm = 'ECDSA_nistP384'; CurveExport = 'CurveName' })) `
    'cng-p384' 'CNG' $Ksp 'EC' 'NCryptSignHash'
ConvertTo-Result (New-TestCertificate 'capi-aes-rsa2048' ($rsa + @{ Provider = $AesCsp; KeySpec = 'Signature' })) `
    'capi-aes-rsa2048' 'CAPI' $AesCsp 'RSA' 'CryptSignHash'
ConvertTo-Result (New-PfxImportedCertificate 'a1-pfx-rsa2048') `
    'a1-pfx-rsa2048' 'CAPI' $EnhancedCsp 'RSA' 'CryptSignHash (PROV_RSA_AES)'

# The Base CSP cannot hash with SHA-2, so its own certificate is self-signed
# with SHA-1; only the key matters here.
try {
    $base = New-TestCertificate 'capi-base-rsa2048' ($rsa + @{
            Provider = $BaseCsp; KeySpec = 'Signature'; HashAlgorithm = 'SHA1'
        })
    ConvertTo-Result $base 'capi-base-rsa2048' 'CAPI' $BaseCsp 'RSA' 'CryptSignHash (PROV_RSA_AES)'
} catch {
    Write-Warning "Skipping capi-base-rsa2048: New-SelfSignedCertificate refused the Base CSP: $_"
}
