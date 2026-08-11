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
FROM otel/opentelemetry-collector-contrib:0.157.0@sha256:f2f01157055a9b2aab9df7118e1f1c9abf345e99b23bc7a2bc791db374a7d0f6

# Network ports
# 4317 - OpenTelemetry gRPC services:
#      - OpenTelemetry Protocol with Apache Arrow
#      - OpenTelemetry Protocol (OTLP)
# 1777 - Profiling support
EXPOSE 4317/tcp 1777/tcp
