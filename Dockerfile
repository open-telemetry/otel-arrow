# Copyright The OpenTelemetry Authors
# SPDX-License-Identifier: Apache-2.0

# This Dockerfile builds the OpenTelemetry Protocol with Apache Arrow
# Collector from the code in this repository.  It builds using the
# "otelarrowcol" configuration.  See collector/otelarrowcol-build.yaml
# for the components that are included in the build, which are all of
# those with sources in this repository plus a few commonly useful
# accessories (e.g., the profiler extension).
FROM golang:1.21@sha256:4746d26432a9117a5f58e95cb9f954ddf0de128e9d5816886514199316e4a2fb AS sandbox

WORKDIR /otel-arrow
COPY . .
ENV CGO_ENABLED=0

# Note the version MUST MATCH otelarrowcol-build.yaml
RUN go install go.opentelemetry.io/collector/cmd/builder@v0.89.0

# This command generates main.go, go.mod but does not update deps.
RUN builder --skip-compilation --skip-get-modules --config=collector/otelarrowcol-build.yaml

# This build will update the go.mod, using the checked-in go.work file
# in the repository.
RUN go build -o otelarrowcol ./collector/cmd/otelarrowcol

# This build uses an Alpine Linux container.
FROM alpine@sha256:a8560b36e8b8210634f77d9f7f9efd7ffa463e380b75e2e74aff4511df3ef88c AS release
COPY --from=sandbox /otel-arrow/otelarrowcol /

# Network ports
# 4317 - OpenTelemetry gRPC services:
#      - OpenTelemetry Protocol with Apache Arrow
#      - OpenTelemetry Protocol (OTLP)
# 1777 - Profiling support
EXPOSE 4317/tcp 1777/tcp

ENTRYPOINT ["/otelarrowcol"]
