// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! utilities for managing context of inbound and outbound requests
//! produced by transform processor.

use slotmap::Key as _;
use std::num::NonZeroUsize;

use crate::{
    accessory::slots::{Key, State},
    pdata::Context,
};

struct Inbound {
    context: Context,
    error_reason: Option<String>,
    num_outbound: usize,
}

struct Outbound {
    inbound_key: Key,
}

/// Contexts manages the context of inbound and outbound batches. The intent here is to keep enough state
/// that if any batches are split by the pipeline, we can Ack/Nack the inbound batch and when all outbound batches
/// are completed.
///
/// It contains two slot maps:
/// - Inbound: manages how many outbound batches are associated with an inbound batch, as well as
///   the error reason if any occurred (either processing the inbound batch, or any outbound batch).
/// - Outbound: maps the inbound key to the outbound key.
pub(super) struct Contexts {
    inbound: State<Inbound>,
    outbound: State<Outbound>,
}

impl Contexts {
    pub fn new(max_inbound: NonZeroUsize, max_outbound: NonZeroUsize) -> Self {
        Self {
            inbound: State::new(max_inbound.into()),
            outbound: State::new(max_outbound.into()),
        }
    }

    /// Insert an inbound batch into the context.
    ///
    /// If the inbound batch does not need to be managed (no subscribers), it is not inserted into the context
    /// and a null key is returned.
    ///
    /// Returns `None` if the inbound slot map is full.
    ///
    /// # Parameters
    ///
    /// - `context`: The context of the inbound batch.
    /// - `error_reason`: The error may have occurred processing the inbound batch.
    pub fn insert_inbound(
        &mut self,
        context: Context,
        error_reason: Option<String>,
    ) -> Option<Key> {
        if !context.has_subscribers() {
            // no point in managing the inbound/outbound the context if there are no subscribers
            return Some(Key::null());
        }

        let inbound = Inbound {
            context,
            num_outbound: 0,
            error_reason,
        };

        self.inbound.allocate(|| (inbound, ())).map(|(key, _)| key)
    }

    /// Inserts an outbound batch into the context. This will update any necessary state related
    /// to the inbound batch (increment count of outbound batches).
    ///
    /// Returns the key of the outbound batch. IF the inbound batch doesn't exist it will return
    /// a null key. Note: even after calling insert_inbound, the inbound batch may not exist
    /// because we determined from the context it doesn't need to be tracked (e.g. it has no
    /// subscribers).
    ///
    /// Returns `None` if the outbound slot map is full.
    pub fn insert_outbound(&mut self, inbound_key: Key) -> Option<Key> {
        // incr inbound
        if let Some(inbound) = self.inbound.get_mut(inbound_key) {
            inbound.num_outbound += 1;

            // insert outbound
            let outbound = Outbound { inbound_key };
            self.outbound
                .allocate(|| (outbound, ()))
                .map(|(key, _)| key)
        } else {
            Some(Key::null())
        }
    }

    /// Set an error message on the inbound context associated with this outbound key explaining
    /// why the batch processing failed. Note - this method does not clear the outbound
    pub fn set_failed(&mut self, outbound_key: Key, error_reason: String) {
        if let Some(inbound_key) = self.outbound.get(outbound_key).map(|o| o.inbound_key) {
            if let Some(inbound) = self.inbound.get_mut(inbound_key) {
                // keep the original error if it exists
                if inbound.error_reason.is_none() {
                    inbound.error_reason = Some(error_reason)
                }
            }
        }
    }

