# Contributing to OTel Arrow

We want to make contributing to this project as easy and transparent
as possible.  Please see the OpenTelemetry
[CONTRIBUTING.md](https://github.com/open-telemetry/community/blob/main/CONTRIBUTING.md)
guidelines for project-wide information, including code of conduct,
and contributor license agreements, copyright notices, and how to
engage with the OpenTelemetry community.

## Our Development Process

### Repository background

The OTel Arrow project was initially developed using the package name
`github.com/f5/otel-arrow-adapter`.  At the time of the [OpenTelemetry
donation](https://github.com/open-telemetry/community/issues/1332),
this repository was an amalgamation of original code and code copied
from the [OTel Arrow
Collector](https://github.com/open-telemetry/otel-arrow-collector) as
part of [our development
process](https://github.com/open-telemetry/otel-arrow-collector/issues/48).

### Source locations

This repository contains the OTel Arrow protocol definition and Golang
libraries for producing and consuming OTel Arrow data.

The OTel Arrow exporter and receiver components for the [OpenTelemetry
Collector](https://github.com/open-telemetry/opentelemetry-collector)
components were developed in parallel with this code base.  They are
maintained as part of the [OTel Arrow
Collector](https://github.com/open-telemetry/otel-arrow-collector)
repository, which contains the branch history that relates the OTel
Arrow exporter and receiver components to the core OTLP
[exporter](https://github.com/open-telemetry/opentelemetry-collector/tree/main/exporter/otlpexporter)
and
[receiver](https://github.com/open-telemetry/opentelemetry-collector/tree/main/receiver/otlpreceiver)
components.  Prior to the donation, these components were copied into
`github.com/f5/otel-arrow-adapter/collector`.  Following the donation,
these components are copied into the [OpenTelemetry Collector
Contrib](https://github.com/open-telemetry/opentelemetry-collector-contrib)
repository.

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

TODO: Update this document with links when the components are
includeded in their first `collector-contrib` release.
