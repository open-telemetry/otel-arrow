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

# This command generates main.go, go.mod but does not update deps.
RUN builder --skip-compilation --skip-get-modules --config=collector/otelarrowcol-build.yaml

# This build will update the go.mod, using the checked-in go.work file
# in the repository.
RUN go build -o otelarrowcol ./collector/cmd/otelarrowcol

# This build uses an Alpine Linux container.
FROM alpine AS release
COPY --from=sandbox /otel-arrow/otelarrowcol /

# Network ports
# 4317 - OpenTelemetry gRPC services:
#      - OpenTelemetry Protocol with Apache Arrow
#      - OpenTelemetry Protocol (OTLP)
# 1777 - Profiling support
EXPOSE 4317/tcp 1777/tcp

ENTRYPOINT ["/otelarrowcol"]
