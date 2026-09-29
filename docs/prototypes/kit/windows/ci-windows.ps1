#Requires -Version 7.2
<#
.SYNOPSIS
    Proof of the Windows key store with software keys: websign-probe lists
    and signs with CNG and legacy CAPI keys under every --ncrypt mode, and
    every outcome is checked against what is expected.

.DESCRIPTION
    Needs $env:PROBE_EXE (the websign-probe binary). When $env:REPORT_PATH is
    set, also writes the probe's Markdown report there. Test certificates are
    created by make-test-certs.ps1 and always removed at the end.

    Required (the script fails otherwise):
      - `list` shows every test certificate as software, with its provider;
      - CNG keys sign every hash with every algorithm, via NCryptSignHash, in
        every mode;
      - CAPI keys sign RSASSA-PKCS1-v1_5 with every hash under --ncrypt allow,
        through the expected call (PROV_RSA_FULL keys via the AES CSP);
      - RSASSA-PSS on CAPI keys under --ncrypt allow fails as "not supported":
        CAPI has no PSS. This is the one failure that is expected.
    Observed only (reported, never fatal): CAPI keys under --ncrypt prefer and
    --ncrypt only, where Windows may open Microsoft CSP keys through the
    software KSP. The outcome is evidence for docs/prototypes/1-windows.md.

    PKCS#11 discovery is switched off for these runs: they prove the Windows
    store, and the runner has no PKCS#11 module anyway.
