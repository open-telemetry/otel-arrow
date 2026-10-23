$ErrorActionPreference = "Stop"

Push-Location "rust\otap-dataflow"
try {
    cargo test --locked `
        -p otel-arrow-dfe-contrib-nodes `
        windows_perf_counters_receiver::pdh::native::tests::live_object_refresh_preserves_thread_language `
        --features windows-perf-counters `
        -- --ignored --test-threads=1
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }
} finally {
    Pop-Location
}
