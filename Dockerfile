# Copyright The OpenTelemetry Authors
# SPDX-License-Identifier: Apache-2.0

# This Dockerfile builds the OpenTelemetry Protocol with Apache Arrow
# Collector from the code in this repository.  It builds using the
# "otelarrowcol" configuration.  See collector/otelarrowcol-build.yaml
# for the components that are included in the build, which are all of
# those with sources in this repository plus a few commonly useful
# accessories (e.g., the profiler extension).
FROM golang:1.26@sha256:c83e68f3ebb6943a2904fa66348867d108119890a2c6a2e6f07b38d0eb6c25c5 AS sandbox

WORKDIR /otel-arrow
COPY . .
ENV CGO_ENABLED=0

# Note the version should match the builder version referenced in the Makefile.
# The version is overridden when running `make builder`.
RUN go install go.opentelemetry.io/collector/cmd/builder@v0.143.0

# This command generates main.go, go.mod but does not update deps.
RUN builder --skip-compilation --skip-get-modules --config=collector/otelarrowcol-build.yaml

# Build from within the collector module directory where go.mod exists.
WORKDIR /otel-arrow/collector/cmd/otelarrowcol
RUN go mod tidy && go build -o /otel-arrow/otelarrowcol .

# This build uses an Alpine Linux container.
FROM alpine@sha256:25109184c71bdad752c8312a8623239686a9a2071e8825f20acb8f2198c3f659 AS release
COPY --from=sandbox /otel-arrow/otelarrowcol /

# Network ports
# 4317 - OpenTelemetry gRPC services:
#      - OpenTelemetry Protocol with Apache Arrow
#      - OpenTelemetry Protocol (OTLP)
# 1777 - Profiling support
EXPOSE 4317/tcp 1777/tcp

ENTRYPOINT ["/otelarrowcol"]
