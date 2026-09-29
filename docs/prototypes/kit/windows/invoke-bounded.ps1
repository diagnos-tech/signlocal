# Shared by the Windows proof scripts (dot-source it): runs a program with a
# time limit. On a CI runner there is no one to click a dialog, so a native
# call that decides to show one waits forever; with a limit, the step fails
# with the name of the call instead of the job timing out in silence.

# Runs $FilePath with $Arguments and returns ExitCode ($null if killed),
# TimedOut, Stdout and Stderr (lines), Lines (both, in arrival order) and
# Seconds. Standard input is closed at once, so the program can never wait on
# it; stdout and stderr are read concurrently, so neither pipe can fill up and
# block the program; every line is echoed as it arrives, so the log shows how
# far it got even when it hangs. After $TimeoutSeconds the whole process tree
# is killed.
function Invoke-Bounded {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)] [string] $FilePath,
        [string[]] $Arguments = @(),
        [int] $TimeoutSeconds = 90,
        # What the log calls this run; default: the program and its arguments.
        [string] $Label,
        # Keep the output out of the log (it is still returned).
        [switch] $Quiet
    )
    if (-not $Label) { $Label = "$(Split-Path -Leaf $FilePath) $($Arguments -join ' ')".Trim() }
    Write-Host "`n== $Label (limit $TimeoutSeconds s)"

    $info = [Diagnostics.ProcessStartInfo]::new($FilePath)
    foreach ($argument in $Arguments) { $info.ArgumentList.Add($argument) }
    $info.UseShellExecute = $false
    $info.CreateNoWindow = $true
    $info.RedirectStandardInput = $true
    $info.RedirectStandardOutput = $true
    $info.RedirectStandardError = $true
    $info.StandardOutputEncoding = [Text.UTF8Encoding]::new($false)
    $info.StandardErrorEncoding = [Text.UTF8Encoding]::new($false)

    $process = [Diagnostics.Process]::new()
    $process.StartInfo = $info
    $clock = [Diagnostics.Stopwatch]::StartNew()
    $null = $process.Start()
    $process.StandardInput.Close()

    $all = [Collections.Generic.List[string]]::new()
    $streams = @(
        @{ Reader = $process.StandardOutput; Lines = [Collections.Generic.List[string]]::new(); Task = $null; Open = $true }
        @{ Reader = $process.StandardError; Lines = [Collections.Generic.List[string]]::new(); Task = $null; Open = $true }
    )
    $limit = [long] $TimeoutSeconds * 1000
    $timedOut = $false
    while ($true) {
        $pending = [Collections.Generic.List[Threading.Tasks.Task]]::new()
        foreach ($stream in $streams | Where-Object { $_.Open }) {
            while ($stream.Open) {
                if ($null -eq $stream.Task) { $stream.Task = $stream.Reader.ReadLineAsync() }
                if (-not $stream.Task.IsCompleted) { $pending.Add($stream.Task); break }
                $line = $stream.Task.GetAwaiter().GetResult()
                $stream.Task = $null
                if ($null -eq $line) { $stream.Open = $false; break }
                $stream.Lines.Add($line)
                $all.Add($line)
                if (-not $Quiet) { Write-Host "  $line" }
            }
        }
        if ($pending.Count -eq 0) { break }
        $left = $limit - $clock.ElapsedMilliseconds
        if ($left -le 0) { $timedOut = $true; break }
        $null = [Threading.Tasks.Task]::WaitAny($pending.ToArray(), [int] [Math]::Min(250, $left))
    }
    # Both pipes are closed, but the program may still be running.
    if (-not $timedOut) {
        $left = [Math]::Max(0, $limit - $clock.ElapsedMilliseconds)
        $timedOut = -not $process.WaitForExit([int] $left)
    }
    if ($timedOut) {
        try { $process.Kill($true) } catch { Write-Host "  (kill failed: $_)" }
        $null = $process.WaitForExit(10000)
    } else {
        # Flushes the asynchronous reads before ExitCode is read.
        $process.WaitForExit()
    }
    $seconds = [Math]::Round($clock.Elapsed.TotalSeconds, 1)
    $exitCode = if ($timedOut) { $null } else { $process.ExitCode }
    $process.Dispose()
    if ($timedOut) {
        Write-Host "  !! TIMEOUT in ${Label}: no exit after $TimeoutSeconds s, process tree killed"
    } else {
        Write-Host "  (exit code $exitCode in $seconds s)"
    }
    [pscustomobject]@{
        ExitCode = $exitCode
        TimedOut = $timedOut
        Stdout   = [string[]] $streams[0].Lines.ToArray()
        Stderr   = [string[]] $streams[1].Lines.ToArray()
        Lines    = [string[]] $all.ToArray()
        Seconds  = $seconds
    }
}

# Runs $Script in a fresh, non-interactive PowerShell 7 with a time limit, for
# cmdlets that may block inside a native call (New-SelfSignedCertificate with
# a legacy CSP, say): a hung child process can be killed, a hung cmdlet in
# this process cannot. Errors stop the script and make the exit code 1.
function Invoke-BoundedPowerShell {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)] [string] $Script,
        [Parameter(Mandatory)] [string] $Label,
        [int] $TimeoutSeconds = 120,
        [switch] $Quiet
    )
    $prelude = "`$ErrorActionPreference = 'Stop'; `$ProgressPreference = 'SilentlyContinue'; " +
        "`$PSStyle.OutputRendering = 'PlainText'; "
    $encoded = [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes($prelude + $Script))
    $pwsh = [Environment]::ProcessPath
    if (-not $pwsh -or (Split-Path -Leaf $pwsh) -notmatch '^pwsh') {
        $pwsh = Join-Path $PSHOME ($IsWindows ? 'pwsh.exe' : 'pwsh')
    }
    Invoke-Bounded -FilePath $pwsh -Label $Label -TimeoutSeconds $TimeoutSeconds -Quiet:$Quiet -Arguments @(
        '-NoLogo', '-NoProfile', '-NonInteractive', '-OutputFormat', 'Text', '-EncodedCommand', $encoded)
}
