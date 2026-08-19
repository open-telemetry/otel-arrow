# Kafka SASL over TLS Local Validation

This example validates the otel-arrow Kafka exporter and receiver against a
real Kafka broker. It sends OTLP protobuf logs through three independent
SASL-over-TLS pipelines:

| Mechanism | Topic | Consumer group |
| --- | --- | --- |
| `PLAIN` | `otlp-logs-plain` | `otap-plain-consumer` |
| `SCRAM-SHA-256` | `otlp-logs-scram-256` | `otap-scram-256-consumer` |
| `SCRAM-SHA-512` | `otlp-logs-scram-512` | `otap-scram-512-consumer` |

The fixed credentials and generated certificates are for local development
only.

## Prerequisites

- Docker Desktop using Linux containers
- The Rust toolchain specified by this repository
- Visual Studio 2022 or Build Tools with the **Desktop development with C++**
  workload
- LLVM with `libclang.dll`
- vcpkg with `openssl:x64-windows-static-md`

Run all commands below from **Developer PowerShell for Visual Studio**. Start
in the repository root and define the example paths once:

```powershell
Set-Location rust/otap-dataflow
$ComposeFile = "examples/kafka-sasl-tls/compose.yaml"
$DataflowConfig = "examples/kafka-sasl-tls/kafka-sasl-tls.yaml"
```

### Prepare the Windows Build Environment

Install LLVM from an elevated PowerShell terminal if it is not already
installed:

```powershell
choco install llvm -y
```

Bootstrap a user-local vcpkg installation if `vcpkg.exe` is not available:

```powershell
$VcpkgRoot = "$Env:USERPROFILE\vcpkg"

if (-not (Test-Path "$VcpkgRoot\vcpkg.exe")) {
    if (-not (Test-Path "$VcpkgRoot\bootstrap-vcpkg.bat")) {
        git clone --depth 1 https://github.com/microsoft/vcpkg.git $VcpkgRoot
    }
    & "$VcpkgRoot\bootstrap-vcpkg.bat" -disableMetrics
}
```

Install OpenSSL and configure the current Developer PowerShell session:

```powershell
& "$VcpkgRoot\vcpkg.exe" install openssl:x64-windows-static-md

$Env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"
$Env:VCPKG_INSTALLATION_ROOT = $VcpkgRoot
$Env:OPENSSL_DIR = "$VcpkgRoot\installed\x64-windows-static-md"
$Env:OPENSSL_ROOT_DIR = $Env:OPENSSL_DIR
$Env:OPENSSL_USE_STATIC_LIBS = "TRUE"

Test-Path "$Env:LIBCLANG_PATH\libclang.dll"
Test-Path "$Env:OPENSSL_DIR\include\openssl\ssl.h"
Test-Path "$Env:OPENSSL_DIR\lib\libssl.lib"
Test-Path "$Env:OPENSSL_DIR\lib\libcrypto.lib"
```

All four checks must print `True`. Set these environment variables again when
using a new Developer PowerShell session.

## Start Kafka

Start Docker Desktop if necessary, validate the Compose file, and start the
broker:

```powershell
Start-Process "$Env:ProgramFiles\Docker\Docker\Docker Desktop.exe"
docker version
docker compose -f $ComposeFile config --quiet
docker compose -f $ComposeFile up -d --wait kafka
docker compose -f $ComposeFile run --rm kafka-init
docker compose -f $ComposeFile ps
```

The startup generates a local CA and broker certificate, creates the SCRAM
users, and creates all three topics. The broker exposes its authenticated
listener at `localhost:9093`.

## Check Broker Authentication

Verify that the broker accepts each SASL mechanism through its TLS listener:

```powershell
& ./examples/kafka-sasl-tls/scripts/Test-KafkaAuth.ps1
```

Expected output:

```text
PASS: PLAIN over TLS - localhost:9093 ...
PASS: SCRAM-SHA-256 over TLS - localhost:9093 ...
PASS: SCRAM-SHA-512 over TLS - localhost:9093 ...
```

This is a broker-only check using Kafka's client tools. The next steps validate
the otel-arrow exporter and receiver.

## Run the Dataflow

First validate the six-pipeline configuration:

```powershell
cargo run --features "kafka-receiver,otap-df-contrib-nodes/kafka-exporter" -- --config $DataflowConfig --validate-and-exit
```

Then start the dataflow and capture its console output:

```powershell
cargo run --features "kafka-receiver,otap-df-contrib-nodes/kafka-exporter" -- --config $DataflowConfig --num-cores 1 2>&1 | Tee-Object -FilePath kafka-sasl-tls.log
```

Each traffic generator publishes OTLP logs through its Kafka exporter. The
matching Kafka receiver consumes and decodes the logs, then sends them to a
console exporter.

## Verify End-to-End Delivery

While the dataflow is running, open another PowerShell terminal from
`rust/otap-dataflow` and inspect all three consumer groups:

```powershell
$ComposeFile = "examples/kafka-sasl-tls/compose.yaml"
$ConsumerGroups = @(
    "otap-plain-consumer"
    "otap-scram-256-consumer"
    "otap-scram-512-consumer"
)

$ConsumerGroups | ForEach-Object {
    docker compose -f $ComposeFile exec -T kafka `
        kafka-consumer-groups --bootstrap-server kafka:29092 `
        --describe --group $_
}
```

The validation succeeds when:

- The dataflow console shows decoded log batches from all three receivers.
- Every consumer group reports `LAG` as `0`.
- The log contains no authentication, TLS, decoding, or connection failures.

Stop the dataflow with `Ctrl-C` after verification.

## Troubleshooting

- `Unable to find libclang`: verify that `LIBCLANG_PATH` contains
  `libclang.dll`.
- `Could NOT find OpenSSL`: verify that `OPENSSL_ROOT_DIR` points to the vcpkg
  `x64-windows-static-md` installation.
- Native compiler or CMake errors: run Cargo from Developer PowerShell for
  Visual Studio.

Inspect the container logs:

```powershell
docker compose -f $ComposeFile logs --no-log-prefix kafka
docker compose -f $ComposeFile logs --no-log-prefix certgen
```

To regenerate the certificates and broker state:

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
