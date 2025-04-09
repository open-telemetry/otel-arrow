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

## Components included in this repository

Several components were developed to facilitate testing and debugging the
primary OpenTelemetry Protocol with Apache Arrow components.  Most importantly,
these tools can be used to report problematic data to the maintainers.  These
components are:

### For production use

- [`processor/concurrentbatchprocessor`][CONCURRENTBATCHPROCESSOR]: Derived from
  the upstream [batchprocessor][UPSTREAMBATCHPROCESSOR], this component is
  enhanced with the ability to send batches concurrently, with an overall
  in-flight-bytes limit.

### For research and validation

- [`processor/obfuscationprocessor`][OBFUSCATIONPROCESSOR]: Supports obfuscation
  of OpenTelemetry data using a [Feistel
  cipher](https://en.wikipedia.org/wiki/Feistel_cipher).

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

- [otelhttpexporter][UPSTREAMHTTPOTLP]:  Useful for debugging, sends standard
      OTLP over HTTP
- [debugexporter][UPSTREAMDEBUG]:   Useful for debugging, prints OTLP data to
      the console

[BUILDING]: ./BUILDING.md
[COLLECTORCONTRIB]:
    https://github.com/open-telemetry/opentelemetry-collector-contrib
[ARROWEXPORTER]:
    https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/exporter/otelarrowexporter/README.md
[ARROWRECEIVER]:
    https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/receiver/otelarrowreceiver/README.md
[UPSTREAMBATCHPROCESSOR]:
    https://github.com/open-telemetry/opentelemetry-collector/blob/main/processor/batchprocessor/README.md
[CONCURRENTBATCHPROCESSOR]: ./processor/concurrentbatchprocessor/README.md
[OBFUSCATIONPROCESSOR]: ./processor/obfuscationprocessor/README.md
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
[UPSTREAMDEBUG]:
    https://github.com/open-telemetry/opentelemetry-collector/blob/main/exporter/debugexporter/README.md
