# OpenTelemetry Protocol with Apache Arrow Collector Components

This directory contains the primary collector components for using
OpenTelemetry Protocol with Apache Arrow as well as a number of useful
accessory components that were developed to assist the project.

The primary components are:

- [Exporter][EXPORTER]: for sending OpenTelemetry Protocol with Apache Arrow data
- [Receiver][RECEIVER]: for receiving OpenTelemetry Protocol with Apache Arrow data

## Building and distributing these components

We are aware that building and distributing OpenTelemetry collectors
is not a simple task and have prepared dedicated instructions for
building and testing the components in this repository.

[Instructions for building a OpenTelemetry Collector with support for
OpenTelemetry Protocol with Apache Arrow](./BUILDING.md).

[We would prefer to include these components in the OpenTelemetry
Contrib Collector, because it is an officially maintained artifact.
At this time, however, these components are new and the migration
process will take some time to complete.][CONTRIBUTION]

## Components included in this repository

Several components were developed to facilitate testing and debugging
the primary OpenTelemetry Protocol with Apache Arrow components.  Most
importantly, these tools can be used to report problematic data to the
maintainers.  These components are:

### For production use

- `processor/concurrentbatchprocessor`: Derived from the upstream
  [batchprocessor](https://github.com/open-telemetry/opentelemetry-collector-contrib/tree/main/processor/batchprocessor),
  this component is enhanced with the ability to send batches
  concurrently, with an overall in-flight-bytes limit.

### For research and validation

- `exporter/fileexporter`: Derived from the upstream
  [fileexporter](https://github.com/open-telemetry/opentelemetry-collector-contrib/tree/main/exporter/fileexporter),
  this component supports writing files that can be read by the
  corresponding `filereceiver` in this package (unlike the upstream).
- `receiver/filereceiver`: Derived from the upstream
  [filereceiver](https://github.com/open-telemetry/opentelemetry-collector-contrib/tree/main/receiver/filereceiver),
  this component supports reading files written by the corresponding
  `fileexporter` in this package (unlike the upstream).
- `processor/obfuscationprocessor`: Supports obfuscation of
  OpenTelemetry data using a [Feistel
  cipher](https://en.wikipedia.org/wiki/Feistel_cipher).
- `processor/experimentprocessor`: A probabilistic routing component
  for conducting experiments between exporters.
- `connector/validationconnector`: A component for on-the-fly
  validation of a local pipeline.

## Other extensions built into `otelarrowcol`

Several Collector-Contrib extensions are included in the build:

- [basicauth](https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/extension/basicauthextension/README.md):    Allows use of username and password
- [headersetter](https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/extension/headerssetterextension/README.md): Allows propagating headers through a pipeline
- [pprof](https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/extension/pprofextension/README.md):        Allows use of Golang profiling tools.

From the core collector repository:

- [otelhttpexporter]():  Useful for debugging, sends standard OTLP over HTTP
- [loggingexporter]():   Useful for debugging, prints OTLP data to the console

Also, a synthetic telemetry data generator:

- [generator](https://github.com/lightstep/telemetry-generator/blob/main/README.md): Produces synthetic telemetry data.


[EXPORTER]: ./exporter/otelarrowexporter/README.md
[RECEIVER]: ./receiver/otelarrowreceiver/README.md
[CONTRIBUTION]: https://github.com/open-telemetry/opentelemetry-collector-contrib/issues/26491
