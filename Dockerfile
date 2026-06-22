# Copyright The OpenTelemetry Authors
# SPDX-License-Identifier: Apache-2.0

# This image provides an OpenTelemetry Collector with the OpenTelemetry
# Protocol with Apache Arrow (OTAP) components for testing and evaluation.
#
# Rather than building a bespoke collector in this repository, we use the
# upstream OpenTelemetry Collector Contrib distribution, which already ships
# the `otelarrow` receiver and exporter alongside the OTLP components and the
# other accessories exercised by the examples (see collector/BUILDING.md and
# collector/examples/).
#
# This image is consumed in two ways:
#   * directly, as the `otelarrowcol` image used by the pipeline perf tests; and
#   * as the source of the `/otelcol-contrib` binary, which `make otelarrowcol`
#     extracts to `bin/otelarrowcol` for the Rust validation test harness.
#
# The pinned tag and digest are kept up to date automatically by Renovate
# (see .github/renovate.json5, `docker:pinDigests`).
FROM otel/opentelemetry-collector-contrib:0.154.0@sha256:b3079f45e19bdb7326bf49cdddce6cf60dfd865138db39f2733ea48ab17bc4cb

# Network ports
# 4317 - OpenTelemetry gRPC services:
#      - OpenTelemetry Protocol with Apache Arrow
#      - OpenTelemetry Protocol (OTLP)
# 1777 - Profiling support
EXPOSE 4317/tcp 1777/tcp
