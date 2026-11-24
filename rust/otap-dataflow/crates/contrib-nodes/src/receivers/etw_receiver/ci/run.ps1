$ErrorActionPreference = "Stop"

# The container end-to-end test needs a Windows Docker engine. Hosted runners
# install it as the `docker` service but do not always start it, so start it
# before the build if no engine answers, and wait for the engine before the
# tests run.
function Test-DockerEngine {
    $serverOs = docker version --format '{{.Server.Os}}' 2>$null
    return ($LASTEXITCODE -eq 0 -and $serverOs -eq "windows")
}

function Start-DockerService {
    if (Test-DockerEngine) {
        return
    }
    $service = Get-Service -Name docker -ErrorAction SilentlyContinue
    if ($null -eq $service) {
        Write-Host "::error::No Windows Docker engine is reachable and no docker service is installed."
        exit 1
    }
    Write-Host "docker service: status=$($service.Status), start type=$($service.StartType)"
    if ($service.Status -ne "Running") {
        try {
            Start-Service -Name docker
        } catch {
            Write-Host "::error::Failed to start the docker service: $_"
            exit 1
        }
    }
}

function Wait-DockerEngine([TimeSpan] $Timeout) {
    $deadline = [DateTime]::UtcNow + $Timeout
    while (-not (Test-DockerEngine)) {
        if ([DateTime]::UtcNow -ge $deadline) {
            docker version
            Write-Host "::error::The Windows Docker engine was not ready within $Timeout."
            exit 1
        }
        Start-Sleep -Seconds 2
    }
}

Start-DockerService

# Each ETW end-to-end test must run in its own process: `one_collect` uses a
# fixed ETW session GUID, and a receiver session lives until its process exits.
$tests = @(
    "receivers::etw_receiver::windows_e2e_tests::etw_receiver_decodes_tracelogging_events_end_to_end",
    "receivers::etw_receiver::windows_container_e2e_tests::etw_receiver_collects_events_from_process_isolated_container_end_to_end"
)
$cargoArgs = @("test", "--locked", "-p", "otel-arrow-dfe-contrib-nodes", "--lib", "--features", "etw")

Push-Location "rust/otap-dataflow"
try {
    # `--exact` succeeds without running anything when a name no longer
    # matches, so check every selector against the listed ignored tests first.
    $listed = cargo @cargoArgs -- --ignored --list
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }
    $missing = @($tests | Where-Object { $listed -notcontains "$($_): test" })
    if ($missing.Count -gt 0) {
        Write-Host "::error::ETW end-to-end tests not found (renamed or removed?): $($missing -join ', ')"
        exit 1
    }

    Wait-DockerEngine ([TimeSpan]::FromMinutes(2))

    foreach ($test in $tests) {
        cargo @cargoArgs -- --ignored --exact $test
        if ($LASTEXITCODE -ne 0) {
            exit $LASTEXITCODE
        }
    }
} finally {
    Pop-Location
}
