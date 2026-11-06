$ErrorActionPreference = "Stop"

Push-Location "rust\otap-dataflow"
try {
    cargo build --locked --no-default-features `
        --features crypto-ring,windows-perf-counters --bin df_engine
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to build df_engine for configuration validation."
    }
    foreach ($config in @(
        "configs\windows-perf-counters-console.yaml",
        "configs\windows-perf-counters-wildcard-console.yaml"
    )) {
        .\target\debug\df_engine.exe --config $config --validate-and-exit
        if ($LASTEXITCODE -ne 0) {
            throw "Configuration validation failed for '$config'."
        }
    }

    foreach ($filter in @(
        "windows_perf_counters_receiver::pdh::tests::live::",
        "windows_perf_counters_receiver::runtime::tests::live_"
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