#>
[CmdletBinding()]
param(
    [string] $Probe = $env:PROBE_EXE,
    [string] $ReportPath = $env:REPORT_PATH
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

if (-not $Probe -or -not (Test-Path -LiteralPath $Probe)) {
    throw 'Set PROBE_EXE to the websign-probe binary.'
}
$Isolate = '--no-known-modules', '--no-p11-kit'
$Modes = 'allow', 'prefer', 'only'
$failures = [Collections.Generic.List[string]]::new()
$rows = [Collections.Generic.List[object]]::new()

function Invoke-Probe([string[]] $Arguments) {
    Write-Host "`n> websign-probe $($Arguments -join ' ')"
    $lines = @(& $Probe @Arguments 2>&1 | ForEach-Object { "$_" })
    $exitCode = $LASTEXITCODE
    $lines | ForEach-Object { Write-Host "  $_" }
    Write-Host "  (exit code $exitCode)"
    [pscustomobject]@{ ExitCode = $exitCode; Lines = $lines }
}

function Assert-That([bool] $Condition, [string] $Message) {
    if (-not $Condition) {
        $failures.Add($Message)
        Write-Host "  !! $Message"
    }
}

# What a signing case must do: 'ok', 'unsupported', or 'observe' (no verdict).
function Get-Expectation($Cert, [string] $Mode, [string] $Algorithm) {
    if ($Cert.Api -eq 'CNG') { return 'ok' }
    if ($Mode -ne 'allow') { return 'observe' }
    if ($Algorithm -eq 'RSASSA-PSS') { return 'unsupported' }
    'ok'
}

function Test-List($Certs) {
    $run = Invoke-Probe (@('list') + $Isolate)
    Assert-That ($run.ExitCode -eq 0) "list exited with $($run.ExitCode)"
    foreach ($cert in $Certs) {
        $line = $run.Lines | Where-Object { $_.Contains($cert.Sha256.Substring(0, 16)) } |
            Select-Object -First 1
        Assert-That ($null -ne $line) "list does not show $($cert.Name)"
        if ($line) {
            Assert-That $line.Contains($cert.Provider) "list shows the wrong provider for $($cert.Name): $line"
            Assert-That $line.Contains(', software;') "list does not mark $($cert.Name) as software: $line"
        }
    }
}

function Test-Sign($Cert, [string] $Mode) {
    $arguments = @('sign', '--cert', $Cert.Sha256.Substring(0, 16), '--hash', 'all', '--pss',
        '--ncrypt', $Mode) + $Isolate
    $run = Invoke-Probe $arguments
    $cases = @($run.Lines | ForEach-Object {
            if ($_ -match '^\s+(OK|FAIL)\s+(SHA-\d+)\s+([A-Za-z0-9_.-]+):?\s*(.*)$') {
                [pscustomobject]@{ Outcome = $Matches[1]; Hash = $Matches[2]; Algorithm = $Matches[3]; Rest = $Matches[4] }
            }
        })
    $expectedCount = if ($Cert.Key -eq 'RSA') { 6 } else { 3 }
    Assert-That ($cases.Count -eq $expectedCount) `
        "$($Cert.Name) --ncrypt ${Mode}: $($cases.Count) signing results, expected $expectedCount"
    foreach ($case in $cases) {
        $expected = Get-Expectation $Cert $Mode $case.Algorithm
        $api = if ($case.Rest -match '^via (.+) in \d+ ms$') { $Matches[1] } else { '' }
        $label = "$($Cert.Name) --ncrypt $Mode $($case.Hash) $($case.Algorithm)"
        switch ($expected) {
            'ok' {
                Assert-That ($case.Outcome -eq 'OK') "${label}: expected OK, got $($case.Rest)"
                $wanted = if ($Mode -eq 'allow' -or $Cert.Api -eq 'CNG') { $Cert.AllowApi } else { $null }
                if ($case.Outcome -eq 'OK' -and $wanted) {
                    Assert-That ($api -eq $wanted) "${label}: signed via '$api', expected '$wanted'"
                }
            }
            'unsupported' {
                if ($case.Outcome -eq 'OK') {
                    Write-Host "  note: $label unexpectedly succeeded via $api"
                } else {
                    Assert-That $case.Rest.Contains('not supported') "${label}: expected 'not supported', got $($case.Rest)"
                }
            }
        }
        $rows.Add([pscustomobject]@{
                Certificate = $Cert.Name; Mode = $Mode; Hash = $case.Hash; Algorithm = $case.Algorithm
                Outcome = $case.Outcome; Expected = $expected
                Detail = if ($api) { $api } else { $case.Rest }
            })
    }
}

function Write-Environment {
    $os = Get-CimInstance Win32_OperatingSystem
    Write-Host "OS: $($os.Caption) $($os.Version) (build $($os.BuildNumber))"
    Write-Host "PowerShell: $($PSVersionTable.PSVersion)"
    Write-Host "Probe: $(& $Probe --version)"
}

function Write-Report {
    if (-not $ReportPath) { return }
    # Warning only: the report depends on sections owned by other commands,
    # and it exits non-zero whenever a case fails, including PSS on CAPI.
    try {
        $run = Invoke-Probe @('report', '--run-signatures', '--all', '--hash', 'all', '--pss', '--out', $ReportPath)
        if (-not (Test-Path -LiteralPath $ReportPath)) {
            Write-Warning "report did not write $ReportPath (exit code $($run.ExitCode))"
        }
    } catch {
        Write-Warning "report failed: $_"
    }
}

Write-Environment
& "$PSScriptRoot/make-test-certs.ps1" -Remove
try {
    $certs = @(& "$PSScriptRoot/make-test-certs.ps1")
    $certs | Format-Table Name, Api, Provider, Thumbprint | Out-String | Write-Host
    foreach ($cert in $certs | Where-Object Api -EQ 'CAPI') {
        Write-Host "certutil view of $($cert.Name):"
        certutil -user -store My $cert.Thumbprint | Select-String 'Provider|KeySpec' | ForEach-Object { Write-Host "  $_" }
    }
    Test-List $certs
    foreach ($mode in $Modes) {
        foreach ($cert in $certs) { Test-Sign $cert $mode }
    }
    Write-Host "`nSummary"
    $rows | Format-Table -AutoSize | Out-String -Width 200 | Write-Host
    Write-Report
} finally {
    & "$PSScriptRoot/make-test-certs.ps1" -Remove
}

if ($failures.Count -gt 0) {
    Write-Host "`n$($failures.Count) required check(s) failed:"
    $failures | ForEach-Object { Write-Host "  - $_" }
    exit 1
}
Write-Host "`nAll required checks passed."
exit 0
