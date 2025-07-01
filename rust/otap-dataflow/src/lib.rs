// SPDX-License-Identifier: Apache-2.0

//! OTAP Pipeline Engine Library
//!
//! This crate provides core functionality for the OTAP dataflow system.
//! The Retryable trait is now defined in the separate `otap-df-traits` crate to avoid circular dependencies.

pub mod error;
/// Retry processor implementation for handling failed message delivery with configurable retry logic.
pub mod retry_processor;

// Re-export key items for convenience
pub use retry_processor::{ErrorDetail, RetryConfig, RetryPolicy, RetryProcessor};
// Re-export the Retryable trait from the traits crate for convenience
pub use otap_df_traits::Retryable;
