# Stops every process the last run started, and only those. Each process is matched by pid and
# start time, so a pid that the OS has reused is left alone.

[CmdletBinding()]
param(
    [string]$RunDir = 'D:\dev\acan-run'
)

. (Join-Path $PSScriptRoot 'lib.ps1')

$state = Get-RunState -RunDir $RunDir
if ($null -eq $state) {
    Write-Output "no run state in $RunDir; nothing to stop"
    return
}

foreach ($entry in $state.processes) {
    $proc = Get-Process -Id $entry.pid -ErrorAction SilentlyContinue
    if ($null -ne $proc -and $proc.StartTime.Ticks -eq $entry.startTicks) {
        # /T takes the process tree, so the tests a worker spawned stop with it.
        & taskkill /T /F /PID $entry.pid *> $null
        Write-Output "stopped $($entry.name) (pid $($entry.pid))"
    } else {
        Write-Output "$($entry.name) was not running"
    }
}
