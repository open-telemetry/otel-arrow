# Building the components in this repository

There are several options for building the components in this
repository.  This is a Golang code base, however users familiar with
building and installing Golang code using `go install` commands will
be disappointed.  Due to complications with Go module dependencies,
the process is a bit more elaborate.

The recommended practice for building the OpenTelemetry Collector uses
a [`builder` tool][BUILDER].  The builder synthesizes a Golang
`go.mod` file and main package with a specific set of components.  If
you are trying to modify and build code in this repository, see
[../CONTRIBUTING.md][CONTRIBUTING], otherwise the instructions here
will help you simply build the code in order to try it out.

## Building the demonstration collector from an installed Golang toolchain

The `./cmd/otelarrowcol` directory contains a pre-generated collector
build with a set of demonstration components detailed in
[`./cmd/otelarrowcol/build-config.yaml`][BUILDCONFIG].

Users who already have a recent Golang toolchain installed and wish to
build a collector for the host operating system can simply run `make
otelarrowcol` at the top level of this repository.  The executable is
placed in `./bin/otelarrowcol_$(GOOS)_$(GOARCH)`.  You will be able to
run the [examples][EXAMPLES] using the resulting artifact.

You can also build and run directly from the directory containing the
main file, e.g.,

```
(cd ./cmd/otelarrowcol && go run . --config ../../examples/bridge/edge-collector.yaml)
```

### Building a custom collector using Docker

Many vendors and platform providers offer pre-built OpenTelemetry
collector "distros" including compoments they recommend for use.  When
your vendors or platform provider adds support for OpenTelemetry
Protocol with Apache Arrow, you will instead use the artifacts
provided by them.

Note: these instructions are crafted from [similar instructions
prepared by Google][GCPSAMPLE]; if you are using Google Cloud
Platform, their instructions will be even more helpful than this
guide.



[BUILDER]: https://github.com/open-telemetry/opentelemetry-collector/blob/main/cmd/builder/README.md
[CONTRIBUTING]: ../CONTRIBUTING.md
[EXAMPLES]: ./examples/README.md
[BUILDCONFIG]: ./cmd/otelarrowcol/build-config.yaml
[GCPSAMPLE]: https://github.com/GoogleCloudPlatform/opentelemetry-collector-builder-sample
