#!/usr/bin/env bash

# Run this in the top-level directory.
rm -rf api
mkdir api

# Get current directory.
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

for dir in $(find ${DIR}/opentelemetry -name '*.proto' -print0 | xargs -0 -n1 dirname | sort | uniq); do
  files=$(find "${dir}" -name '*.proto')

  # Generate all files with protoc-gen-go.
  echo ${files}
  protoc -I ${DIR} --go_out=api --go-grpc_out=api ${files}
done

# mv api/github.com/open-telemetry/otel-arrow/api/experimental api
# rm -rf api/github.com

# # Generate the mock files
# go install go.uber.org/mock/mockgen@latest

# mkdir -p api/experimental/arrow/v1/mock
# mockgen -package mock github.com/open-telemetry/otel-arrow/api/experimental/arrow/v1 ArrowTracesServiceClient,ArrowTracesService_ArrowTracesClient,ArrowTracesServiceServer,ArrowTracesService_ArrowTracesServer,ArrowLogsServiceClient,ArrowLogsService_ArrowLogsClient,ArrowLogsServiceServer,ArrowLogsService_ArrowLogsServer,ArrowMetricsServiceClient,ArrowMetricsService_ArrowMetricsClient,ArrowMetricsServiceServer,ArrowMetricsService_ArrowMetricsServer > api/experimental/arrow/v1/mock/arrow_service_mock.go
# go mod tidy
