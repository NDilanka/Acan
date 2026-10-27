# Stages one take of the hackathon demo on a fresh copy of the demo repository.
#
# It clones the demo repo into a run folder, copies the live routing and provider key file in,
# starts `acan serve` with the UI and the demo app, mints tokens for the owner and two agents,
# registers the agents, posts the order with its witnesses, and starts two worker slots (worker-slot.ps1),
# each running acan-worker one task at a time. It returns once everything is running. Run watch.ps1
# next to follow the take and settle clashes.
#
# The owner token is never printed. It is in the run folder, in .acan\tokens.json and in
# logs\ui-url.txt, which holds the UI URL with the token in its fragment.

[CmdletBinding()]
param(
    [string]$RunDir = 'D:\dev\acan-run',
    [string]$DemoRepo = 'D:\dev\acan-demo',
    [int]$Port = 7311,
    [int]$DemoPort = 3000,
    [string]$Machine = 'src',
    [string]$Sentence = 'Add rate limiting to the API in src/middleware.ts. Add a limit per client address to runRoute, backed by a fixed-window store with expiry cleanup, and send 429 responses with Retry-After and X-RateLimit-Remaining headers. Add a limit per signed-in account to runRoute too, using the same store shape. Each limit is a check that runs before the route handler, so every route passes both. Keep the login flow untouched.',
    [int]$WaitSecs = 900
)

. (Join-Path $PSScriptRoot 'lib.ps1')

$repo = $script:WorktreeRoot
$acan = Join-Path $repo 'target\debug\acan.exe'
$worker = Join-Path $repo 'target\debug\acan-worker.exe'
$www = Join-Path $repo 'crates\acan-render-web\www'
$llmSource = Join-Path $repo 'crates\acan-llm\llm.toml'
$envSource = Join-Path $repo 'crates\acan-llm\.env'

# One take at a time. The previous take's folder is renamed aside, not deleted, so its logs stay.
& (Join-Path $PSScriptRoot 'stop.ps1') -RunDir $RunDir
if (Test-Path -LiteralPath $RunDir) {
    $aside = "$RunDir-prev-$(Get-Date -Format 'yyyyMMdd-HHmmss')"
    Move-Item -LiteralPath $RunDir -Destination $aside
    Write-Output "moved the previous run folder to $aside"
}

git clone --quiet $DemoRepo $RunDir
if ($LASTEXITCODE -ne 0) { throw "git clone of $DemoRepo failed with exit code $LASTEXITCODE" }

$logs = Join-Path $RunDir 'logs'
New-Item -ItemType Directory -Force -Path $logs | Out-Null
$state = @{
    runDir    = $RunDir
    port      = $Port
    demoPort  = $DemoPort
    machine   = $Machine
    sentence  = $Sentence
    processes = New-Object System.Collections.ArrayList
}
Save-RunState -State $state -RunDir $RunDir

# The copied key file must not show up as untracked in the run repo, since that blocks a land.
Add-Content -LiteralPath (Join-Path $RunDir '.git\info\exclude') -Value '/.env'

$acanDir = Join-Path $RunDir '.acan'
New-Item -ItemType Directory -Force -Path $acanDir | Out-Null
Copy-Item -LiteralPath $llmSource -Destination (Join-Path $acanDir 'llm.toml')
Copy-Item -LiteralPath $envSource -Destination (Join-Path $RunDir '.env')

# Tokens go in before the first start, so the server loads them and mints no owner token of its own.
$owner = 'acan_' + [guid]::NewGuid().ToString('N')
$agentOne = 'acan_' + [guid]::NewGuid().ToString('N')
$agentTwo = 'acan_' + [guid]::NewGuid().ToString('N')
$tokens = @(
    @{ token = $owner; principal = @{ kind = 'member'; id = 'owner' } },
    @{ token = $agentOne; principal = @{ kind = 'agent'; id = 'agent-1' } },
    @{ token = $agentTwo; principal = @{ kind = 'agent'; id = 'agent-2' } }
)
$tokens | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $acanDir 'tokens.json') -Encoding utf8

Push-Location $repo
try {
    cargo build -p acan-server -p acan-agent --bins *> (Join-Path $logs 'build.log')
    if ($LASTEXITCODE -ne 0) { throw "cargo build failed; see $logs\build.log" }
} finally {
    Pop-Location
}

