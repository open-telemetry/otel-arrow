# OpenTelemetry Protocol with Apache Arrow examples

Examples demonstrating how to configure and test an OpenTelemetry
Collector with OpenTelemetry Protocol with Apache Arrow components.

To run any of the following examples, first build a collector using
one of the methods document in [BUILDING](../BUILDING.md).

- [`bridge`](./bridge/README.md): A compression bridge between "edge"
  (gateway) and "saas" (reverse gateway) collectors.
- [`metadata-bridge`](./metadata-bridge/README.md): A compression
  bridge between "edge" (gateway) and "saas" (reverse gateway)
  collectors with metadata support, allowing request headers through.
- [`recorder`](./recorder/README.md): A collector with support for
  recording data files for diagnostic and benchmark purposes.
  
Here are some tools used in running these examples:

- [printer](./printer/README.md): Small Golang tool for printing raw
  HTTP request data (including headers).
- [generator](./generator/README.md): Collector example for sending
  synthetic telemetry.

For each example directory, change your the working directory to the
example.  Set a `COLLECTOR` in your shell according to the build
method used.

If you used docker,

```
COLLECTOR=`docker run -v `pwd`:/config -w /config otelarrowcol`
```

if you used an installed Golang toolchain and local sources,

```
COLLECTOR=../../../bin/otelarrowcol
```

and if you used the `go install` method,

```
COLLECTOR=${GOPATH}/bin/otelarrowcol
```
