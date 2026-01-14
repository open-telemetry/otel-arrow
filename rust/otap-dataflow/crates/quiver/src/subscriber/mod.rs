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
//! 4. If retry is needed, calls `handle.defer()` to release the claim
//! 5. On retry, either:
//!    - Call `next_bundle()` again (deferred bundle will be returned since
//!      it's the oldest unresolved unclaimed bundle), or
//!    - Call `claim_bundle(bundle_ref)` to explicitly re-acquire a specific bundle
//!
//! **Note:** Deferred bundles are automatically rescheduled. When `defer()` is
//! called, the bundle's claim is released and it becomes eligible for redelivery
//! via `next_bundle()`. The embedding layer does NOT need to track deferred
//! bundles separately—Quiver handles this internally.
//!
//! The embedding layer (e.g., otap-dataflow's persistence_processor) may still
//! choose to track deferred bundles for custom retry timing, backoff strategies,
//! or priority scheduling—but this is optional.
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
    SegmentProgressEntry, SubscriberProgress, delete_progress_file, delete_progress_file_sync,
    progress_file_path, read_progress_file, scan_progress_files, write_progress_file,
    write_progress_file_sync,
};
pub use registry::{RegistryCallback, RegistryConfig, SegmentProvider, SubscriberRegistry};
pub use state::SubscriberState;
pub use types::{AckOutcome, BundleIndex, BundleRef, SubscriberId};
