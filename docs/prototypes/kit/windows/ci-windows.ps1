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
    store, and the runner has no PKCS#11 module anyway. They also pass
    --silent: software keys never need UI, so a provider that wants a dialog
    fails at once (NTE_SILENT_CONTEXT) instead of waiting for a click.

    Nothing can hang the job: every probe run has its own time limit
    (-ProbeTimeoutSeconds) after which it is killed and reported as TIMEOUT,
    with WEBSIGN_PROBE_TRACE=1 its last trace line on stderr names the native
    call it was stuck in, and after $MaxTimeouts time-outs the remaining runs
    are skipped so the summary still gets printed.
#>
[CmdletBinding()]
param(
    [string] $Probe = $env:PROBE_EXE,
    [string] $ReportPath = $env:REPORT_PATH,
    [int] $ProbeTimeoutSeconds = 90
)

$ErrorActionPreference = 'Stop'
# A failing probe run is data here (its exit code is checked), never a reason
# to stop the script, whatever the PowerShell version's default is.
$PSNativeCommandUseErrorActionPreference = $false
Set-StrictMode -Version Latest
. "$PSScriptRoot/invoke-bounded.ps1"

if (-not $Probe -or -not (Test-Path -LiteralPath $Probe)) {
    throw 'Set PROBE_EXE to the websign-probe binary.'
}
# Each step the probe takes, on stderr (no personal data): a hang names its call.
if (-not $env:WEBSIGN_PROBE_TRACE) { $env:WEBSIGN_PROBE_TRACE = '1' }
$Isolate = '--no-known-modules', '--no-p11-kit', '--silent'
$MaxTimeouts = 3
$script:timeouts = 0
$Modes = 'allow', 'prefer', 'only'
$failures = [Collections.Generic.List[string]]::new()
$rows = [Collections.Generic.List[object]]::new()

# Runs the probe with a time limit. Lines holds stdout only: trace and
# warnings go to stderr, which is echoed but never parsed.
function Invoke-Probe([string[]] $Arguments) {
    $run = Invoke-Bounded -FilePath $Probe -Arguments $Arguments -TimeoutSeconds $ProbeTimeoutSeconds `
        -Label "websign-probe $($Arguments -join ' ')"
    if ($run.TimedOut) {
        $script:timeouts++
        $failures.Add("TIMEOUT in websign-probe $($Arguments -join ' ') (killed after $ProbeTimeoutSeconds s)")
    }
    [pscustomobject]@{ ExitCode = $run.ExitCode; TimedOut = $run.TimedOut; Lines = $run.Stdout }
}

function Test-TooManyTimeouts {
    $script:timeouts -ge $MaxTimeouts
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
    if ($run.TimedOut) { return }
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
    if (Test-TooManyTimeouts) {
        Write-Host "`n== skipped: sign $($Cert.Name) --ncrypt $Mode ($MaxTimeouts time-outs already)"
        return
    }
    $arguments = @('sign', '--cert', $Cert.Sha256.Substring(0, 16), '--hash', 'all', '--pss',
        '--ncrypt', $Mode) + $Isolate
    $run = Invoke-Probe $arguments
    if ($run.TimedOut) { return }
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
    Write-Host "Trace: WEBSIGN_PROBE_TRACE=$env:WEBSIGN_PROBE_TRACE"
    $version = Invoke-Bounded -FilePath $Probe -Arguments '--version' -TimeoutSeconds 30 -Quiet
    Write-Host "Probe: $($version.Stdout -join ' ')"
}

function Write-Report {
    if (-not $ReportPath) { return }
    if (Test-TooManyTimeouts) {
        Write-Host "`n== skipped: report ($MaxTimeouts time-outs already)"
        return
    }
    # Warning only: the report depends on sections owned by other commands,
    # and it exits non-zero whenever a case fails, including PSS on CAPI.
    try {
        # PKCS#11 discovery stays on here: what it finds is part of the evidence.
        $run = Invoke-Bounded -FilePath $Probe -TimeoutSeconds ($ProbeTimeoutSeconds * 2) -Arguments @(
            'report', '--run-signatures', '--all', '--hash', 'all', '--pss', '--silent', '--out', $ReportPath)
        if ($run.TimedOut) {
            Write-Warning 'report timed out; see its last trace line above'
        } elseif (-not (Test-Path -LiteralPath $ReportPath)) {
            Write-Warning "report did not write $ReportPath (exit code $($run.ExitCode))"
        }
    } catch {
        Write-Warning "report failed: $_"
    }
}

Write-Environment
Write-Host "`n== removing test certificates left by an earlier run"
& "$PSScriptRoot/make-test-certs.ps1" -Remove
try {
    Write-Host "`n== creating test certificates"
    $certs = @(& "$PSScriptRoot/make-test-certs.ps1")
    $certs | Format-Table Name, Api, Provider, Thumbprint | Out-String | Write-Host
    $certutil = Join-Path ([Environment]::SystemDirectory) 'certutil.exe'
    foreach ($cert in $certs | Where-Object Api -EQ 'CAPI') {
        $view = Invoke-Bounded -FilePath $certutil -Arguments '-user', '-store', 'My', $cert.Thumbprint `
            -TimeoutSeconds 60 -Quiet -Label "certutil view of $($cert.Name)"
        $view.Lines | Select-String 'Provider|KeySpec' | ForEach-Object { Write-Host "  $_" }
    }
    Test-List $certs
    foreach ($mode in $Modes) {
        foreach ($cert in $certs) { Test-Sign $cert $mode }
    }
    Write-Host "`nSummary"
    $rows | Format-Table -AutoSize | Out-String -Width 200 | Write-Host
    Write-Report
} finally {
    Write-Host "`n== removing test certificates"
    & "$PSScriptRoot/make-test-certs.ps1" -Remove
}

if ($failures.Count -gt 0) {
    Write-Host "`n$($failures.Count) required check(s) failed:"
    $failures | ForEach-Object { Write-Host "  - $_" }
    exit 1
}
Write-Host "`nAll required checks passed."
exit 0
