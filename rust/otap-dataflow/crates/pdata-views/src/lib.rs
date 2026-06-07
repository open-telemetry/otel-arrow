// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Zero-dependency view traits for OTLP/OTAP telemetry data.
//!
//! This crate provides backend-agnostic trait definitions for traversing
//! hierarchical OTLP data structures without any external dependencies.

/// A 128-bit trace identifier, represented as a 16-byte array.
pub type TraceId = [u8; 16];

/// A 64-bit span identifier, represented as an 8-byte array.
pub type SpanId = [u8; 8];

pub mod views;
