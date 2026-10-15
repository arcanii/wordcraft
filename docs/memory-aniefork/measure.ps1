# Run a program and report its memory: peak and final private bytes (what Task Manager calls
# "Memory") and working set, sampled every 50 ms, plus a coarse timeline.
param([Parameter(Mandatory)][string]$Label, [Parameter(Mandatory)][string]$Exe, [string]$ArgLine = "", [hashtable]$Env = @{})
$psi = New-Object System.Diagnostics.ProcessStartInfo $Exe
$psi.Arguments = $ArgLine
$psi.UseShellExecute = $false
$psi.RedirectStandardOutput = $true
$psi.RedirectStandardError = $true
foreach ($k in $Env.Keys) { $psi.EnvironmentVariables[$k] = $Env[$k] }
$sw = [System.Diagnostics.Stopwatch]::StartNew()
$p = [System.Diagnostics.Process]::Start($psi)
$out = $p.StandardOutput.ReadToEndAsync(); $err = $p.StandardError.ReadToEndAsync()
$maxPriv = 0; $maxWs = 0; $lastPriv = 0; $lastWs = 0; $timeline = @(); $nextMark = 0
while (-not $p.HasExited) {
    try {
        $p.Refresh()
        $priv = $p.PrivateMemorySize64; $ws = $p.WorkingSet64
        if ($priv -gt 0) { $lastPriv = $priv; $lastWs = $ws }
        if ($priv -gt $maxPriv) { $maxPriv = $priv }
        if ($ws -gt $maxWs) { $maxWs = $ws }
        if ($sw.ElapsedMilliseconds -ge $nextMark) { $timeline += ("{0:N1}s:{1}" -f ($sw.ElapsedMilliseconds / 1000), [math]::Round($priv / 1MB)); $nextMark += 1000 }
    } catch {}
    Start-Sleep -Milliseconds 50
}
$p.WaitForExit()
$mb = { param($b) [math]::Round($b / 1MB) }
"{0,-34} exit {1}  peak private {2} MB  peak WS {3} MB  last private {4} MB  last WS {5} MB  ({6:N1} s)" -f $Label, $p.ExitCode, (& $mb $maxPriv), (& $mb $maxWs), (& $mb $lastPriv), (& $mb $lastWs), ($sw.ElapsedMilliseconds / 1000)
"  timeline (s:MB private): " + ($timeline -join " ")
$e = $err.Result.Trim(); if ($p.ExitCode -ne 0 -and $e) { "  stderr: " + ($e.Substring(0, [math]::Min(300, $e.Length))) }
