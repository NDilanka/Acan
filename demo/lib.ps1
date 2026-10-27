# Shared helpers for the demo scripts. Dot-sourced by run.ps1, watch.ps1 and stop.ps1.
# Nothing here prints a token or a provider key.

$ErrorActionPreference = 'Stop'

$script:WorktreeRoot = Split-Path -Parent $PSScriptRoot

function Get-RunState {
    param([string]$RunDir)
    $path = Join-Path $RunDir 'logs\run.json'
    if (-not (Test-Path -LiteralPath $path)) { return $null }
    Get-Content -LiteralPath $path -Raw | ConvertFrom-Json
}

function Save-RunState {
    param([hashtable]$State, [string]$RunDir)
    $dir = Join-Path $RunDir 'logs'
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
    $State | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $dir 'run.json') -Encoding utf8
}

function Get-OwnerToken {
    param([string]$RunDir)
    $entries = Get-Content -LiteralPath (Join-Path $RunDir '.acan\tokens.json') -Raw | ConvertFrom-Json
    ($entries | Where-Object { $_.principal.kind -eq 'member' } | Select-Object -First 1).token
}

# Starts a process with stdout and stderr in the run's logs folder and records it in the run state.
function Start-Logged {
    param(
        [hashtable]$State,
        [string]$RunDir,
        [string]$Name,
        [string]$File,
        [string[]]$Arguments,
        [string]$WorkingDirectory
    )
    $logs = Join-Path $RunDir 'logs'
    New-Item -ItemType Directory -Force -Path $logs | Out-Null
    # Start-Process joins the argument array without quotes, so a value with spaces is quoted here.
    $quoted = $Arguments | ForEach-Object { if ($_ -match '\s') { '"' + $_ + '"' } else { $_ } }
    $proc = Start-Process -FilePath $File -ArgumentList $quoted -WorkingDirectory $WorkingDirectory `
        -RedirectStandardOutput (Join-Path $logs "$Name.out.log") `
        -RedirectStandardError (Join-Path $logs "$Name.err.log") `
        -PassThru -WindowStyle Hidden
    $State.processes.Add([ordered]@{ name = $Name; pid = $proc.Id; startTicks = $proc.StartTime.Ticks }) | Out-Null
    Save-RunState -State $State -RunDir $RunDir
    $proc
}

function Invoke-Api {
    param([int]$Port, [string]$Token, [string]$Method, [string]$Path, $Body = $null)
    $params = @{
        Method     = $Method
        Uri        = "http://127.0.0.1:$Port$Path"
        Headers    = @{ Authorization = "Bearer $Token" }
        TimeoutSec = 60
    }
    if ($null -ne $Body) {
        $params.Body = $Body | ConvertTo-Json -Depth 12 -Compress
        $params.ContentType = 'application/json'
    }
    Invoke-RestMethod @params
}

function Wait-Http200 {
    param([string]$Url, [int]$Seconds)
    $deadline = (Get-Date).AddSeconds($Seconds)
    while ((Get-Date) -lt $deadline) {
        try {
            $response = Invoke-WebRequest -Uri $Url -TimeoutSec 3
            if ($response.StatusCode -eq 200) { return $true }
        } catch {
            # Not up yet; retry until the deadline.
        }
        Start-Sleep -Milliseconds 300
    }
    $false
}
