# Copyright The OpenTelemetry Authors
# SPDX-License-Identifier: Apache-2.0

FROM golang:1.21
WORKDIR /main
COPY . .
RUN go mod download
RUN go build -o printer printer.go

# This is hard-coded, for use with otlphttp exporters
EXPOSE 8101/tcp

ENTRYPOINT ["/main/printer"]
