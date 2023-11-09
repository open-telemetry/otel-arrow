# Copyright The OpenTelemetry Authors
# SPDX-License-Identifier: Apache-2.0

# This Dockerfile builds the OpenTelemetry Protocol with Apache Arrow
# Collector from the code in this repository.  It builds using the
# "otelarrowcol" configuration.  See collector/otelarrowcol-build.yaml
# for the components that are included in the build, which are all of
# those with sources in this repository plus a few commonly useful
# accessories (e.g., the profiler extension).
FROM golang:1.21 AS sandbox

WORKDIR /otel-arrow
COPY . .
ENV CGO_ENABLED=0

# Note we recommend using the latest released builder, which will
# update the core OpenTelemetry collector libraries to the newest
# possible versions.  When the latest set of collector dependencies
# leads to a broken build here, this `latest` can instead be set
# to the last-successful version of the OpenTelemetry collector.
RUN go install go.opentelemetry.io/collector/cmd/builder@latest

# This command generates main.go, go.mod, and then builds using the
# container's go toolchain.  Note the 'exit 0' at the end of this
# command ignores the result of the builder.  See commands in
# Makefile above the `genotelarrow` rule for an explanation.
RUN builder --skip-compilation --config=collector/otelarrowcol-build.yaml; exit 0

# This two-stage build will succeed because there is a `go.work`
# checked-in to the repository.
RUN go install ./collector/cmd/otelarrowcol

# This build uses an Alpine Linux container.
FROM alpine AS release
COPY --from=sandbox /otel-arrow/collector/cmd/otelarrowcol/otelarrowcol /

# Network ports
# 4317 - OpenTelemetry gRPC services:
#      - OpenTelemetry Protocol with Apache Arrow
#      - OpenTelemetry Protocol (OTLP)
# 1777 - Profiling support
EXPOSE 4317/tcp 1777/tcp

CMD ["/otelarrowcol"]
