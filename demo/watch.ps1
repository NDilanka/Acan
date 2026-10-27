# Follows one take through the owner's event log and writes a timeline and a summary under the
# run's logs folder. With -Settle 1, 2 or 3 it settles each clash the way a click on that menu
# row would, so a take runs without the UI. With -Settle 0 a human settles in the UI.
#
# "Clash before write" means the clash was recorded before the origin cleared any task to build.
# The origin clears a task only after its declarations are free of blocking clashes, and a worker
# writes only after that clearance, so no file is written before the clash.

[CmdletBinding()]
param(
    [string]$RunDir = 'D:\dev\acan-run',
    [int]$Port = 7311,
    [ValidateRange(0, 3)][int]$Settle = 0,
    [int]$TimeoutSecs = 1500,
    [int]$PollMs = 500
)

. (Join-Path $PSScriptRoot 'lib.ps1')

$state = Get-RunState -RunDir $RunDir
if ($null -eq $state) { throw "no run state in $RunDir; run run.ps1 first" }
$owner = Get-OwnerToken -RunDir $RunDir
$logs = Join-Path $RunDir 'logs'
$timelinePath = Join-Path $logs 'timeline.txt'
$rawPath = Join-Path $logs 'events.jsonl'
Set-Content -LiteralPath $timelinePath -Value '' -Encoding utf8
Set-Content -LiteralPath $rawPath -Value '' -Encoding utf8

$script:originMs = $null
$script:tasks = @()
$script:landed = @{}
$script:clashes = New-Object System.Collections.ArrayList
$script:firstClearSeq = $null
$script:modelSteps = @{}
$script:firstAt = @{}
$script:readbackAts = New-Object System.Collections.ArrayList
$script:declaredAts = New-Object System.Collections.ArrayList
$script:claimAts = New-Object System.Collections.ArrayList
$script:settled = @{}
$script:declaredBy = @{}

function Get-Elapsed([uint64]$AtMs) {
    if ($null -eq $script:originMs) { return 0.0 }
    [math]::Round(([double]$AtMs - [double]$script:originMs) / 1000.0, 1)
}

function Write-Line([string]$Text) {
    Add-Content -LiteralPath $timelinePath -Value $Text -Encoding utf8
    Write-Output $Text
}

function Get-ClashChoice($clash, [int]$Option) {
    $a = $clash.key.a
    $b = $clash.key.b
    if ($clash.kind -eq 'contract' -and $null -ne $clash.contract) {
        $owner = $clash.contract.owner
        $other = if ($owner -eq $a) { $b } else { $a }
        $menu = @(
            @{ choice = 'split_at_contract'; owner = $owner; contract = $clash.contract.interface.name },
            @{ choice = 'serialize'; first = $owner; then = $other }
        )
    } else {
        $menu = @(
            @{ choice = 'narrow'; task = $a },
            @{ choice = 'narrow'; task = $b },
            @{ choice = 'serialize'; first = $a; then = $b }
        )
    }
    if ($Option -gt $menu.Count) { return $null }
    $menu[$Option - 1]
}

function Format-Event($stamped) {
    $e = $stamped.event
    switch ($e.type) {
        'order_posted' { return "order o-$($e.order) posted on $($e.machine): $($e.sentence) ($(@($e.witnesses).Count) witnesses)" }
        'order_split' {
            $script:tasks = @($e.tasks)
            $titles = @($e.tasks | ForEach-Object { "t-$($_[0]) $($_[1].title)" })
            return "split into $($titles.Count) task(s): $($titles -join '; ')"
        }
        'readback_posted' {
            return "readback r-$($e.readback.id) on t-$($e.readback.task): $($e.readback.summary)"
        }
        'readbacks_reviewed' {
            $verdicts = @($e.verdicts | ForEach-Object { "r-$($_[0]) $(($_[1] | ConvertTo-Json -Compress))" })
            return "reviewed: $($verdicts -join '; ')"
        }
        'lease_granted' { return "claim t-$($e.task) granted (generation $($e.lease.generation), readback r-$($e.readback))" }
        'declared' { return "declared t-$($e.decl.task): $(@($e.decl.footprint.files) -join ', ')" }
        'clash_found' { return "CLASH c-$($e.clash) $($e.kind) on t-$($e.key.a) and t-$($e.key.b) at $($e.key.at | ConvertTo-Json -Compress)" }
        'clash_settled' { return "clash c-$($e.clash) settled: $($e.choice | ConvertTo-Json -Compress)" }
        'clash_proposed' { return "clash c-$($e.clash) proposed: $($e.choice | ConvertTo-Json -Compress)" }
        'clash_dissolved' { return "clash c-$($e.clash) dissolved" }
        'clash_explained' { return "clash c-$($e.clash) explained: $($e.explanation.consequence)" }
        'build_cleared' { return "build cleared for t-$($e.task)" }
        'build_reported' { return "built t-$($e.task) at $($e.head); gate run g-$($e.run)" }
        'witness_reported' { return "witness w-$($e.witness) on g-$($e.run): $($e.outcome | ConvertTo-Json -Compress)" }
        'gate_closed' { return "gate g-$($e.run) closed: $($e.verdict | ConvertTo-Json -Compress)" }
        'landed' { return "LANDED t-$($e.task) as $($e.commit)" }
        'land_failed' { return "land failed for t-$($e.task): $($e.reason)" }
        'lease_ended' { return "lease on t-$($e.task) ended: $($e.why | ConvertTo-Json -Compress)" }
        'model_used' { return "model $($e.model) for $($e.step): $($e.input_tokens) in, $($e.output_tokens) out" }
        default { return "$($e.type)" }
    }
}

