# Contributing to the OpenTelemetry Protocol with Apache Arrow project

## Introduction

Welcome to the OpenTelemetry Protocol with Apache Arrow project! :tada:
This repository defines and supports Golang libraries for producing and
consuming telemetry data streams using the OpenTelemetry Protocol with Apache
Arrow.

We value all contributions, whether big or small, and encourage you to join us
in improving this project. If you have questions, don't hesitate to reach out to
the OpenTelemetry community - we're here to help!

## Pre-requisites

To work with this repository, you'll need:

- **Go (Golang):** v1.18 or higher. [Installation
  Guide](https://golang.org/doc/install)
- **Protocol Buffer Compiler (protoc):** Required for regenerating gRPC
  services. [Installation Guide](https://grpc.io/docs/protoc-installation/)
- **protoc-gen-go:** Install using:

  ```shell
  go install google.golang.org/protobuf/cmd/protoc-gen-go@v1.28
  ```

## Local Run/Build

### How to set up and run a local OTel-Arrow collector

See [collector/README.md](./collector/README.md) for instructions on running the
examples. See [collector/BUILDING.md](./collector/BUILDING.md) for instructions
on building a collector from local sources.

## Testing

-How to run the test suite for the repository. (TBD)

-Explanation of different types of tests (e.g., unit, integration, or
functional). (TBD)

-Tools and frameworks used for testing (TBD)

-How to interpret test results and resolve common test failures. (TBD)

-Mention code coverage expectations or reporting tools if applicable. (TBD)

## Contribution Guidelines

This guide outlines best practices and requirements to ensure a smooth and
effective contribution process.

### Adhering to Coding Standards

All contributions must align with the project's coding standards and guidelines.
These standards ensure consistency, readability, and maintainability of the
codebase. Please review the following:

- Use clear and concise naming conventions for variables, functions, and files.
- Run linters or formatters where applicable to maintain consistent code style.

### Writing Meaningful Commit Messages

Commit messages are essential for understanding the history and context of
changes. Follow these tips for writing effective commit messages:

- Use the conventional commit format:

Examples of `<type>`: `feat` (new feature), `fix` (bug fix), `docs`
(documentation updates), `test` (test-related updates), etc.

- Include a brief description of what and why, avoiding overly technical jargon.
- Use present-tense verbs, e.g., "Add" instead of "Added."

### Including Tests for New Features or Bug Fixes

Testing is crucial to ensure code reliability. When contributing:

- Write unit tests for new features and bug fixes.
- Run the test suite before submitting a pull request to verify changes. -Ensure
test coverage remains high, and add tests for edge cases when applicable.

### Community Standards and Style Guides

This project adheres to the OpenTelemetry community's standards. Please ensure
you:

Follow the [Code of
Conduct](https://github.com/open-telemetry/community/blob/main/code-of-conduct.md).

Align with any specific style guides, such as [Go Style
Guide.](https://google.github.io/styleguide/go/)

### Contributor License Agreement (CLA)

Before contributing, you may need to sign a Contributor License Agreement (CLA).
This ensures that the community can freely use your contributions.

Instructions for signing the CLA:

Visit [OpenTelemetry CLA
Portal](https://docs.linuxfoundation.org/lfx/easycla/contributors) and follow
the steps to sign electronically.

## Further Help

If you have any questions or run into issues:

Join the OpenTelemetry
[Slack](https://cloud-native.slack.com/archives/C07S4Q67LTF) Community.

## Our Development Process

### Repository background

The OpenTelemetry Protocol with Apache Arrow project was initially developed in
the `github.com/f5/otel-arrow-adapter` repository.  At the time of the
[OpenTelemetry donation][DONATION], this repository was a construction of
original code and code copied from the [OpenTelemetry Protocol with Apache Arrow
Collector][OACGH], which is a fork of the [OpenTelemetry Collector][OTCGH], as
part of [our development process][DEVPROCESS].

### Source locations

This repository contains the OpenTelemetry Protocol with Apache Arrow definition
and Golang libraries for producing and consuming streams of data in this format.

Exporter and receiver components for the [OpenTelemetry Collector][OTCDOCS] were
developed in parallel, maintained in this repository through release v0.24.0,
and now they are included in release of the OpenTelemetry Collector-Contrib
repository.

- [Exporter][EXPORTER]: Send telemetry data using OpenTelemetry Protocol with
      Apache Arrow
- [Receiver][RECEIVER]: Receive telemetry data using OpenTelemetry Protocol with
      Apache Arrow.

Historically, the exporter and receiver components were forked from the
Collector's core [OTLP Exporter][OTLPEXPORTER] and [OTLP
Receiver][OTLPRECEIVER], and the original branch history is now archived in the
[OpenTelemetry Protocol with Apache Arrow Collector][OACGH] repository.

### How to change the protobuf specification

To (re)generate the ArrowStreamService gRPC service, you need to install the
`protoc` compiler and the `protoc-gen-grpc` plugin.

```shell
go install google.golang.org/protobuf/cmd/protoc-gen-go@v1.28
go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@v1.2
export PATH="$PATH:$(go env GOPATH)/bin"
./proto/generate.sh
```

Once the `*.pb.go` files are generated, you need to replace the content of the
`api/collector/arrow/v1` directory by the generated files present in the
`./proto/api/collector/arrow/v1` directory.

### Releasing this repository

See the instructions in [RELEASING.md][].

### Local development issues

This repository contains a top-level `go.work` file.  This enables the Go
modules defined here to avoid relative replace statements, which interfere with
the ability to run them via simple `go install` and `go run` commands.  The
`go.work` file names all the module definitions inside this repository and
allows them all to be used at once during local development.

### Upgrading OpenTelemetry Collector dependencies

When a new version of the OpenTelemetry collector, is available, the easiest way
to upgrade this repository is:

1. Update the `distribution::otelcol_version` field in `otelarrowcol-build.yaml`
2. Modify any components from the core or contrib repositories to use the
   corresponding versions (e.g., pprofextension's module version should match
   the new collector release).
3. Regenerate `otelarrowcol` via `make genotelarrowcol`
4. Run `go work sync` to update the other modules with fresh dependencies.

## OpenTelemetry-Arrow Team

It takes a team to keep a repository like this functioning.  We use
the OpenTelemetry the [Maintainer][MAINTAINERROLE] and
[Approver][APPROVERROLE] roles to organize our work.

The current [OpenTelemetry-Arrow maintainers
(@open-telemetry/arrow-maintainers)][MAINTAINERS] are:

- [Laurent Qu&#xE9;rel](https://github.com/lquerel), F5
- [Joshua MacDonald](https://github.com/jmacd), Microsoft
- [Drew Relmas](https://github.com/drewrelmas), Microsoft

The current [OpenTelemetry-Arrow approvers
(@open-telemetry/arrow-approvers)][APPROVERS] are:

- [Lei Huang](https://github.com/v0y4g3r), Greptime
- [Albert Lockett](https://github.com/albertlockett), F5

The people who filled these roles in the past:

- [Moh Osman](https://github.com/moh-osman3)
- [Alex Boten](https://github.com/codeboten)

Thanks to all the contributors!

[![OpenTelemetry-Arrow contributors](https://contributors-img.web.app/image?repo=open-telemetry/otel-arrow)](https://github.com/open-telemetry/otel-arrow/graphs/contributors)

[RELEASING.md]: ./RELEASING.md
[OTCDOCS]: https://opentelemetry.io/docs/collector/
[OTCGH]: https://github.com/open-telemetry/opentelemetry-collector
[OACGH]: https://github.com/open-telemetry/otel-arrow-collector
[EXPORTER]:
    https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/exporter/otelarrowexporter/README.md
[RECEIVER]:
    https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/receiver/otelarrowreceiver/README.md
[DONATION]: https://github.com/open-telemetry/community/issues/1332
[DEVPROCESS]: https://github.com/open-telemetry/otel-arrow-collector/issues/48
[OTLPRECEIVER]:
    https://github.com/open-telemetry/opentelemetry-collector/receiver/otlpreceiver
[OTLPEXPORTER]:
    https://github.com/open-telemetry/opentelemetry-collector/exporter/otlpexporter
[APPROVERS]: https://github.com/orgs/open-telemetry/teams/arrow-approvers
[MAINTAINERS]: https://github.com/orgs/open-telemetry/teams/arrow-maintainers
[MAINTAINERROLE]: https://github.com/open-telemetry/community/blob/main/guides/contributor/membership.md#maintainer
[APPROVERROLE]: https://github.com/open-telemetry/community/blob/main/guides/contributor/membership.md#approver
