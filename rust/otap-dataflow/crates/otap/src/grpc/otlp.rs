// SPDX-License-Identifier: Apache-2.0
// Copyright The OpenTelemetry Authors

//! crate containing GRPC Server implementations for the OTLP services that
//! convert the received OTLP signals into OTAP

pub mod client;

pub mod server;

const LOGS_SERVICE_NAME: &str = "opentelemetry.proto.collector.logs.v1.LogsService";
const LOGS_SERVICE_EXPORT_PATH: &str = "/opentelemetry.proto.collector.logs.v1.LogsService/Export";
const METRICS_SERVICE_NAME: &str = "opentelemetry.proto.collector.metrics.v1.MetricsService";
const METRICS_SERVICE_EXPORT_PATH: &str =
    "/opentelemetry.proto.collector.metrics.v1.MetricsService/Export";
const TRACE_SERVICE_NAME: &str = "opentelemetry.proto.collector.trace.v1.TraceService";
const TRACE_SERVICE_EXPORT_PATH: &str =
    "/opentelemetry.proto.collector.trace.v1.TraceService/Export";