function Handle-Event($stamped) {
    $e = $stamped.event
    $seq = [uint64]$stamped.seq
    $at = [uint64]$stamped.at
    if ($e.type -eq 'order_posted' -and $null -eq $script:originMs) { $script:originMs = $at }
    if ($e.type -eq 'model_used') {
        $script:modelSteps[$e.step] = 1 + [int]$script:modelSteps[$e.step]
    }
    if ($e.type -eq 'build_cleared' -and $null -eq $script:firstClearSeq) { $script:firstClearSeq = $seq }
    if ($e.type -eq 'readback_posted') { $script:readbackAts.Add($at) | Out-Null }
    if ($e.type -eq 'declared') {
        $script:declaredAts.Add($at) | Out-Null
        if (-not $script:declaredBy.ContainsKey("t-$($e.decl.task)")) { $script:declaredBy["t-$($e.decl.task)"] = $at }
    }
    if ($e.type -eq 'lease_granted') { $script:claimAts.Add($at) | Out-Null }
    if ($e.type -eq 'landed') { $script:landed[[string]$e.task] = $at }
    foreach ($name in @('order_split', 'readbacks_reviewed', 'clash_settled', 'clash_found')) {
        if ($e.type -eq $name -and -not $script:firstAt.ContainsKey($name)) { $script:firstAt[$name] = $at }
    }

    Write-Line ('{0,9}  seq {1,-4} {2}' -f ("+{0}s" -f (Get-Elapsed $at)), $seq, (Format-Event $stamped))

    if ($e.type -eq 'clash_found') {
        $script:clashes.Add([ordered]@{
            clash = $e.clash; kind = $e.kind; seq = $seq; atMs = $at
            file = $e.key.at.file; a = "t-$($e.key.a)"; b = "t-$($e.key.b)"
        }) | Out-Null
        if ($Settle -gt 0 -and -not $script:settled.ContainsKey([string]$e.clash)) {
            $choice = Get-ClashChoice -clash $e -Option $Settle
            if ($null -ne $choice) {
                $script:settled[[string]$e.clash] = $true
                try {
                    Invoke-Api -Port $Port -Token $owner -Method POST -Path '/api/commands' -Body @{
                        cmd    = 'settle_clash'
                        clash  = $e.clash
                        choice = $choice
                    } | Out-Null
                    Write-Line ("          settled c-{0} with option {1} as a human would: {2}" -f $e.clash, $Settle, ($choice | ConvertTo-Json -Compress))
                } catch {
                    Write-Line ("          settle of c-{0} refused: {1}" -f $e.clash, $_.ErrorDetails.Message)
                }
            }
        }
    }
}

# The worker logs one line per write, stamped when the write returned. Those stamps are the only
# record of a write in the run, since the origin logs a clearance, not a write.
function Get-Writes {
    param([string]$LogsDir)
    foreach ($file in Get-ChildItem -LiteralPath $LogsDir -Filter 'worker-*.err.log') {
        foreach ($line in Get-Content -LiteralPath $file.FullName) {
            if ($line -match 'wrote (\S+) on (t-\d+)') {
                $path = $Matches[1]
                $task = $Matches[2]
                if ($line -match '(\d{4}-\d{2}-\d{2}T[\d:.]+Z)') {
                    [pscustomobject]@{ utc = [datetime]::Parse($Matches[1]).ToUniversalTime(); path = $path; task = $task }
                }
            }
        }
    }
}

$deadline = (Get-Date).AddSeconds($TimeoutSecs)
$next = 0
Write-Line "watching $RunDir (settle option: $(if ($Settle -eq 0) { 'by hand in the UI' } else { $Settle }), timeout ${TimeoutSecs}s)"
while ((Get-Date) -lt $deadline) {
    try {
        $batch = Invoke-Api -Port $Port -Token $owner -Method GET -Path "/api/events?from=$next"
    } catch {
        Write-Line "event poll failed: $($_.Exception.Message)"
        Start-Sleep -Milliseconds $PollMs
        continue
    }
    foreach ($stamped in @($batch | Where-Object { $null -ne $_ })) {
        Add-Content -LiteralPath $rawPath -Value ($stamped | ConvertTo-Json -Depth 20 -Compress) -Encoding utf8
        Handle-Event $stamped
        $next = [uint64]$stamped.seq + 1
    }
    $taskCount = @($script:tasks).Count
    if ($taskCount -gt 0 -and $script:landed.Count -eq $taskCount) { break }
    Start-Sleep -Milliseconds $PollMs
}

