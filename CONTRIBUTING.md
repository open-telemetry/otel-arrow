# Contributing to the OpenTelemetry Protocol with Apache Arrow project

We want to make contributing to this project as easy and transparent
as possible.  Please see the OpenTelemetry
[CONTRIBUTING.md](https://github.com/open-telemetry/community/blob/main/CONTRIBUTING.md)
guidelines for project-wide information, including code of conduct,
and contributor license agreements, copyright notices, and how to
engage with the OpenTelemetry community.

## Our Development Process

### Repository background

The OpenTelemetry Protocol with Apache Arrow project was initially
developed in the `github.com/f5/otel-arrow-adapter` repository.  At
the time of the [OpenTelemetry donation][DONATION], this repository
was a construction of original code and code copied from the
[OpenTelemetry Protocol with Apache Arrow Collector][OACGH], which is
a fork of the [OpenTelemetry Collector][OTCGH], as part of [our
development process][DEVPROCESS].

### Source locations

This repository contains the OpenTelemetry Protocol with Apache Arrow
definition and Golang libraries for producing and consuming streams of
data in this format.

Exporter and receiver components for the [OpenTelemetry
Collector][OTCDOCS] were developed in parallel and are currently
maintained in this repository.

- [Exporter][EXPORTER]: Send telemetry data using OpenTelemetry Protocol with Apache Arrow
- [Receiver][RECEIVER]: Receive telemetry data using OpenTelemetry Protocol with Apache Arrow.

Historically, the exporter and receiver components were forked from
the Collector's core [OTLP Exporter][OTLPEXPORTER] and [OTLP
Receiver][OTLPRECEIVER], and the original branch history is now
archived in the [OpenTelemetry Protocol with Apache Arrow
Collector][OACGH] repository.

### How to change the protobuf specification

To (re)generate the ArrowStreamService gRPC service, you need to install the `protoc` compiler and the `protoc-gen-grpc` plugin.
```shell
go install google.golang.org/protobuf/cmd/protoc-gen-go@v1.28
go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@v1.2
export PATH="$PATH:$(go env GOPATH)/bin"
./proto/generate.sh
```
Once the `*.pb.go` files are generated, you need to replace the content of the `api/collector/arrow/v1` directory by the
generated files present in the `./proto/api/collector/arrow/v1` directory.

### Releasing this repository

See the instructions in [RELEASING.md][./RELEASING.md].

### Local development issues

This repository contains a top-level `go.work` file, as an experiment.
This enables the Go modules defined here to avoid relative replace
statements, which interfere with the ability to run them via simple
`go install` and `go run` commands.  The `go.work` file names the
three module definitions inside this repository and allows them all to
be used at once during local development.

### Upgrading OpenTelemetry Collector dependencies

When a new version of the OpenTelemetry collector, is available,
the easiest way to upgrade this repository is:

1. Update the `distribution::otelcol_version` field in `otelarrowcol-build.yaml`
2. Modify any components from the core or contrib repositories to use
   the corresponding versions (e.g., pprofextension's module version
   should match the new collector release).
3. Regenerate `otelarrowcol` via `make genotelarrowcol`
4. Run `go work sync` to update the other modules with fresh dependencies.

[OTCDOCS]: https://opentelemetry.io/docs/collector/
[OTCGH]: https://github.com/open-telemetry/opentelemetry-collector
[OACGH]: https://github.com/open-telemetry/otel-arrow-collector
[EXPORTER]: ./collector/exporter/otelarrowexporter/README.md
[RECEIVER]: ./collector/receiver/otelarrowreceiver/README.md
[DONATION]: https://github.com/open-telemetry/community/issues/1332
[DEVPROCESS]: https://github.com/open-telemetry/otel-arrow-collector/issues/48
[OTLPRECEIVER]: https://github.com/open-telemetry/opentelemetry-collector/receiver/otlpreceiver
[OTLPEXPORTER]: https://github.com/open-telemetry/opentelemetry-collector/exporter/otlpexporter
