//! # OTLP Crate
//!
//! This crate provides OpenTelemetry Protocol (OTLP) support for the OTAP dataflow system.
//! It includes functionality for:
//! - gRPC service implementation for OTLP protocol
//! - Batch processing of telemetry data (traces, metrics, logs)
//! - OTLP exporter for sending telemetry data
//! - Mock services for testing all OTLP service types (traces, metrics, logs, and profiles)
//!
//! The crate is designed to handle OpenTelemetry data in a high-performance, reliable manner
//! as part of the OTAP dataflow pipeline.
//!
//! ## Mock Services
//! The `mock` module provides mock implementations for all OTLP services:
//! - `LogsServiceMock`: Mocks the OTLP Logs service
//! - `MetricsServiceMock`: Mocks the OTLP Metrics service
//! - `TraceServiceMock`: Mocks the OTLP Trace service
//! - `ProfilesServiceMock`: Mocks the OTLP Profiles service
//!
//! These mocks are useful for testing OTLP clients and verifying that the correct data is being sent.

#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

/// Simple batch processor implementation
mod simple_generic_batch_processor;

/// gRPC service implementation
pub mod grpc;
/// Implementation of OTLP Receiver that implements the receiver trait
// pub mod otlp_receiver;
/// Generated protobuf files
pub mod proto;

/// otlp exporter implementation
pub mod otlp_exporter;

/// grpc mock server for testing
pub mod mock;

pub use grpc::OTLPData;
