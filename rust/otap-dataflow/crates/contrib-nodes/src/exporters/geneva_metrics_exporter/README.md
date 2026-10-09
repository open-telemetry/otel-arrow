# Geneva Metrics Exporter

## Metadata

- Type: Exporter
- Feature gate: `geneva` (shared with `geneva_exporter`)
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
`bearer_token_provider`. Providers backed by Azure managed identity are
supported, as are other bearer providers configured for the Geneva publication
resource. The endpoint must use HTTPS.

The `geneva` feature does not include a bearer token provider. To use Azure
managed identity, also enable the `azure_identity_auth` extension, for example
`--features geneva,azure-identity-auth`.

The exporter publishes to the single monitoring account configured for the
component instance; it does not route metrics to different accounts per
request.

## Testing

Run the current Geneva metrics tests with:

```bash
cargo test --manifest-path rust/otap-dataflow/Cargo.toml \
  -p otel-arrow-dfe-contrib-nodes \
  --features geneva \
  geneva_metrics_exporter
```

## License

Apache 2.0