# The summary is computed from the stored events, so it matches the raw log.
$origin = $script:originMs
function Rel($ms) { if ($null -eq $ms) { return $null }; Get-Elapsed ([uint64]$ms) }
$clash0 = if ($script:clashes.Count -gt 0) { $script:clashes[0] } else { $null }
$clashSeq = if ($null -ne $clash0) { [uint64]$clash0.seq } else { $null }
$clashUtc = if ($null -ne $clash0) { [DateTimeOffset]::FromUnixTimeMilliseconds([long]$clash0.atMs).UtcDateTime } else { $null }
$writes = @(Get-Writes -LogsDir $logs)
$firstWriteUtc = if ($writes.Count -gt 0) { ($writes | Sort-Object utc | Select-Object -First 1).utc } else { $null }
# "Before any write" is measured against the contested file: no task wrote it before the clash.
$contested = if ($null -ne $clash0) { @($writes | Where-Object { $_.path -eq $clash0.file }) } else { @() }
$firstContestedUtc = if ($contested.Count -gt 0) { ($contested | Sort-Object utc | Select-Object -First 1).utc } else { $null }
$clashBeforeWrite = ($null -ne $clashUtc) -and ($null -eq $firstContestedUtc -or $clashUtc -lt $firstContestedUtc)
# The blocked task is the one that declared second. It is the task the clash holds back.
$blockedTask = $null
$blockedWriteUtc = $null
$clashBeforeBlockedWrite = $null
if ($null -ne $clash0) {
    $da = $script:declaredBy[$clash0.a]
    $db = $script:declaredBy[$clash0.b]
    $blockedTask = if ($null -ne $da -and $null -ne $db -and $db -gt $da) { $clash0.b } else { $clash0.a }
    $blockedWrites = @($contested | Where-Object { $_.task -eq $blockedTask })
    $blockedWriteUtc = if ($blockedWrites.Count -gt 0) { ($blockedWrites | Sort-Object utc | Select-Object -First 1).utc } else { $null }
    $clashBeforeBlockedWrite = ($null -eq $blockedWriteUtc) -or ($clashUtc -lt $blockedWriteUtc)
}
$taskCount = @($script:tasks).Count
$allLanded = $taskCount -gt 0 -and $script:landed.Count -eq $taskCount
$outcome = if ($null -eq $clashSeq) {
    if ($allLanded) { 'no_clash_landed' } else { 'no_clash_incomplete' }
} elseif (-not $clashBeforeWrite) {
    'clash_after_write'
} elseif ($allLanded) {
    'clash_before_write_landed'
} else {
    'clash_before_write_not_landed'
}

$usage = $null
try { $usage = Invoke-Api -Port $Port -Token $owner -Method GET -Path '/api/usage' } catch { }
$liveFailures = 0
$serveErr = Join-Path $logs 'acan-serve.err.log'
if (Test-Path -LiteralPath $serveErr) {
    $liveFailures = @(Select-String -LiteralPath $serveErr -Pattern 'live model call failed').Count
}

$summary = [ordered]@{
    outcome                = $outcome
    tasks                  = $taskCount
    landed                 = $script:landed.Count
    clashes                = @($script:clashes)
    clash_before_write     = $clashBeforeWrite
    clash_utc              = if ($null -ne $clashUtc) { $clashUtc.ToString('o') } else { $null }
    first_write_utc        = if ($null -ne $firstWriteUtc) { $firstWriteUtc.ToString('o') } else { $null }
    writes                 = $writes.Count
    seconds                = [ordered]@{
        split              = Rel $script:firstAt['order_split']
        last_readback      = Rel (($script:readbackAts | Measure-Object -Maximum).Maximum)
        reviewed           = Rel $script:firstAt['readbacks_reviewed']
        first_claim        = Rel (($script:claimAts | Measure-Object -Minimum).Minimum)
        first_declare      = Rel (($script:declaredAts | Measure-Object -Minimum).Minimum)
        clash_found        = Rel $script:firstAt['clash_found']
        settled            = Rel $script:firstAt['clash_settled']
        last_land          = Rel (($script:landed.Values | Measure-Object -Maximum).Maximum)
    }
    model_steps            = $script:modelSteps
    live_call_failures     = $liveFailures
    usage                  = $usage
    contested_file         = if ($null -ne $clash0) { $clash0.file } else { $null }
    blocked_task           = $blockedTask
    clash_before_blocked_write = $clashBeforeBlockedWrite
    blocked_first_write_utc = if ($null -ne $blockedWriteUtc) { $blockedWriteUtc.ToString('o') } else { $null }
    first_contested_write_utc = if ($null -ne $firstContestedUtc) { $firstContestedUtc.ToString('o') } else { $null }
    declared_ms            = $script:declaredBy
    origin_known           = ($null -ne $origin)
}
$summary | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $logs 'summary.json') -Encoding utf8
Write-Line "OUTCOME: $outcome (tasks $taskCount, landed $($script:landed.Count), clashes $($script:clashes.Count), live call failures $liveFailures)"
Write-Output (Get-Content -LiteralPath (Join-Path $logs 'summary.json') -Raw)
