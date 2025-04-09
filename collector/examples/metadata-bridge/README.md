# Example: OpenTelemetry Protocol with Apache Arrow metadata bridge

This example demonstrates how to setup basic authentication and propagate header
context through the bridge.

To run the exporting side of the bridge,

```shell
$COLLECTOR --config edge-collector.yaml
```

To run the receiving side of the bridge,

```shell
$COLLECTOR --config saas-collector.yaml
```

You may use the
[`telemetrygen`](https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/cmd/telemetrygen/README.md)
generator to exercise this pipeline.  For example, to send traces:

```shell
telemetrygen traces --otlp-insecure --duration 1000s
```
