// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Subscriber management and acknowledgment tracking for Quiver.
//!
//! This module implements the consumer side of Quiver's persistence layer,
//! enabling multiple independent subscribers to consume bundles from segments
//! with durable acknowledgment tracking.
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
//! - **quiver.ack**: A durable append-only log recording subscriber outcomes.
//!   Replayed on recovery to rebuild subscriber state.
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
//! | `ack_log.rs`  | Durable ack log writer and reader                 |
//! | `state.rs`    | Per-subscriber progress tracking                  |
//! | `handle.rs`   | RAII bundle handle for consumption                |
//! | `registry.rs` | Subscriber lifecycle management                   |

mod ack_log;
mod error;
mod handle;
mod registry;
mod state;
mod types;

pub use ack_log::{AckLogEntry, AckLogReader, AckLogWriter};
pub use error::SubscriberError;
pub use handle::{BundleHandle, ResolutionCallback};
pub use registry::{SegmentProvider, SubscriberRegistry};
pub use state::SubscriberState;
pub use types::{AckOutcome, BundleIndex, BundleRef, SubscriberId};
