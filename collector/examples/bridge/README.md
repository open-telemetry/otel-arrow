# Example: OpenTelemetry Protocol with Apache Arrow bridge

This example demonstrates the most basic setup for sending and
receiving OpenTelemetry Protocol with Apache Arrow data.

To run the exporting side of the bridge,

```
$COLLECTOR --config edge-collector.yaml
```

To run the receiving side of the bridge,

```
$COLLECTOR --config saas-collector.yaml
```

You may use the [`generator`](../generator/README.md) example to
produce data and the [`printer`](../printer/README.md) to display data
received on the other end.

Prometheus metrics describing the OpenTelemetry Protocol with Apache
Arrow components are available at `127.0.0.1:8888` and `127.0.0.1:8889`.
