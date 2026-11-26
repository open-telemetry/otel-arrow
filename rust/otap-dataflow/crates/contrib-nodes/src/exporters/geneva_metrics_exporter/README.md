# Geneva Metrics Exporter

## Metadata

- Type: `urn:otel:exporter:geneva_metrics`
- Feature gate: `geneva` (shared with `geneva_exporter`)
- Stability: WIP; metrics support is under development

## Overview

The Geneva Metrics Exporter is designed for Microsoft products to send OTLP and
OTAP metrics to one configured Geneva monitoring account.
For logs and traces, use `geneva_exporter`.

## Getting Started

Requires an HTTPS `endpoint`, `monitoring_account`, `metric_namespace`, an explicit
`auth.type`, and a bound `bearer_token_provider` for the target service's audience.

### Token exchange

Exchange the provider's source token for a Geneva publication token and endpoint:

```yaml
type: urn:otel:exporter:geneva_metrics
capabilities:
  bearer_token_provider: azure_identity
config:
  endpoint: https://exchange.example.com
  monitoring_account: example-account
  metric_namespace: example-namespace
  auth:
    type: token_exchange
```

Use your Geneva home-stamp endpoint, with no path, query, or fragment.
The exporter uses the first part of its hostname as `stampId`: for example,
`https://my-stamp.example.com` gives `stampId=my-stamp`.
The [complete sample pipeline](otlp-geneva-metrics.yaml) includes an OTLP receiver
and the Azure Identity Auth extension. Replace its example scope, endpoint,
account and namespace. The scope must match the exchange service's Entra audience.

### Direct bearer

Keep the account and namespace fields, but bind a provider that supplies a token
accepted by the publication endpoint. In the exporter `config`, set:

```yaml
endpoint: https://ingest.example.com/your-publication-path
auth:
  type: bearer
```

Use the full publication URL, including any required query parameters. No token
exchange occurs in this mode.

## Build and run

Copy the sample to `config.yaml` in `rust/otap-dataflow` and fill in your deployment
values. From that directory, run:

```bash
cargo run --release --features geneva,azure-identity-auth -- --config config.yaml --num-cores 1
```

Add `--validate-and-exit` to validate configuration without starting the pipeline.

## Delivery notes

Retries can duplicate metrics. Inputs exceeding `max_mapping_bytes` are
permanently rejected; reduce batch size or increase the limit.

## Testing

From `rust/otap-dataflow`, run:

```bash
cargo test -p otel-arrow-dfe-contrib-nodes --features geneva geneva_metrics_exporter
```

## License

Apache 2.0
