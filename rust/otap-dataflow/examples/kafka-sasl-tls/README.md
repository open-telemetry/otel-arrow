# Kafka SASL over TLS Local Validation

This example validates the Kafka receiver against a real TLS and SASL
handshake. It exercises three independent consumer groups:

- `otap-plain-consumer` using `PLAIN`
- `otap-scram-256-consumer` using `SCRAM-SHA-256`
- `otap-scram-512-consumer` using `SCRAM-SHA-512`

The local credentials are intentionally fixed and must not be reused outside
this development environment.

## Prerequisites

- Docker Desktop with Linux containers
- Docker Compose
- The Rust toolchain specified by this repository
- LLVM with `libclang.dll` on Windows. The Kafka dependency enables Zstandard,
  whose native build uses bindgen.

Run the following commands in PowerShell from the repository root:

```powershell
Set-Location rust/otap-dataflow
$ComposeFile = "examples/kafka-sasl-tls/compose.yaml"
$DataflowConfig = "examples/kafka-sasl-tls/kafka-sasl-tls.yaml"
```

If LLVM is not installed, install it from an elevated PowerShell terminal:

```powershell
choco install llvm -y
```

Set `LIBCLANG_PATH` in every PowerShell session used to build the dataflow:

```powershell
$Env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"
Test-Path "$Env:LIBCLANG_PATH\libclang.dll"
```

The last command must print `True`.

## Check Docker

Start Docker Desktop if necessary, then verify that its Linux engine responds:

```powershell
Start-Process "$Env:ProgramFiles\Docker\Docker\Docker Desktop.exe"
docker version
docker compose version
docker compose -f $ComposeFile config --quiet
```

`docker compose config` produces no output when the Compose file is valid.

## Start Kafka

Generate a local CA and broker certificate, start Kafka, create the SCRAM
credentials, and create the three topics:

```powershell
docker compose -f $ComposeFile up -d --wait kafka
docker compose -f $ComposeFile run --rm kafka-init
docker compose -f $ComposeFile ps
```

The broker exposes only its authenticated client listener to the host at
`localhost:9093`. Its controller and administrative listener remain inside the
Compose network.

The initialization command is idempotent. It creates these topics:

- `otlp-logs-plain`
- `otlp-logs-scram-256`
- `otlp-logs-scram-512`

## Test the SASL and TLS Handshakes

Run the PowerShell handshake test against the broker's `SASL_SSL` listener:

```powershell
& ./examples/kafka-sasl-tls/scripts/Test-KafkaAuth.ps1
```

Expected output:

```text
PASS: PLAIN over TLS - localhost:9093 ...
PASS: SCRAM-SHA-256 over TLS - localhost:9093 ...
PASS: SCRAM-SHA-512 over TLS - localhost:9093 ...
```

This check uses Kafka's client tools and the generated truststore. It proves
that the broker accepts each SASL mechanism through its TLS listener before the
dataflow receiver is started.

## Validate and Run the Dataflow

The config contains three finite traffic generators and three Kafka receivers.
The generators publish OTLP protobuf logs using the same mechanism as the
corresponding receiver, and each receiver writes decoded telemetry to a console
exporter.

```powershell
cargo run --features "kafka-receiver,otap-df-contrib-nodes/kafka-exporter" -- --config $DataflowConfig --validate-and-exit
```

Start the dataflow and save its output:

```powershell
cargo run --features "kafka-receiver,otap-df-contrib-nodes/kafka-exporter" -- --config $DataflowConfig --num-cores 1 2>&1 | Tee-Object -FilePath kafka-sasl-tls.log
```

After console output appears for all three receiver pipelines, inspect the
consumer groups in another terminal:

```powershell
$ConsumerGroups = @(
    "otap-plain-consumer"
    "otap-scram-256-consumer"
    "otap-scram-512-consumer"
)

$ConsumerGroups | ForEach-Object {
    docker compose -f $ComposeFile exec -T kafka kafka-consumer-groups --bootstrap-server kafka:29092 --describe --group $_
}
```

Successful delivery and group descriptions validate TLS server verification,
the selected SASL handshake, group membership, OTLP decoding, and downstream
delivery. Stop the dataflow with `Ctrl-C`.

To search the captured output for Kafka authentication, decoding, or connection
errors:

```powershell
Select-String -Path kafka-sasl-tls.log -Pattern "authentication|SSL|SASL|decode|connection|error"
```

## Troubleshooting

Inspect the broker and certificate generator logs:

```powershell
docker compose -f $ComposeFile logs --no-log-prefix kafka
docker compose -f $ComposeFile logs --no-log-prefix certgen
```

Regenerate the local CA and broker certificate:

```powershell
docker compose -f $ComposeFile down -v
Remove-Item -Recurse -Force examples/kafka-sasl-tls/certs -ErrorAction SilentlyContinue
docker compose -f $ComposeFile up -d --wait kafka
docker compose -f $ComposeFile run --rm kafka-init
```

## Clean Up

```powershell
docker compose -f $ComposeFile down -v
Remove-Item -Recurse -Force examples/kafka-sasl-tls/certs -ErrorAction SilentlyContinue
Remove-Item kafka-sasl-tls.log -ErrorAction SilentlyContinue
```
