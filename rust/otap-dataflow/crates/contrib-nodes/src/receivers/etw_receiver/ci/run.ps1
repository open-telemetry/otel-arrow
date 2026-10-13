$ErrorActionPreference = "Stop"

Push-Location "rust/otap-dataflow"
try {
    cargo test --locked `
        -p otel-arrow-dfe-contrib-nodes `
        etw_receiver_decodes_tracelogging_events_end_to_end `
        --features etw `
        -- --ignored
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }
} finally {
    Pop-Location
}
