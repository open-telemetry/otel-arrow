# Building the components in this repository

The OpenTelemetry Protocol with Apache Arrow (OTAP) components -- the `otelarrow`
receiver and exporter -- live in the [OpenTelemetry Collector Contrib
repository][CONTRIB] and ship in the upstream **OpenTelemetry Collector Contrib
distribution**. Rather than generating and checking in a bespoke collector in
this repository, we test and evaluate the OTAP components using that
distribution.

If you are trying to modify and build the Go code in this repository, see
[CONTRIBUTING.md][]; the instructions here help you simply run a collector that
includes the OTAP components.

## Running a collector using the Collector Contrib distribution

The simplest way to try the OTAP components is to run the upstream
Collector Contrib image, which already bundles the `otelarrow` receiver and
exporter alongside the OTLP components and the other accessories used by the
[examples][EXAMPLES] (debug/file exporters, OTLP/JSON-file and syslog receivers,
the filter processor, and the headerssetter/basicauth/pprof extensions):

```shell
docker run --rm -v "$(pwd):/config" -w /config \
  otel/opentelemetry-collector-contrib:latest \
  --config <path-to-config.yaml>
```

Pin a specific released version (recommended) by replacing `latest` with a
version tag, e.g. `otel/opentelemetry-collector-contrib:0.153.0`.

## Building the `otelarrowcol` image from this repository

The top-level `Dockerfile` in this repository is a thin wrapper that pins a
specific Collector Contrib distribution. It is used by the pipeline performance
tests and to obtain a collector binary for the Rust validation tests. Build it
with:

```shell
make docker-otelarrowcol
```

This produces an `otelarrowcol` image equivalent to the pinned upstream
distribution. You can run the [examples][EXAMPLES] against it just like the
upstream image:

```shell
docker run --rm -v "$(pwd):/config" -w /config otelarrowcol \
  --config <path-to-config.yaml>
```

## Obtaining a collector binary

The Rust validation test harness runs a collector as a child process and expects
a binary at `./bin/otelarrowcol`. The `otelarrowcol` make target builds the thin
image and extracts the collector binary from it:

```shell
make otelarrowcol
./bin/otelarrowcol --config collector/examples/bridge/saas-collector.yaml
```

## Upgrading the collector version

The collector version is pinned in the top-level `Dockerfile` via the
`FROM otel/opentelemetry-collector-contrib:<tag>@sha256:<digest>` line. Renovate
keeps the tag and digest up to date automatically (see
[`.github/renovate.json5`](../.github/renovate.json5), which enables
`docker:pinDigests`). To upgrade manually, edit the tag/digest on that line.

[CONTRIB]: https://github.com/open-telemetry/opentelemetry-collector-contrib
[CONTRIBUTING.md]: ../CONTRIBUTING.md
[EXAMPLES]: ./examples/README.md
