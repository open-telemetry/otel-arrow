# Geneva Metrics Exporter

## Metadata

- Type: Exporter
- Feature gate: `geneva-metrics`
- Optional certificate authentication: `geneva-metrics-certificate-auth` (disabled by default)
- Stability: WIP; metrics support is under development

## Overview

The Geneva Metrics Exporter is designed for Microsoft products to send OTLP
metrics to the Geneva monitoring backend. It maps OTLP metrics to the Geneva
metric model, encodes Geneva metrics ingestion protocol and publishes them to Geneva.

The exporter is separate from `geneva_exporter`, which publishes logs and
traces through a different Geneva protocol and client.

The current implementation contains the protocol model, encoder, compatibility
fixtures, Geneva-compatible mapping for OTLP and OTAP metrics views,
authenticated HTTP publication, exporter registration, and runtime
configuration. The registered exporter accepts OTLP metrics payloads.

Password-protected PKCS#12 certificate authentication is excluded by default.
Build with `--features geneva-metrics-certificate-auth` only when certificate
authentication is required.

## Testing

Run the current Geneva metrics tests with:

```bash
cargo test --manifest-path rust/otap-dataflow/Cargo.toml \
  -p otel-arrow-dfe-contrib-nodes \
  --features geneva-metrics \
  geneva_metrics_exporter
```

## License

Apache 2.0
