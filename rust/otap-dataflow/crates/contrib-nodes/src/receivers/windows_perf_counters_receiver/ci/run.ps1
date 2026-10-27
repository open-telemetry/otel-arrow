$ErrorActionPreference = "Stop"

Push-Location "rust\otap-dataflow"
try {
    foreach ($filter in @(
        "windows_perf_counters_receiver::pdh::tests::live::live_pdh_",
        "windows_perf_counters_receiver::pdh::native::tests::live_object_refresh_preserves_thread_language"
    )) {
        $output = cargo test --locked `
            -p otel-arrow-dfe-contrib-nodes `
            --lib $filter `
            --features windows-perf-counters `
            -- --ignored --test-threads=1
        $code = $LASTEXITCODE
        $output | Write-Output
        if ($code -ne 0) {
            throw "Native test filter '$filter' failed with exit code $code."
        }
        if (-not ($output -match '^test result: ok\. [1-9]\d* passed; 0 failed;')) {
            throw "Native test filter '$filter' ran no passing tests."
        }
    }
} finally {
    Pop-Location
}
