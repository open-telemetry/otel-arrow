# Example: loopback and experiment processor configuration

This example demonstrates how to configure an OpenTelemetry Protocol
with Apache Arrow protocol "loopback", where one pipeline sends to
another pipeline in the same collector process.

To run,

```
$COLLECTOR --config config.yaml
```
