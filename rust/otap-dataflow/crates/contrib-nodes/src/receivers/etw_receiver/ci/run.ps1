$ErrorActionPreference = "Stop"

# Each ETW end-to-end test must run in its own process: `one_collect` uses a
# fixed ETW session GUID, and a receiver session lives until its process exits.
$tests = @(
    "receivers::etw_receiver::windows_e2e_tests::etw_receiver_decodes_tracelogging_events_end_to_end",
    "receivers::etw_receiver::windows_container_e2e_tests::etw_receiver_collects_events_from_process_isolated_container_end_to_end"
)

Push-Location "rust/otap-dataflow"
try {
    foreach ($test in $tests) {
        cargo test --locked `
            -p otel-arrow-dfe-contrib-nodes `
            --lib `
            --features etw `
            -- --ignored --exact $test
        if ($LASTEXITCODE -ne 0) {
            exit $LASTEXITCODE
        }
    }
} finally {
    Pop-Location
}
