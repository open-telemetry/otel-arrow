# Building the components in this repository

There are several options for building the components in this repository.  This
is a Golang code base, so users familiar with building and installing Golang
will find this easy to do using Golang tools.  Docker build instructions are
also provided.

The recommended practice for building the OpenTelemetry Collector uses a
[`builder` tool][BUILDER].  The builder synthesizes a Golang `go.mod` file and
main package with a specific set of components.  If you are trying to modify and
build code in this repository, see [CONTRIBUTING.md][], otherwise the
instructions here will help you simply build the code in order to try it out.

## Building a collector from local sources using an installed Golang toolchain

The `./cmd/otelarrowcol` directory contains a pre-generated collector build with
a set of demonstration components detailed in
[`./cmd/otelarrowcol/build-config.yaml`][BUILDCONFIG].

Users who have a recent Golang toolchain installed and wish to build a collector
for the host operating system can simply run `make otelarrowcol` at the top
level of this repository.  The executable is placed in `./bin/otelarrowcol`.
You will be able to run the [examples][EXAMPLES] using the resulting artifact.

```shell
make otelarrowcol
./bin/otelarrowcol_darwin_arm64 --config collector/examples/bridge/saas-collector.yaml
```

You can also build and run directly from the directory containing the main file,
e.g.,

```shell
go run ./collector/cmd/otelarrowcol --config collector/examples/bridge/edge-collector.yaml
```

Note that the `go.work` file in the top-level of this repository enables running
`otelarrowcol` from the top-level directory.

## Installing a collector using remote sources with an installed Golang toolchain

The `go install` command can build and install a collector by downloading the
sources to the latest release itself.

```shell
go install github.com/open-telemetry/otel-arrow/collector/cmd/otelarrowcol
```

This installs `otelarrowcol`, which can be run with, e.g.,

```shell
otelarrowcol --config <path-to-config.yaml>
```

## Building a collector from local sources using Docker

Some vendors and platform providers offer pre-built OpenTelemetry collector
"distros" including compoments they recommend for use.  When your vendor or
platform provider adds support for OpenTelemetry Protocol with Apache Arrow, you
may wish to use the artifacts provided by them.

The top-level `Dockerfile` in this repository demonstrates how to build
`otelarrowcol` as a container image, and it can easily be modified for
integration into a custom build and release pipeline.

With Docker installed, simply run:

```shell
make docker-otelarrowcol
```

You will now be able to run the [examples][EXAMPLES] using the resulting
`otelarrowcol` image.

## Upgrading collector dependencies

The collector's Go dependencies (OpenTelemetry Collector core and contrib
modules) are managed through the `BUILDER_VERSION` variable in the top-level
`Makefile`. To upgrade all collector dependencies to a new version:

1. **Update `BUILDER_VERSION`** in the top-level `Makefile` to the desired
   version:

   ```makefile
   BUILDER_VERSION = v0.147.0
   ```

2. **Regenerate the collector source and tidy modules**:

   ```shell
   make genotelarrowcol
   ```

   This single command will:
   - Install the `builder` tool at the specified version.
   - Update all `go.opentelemetry.io/collector/...` and
     `github.com/open-telemetry/opentelemetry-collector-contrib/...` module
     versions in `collector/otelarrowcol-build.yaml` and `Dockerfile`.
   - Remove and regenerate all files in `collector/cmd/otelarrowcol/`
     (including `go.mod`, `go.sum`, `components.go`, and `main.go`).
   - Run `go mod tidy` on all modules.

[BUILDER]:
    https://github.com/open-telemetry/opentelemetry-collector/blob/main/cmd/builder/README.md
[CONTRIBUTING.md]: ../CONTRIBUTING.md
[EXAMPLES]: ./examples/README.md
[BUILDCONFIG]: ./otelarrowcol-build.yaml
