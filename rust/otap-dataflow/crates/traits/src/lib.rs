// SPDX-License-Identifier: Apache-2.0

//! Core traits for the OTAP dataflow system.
//!
//! This crate contains the fundamental traits that define interfaces for components
//! in the OTAP (OpenTelemetry Arrow Protocol) dataflow pipeline. By keeping traits
//! in a separate crate, we avoid circular dependencies between implementation crates.
//!
//! ## Architecture Benefits
//!
//! - **Dependency Clarity**: All implementation crates depend on this lightweight trait crate
//! - **No Circular Dependencies**: Traits are defined once and imported where needed
//! - **Clean Separation**: Interface definitions are separated from implementations
//! - **Testability**: Tests can use any implementation of the traits without complex dependencies

#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

use std::time::Instant;

/// Trait for data items that can be retried in the pipeline.
///
/// This trait captures the minimal capabilities needed for retry logic:
/// - Items must be cloneable for retry attempts
/// - Items must be Send for multi-threaded operation  
/// - Items must have a stable 'static lifetime
///
/// ## Design Philosophy
///
/// This trait is intentionally minimal to allow maximum flexibility in implementations.
/// It only requires the essential capabilities needed for retry mechanisms:
///
/// 1. **Unique identification** via `id()` for ACK/NACK correlation
/// 2. **Optional deadline** via `deadline()` for expiration handling
///
/// ## Example Implementation
///
/// ```rust
/// use otap_df_traits::Retryable;
/// use std::time::Instant;
///
/// #[derive(Clone)]
/// struct MyData {
///     id: u64,
///     payload: String,
/// }
///
/// impl Retryable for MyData {
///     fn id(&self) -> u64 {
///         self.id
///     }
///     
///     fn deadline(&self) -> Option<Instant> {
///         None // No deadline for this simple example
///     }
/// }
/// ```
pub trait Retryable: Clone + Send + 'static {
    /// Unique identifier for the item, used for ACK/NACK correlation.
    ///
    /// This ID should be:
    /// - **Deterministic**: Same logical data produces the same ID
    /// - **Stable**: Multiple calls on the same data return the same ID
    /// - **Unique**: Different logical data produces different IDs
    ///
    /// ## Implementation Notes
    ///
    /// - For protobuf-based data, consider hashing the encoded content
    /// - For structured data with natural IDs, use those directly
    /// - For composite data, combine multiple fields into a unique identifier
    fn id(&self) -> u64;

    /// Optional deadline for processing the item.
    ///
    /// If set, the retry logic can use this to:
    /// - Determine if an item has expired and should no longer be retried
    /// - Prioritize items based on urgency
    /// - Implement deadline-aware backoff strategies
    ///
    /// ## Return Value
    ///
    /// - `Some(instant)`: Item should be processed before this time
    /// - `None`: No deadline constraint (item can be retried indefinitely)
    fn deadline(&self) -> Option<Instant>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Simple test data for testing the trait
    #[derive(Clone, Debug)]
    struct TestRetryableData {
        id: u64,
        content: String,
    }

    impl Retryable for TestRetryableData {
        fn id(&self) -> u64 {
            self.id
        }

        fn deadline(&self) -> Option<Instant> {
            None
        }
    }

    #[test]
    fn test_retryable_trait_basics() {
        let data = TestRetryableData {
            id: 12345,
            content: "test content".to_string(),
        };

        // Test that ID generation is deterministic
        assert_eq!(data.id(), 12345);
        assert_eq!(data.id(), data.id());

        // Test deadline
        assert!(data.deadline().is_none());

        // Test that content is preserved (ensuring it's used)
        assert_eq!(data.content, "test content");

        // Test cloning preserves all fields
        let cloned_data = data.clone();
        assert_eq!(cloned_data.id, data.id);
        assert_eq!(cloned_data.content, data.content);

        // Test trait bounds
        fn requires_retryable<T: Retryable>(_: T) {}
        requires_retryable(data);
    }
}
