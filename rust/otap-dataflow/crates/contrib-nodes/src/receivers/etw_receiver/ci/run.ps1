$ErrorActionPreference = "Stop"

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

    foreach ($test in $tests) {
        cargo @cargoArgs -- --ignored --exact $test
        if ($LASTEXITCODE -ne 0) {
            exit $LASTEXITCODE
        }
    }
} finally {
    Pop-Location
}
