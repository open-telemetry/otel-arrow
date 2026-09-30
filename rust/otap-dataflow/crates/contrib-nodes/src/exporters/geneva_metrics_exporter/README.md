# Geneva Metrics Exporter

## Metadata

- Type: Exporter
- Feature gate: `geneva-metrics`
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

The exporter requires `auth.type: bearer` and a bound
`bearer_token_provider` supplied by the Azure Identity extension. The provider
uses managed identity to acquire publication tokens, and the endpoint must use
HTTPS.

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
