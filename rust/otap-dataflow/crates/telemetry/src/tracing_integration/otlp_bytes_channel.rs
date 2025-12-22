// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OTLP bytes channel abstraction for multi-threaded telemetry.
//!
//! This provides a common pattern used in multiple places:
//! - Admin runtime: 3rd party logging via global tracing subscriber
//! - Internal telemetry receiver: Component logging bridge to OTAP pipeline
//! - Thread-per-core: Per-thread logging with dedicated channels
//!
//! Architecture:
//! ```text
//! Producer(s) → mpsc::Sender<Bytes> → Channel → mpsc::Receiver<Bytes> → Consumer
//!                                                                             ↓
//!                                                      Console | OTLP | Custom handler
//! ```

use bytes::Bytes;
use std::sync::mpsc;

/// Configuration for how to consume OTLP bytes from the channel.
///
/// All 3rd party logging goes through our custom subscriber → OTLP bytes → channel.
/// This enum determines how those bytes are consumed in the admin runtime:
///
/// - **Console**: Human-readable formatting (our builtin formatter)
/// - **InternalReceiver**: Forward to OTAP pipeline (our builtin OTLP path)
/// - **OtelSdkExporter**: Use any OpenTelemetry SDK exporter (stdout, OTLP, custom)
///
/// This unified architecture means:
/// 1. ALL 3rd party logs use the same channel-based path
/// 2. No need for OpenTelemetryTracingBridge (we decode OTLP → OTel format if needed)
/// 3. Flexible backend choice while keeping single-threaded consumption
#[derive(Debug, Clone)]
pub enum OtlpBytesConsumerConfig {
    /// Format and write to console (stdout/stderr based on level).
    /// Uses our builtin formatter for human-readable output.
    Console {
        /// Enable ANSI color codes
        ansi: bool,
        /// Include ISO8601 timestamps
        timestamp: bool,
        /// Include log level (INFO, WARN, etc.)
        level: bool,
        /// Include target/scope name
        target: bool,
        /// Include event name field
        event_name: bool,
        /// Include thread names
        thread_names: bool,
    },
    
    /// Forward to internal telemetry receiver (bridges to OTAP pipeline).
    /// Uses our builtin OTLP exporter to send to the internal receiver,
    /// which then goes through the OTAP pipeline for processing/export.
    InternalReceiver {
        // Future: configuration for the internal receiver
    },
    
    /// Use an OpenTelemetry SDK exporter.
    /// OTLP bytes are decoded to OpenTelemetry LogData and passed to the SDK exporter.
    /// This allows using any OTel SDK exporter (stdout, OTLP, custom) while keeping
    /// our unified channel-based architecture.
    OtelSdkExporter {
        /// Exporter type identifier (e.g., "stdout", "otlp-grpc", "otlp-http")
        exporter_type: String,
        /// Configuration for the specific exporter (JSON or similar)
        config: std::collections::HashMap<String, String>,
    },
}

impl OtlpBytesConsumerConfig {
    /// Create default console configuration (matches current behavior)
    pub fn default_console() -> Self {
        Self::Console {
            ansi: true,
            timestamp: true,
            level: true,
            target: true,
            event_name: false,
            thread_names: true,
        }
    }
}

/// OTLP bytes channel for single-producer, single-consumer telemetry.
///
/// This encapsulates the mpsc channel pattern used throughout the telemetry system.
/// Multiple producers can share the sender (wrapped in Arc), but there's typically
/// one consumer task per channel.
pub struct OtlpBytesChannel {
    sender: mpsc::SyncSender<Bytes>,
    receiver: mpsc::Receiver<Bytes>,
}

impl OtlpBytesChannel {
    /// Create a new OTLP bytes channel with bounded capacity.
    ///
    /// # Arguments
    /// * `capacity` - Maximum number of OTLP byte buffers to queue
    ///
    /// When the channel is full, senders will block until space is available.
    /// This provides backpressure.
    pub fn new(capacity: usize) -> Self {
        let (sender, receiver) = mpsc::sync_channel(capacity);
        Self { sender, receiver }
    }
    
    /// Split into sender and receiver parts.
    ///
    /// The sender can be cloned and shared across multiple producers.
    /// The receiver should be moved to a single consumer task.
    pub fn split(self) -> (mpsc::SyncSender<Bytes>, mpsc::Receiver<Bytes>) {
        (self.sender, self.receiver)
    }
    
    /// Get a reference to the sender (for cloning).
    pub fn sender(&self) -> &mpsc::SyncSender<Bytes> {
        &self.sender
    }
    
    /// Take the receiver (consumes self).
    pub fn into_receiver(self) -> mpsc::Receiver<Bytes> {
        self.receiver
    }
}

/// Statistics about OTLP bytes channel consumption.
#[derive(Debug, Default, Clone)]
pub struct OtlpBytesChannelStats {
    /// Total number of OTLP byte buffers received
    pub buffers_received: u64,
    
    /// Total bytes processed
    pub bytes_processed: u64,
    
    /// Number of format/forward errors
    pub errors: u64,
}

impl OtlpBytesChannelStats {
    /// Create new statistics tracker.
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Record a successfully processed buffer.
    pub fn record_buffer(&mut self, size: usize) {
        self.buffers_received += 1;
        self.bytes_processed += size as u64;
    }
    
    /// Record an error during processing.
    pub fn record_error(&mut self) {
        self.errors += 1;
    }
}
