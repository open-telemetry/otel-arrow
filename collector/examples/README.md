# OpenTelemetry Protocol with Apache Arrow examples

Examples demonstrating how to configure and test an OpenTelemetry
Collector with OpenTelemetry Protocol with Apache Arrow components.

- [`bridge`](./bridge/README.md): A compression bridge between "edge"
  (gateway) and "saas" (reverse gateway) collectors.
- [`metadata-bridge`](./metadata-bridge/README.md): A compression
  bridge between "edge" (gateway) and "saas" (reverse gateway)
  collectors with metadata support, allowing request headers through.
- [`loopback`](./loopback/README.md): A collector that writes Arrow to
  and from itself.
- [`recorder`](./recorder/README.md): A collector with support for
  recording data files for diagnostic and benchmark purposes.
- [`synthesize`](./synthesize/README.md): A collector with support for
  synthesizing telemetry data using a
  [telemetry-generator](https://github.com/lightstep/telemetry-generator)
  component.
