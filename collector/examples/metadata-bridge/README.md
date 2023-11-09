# Example: OpenTelemetry Protocol with Apache Arrow metadata bridge

This example demonstrates how to setup basic authentication and
propagate header context through the bridge.

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

