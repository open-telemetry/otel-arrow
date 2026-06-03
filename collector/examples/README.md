# OpenTelemetry Protocol with Apache Arrow examples

Examples demonstrating how to configure and test an OpenTelemetry Collector with
OpenTelemetry Protocol with Apache Arrow components.

To run any of the following examples, first build a collector using one of the
methods document in [BUILDING](../BUILDING.md).

- [`bridge`](./bridge/README.md): A compression bridge between "edge" (gateway)
  and "saas" (reverse gateway) collectors.
- [`metadata-bridge`](./metadata-bridge/README.md): A compression bridge between
  "edge" (gateway) and "saas" (reverse gateway) collectors with metadata
  support, allowing request headers through.
- [`recorder`](./recorder/README.md): A collector with support for recording
  data files for diagnostic and benchmark purposes.
- [`shutdown`](./shutdown/README.md): Sets up two OTAP bridges with different
  stream lifetimes to exercise gRPC stream failure modes.

For each example directory, change your working directory to the example.
Set a `COLLECTOR` in your shell according to the method used in
[BUILDING](../BUILDING.md).

If you ran the Collector Contrib distribution image directly,

```shell
COLLECTOR="docker run --rm -v `pwd`:/config -w /config otel/opentelemetry-collector-contrib:latest"
```

if you built the `otelarrowcol` image from this repository,

```shell
COLLECTOR="docker run --rm -v `pwd`:/config -w /config otelarrowcol"
```

and if you extracted the collector binary with `make otelarrowcol`,

```shell
COLLECTOR=../../../bin/otelarrowcol
```
