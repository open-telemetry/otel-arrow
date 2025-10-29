# Geneva Exporter

**Status:** ALPHA (Functional - supports logs and traces)

The Geneva Exporter is designed for Microsoft products to send telemetry data
to Microsoft's Geneva monitoring backend. It is not meant to be used outside
of Microsoft products and is open sourced to demonstrate best practices and to
be transparent about what is being collected.

## Build df_engine with Geneva Exporter

From the `otap-dataflow` directory:

```bash
cargo build --release --features geneva-exporter
```

## Verify the exporter is registered

```bash
./target/release/df_engine --help
```

You should see `urn:otel:geneva:exporter` in the Exporters list.

## Usage

### Running

```bash
./target/release/df_engine --pipeline config.yaml --num-cores 4
```

## Test Configuration

To test using the configuration file `otlp-geneva.yaml` provided
in this directory:

```bash
# Start the collector
./target/release/df_engine \
  --pipeline crates/otap/src/experimental/geneva_exporter/\
test-config-otlp-receiver.yaml \
  --num-cores 1

# In another terminal, send test data:

# Option A: Using telemetrygen (easiest)
telemetrygen logs --otlp-endpoint localhost:4317 --otlp-insecure --logs 10
telemetrygen traces --otlp-endpoint localhost:4317 --otlp-insecure --traces 10

# Option B: Using grpcurl (for manual testing)
grpcurl -plaintext \
  -import-path ../otel-arrow/proto/opentelemetry-proto \
  -proto opentelemetry/proto/logs/v1/logs.proto \
  -proto opentelemetry/proto/collector/logs/v1/logs_service.proto \
  -proto opentelemetry/proto/common/v1/common.proto \
  -proto opentelemetry/proto/resource/v1/resource.proto \
  -d '{
    "resourceLogs": [{
      "scopeLogs": [{
        "logRecords": [{
          "body": {"stringValue": "test"}
        }]
      }]
    }]
  }' \
  localhost:4317 \
  opentelemetry.proto.collector.logs.v1.LogsService/Export

# Option C: Configure your instrumented app to send to localhost:4317
```

## License

Apache 2.0