    /// Clears the outbound slot and returns the context and error reason if the inbound slot is now empty.
    ///
    /// Returns `Some((context, error_reason))` if the inbound slot is now empty. This would mean that
    /// all outbound batches for this inbound slot have been processed and the inbound batch could be
    /// Ack/NAck'd
    pub fn clear_outbound(&mut self, outbound_key: Key) -> Option<(Context, Option<String>)> {
        let inbound_key = {
            let outbound = self.outbound.take(outbound_key)?;
            outbound.inbound_key
        };

        let num_outbound = {
            let inbound = self.inbound.get_mut(inbound_key)?;
            inbound.num_outbound -= 1;
            inbound.num_outbound
        };

        if num_outbound == 0 {
            let inbound = self.inbound.take(inbound_key)?;
            Some((inbound.context, inbound.error_reason))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::num::NonZeroUsize;

    fn new_contexts() -> Contexts {
        Contexts::new(
            NonZeroUsize::new(10).unwrap(),
            NonZeroUsize::new(20).unwrap(),
        )
    }

    // Helper to create a test context with subscribers
    fn create_context_with_subscribers() -> Context {
        let mut ctx = Context::default();
        use otap_df_engine::control::Context8u8;
        ctx.subscribe_to(
            otap_df_engine::Interests::ACKS,
            smallvec::smallvec![Context8u8::from(1u64)],
            1,
        );
        ctx
    }

    // Helper to create a test context without subscribers
    fn create_context_without_subscribers() -> Context {
        Context::default()
    }

    #[test]
    fn test_with_subscribers() {
        let mut contexts = new_contexts();
        let original_context = create_context_with_subscribers();
        let inbound_key = contexts
            .insert_inbound(original_context.clone(), None)
            .unwrap();
        assert!(
            !inbound_key.is_null(),
            "inbound key should not be null when there are subscribers"
        );

        let outbound_key = contexts.insert_outbound(inbound_key).unwrap();
        assert!(
            !outbound_key.is_null(),
            "outbound key should not be null when there are subscribers"
        );

        let (inbound_ctx, error_reason) = contexts.clear_outbound(outbound_key).unwrap();
        assert_eq!(original_context, inbound_ctx);
        assert_eq!(error_reason, None);
    }

    #[test]
    fn test_without_subscribers() {
        let mut contexts = new_contexts();
        let original_context = create_context_without_subscribers();
        let key = contexts
            .insert_inbound(original_context.clone(), None)
            .unwrap();
        assert!(
            key.is_null(),
            "inbound key should be null when there are no subscribers"
        );
        let outbound_key = contexts.insert_outbound(key).unwrap();
        assert!(
            outbound_key.is_null(),
            "outbound key should be null when there are no subscribers"
        );

        assert!(contexts.clear_outbound(outbound_key).is_none());
    }

    #[test]
    fn test_insert_multiple_outbounds() {
        let mut contexts = new_contexts();

        // Insert an inbound
        let original_context = create_context_with_subscribers();
        let inbound_key = contexts
            .insert_inbound(original_context.clone(), None)
            .unwrap();

        // Insert multiple outbounds
        let outbound_key1 = contexts.insert_outbound(inbound_key).unwrap();
        let outbound_key2 = contexts.insert_outbound(inbound_key).unwrap();
        let outbound_key3 = contexts.insert_outbound(inbound_key).unwrap();

        assert!(!outbound_key1.is_null());
        assert!(!outbound_key2.is_null());
        assert!(!outbound_key3.is_null());

        // clear the outbounds
        assert!(contexts.clear_outbound(outbound_key2).is_none());
        assert!(contexts.clear_outbound(outbound_key1).is_none());

        let (inbound_ctx, error_reason) = contexts.clear_outbound(outbound_key3).unwrap();
        assert_eq!(original_context, inbound_ctx);
        assert_eq!(error_reason, None);
    }

    #[test]
    fn test_clear_outbound_with_invalid_key() {
        let mut contexts = new_contexts();

        // Create a key that doesn't exist, just to ensure we handle correctly
        let invalid_key = {
            let mut temp_contexts = new_contexts();
            let ctx = create_context_with_subscribers();
            let inbound_key = temp_contexts.insert_inbound(ctx, None).unwrap();
            temp_contexts.insert_outbound(inbound_key).unwrap()
        };

        let result = contexts.clear_outbound(invalid_key);
        assert!(result.is_none());
    }

    #[test]
    fn test_clear_outbound_returns_error_reason() {
        let mut contexts = new_contexts();

        // Insert inbound with error
        let context = create_context_with_subscribers();
        let error_msg = "pipeline processing failed".to_string();
        let inbound_key = contexts
            .insert_inbound(context, Some(error_msg.clone()))
            .unwrap();
        let outbound_key = contexts.insert_outbound(inbound_key).unwrap();

        // Clear outbound and check error is returned
        let result = contexts.clear_outbound(outbound_key);
        assert!(result.is_some());
        let (_, error_reason) = result.unwrap();
        assert!(error_reason.is_some());
        assert_eq!(error_reason.unwrap(), error_msg);
    }

    #[test]
    fn test_double_clear_same_outbound() {
        let mut contexts = new_contexts();
        let context = create_context_with_subscribers();
        let inbound_key = contexts.insert_inbound(context, None).unwrap();
        let outbound_key = contexts.insert_outbound(inbound_key).unwrap();

        // First clear should succeed
        let result1 = contexts.clear_outbound(outbound_key);
        assert!(result1.is_some());

        // Second clear with same key should fail (key is no longer valid)
        let result2 = contexts.clear_outbound(outbound_key);
        assert!(result2.is_none());
    }

    #[test]
    fn test_set_failed_single_outbound() {
        let mut contexts = new_contexts();
        let context = create_context_with_subscribers();
        let inbound_key = contexts.insert_inbound(context, None).unwrap();
        let outbound_key = contexts.insert_outbound(inbound_key).unwrap();

        // Set the outbound as failed
        let error_msg = "export failed".to_string();
        contexts.set_failed(outbound_key, error_msg.clone());

        // Clear the outbound and verify error is returned
        let result = contexts.clear_outbound(outbound_key);
        assert!(result.is_some());
        let (_, error_reason) = result.unwrap();
        assert!(error_reason.is_some());
        assert_eq!(error_reason.unwrap(), error_msg);
    }

    #[test]
    fn test_set_failed_multiple_outbounds_first_error_wins() {
        let mut contexts = new_contexts();
        let context = create_context_with_subscribers();
        let inbound_key = contexts.insert_inbound(context, None).unwrap();

        let outbound_key1 = contexts.insert_outbound(inbound_key).unwrap();
        let outbound_key2 = contexts.insert_outbound(inbound_key).unwrap();
        let outbound_key3 = contexts.insert_outbound(inbound_key).unwrap();

        // Set first outbound as failed
        let error_msg1 = "first error".to_string();
        contexts.set_failed(outbound_key1, error_msg1.clone());

        // Set second outbound as failed (should be ignored since error_reason is already set)
        let error_msg2 = "second error".to_string();
        contexts.set_failed(outbound_key2, error_msg2.clone());

        // Clear all outbounds
        assert!(contexts.clear_outbound(outbound_key1).is_none());
        assert!(contexts.clear_outbound(outbound_key2).is_none());

        // When clearing the last outbound, the first error should be returned
        let result = contexts.clear_outbound(outbound_key3);
        assert!(result.is_some());
        let (_, error_reason) = result.unwrap();
        assert!(error_reason.is_some());
        assert_eq!(
            error_reason.unwrap(),
            error_msg1,
            "First error should be preserved"
        );
    }

    #[test]
    fn test_set_failed_with_invalid_key() {
        let mut contexts = new_contexts();

        // Create a key that doesn't exist
        let invalid_key = {
            let mut temp_contexts = new_contexts();
            let ctx = create_context_with_subscribers();
            let inbound_key = temp_contexts.insert_inbound(ctx, None).unwrap();
            temp_contexts.insert_outbound(inbound_key).unwrap()
        };

        // Setting failed with invalid key should not panic
        contexts.set_failed(invalid_key, "error".to_string());
    }

    #[test]
    fn test_set_failed_with_null_key() {
        let mut contexts = new_contexts();

        // Create a context without subscribers (results in null key)
        let context = create_context_without_subscribers();
        let inbound_key = contexts.insert_inbound(context, None).unwrap();
        let outbound_key = contexts.insert_outbound(inbound_key).unwrap();

        assert!(outbound_key.is_null());

        // Setting failed with null key should not panic
        contexts.set_failed(outbound_key, "error".to_string());
    }

    #[test]
    fn test_set_failed_does_not_override_inbound_error() {
        let mut contexts = new_contexts();
        let context = create_context_with_subscribers();

        // Insert inbound with an initial error
        let inbound_error = "initial inbound error".to_string();
        let inbound_key = contexts
            .insert_inbound(context, Some(inbound_error.clone()))
            .unwrap();
        let outbound_key = contexts.insert_outbound(inbound_key).unwrap();

        // Try to set a different error via set_failed
        let outbound_error = "outbound error".to_string();
        contexts.set_failed(outbound_key, outbound_error);

        // Clear outbound and verify the original inbound error is preserved
        let result = contexts.clear_outbound(outbound_key);
        assert!(result.is_some());
        let (_, error_reason) = result.unwrap();
        assert!(error_reason.is_some());
        assert_eq!(
            error_reason.unwrap(),
            inbound_error,
            "Original inbound error should be preserved"
        );
    }

    #[test]
    fn test_clear_outbound_removes_outbound_from_slotmap() {
        let mut contexts = new_contexts();
        let context = create_context_with_subscribers();
        let inbound_key = contexts.insert_inbound(context, None).unwrap();

        // Create two outbounds
        let outbound_key1 = contexts.insert_outbound(inbound_key).unwrap();
        let outbound_key2 = contexts.insert_outbound(inbound_key).unwrap();

        // Clear outbound1 once (should decrement counter to 1)
        let result1 = contexts.clear_outbound(outbound_key1);
        assert!(
            result1.is_none(),
            "Should not complete because there's still one outbound"
        );

        // Try to clear outbound1 again - this should fail because the outbound
        // should have been removed from the slotmap on the first clear.
        // If clear_outbound doesn't call self.outbound.take(), this will incorrectly
        // decrement the counter again and potentially complete the inbound prematurely.
        let result2 = contexts.clear_outbound(outbound_key1);
        assert!(
            result2.is_none(),
            "Clearing the same outbound twice should not decrement the counter again"
        );

        // Clear outbound2 - this should complete since we have 1 outbound remaining
        let result3 = contexts.clear_outbound(outbound_key2);
        assert!(
            result3.is_some(),
            "Should complete after clearing the second (and last) outbound"
        );
    }
}
