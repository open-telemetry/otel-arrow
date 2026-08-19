$ErrorActionPreference = "Stop"

$fixtureDir = $PSScriptRoot
$composeArgs = @("compose", "--project-directory", $fixtureDir, "-f", (Join-Path $fixtureDir "docker-compose.yaml"))
$env:KAFKA_SASL_TLS_PORT = if ($env:KAFKA_SASL_TLS_PORT) { $env:KAFKA_SASL_TLS_PORT } else { "39093" }
$env:KAFKA_SASL_TLS_HOST = "host.docker.internal"
$env:OTEL_ARROW_REPO_ROOT = (& git -C $fixtureDir rev-parse --show-toplevel).Trim()
if ($LASTEXITCODE -ne 0) {
    throw "Could not resolve the otel-arrow repository root"
}

& docker @composeArgs down --volumes --remove-orphans
if ($LASTEXITCODE -ne 0) {
    throw "Could not remove stale Kafka SASL/TLS fixture state"
}

try {
    & docker @composeArgs up --detach --wait --wait-timeout 180 broker
    if ($LASTEXITCODE -ne 0) {
        throw "Kafka SASL/TLS broker failed to become healthy"
    }

    & docker @composeArgs run --build --rm --no-deps receiver-test
    if ($LASTEXITCODE -ne 0) {
        throw "Kafka receiver SASL/TLS validation failed"
    }
}
catch {
    & docker @composeArgs logs broker
    throw
}
finally {
    & docker @composeArgs down --volumes --remove-orphans
}
