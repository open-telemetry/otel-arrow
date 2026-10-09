$ErrorActionPreference = "Stop"

$artifactDir = $env:OTEL_ARROW_INTEGRATION_ARTIFACT_DIR
if (-not $artifactDir) {
    throw "OTEL_ARROW_INTEGRATION_ARTIFACT_DIR is required"
}
New-Item -ItemType Directory -Force -Path $artifactDir | Out-Null
$output = Join-Path $artifactDir "cargo-test.log"

Push-Location "rust/otap-dataflow"
try {
    cargo test --locked `
        -p otel-arrow-dfe-contrib-nodes `
        etw_receiver_decodes_tracelogging_events_end_to_end `
        --features etw `
        -- --ignored --nocapture 2>&1 | Tee-Object -FilePath $output
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }
} finally {
    Pop-Location
}

$testOutput = Get-Content -Raw -Path $output
if ($testOutput -notmatch "test result: ok\. 1 passed; 0 failed; 0 ignored;") {
    throw "Expected exactly one ETW smoke test to execute."
}
