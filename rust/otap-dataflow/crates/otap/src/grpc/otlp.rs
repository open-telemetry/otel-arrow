// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OTLP gRPC clients, services, and shared RPC names.
//!
//! Receivers preserve protobuf bytes; consumers decode them through the pipeline
//! codec service only when native OTAP records are required.

pub mod client;

pub mod server;
/// Typed OTLP services used by fixtures and benchmarks.
pub mod typed;

const LOGS_SERVICE_NAME: &str = "opentelemetry.proto.collector.logs.v1.LogsService";
const LOGS_SERVICE_EXPORT_PATH: &str = "/opentelemetry.proto.collector.logs.v1.LogsService/Export";
const METRICS_SERVICE_NAME: &str = "opentelemetry.proto.collector.metrics.v1.MetricsService";
const METRICS_SERVICE_EXPORT_PATH: &str =
    "/opentelemetry.proto.collector.metrics.v1.MetricsService/Export";
const TRACE_SERVICE_NAME: &str = "opentelemetry.proto.collector.trace.v1.TraceService";
const TRACE_SERVICE_EXPORT_PATH: &str =
    "/opentelemetry.proto.collector.trace.v1.TraceService/Export";
