This processor supports randomized routing in an OpenTelemetry
collector processor.  This code's structure is copied from the
collector-contrib
[routingprocessor](https://github.com/open-telemetry/opentelemetry-collector-contrib/tree/main/processor/routingprocessor), but it is substantially simpler. 

The routing table consists of a table of weights with associated list
of exporters (by name).  This method of traffic splitting is modeled
on [Envoy's support for multiple
"upstreams"](https://www.envoyproxy.io/docs/envoy/latest/configuration/http/http_conn_man/traffic_splitting#traffic-splitting-across-multiple-upstreams).

For example, to send 1% of data to an experimental exporter, use the
following configuration.

```
processors:
  experiment:
    table:
    - weight: 1
      exporters: [otlp/experiment]
    - weight: 99
      exporters: [otlp/standard]
```
