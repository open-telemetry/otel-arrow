$ErrorActionPreference = "Stop"

Push-Location "rust\otap-dataflow"
try {
    cargo test --locked `
        -p otel-arrow-dfe-contrib-nodes `
        windows_perf_counters_receiver::pdh::tests::live_pdh_ `
        --features windows-perf-counters `
        -- --ignored --test-threads=1
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }
} finally {
    Pop-Location
}
