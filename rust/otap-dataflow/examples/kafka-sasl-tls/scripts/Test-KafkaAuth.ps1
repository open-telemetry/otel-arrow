[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$exampleDirectory = Split-Path -Parent $PSScriptRoot
$composeFile = Join-Path $exampleDirectory 'compose.yaml'

$runningServices = @(docker compose -f $composeFile ps --status running --services 2>&1)
if ($LASTEXITCODE -ne 0) {
    $runningServices | Write-Host
    throw "Failed to inspect the Kafka Compose services."
}
if (-not ($runningServices | Where-Object { $_ -eq 'kafka' })) {
    throw "Kafka is not running. Run: docker compose -f `"$composeFile`" up -d --wait kafka"
}

$tests = @(
    @{
        Name = 'PLAIN'
        User = 'plain'
        Password = 'plain-secret'
        LoginModule = 'plain.PlainLoginModule'
    },
    @{
        Name = 'SCRAM-SHA-256'
        User = 'scram256'
        Password = 'scram256-secret'
        LoginModule = 'scram.ScramLoginModule'
    },
    @{
        Name = 'SCRAM-SHA-512'
        User = 'scram512'
        Password = 'scram512-secret'
        LoginModule = 'scram.ScramLoginModule'
    }
)

foreach ($test in $tests) {
    $jaas = 'sasl.jaas.config=org.apache.kafka.common.security.{0} required username="{1}" password="{2}";' -f `
        $test.LoginModule, $test.User, $test.Password
    $properties = @(
        'security.protocol=SASL_SSL'
        'ssl.truststore.location=/etc/kafka/secrets/kafka.truststore.jks'
        'ssl.truststore.password=changeit'
        "sasl.mechanism=$($test.Name)"
        $jaas
    ) -join "`n"

    $properties | docker compose -f $composeFile exec -T kafka `
        sh -c 'cat >/tmp/client.properties'
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to write the $($test.Name) Kafka client configuration."
    }

    $output = docker compose -f $composeFile exec -T kafka `
        kafka-broker-api-versions `
        --bootstrap-server localhost:9093 `
        --command-config /tmp/client.properties 2>&1
    if ($LASTEXITCODE -ne 0) {
        $output | Write-Host
        throw "$($test.Name) authentication over TLS failed."
    }

    $broker = $output | Select-Object -First 1
    Write-Host "PASS: $($test.Name) over TLS - $broker"
}