$serve = Start-Logged -State $state -RunDir $RunDir -Name 'acan-serve' -File $acan -WorkingDirectory $RunDir -Arguments @(
    'serve',
    '--repo', $RunDir,
    '--port', "$Port",
    '--web', $www,
    '--llm', (Join-Path $acanDir 'llm.toml'),
    '--protected-branch', 'main',
    '--witness-http-base-url', "http://127.0.0.1:$DemoPort",
    '--witness-concurrency', '2'
)
if (-not (Wait-Http200 -Url "http://127.0.0.1:$Port/health" -Seconds 60)) {
    throw "acan serve did not answer on port $Port; see $logs\acan-serve.err.log"
}

$env:PORT = "$DemoPort"
$bun = (Get-Command bun -CommandType Application).Source
$app = Start-Logged -State $state -RunDir $RunDir -Name 'demo-app' -File $bun -WorkingDirectory $RunDir -Arguments @('src/server.ts')
Remove-Item Env:PORT
if (-not (Wait-Http200 -Url "http://127.0.0.1:$DemoPort/health" -Seconds 60)) {
    throw "the demo app did not answer on port $DemoPort; see $logs\demo-app.err.log"
}

$plant = Invoke-Api -Port $Port -Token $owner -Method GET -Path '/api/plant'
$machines = @($plant.machines | ForEach-Object { $_.name })
Write-Output "plant machines: $($machines -join ', ')"
if ($machines -notcontains $Machine) { throw "machine '$Machine' is not on the plant" }

foreach ($name in @('agent-1', 'agent-2')) {
    Invoke-Api -Port $Port -Token $owner -Method POST -Path '/api/commands' -Body @{ cmd = 'register_agent'; agent = $name } | Out-Null
}

# The witnesses are the bar the build must clear: the login flow, the links flow, and the health route.
$witnesses = @(
    @{ check = @{ kind = 'test'; command = 'bun test test/login.test.ts' }; reason = 'the login flow stays untouched' },
    @{ check = @{ kind = 'test'; command = 'bun test test/links.test.ts' }; reason = 'links keep working' },
    @{ check = @{ kind = 'http'; method = 'GET'; path = '/health'; expect_status = 200; expect_body = 'ok' }; reason = 'the app still answers its health check' }
)
$posted = Invoke-Api -Port $Port -Token $owner -Method POST -Path '/api/commands' -Body @{
    cmd       = 'post_order'
    machine   = $Machine
    sentence  = $Sentence
    witnesses = $witnesses
}
$state.orderPostedUtc = (Get-Date).ToUniversalTime().ToString('o')
Save-RunState -State $state -RunDir $RunDir
Write-Output "order posted: $($posted.Count) event(s) returned"

# Each slot runs acan-worker again after every task, so the two slots can build a split of three.
$slot = Join-Path $PSScriptRoot 'worker-slot.ps1'
$pwsh = [Environment]::ProcessPath
foreach ($pair in @(@('worker-1', 'agent-1'), @('worker-2', 'agent-2'))) {
    Start-Logged -State $state -RunDir $RunDir -Name $pair[0] -File $pwsh -WorkingDirectory $RunDir -Arguments @(
        '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', $slot,
        '-Name', $pair[0], '-Agent', $pair[1], '-RunDir', $RunDir,
        '-Worker', $worker, '-Port', "$Port", '-WaitSecs', "$WaitSecs"
    ) | Out-Null
}

# The URL with the owner token in its fragment. The token never goes to the console.
$uiUrl = "http://127.0.0.1:$Port/#token=$owner"
Set-Content -LiteralPath (Join-Path $logs 'ui-url.txt') -Value $uiUrl -Encoding utf8
Write-Output "run folder: $RunDir"
Write-Output "UI: http://127.0.0.1:$Port/  (open $logs\ui-url.txt for the URL with the owner token)"
Write-Output "watch: pwsh $PSScriptRoot\watch.ps1 -RunDir `"$RunDir`" -Settle 3"
Write-Output "stop: pwsh $PSScriptRoot\stop.ps1 -RunDir `"$RunDir`""
