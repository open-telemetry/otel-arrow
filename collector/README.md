# OpenTelemetry Protocol with Apache Arrow Collector Components

This directory contains the primary collector components for using
OpenTelemetry Protocol with Apache Arrow as well as a number of useful
accessory components that were developed to assist the project.

The primary components are:

- [Exporter][EXPORTER]: for sending OpenTelemetry Protocol with Apache Arrow data
- [Receiver][RECEIVER]: for receiving OpenTelemetry Protocol with Apache Arrow data

## Building and distributing these components

The exporter and receiver components are included in the official
OpenTelemetry Collector-Contrib release images since v0.105.0.

## Components included in this repository

Several components were developed to facilitate testing and debugging
the primary OpenTelemetry Protocol with Apache Arrow components.  Most
importantly, these tools can be used to report problematic data to the
maintainers.  These components are:

### For production use

- [`processor/concurrentbatchprocessor`][CONCURRENTBATCHPROCESSOR]:
  Derived from the upstream [batchprocessor][UPSTREAMBATCHPROCESSOR],
  this component is enhanced with the ability to send batches
  concurrently, with an overall in-flight-bytes limit.

### For research and validation

- [`exporter/fileexporter`][ARROWFILEEXPORTER]: Derived from the
  upstream [fileexporter][UPSTREAMFILEEXPORTER], this component
  supports writing files that can be read by the corresponding
  `filereceiver` in this package (which the upstream cannot do).
- [`receiver/filereceiver`][ARROWFILERECEIVER]: Derived from the
  upstream [filereceiver][UPSTREAMFILERECEIVER], this component
  supports reading files written by the corresponding `fileexporter`
  in this package (unlike the upstream).
- [`processor/obfuscationprocessor`][OBFUSCATIONPROCESSOR]: Supports
  obfuscation of OpenTelemetry data using a [Feistel
  cipher](https://en.wikipedia.org/wiki/Feistel_cipher).

## Other components built into `otelarrowcol`

Several Collector-Contrib extensions are included in the build:

- [basicauth][BASICAUTHEXT]: Allows use of username and password for
  authorization.
- [headersetter][HEADERSETTEREXT]: Allows propagating headers through
  a pipeline
- [pprof][PPROFEXT]: Allows use of Golang profiling tools.

From the core collector repository:

- [otelhttpexporter][UPSTREAMHTTPOTLP]:  Useful for debugging, sends standard OTLP over HTTP
- [debugexporter][UPSTREAMDEBUG]:   Useful for debugging, prints OTLP data to the console

[BUILDING]: ./BUILDING.md
[EXPORTER]: ./exporter/otelarrowexporter/README.md
[RECEIVER]: ./receiver/otelarrowreceiver/README.md
[CONTRIBUTION]: https://github.com/open-telemetry/opentelemetry-collector-contrib/issues/26491
[UPSTREAMBATCHPROCESSOR]: https://github.com/open-telemetry/opentelemetry-collector/blob/main/processor/batchprocessor/README.md
[CONCURRENTBATCHPROCESSOR]: ./processor/concurrentbatchprocessor/README.md
[ARROWFILEEXPORTER]: ./exporter/fileexporter/README.md
[ARROWFILERECEIVER]: ./receiver/filereceiver/README.md
[UPSTREAMFILEEXPORTER]: https://github.com/open-telemetry/opentelemetry-collector-contrib/tree/main/exporter/fileexporter/README.md
[UPSTREAMFILERECEIVER]: https://github.com/open-telemetry/opentelemetry-collector-contrib/tree/main/receiver/filereceiver/README.md
[OBFUSCATIONPROCESSOR]: ./processor/obfuscationprocessor/README.md
[BASICAUTHEXT]: https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/extension/basicauthextension/README.md
[HEADERSETTEREXT]: https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/extension/headerssetterextension/README.md
[PPROFEXT]: https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/extension/pprofextension/README.md
[UPSTREAMHTTPOTLP]: https://github.com/open-telemetry/opentelemetry-collector/blob/main/exporter/otlphttpexporter/README.md
[UPSTREAMDEBUG]: https://github.com/open-telemetry/opentelemetry-collector/blob/main/exporter/debugexporter/README.md
[EXAMPLES]: ./examples/README.md
