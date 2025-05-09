# OpenTelemetry Protocol with Apache Arrow Collector Components

This directory contains the primary collector components for using OpenTelemetry
Protocol with Apache Arrow as well as a number of useful accessory components
that were developed to assist the project.

The primary components (now included in the upstream [Collector-Contrib
repository][COLLECTORCONTRIB]) are:

- [OTel Arrow Exporter][ARROWEXPORTER]: for sending OpenTelemetry Protocol with
      Apache Arrow data
- [OTel Arrow Receiver][ARROWRECEIVER]: for receiving OpenTelemetry Protocol
      with Apache Arrow data

Note that the exporter and receiver components have been included in the
official OpenTelemetry Collector-Contrib release images since v0.105.0.

## Building and distributing these components

See [Building][BUILDING].

## Other components built into `otelarrowcol`

Several Collector-Contrib extensions are included in the build:

- [basicauth][BASICAUTHEXT]: Allows use of username and password for
  authorization.
- [headersetter][HEADERSETTEREXT]: Allows propagating headers through a pipeline
- [pprof][PPROFEXT]: Allows use of Golang profiling tools.
- [fileexporter][FILEEXPORTER]: Writes telemetry data to files on disk.
- [otlpjsonfilereceiver][FILERECEIVER]: Reads telemetry data from JSON files.

Note that while previously this repository had its own local versions of a basic
fileexporter and filereceiver derived from upstream, it is now recommended to
use the upstream [fileexporter][FILEEXPORTER] and
[otlpjsonfilereceiver][FILERECEIVER] instead.

From the core collector repository:

- [otlphttpexporter][UPSTREAMHTTPOTLP]:  Useful for debugging, sends standard
      OTLP over HTTP
- [otlpexporter][UPSTREAMOTLP]:  Useful for testing and validation, the
      core OTLP exporter
- [debugexporter][UPSTREAMDEBUG]:   Useful for debugging, prints OTLP data to
      the console

## Phase 1 components (removed)

During Phase 1 of the project, several components were built that
could not be maintained given the pace of OpenTelemetry Collector
development.  Notable former components:

- **Concurrent Batch Processor**: This component is an improvement on the
  OpenTelemetry core `batchprocessor`. Today the `exporterhelper` has
  built-in support for batching later in the pipeline, which we
  endorse.
- **Obfuscation Processor**: This component applied a Feistel cipher to all
  string fields of the OpenTelemetry data model. This could be revived as
  a Collector-Contrib component.
- **Validation Connector**: This component routed telemetry in two
  ways such that a collector could self-validate an OTel-Arrow
  bridge. It was difficult to make ensure reliable connector ordering
  at startup, required changes in the core Collector libraries.

[BUILDING]: ./BUILDING.md
[COLLECTORCONTRIB]:
    https://github.com/open-telemetry/opentelemetry-collector-contrib
[ARROWEXPORTER]:
    https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/exporter/otelarrowexporter/README.md
[ARROWRECEIVER]:
    https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/receiver/otelarrowreceiver/README.md
[FILEEXPORTER]:
    https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/exporter/fileexporter/README.md
[FILERECEIVER]:
    https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/receiver/otlpjsonfilereceiver/README.md
[BASICAUTHEXT]:
    https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/extension/basicauthextension/README.md
[HEADERSETTEREXT]:
    https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/extension/headerssetterextension/README.md
[PPROFEXT]:
    https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/extension/pprofextension/README.md
[UPSTREAMHTTPOTLP]:
    https://github.com/open-telemetry/opentelemetry-collector/blob/main/exporter/otlphttpexporter/README.md
[UPSTREAMOTLP]:
    https://github.com/open-telemetry/opentelemetry-collector/blob/main/exporter/otlpexporter/README.md
[UPSTREAMDEBUG]:
    https://github.com/open-telemetry/opentelemetry-collector/blob/main/exporter/debugexporter/README.md
