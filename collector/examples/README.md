TODO

#### Example collector configurations

Examples demonstrating how to configure and test an OpenTelemetry
Collector with OTel-Arrow exporter and receiver components are located
in `./collector/examples`, including:

- [`examples/bridge`](https://github.com/f5/otel-arrow-adapter/tree/main/collector/examples/bridge):
  A compression bridge between "edge" and "saas" collectors.
- [`examples/metadata-bridge`](https://github.com/f5/otel-arrow-adapter/tree/main/collector/examples/metadata-bridge):
  A compression bridge between "edge" and "saas" collectors with metadata support, allowing request headers to transit via OTel-Arrow.
- [`examples/loopback`](https://github.com/f5/otel-arrow-adapter/tree/main/collector/examples/loopback):
  A collector that writes Arrow to and from itself.
- [`examples/recorder`](https://github.com/f5/otel-arrow-adapter/tree/main/collector/examples/recorder):
  A collector with support for recording data files for diagnostic and benchmark purposes.
- [`examples/synthesize`](https://github.com/f5/otel-arrow-adapter/tree/main/collector/examples/synthesize):
  A collector with support for synthesizing telemetry data using a [telemetry-generator](https://github.com/lightstep/telemetry-generator) component.

2
