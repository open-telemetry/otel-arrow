// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Subscriber management and progress tracking for Quiver.
//!
//! This module implements the consumer side of Quiver's persistence layer,
//! enabling multiple independent subscribers to consume bundles from segments
//! with durable progress tracking.
//!
//! # Concepts
//!
//! - **Subscriber**: A named consumer that maintains independent progress
//!   through the segment stream. Examples: "exporter-otlp", "backup-s3".
//!
//! - **BundleRef**: A lightweight reference to a specific bundle within a
//!   segment, consisting of `(segment_seq, bundle_index)`.
//!
//! - **AckOutcome**: The terminal outcome of bundle processing—either
//!   successfully acknowledged or permanently dropped.
//!
//! - **Progress File** (`quiver.sub.<id>`): Each subscriber maintains an
//!   independent progress file that snapshots its current state. Progress is
//!   atomically updated via write-fsync-rename.
//!
//! # Architecture
//!
//! Quiver uses a handle-based consumption model:
//!
//! 1. Subscriber calls `next_bundle()` to claim the next pending bundle
//! 2. Receives a `BundleHandle` wrapping the data and resolution methods
//! 3. After processing, calls `handle.ack()` or `handle.reject()`
//! 4. If retry is needed, calls `handle.defer()` and schedules retry
//! 5. On retry, calls `claim_bundle(bundle_ref)` to re-acquire
//!
//! The embedding layer (e.g., otap-dataflow's persistence_processor) handles
//! retry timing, backoff, and dispatch—Quiver only records terminal outcomes.
//!
//! # Module Organization
//!
//! | File          | Purpose                                           |
//! |---------------|---------------------------------------------------|
//! | `types.rs`    | Core type definitions (SubscriberId, BundleRef)   |
//! | `error.rs`    | Subscriber-specific error types                   |
//! | `progress.rs` | Per-subscriber progress file (write/read)         |
//! | `state.rs`    | Per-subscriber in-memory progress tracking        |
//! | `handle.rs`   | RAII bundle handle for consumption                |
//! | `registry.rs` | Subscriber lifecycle management                   |

mod error;
mod handle;
mod progress;
mod registry;
mod state;
mod types;

pub use error::{Result, SubscriberError};
pub use handle::{BundleHandle, ResolutionCallback};
pub use progress::{
    delete_progress_file, progress_file_path, read_progress_file, scan_progress_files,
    write_progress_file, SegmentProgressEntry, SubscriberProgress,
};
pub use registry::{RegistryConfig, SegmentProvider, SubscriberRegistry};
pub use state::SubscriberState;
pub use types::{AckOutcome, BundleIndex, BundleRef, SubscriberId};
