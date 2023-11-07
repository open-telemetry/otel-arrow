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

### Local development issues

The network of dependencies involved in building OpenTelemetry
Collector images has at times pushed the `go mod` toolchain to its
limits.  While we would like to recommend the `go work` tool for local
development, there are currently unresolvable dependency problems that
happen as a result of this.

The traditional solutiuon to multi-module repositories before `go
work` was introduced is the Go module `replace` statement, which
allows mapping inter-repository dependencies to local directory paths,
allowing you to build and test an OpenTelemetry collector with
locally-modified sources.

While the use of replace statements works to enable local development,
it prevents running code directly from the repository, which raises a
barrier to entry.  To work around this problem, the checked-in
contents of `./collector/cmd/otelarrowcol/go.mod` must not contain
`replace` statements.  To build an `otelarrowcol` from locally
modified sources requires uncommenting the `replaces` directive in
`./collector/cmd/otelarrowcol/build-config.yaml` and re-running `make
genotelarrowcol otelarrowcol`.

[OTCDOCS]: https://opentelemetry.io/docs/collector/
[OTCGH]: https://github.com/open-telemetry/opentelemetry-collector
[OACGH]: https://github.com/open-telemetry/otel-arrow-collector
[EXPORTER]: https://github.com/open-telemetry/otel-arrow/collector/exporter/otelarrowexporter
[RECEIVER]: https://github.com/open-telemetry/otel-arrow/collector/receiver/otelarrowreceiver
[DONATION]: https://github.com/open-telemetry/community/issues/1332
[DEVPROCESS]: https://github.com/open-telemetry/otel-arrow-collector/issues/48
[OTLPRECEIVER]: https://github.com/open-telemetry/opentelemetry-collector/receiver/otlpreceiver
[OTLPEXPORTER]: https://github.com/open-telemetry/opentelemetry-collector/exporter/otlpexporter
