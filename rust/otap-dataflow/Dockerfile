# Copyright The OpenTelemetry Authors
# SPDX-License-Identifier: Apache-2.0

# This Dockerfile builds the otap-dataflow engine from the source in this
# directory. It relies on protobuf files in the root /proto directory.
# In order to include these in the build context, and to avoid conflicting
# with the arrow enabled go collector build files in the repo root, the image
# must be built with additional --build-context flags as follows:
# docker build --build-context otel-arrow=../../ -f Dockerfile -t df_engine .
ARG FEATURES=""
ARG RUSTFLAGS=""
FROM --platform=$BUILDPLATFORM docker.io/rust:1.91@sha256:4a29b0db5c961cd530f39276ece3eb6e66925b59599324c8c19723b72a423615 AS dataflow-build
WORKDIR /build/rust/dataflow
ARG BUILDPLATFORM
ARG TARGETPLATFORM
ARG FEATURES

RUN apt-get update && apt-get install -y protobuf-compiler

# The build process relies on these libraries outside of the current context.
# Build command from the otap-dataflow directory:
# docker build --build-context otel-arrow=../../ -f Dockerfile -t df_engine .
COPY --from=otel-arrow /proto/opentelemetry/proto /build/proto/opentelemetry/proto
COPY --from=otel-arrow /proto/opentelemetry-proto /build/proto/opentelemetry-proto

COPY . /build/rust/dataflow/.

# Build dataflow engine
ENV FEATURES=${FEATURES}
ENV RUSTFLAGS=${RUSTFLAGS}
RUN ./cross-arch-build.sh

# The runtime image
FROM docker.io/alpine:3.22@sha256:4b7ce07002c69e8f3d704a9c5d6fd3053be500b7f1c69fc0d80990c2ad8dd412
LABEL maintainer="The OpenTelemetry Authors"

ARG DEBUG="0"
RUN if [ "$DEBUG" = "1" ]; then apk add --no-cache gdb; fi

RUN addgroup dataflow \
  && adduser \
  --ingroup dataflow \
  --disabled-password \
  dataflow
WORKDIR /home/dataflow

COPY --from=dataflow-build --chown=dataflow:dataflow /build/rust/dataflow/df_engine .
USER dataflow
ENTRYPOINT ["/home/dataflow/df_engine"]
