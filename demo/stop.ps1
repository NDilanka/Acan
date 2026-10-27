# Stops every process the take started, and only those. Each process is matched by pid and start
# time, so a pid that the OS has reused is left alone. With no -RunDir it stops the latest take.

[CmdletBinding()]
param(
    [string]$RunDir = ''
)

. (Join-Path $PSScriptRoot 'lib.ps1')

if (-not $RunDir) { $RunDir = Get-CurrentRunDir }
if (-not $RunDir) {
    Write-Output 'no run folder given and no current run; nothing to stop'
    return
}
$state = Get-RunState -RunDir $RunDir
if ($null -eq $state) {
    Write-Output "no run state in $RunDir; nothing to stop"
    return
}

foreach ($entry in $state.processes) {
    if (Test-RecordedProcess -ProcessId $entry.pid -StartTicks $entry.startTicks) {
        # /T takes the process tree, so the tests a worker spawned stop with it.
        & taskkill /T /F /PID $entry.pid *> $null
        Write-Output "stopped $($entry.name) (pid $($entry.pid))"
    } else {
        Write-Output "$($entry.name) was not running"
    }
}
