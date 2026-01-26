# Copyright The OpenTelemetry Authors
# SPDX-License-Identifier: Apache-2.0

# This Dockerfile builds the otap-dataflow engine from the source in this
# directory. It relies on protobuf files in the root /proto directory.
# In order to include these in the build context, and to avoid conflicting
# with the arrow enabled go collector build files in the repo root, the image
# must be built with additional --build-context flags as follows:
# docker build --build-context otel-arrow=../../ -f Dockerfile -t df_engine .
FROM --platform=$BUILDPLATFORM docker.io/rust:1.93@sha256:4c7eb947d7e078f5c076e086c7b75c36ea0ec7c685f2244b3d79306deb7e44b7 AS dataflow-build
WORKDIR /build/rust/dataflow
ARG BUILDPLATFORM
ARG TARGETPLATFORM

RUN apt-get update && apt-get install -y protobuf-compiler

# The build process relies on these libraries outside of the current context.
# Build command from the otap-dataflow directory:
# docker build --build-context otel-arrow=../../ -f Dockerfile -t df_engine .
COPY --from=otel-arrow /proto/opentelemetry/proto /build/proto/opentelemetry/proto
COPY --from=otel-arrow /proto/opentelemetry-proto /build/proto/opentelemetry-proto
COPY --from=otel-arrow /rust/experimental /build/rust/experimental

COPY . /build/rust/dataflow/.

# RUSTFLAGS can be used to pass argument to rustc e.g. 
# --build-arg RUSTFLAGS="-C target-feature=-gfni"
ARG RUSTFLAGS="-C target-cpu=native"
ENV RUSTFLAGS=${RUSTFLAGS}

# These are used to enable features and are passed to carg's --features argument
# e.g. --build-arg FEATURES="azure,persistence"
ARG FEATURES
ENV FEATURES=${FEATURES}

# Build dataflow engine
RUN ./cross-arch-build.sh

# The runtime image
FROM docker.io/alpine:3.23@sha256:865b95f46d98cf867a156fe4a135ad3fe50d2056aa3f25ed31662dff6da4eb62
LABEL maintainer="The OpenTelemetry Authors"

# Passing --build-arg BUILD=1 adds additional debugging features to the image.
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
