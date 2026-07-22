# Copyright The OpenTelemetry Authors
# SPDX-License-Identifier: Apache-2.0

# This Dockerfile builds the otap-dataflow engine from the source in this
# directory. It relies on protobuf files in the root /proto directory.
# In order to include these in the build context, and to avoid conflicting
# with the arrow enabled go collector build files in the repo root, the image
# must be built with additional --build-context flags as follows:
# docker build --build-context otel-arrow=../../ -f Dockerfile -t df_engine .
FROM --platform=$BUILDPLATFORM docker.io/rust:1.97@sha256:44637ff22d0a6571a221bfaf137849711ad02ff4723dbb4736e297538f6a3e60 AS dataflow-build
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
COPY --from=otel-arrow /THIRD_PARTY_NOTICES.txt /build/THIRD_PARTY_NOTICES.txt

COPY . /build/rust/dataflow/.

# RUSTFLAGS can be used to pass argument to rustc e.g. 
# --build-arg RUSTFLAGS="-C target-feature=-gfni"
ARG RUSTFLAGS="-C target-cpu=native"
ENV RUSTFLAGS=${RUSTFLAGS}

# These are used to enable features and are passed to cargo's --features argument
# e.g. --build-arg FEATURES="azure"
ARG FEATURES
ENV FEATURES=${FEATURES}

# Build dataflow engine
RUN ./cross-arch-build.sh

# Runtime image
FROM gcr.io/distroless/cc-debian13:nonroot@sha256:d3cda6e91129130d7229a1806b6a73d292ef245ab032da7851907798024cefba
LABEL maintainer="The OpenTelemetry Authors"

WORKDIR /home/nonroot
COPY --from=dataflow-build --chown=nonroot:nonroot /build/rust/dataflow/df_engine .
USER nonroot
ENTRYPOINT ["/home/nonroot/df_engine"]
