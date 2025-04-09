# Example: OpenTelemetry Protocol with Apache Arrow bridge

This example demonstrates the most basic setup for sending and receiving
OpenTelemetry Protocol with Apache Arrow data.

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

Prometheus metrics describing the OpenTelemetry Protocol with Apache Arrow
components are available at `127.0.0.1:8888` and `127.0.0.1:8889`.
