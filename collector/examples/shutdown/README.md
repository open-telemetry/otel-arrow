# Shutdown test example

This example configures three collectors with two Arrow bridges with
different stream lifetimes.  It can be used to experiment with
mismatched lifetime and keepalive settings, too.

To run this setup, start the three collectors as follows:

```
$COLLECTOR --config saas-collector.yaml
$COLLECTOR --config middle-collector.yaml
$COLLECTOR --config edge-collector.yaml
```

You may use the
[`telemetrygen`](https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/cmd/telemetrygen/README.md)
generator to exercise this pipeline.  For example, to send traces:

```
telemetrygen traces --otlp-insecure --duration 1000s
```

To test debug-level logging, change the service configurations, e.g.,:

```
    logs:
      level: debug
```

To test a condition where max-stream-lifetime is too short, lower
keepalive `max_connection_age_grace`, e.g.,

```
        keepalive:
          server_parameters:
            max_connection_age: 5s
            max_connection_age_grace: 10s
```





