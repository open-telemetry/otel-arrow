# Geneva Metrics Exporter

## Metadata

- Type: Not registered
- Feature gate: `geneva-metrics-exporter`
- Stability: WIP; metrics support is under development

## Overview

The Geneva Metrics Exporter is designed for Microsoft products to send OTLP
metrics to the Geneva monitoring backend. It maps OTLP metrics to the Geneva
metric model, encodes Geneva metrics ingestion protocol and publishes them to Geneva.

The exporter is separate from `geneva_exporter`, which publishes logs and
traces through a different Geneva protocol and client.

The current implementation contains the protocol model, encoder, and
compatibility fixtures. OTLP mapping, publication, authentication, and runtime
configuration are introduced by follow-up changes.

## Testing

Run the current Geneva metrics tests with:

```bash
cargo test --manifest-path rust/otap-dataflow/Cargo.toml \
  -p otap-df-contrib-nodes \
  --features geneva-metrics-exporter \
  geneva_metrics_exporter
```

A runtime YAML test configuration is not included because the exporter is not
registered on this branch.

## License

Apache 2.0
