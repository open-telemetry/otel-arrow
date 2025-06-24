// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! # Backend-Agnostic View traits for OTLP Data traversal.
//!
//! This module defines a unified trait system for traversing hierarchical OTLP data structures
//! in a backend-agnostic manner.The traits provide zero-copy abstractions that work seamlessly
//! across different data representations without requiring data copying or conversion
//!
//! The view system follows the visitor pattern and uses associated types with lifetime parameters
//! to enable efficient traversal of nested log structures. Each trait represents a level in the
//! OpenTelemetry-like hierarchy:
//!
//! ```text
//! LogsView
//! └── ResourceLogsView (groups by resource/host)
//!     └── ScopeLogsView (groups by application/service scope)
//!         └── LogRecordView (individual log entries)
//!             └── AttributeView (key-value metadata)
//!                 └── AnyValueView (typed values)
//! ```
//!
//! ## Supported Backends
//! - **Struct Backend**: Native Rust structs with owned data
//!
//! ## Supported Backends Roadmap
//! - **OTLP Bytes Backend**: serialized otlp bytes representation
//! - **JSON Backend**: serde_json::Value for dynamic JSON processing
//! - **SYSLOG Backend**: Zero-allocation parsing of syslog/CEF strings

pub mod otlp;
pub mod views;
