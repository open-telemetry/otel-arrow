// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Log encoding and formatting for Tokio tracing events.  This module
//! stores pre-calculated encodings for the LogRecord event_name and
//! avoids unnecessary encoding work for primitive fields (e.g., timestamp).
//!
//! The intermediate representation is InternalLogRecord, includes the
//! primitive fields and static references. The remaining data are
//! placed in a partial OTLP encoding.

pub mod encoder;
pub mod formatter;

use bytes::Bytes;
use std::collections::HashMap;
use tracing::callsite::Identifier;

pub use formatter::{ConsoleWriter, Layer as RawLoggingLayer};

/// A log record with structural metadata and pre-encoded body/attributes.
#[derive(Debug, Clone)]
pub struct LogRecord {
    /// Callsite identifier used to look up cached callsite info
    pub callsite_id: Identifier,

    /// Timestamp in UNIX epoch nanoseconds
    pub timestamp_ns: u64,

    /// Severity level, OpenTelemetry defined
    pub severity_level: u8,

    /// Severity text
    pub severity_text: &'static str,

    /// Pre-encoded body and attributes
    pub body_attrs_bytes: Bytes,
}

/// Saved callsite information, populated via `register_callsite` hook.
#[derive(Debug, Clone)]
pub struct SavedCallsite {
    /// Target (e.g., module path)
    pub target: &'static str,

    /// Event name
    pub name: &'static str,

    /// Source file
    pub file: Option<&'static str>,

    /// Source line
    pub line: Option<u32>,
}

/// Map callsite information by `Identifier`.
#[derive(Debug, Default)]
pub struct CallsiteMap {
    callsites: HashMap<Identifier, SavedCallsite>,
}

impl CallsiteMap {
    /// Create a new empty cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a callsite from its metadata.
    pub fn register(&mut self, metadata: &'static tracing::Metadata<'static>) {
        let id = metadata.callsite();
        let _ = self.callsites.entry(id).or_insert_with(|| SavedCallsite {
            target: metadata.target(),
            name: metadata.name(),
            file: metadata.file(),
            line: metadata.line(),
        });
    }

    /// Get cached callsite info by identifier.
    pub fn get(&self, id: &Identifier) -> Option<&SavedCallsite> {
        self.callsites.get(id)
    }
}
