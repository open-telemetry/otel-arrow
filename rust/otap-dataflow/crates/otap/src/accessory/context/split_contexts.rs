// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Utilities for managing context of inbound and outbound requests
//! produced by processors that may split the incoming batch into
//! multiple outbound batches

use otel_arrow_dfe_engine::control::NackCause;
use otel_arrow_dfe_pdata::OtapPayload;
use slotmap::Key as _;
use std::num::NonZeroUsize;

use crate::{
    accessory::slots::{Key, State},
    pdata::Context,
};

/// Context for inbound batch
pub struct Inbound {
    /// the pdata context for the inbound batch
    pub context: Context,

    /// the payload for the inbound batch
    pub payload: Option<OtapPayload>,

    /// error that may have been produced via processing for some outbound batch
    pub error: Option<OutboundError>,

    num_outbound: usize,

    /// Whether all the downstream batches resulted in errors which were transient.
    /// If so, we could emit a Nack for the inbound batch that is transient, otherwise
    /// if there are any errors we must emit a permanent Nack.
    pub outbound_all_transient_errors: bool,
}

/// represents error that may have been produced via processing for some outbound batch
pub struct OutboundError {
    /// reason for the error
    pub reason: String,

    /// identifier of cause of the error
    pub cause: NackCause,
}

struct Outbound {
    inbound_key: Key,
}

/// Manages inbound contexts until every outbound split has completed.
///
/// This preserves both Ack/Nack routing and pipeline metric frames until the
/// inbound batch can be completed.
///
/// It contains two slot maps:
/// - Inbound: manages how many outbound batches are associated with an inbound batch, as well as
///   the error reason if any occurred (either processing the inbound batch, or any outbound batch).
/// - Outbound: maps the inbound key to the outbound key.
pub struct Contexts {
    inbound: State<Inbound>,
    outbound: State<Outbound>,
}

impl Contexts {
    /// Create a new instance of [`Contexts`] with limits on the number of inbound/outbound slots.
    #[must_use]
    pub fn new(max_inbound: NonZeroUsize, max_outbound: NonZeroUsize) -> Self {
        Self {
            inbound: State::new(max_inbound.into()),
            outbound: State::new(max_outbound.into()),
        }
    }

    /// Insert an inbound batch into the context.
    ///
    /// If the inbound batch does not need completion tracking, it is not inserted
    /// into the context and a null key is returned.
    ///
    /// Returns `None` if the inbound slot map is full.
    ///
    /// # Parameters
    ///
    /// - `context`: The context of the inbound batch.
    /// - `payload`: The payload of the inbound batch.
    /// - `error_reason`: The error may have occurred processing the inbound batch.
    pub fn insert_inbound(
        &mut self,
        context: Context,
        payload: Option<OtapPayload>,
        error: Option<OutboundError>,
    ) -> Option<Key> {
        if !context.needs_completion_tracking() {
            // No completion routing or metrics unwinding depends on this context.
            return Some(Key::null());
        }

        let inbound = Inbound {
            context,
            num_outbound: 0,
            payload,
            error,

            // initialize to true, can be set to false if/when there are any outbound batch results
            // that are not a non-permanent Nack
            outbound_all_transient_errors: true,
        };

        self.inbound.allocate(|| (inbound, ())).map(|(key, _)| key)
    }

    /// Inserts an outbound batch into the context. This will update any necessary state related
    /// to the inbound batch (increment count of outbound batches).
    ///
    /// Returns the key of the outbound batch. If the inbound batch doesn't exist,
    /// it returns a null key. Even after calling `insert_inbound`, the inbound
    /// batch may not exist because its context did not require completion
    /// tracking.
    ///
    /// Returns `None` if the outbound slot map is full.
    pub fn insert_outbound(&mut self, inbound_key: Key) -> Option<Key> {
        if let Some(inbound) = self.inbound.get_mut(inbound_key) {
            // try insert outbound while increment inbound's counter if slot is available
            let outbound = Outbound { inbound_key };
            self.outbound
                .allocate(|| {
                    inbound.num_outbound += 1;
                    (outbound, ())
                })
                .map(|(key, _)| key)
        } else {
            Some(Key::null())
        }
    }

    /// Set an error message on the inbound context associated with this outbound key explaining
    /// why the batch processing failed. Note - this method does not clear the outbound
    pub fn set_failed_outbound(&mut self, outbound_key: Key, error: OutboundError) {
        if let Some(inbound_key) = self.outbound.get(outbound_key).map(|o| o.inbound_key) {
            self.set_failed_inbound(inbound_key, error);
        }
    }

    /// Set an error message on the inbound context associated with this key explaining why the
    /// batch processing failed.
    pub fn set_failed_inbound(&mut self, inbound_key: Key, error: OutboundError) {
        if let Some(inbound) = self.inbound.get_mut(inbound_key) {
            // keep the original error if it exists
            if inbound.error.is_none() {
                inbound.error = Some(error)
            }
        }
    }

    /// Clears the inbound slot associated with this key.
    ///
    /// Note - this does not clear any outbound slots referencing this inbound key
    pub fn clear_inbound(&mut self, inbound_key: Key) {
        self.inbound.cancel(inbound_key);
    }

    /// Clears the outbound slot and returns the context and error reason if the inbound slot is now empty.
    ///
    /// Returns `Some((context, error_reason))` if the inbound slot is now empty. This would mean that
    /// all outbound batches for this inbound slot have been processed and the
    /// inbound batch can be completed.
    pub fn clear_outbound(&mut self, outbound_key: Key) -> Option<Inbound> {
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
            Some(inbound)
        } else {
            None
        }
    }

    /// Set the value of `outbound_all_transient_errors` on the inbound context associated with
    /// this outbound key. This should be set to `false` when any outbound succeeds (is ACk'd) or
    /// is Nack'd with a permanent error.
    pub fn set_outbound_all_transient_errors(&mut self, outbound_key: Key, value: bool) {
        if let Some(inbound_key) = self.outbound.get(outbound_key).map(|o| o.inbound_key) {
            if let Some(inbound) = self.inbound.get_mut(inbound_key) {
                inbound.outbound_all_transient_errors = value
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::testing::create_test_pdata;
    use std::num::NonZeroUsize;

    fn new_contexts() -> Contexts {
        Contexts::new(
            NonZeroUsize::new(10).unwrap(),
            NonZeroUsize::new(20).unwrap(),
        )
    }

    // Helper to create a test context with subscribers
    fn create_context_with_subscribers() -> Context {
        let pdata = create_test_pdata().test_subscribe_to(
            otel_arrow_dfe_engine::Interests::ACKS,
            smallvec::smallvec![otel_arrow_dfe_engine::control::Context8u8::from(1u64)],
            1,
        );
        let (ctx, _payload) = pdata.into_parts();
        ctx
    }

    // Helper to create a test context without completion interests.
    fn create_context_without_subscribers() -> Context {
        Context::default()
    }

    #[test]
    fn test_with_subscribers() {
        let mut contexts = new_contexts();
        let original_context = create_context_with_subscribers();
        let inbound_key = contexts
            .insert_inbound(original_context.clone(), None, None)
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

        let inbound = contexts.clear_outbound(outbound_key).unwrap();
        assert_eq!(inbound.context, original_context);
        assert!(inbound.error.is_none());
        assert!(inbound.outbound_all_transient_errors);
        assert!(inbound.payload.is_none());
    }

    #[test]
    fn test_without_subscribers() {
        let mut contexts = new_contexts();
        let original_context = create_context_without_subscribers();
        let key = contexts
            .insert_inbound(original_context.clone(), None, None)
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
            .insert_inbound(original_context.clone(), None, None)
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

        let inbound = contexts.clear_outbound(outbound_key3).unwrap();
        assert_eq!(inbound.context, original_context);
        assert!(inbound.error.is_none());
        assert!(inbound.outbound_all_transient_errors);
        assert!(inbound.payload.is_none());
    }

    #[test]
    fn test_clear_outbound_with_invalid_key() {
        let mut contexts = new_contexts();

        // Create a key that doesn't exist, just to ensure we handle correctly
        let invalid_key = {
            let mut temp_contexts = new_contexts();
            let ctx = create_context_with_subscribers();
            let inbound_key = temp_contexts.insert_inbound(ctx, None, None).unwrap();
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
            .insert_inbound(
                context,
                None,
                Some(OutboundError {
                    reason: error_msg.clone(),
                    cause: NackCause::Refused,
                }),
            )
            .unwrap();
        let outbound_key = contexts.insert_outbound(inbound_key).unwrap();

        // Clear outbound and check error is returned
        let result = contexts.clear_outbound(outbound_key);
        assert!(result.is_some());
        let inbound = result.unwrap();
        assert!(inbound.error.is_some());
        let inbound_err = inbound.error.unwrap();
        assert_eq!(inbound_err.reason, error_msg);
        assert_eq!(inbound_err.cause, NackCause::Refused);
    }

    #[test]
    fn test_double_clear_same_outbound() {
        let mut contexts = new_contexts();
        let context = create_context_with_subscribers();
        let inbound_key = contexts.insert_inbound(context, None, None).unwrap();
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
        let inbound_key = contexts.insert_inbound(context, None, None).unwrap();
        let outbound_key = contexts.insert_outbound(inbound_key).unwrap();

        // Set the outbound as failed
        let error_msg = "export failed".to_string();
        contexts.set_failed_outbound(
            outbound_key,
            OutboundError {
                reason: error_msg.clone(),
                cause: NackCause::RouteFull,
            },
        );

        // Clear the outbound and verify error is returned
        let result = contexts.clear_outbound(outbound_key);
        assert!(result.is_some());
        let inbound = result.unwrap();
        let error = inbound.error.unwrap();
        assert_eq!(error.reason, error_msg);
        assert_eq!(error.cause, NackCause::RouteFull)
    }

    #[test]
    fn test_set_failed_multiple_outbounds_first_error_wins() {
        let mut contexts = new_contexts();
        let context = create_context_with_subscribers();
        let inbound_key = contexts.insert_inbound(context, None, None).unwrap();

        let outbound_key1 = contexts.insert_outbound(inbound_key).unwrap();
        let outbound_key2 = contexts.insert_outbound(inbound_key).unwrap();
        let outbound_key3 = contexts.insert_outbound(inbound_key).unwrap();

        // Set first outbound as failed
        let error_msg1 = "first error".to_string();
        contexts.set_failed_outbound(
            outbound_key1,
            OutboundError {
                reason: error_msg1.clone(),
                cause: NackCause::NodeShutdown,
            },
        );

        // Set second outbound as failed (should be ignored since error_reason is already set)
        let error_msg2 = "second error".to_string();
        contexts.set_failed_outbound(
            outbound_key2,
            OutboundError {
                reason: error_msg2.clone(),
                cause: NackCause::Unspecified,
            },
        );

        // Clear all outbounds
        assert!(contexts.clear_outbound(outbound_key1).is_none());
        assert!(contexts.clear_outbound(outbound_key2).is_none());

        // When clearing the last outbound, the first error should be returned
        let result = contexts.clear_outbound(outbound_key3);
        assert!(result.is_some());
        let inbound = result.unwrap();
        let error = inbound.error.unwrap();
        assert_eq!(error.reason, error_msg1, "First error should be preserved");
        assert_eq!(error.cause, NackCause::NodeShutdown)
    }

    /// Scenario: A split input has pipeline metric interests but no Ack/Nack subscriber.
    /// Guarantees: The original context is retained until every split output completes.
    #[test]
    fn test_with_metrics_only_context() {
        let mut contexts = new_contexts();
        let pdata = create_test_pdata().test_subscribe_to(
            otel_arrow_dfe_engine::Interests::PRODUCER_METRICS,
            smallvec::smallvec![],
            1,
        );
        let (original_context, _) = pdata.into_parts();

        let inbound_key = contexts
            .insert_inbound(original_context.clone(), None, None)
            .unwrap();
        assert!(!inbound_key.is_null());

        let outbound_key = contexts.insert_outbound(inbound_key).unwrap();
        let inbound = contexts.clear_outbound(outbound_key).unwrap();
        assert_eq!(inbound.context, original_context);
        assert!(inbound.error.is_none());
    }

    #[test]
    fn test_set_failed_with_invalid_key() {
        let mut contexts = new_contexts();

        // Create a key that doesn't exist
        let invalid_key = {
            let mut temp_contexts = new_contexts();
            let ctx = create_context_with_subscribers();
            let inbound_key = temp_contexts.insert_inbound(ctx, None, None).unwrap();
            temp_contexts.insert_outbound(inbound_key).unwrap()
        };

        // Setting failed with invalid key should not panic
        contexts.set_failed_outbound(
            invalid_key,
            OutboundError {
                reason: "error".to_string(),
                cause: NackCause::Refused,
            },
        );
    }

    #[test]
    fn test_set_failed_with_null_key() {
        let mut contexts = new_contexts();

        // Create a context without subscribers (results in null key)
        let context = create_context_without_subscribers();
        let inbound_key = contexts.insert_inbound(context, None, None).unwrap();
        let outbound_key = contexts.insert_outbound(inbound_key).unwrap();

        assert!(outbound_key.is_null());

        // Setting failed with null key should not panic
        contexts.set_failed_outbound(
            outbound_key,
            OutboundError {
                reason: "error".to_string(),
                cause: NackCause::Refused,
            },
        );
    }

    #[test]
    fn test_set_failed_does_not_override_inbound_error() {
        let mut contexts = new_contexts();
        let context = create_context_with_subscribers();

        // Insert inbound with an initial error
        let inbound_error = "initial inbound error".to_string();
        let inbound_key = contexts
            .insert_inbound(
                context,
                None,
                Some(OutboundError {
                    reason: inbound_error.clone(),
                    cause: NackCause::RouteClosed,
                }),
            )
            .unwrap();
        let outbound_key = contexts.insert_outbound(inbound_key).unwrap();

        // Try to set a different error via set_failed
        let outbound_error = "outbound error".to_string();
        contexts.set_failed_outbound(
            outbound_key,
            OutboundError {
                reason: outbound_error,
                cause: NackCause::Refused,
            },
        );

        // Clear outbound and verify the original inbound error is preserved
        let result = contexts.clear_outbound(outbound_key);
        assert!(result.is_some());
        let inbound = result.unwrap();
        let error = inbound.error.unwrap();

        assert_eq!(
            error.reason, inbound_error,
            "Original inbound error should be preserved"
        );
        assert_eq!(
            error.cause,
            NackCause::RouteClosed,
            "Original inbound error should be preserved"
        )
    }

    #[test]
    fn test_clear_outbound_removes_outbound_from_slotmap() {
        let mut contexts = new_contexts();
        let context = create_context_with_subscribers();
        let inbound_key = contexts.insert_inbound(context, None, None).unwrap();

        // Create two outbounds
        let outbound_key1 = contexts.insert_outbound(inbound_key).unwrap();
        let outbound_key2 = contexts.insert_outbound(inbound_key).unwrap();

        // Clear outbound1 once (should decrement counter to 1)
        let result1 = contexts.clear_outbound(outbound_key1);
        assert!(
            result1.is_none(),
            "Should not complete because there's still one outbound"
        );

        // Try to clear outbound1 again - this should return None because the outbound
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

    /// Scenario: An outbound key's transient-errors flag is explicitly set to false.
    /// Guarantees: The inbound returned from clear_outbound reflects the updated flag value.
    #[test]
    fn test_set_outbound_all_transient_errors_to_false() {
        let mut contexts = new_contexts();
        let context = create_context_with_subscribers();
        let inbound_key = contexts.insert_inbound(context, None, None).unwrap();
        let outbound_key = contexts.insert_outbound(inbound_key).unwrap();

        // set the flag to false (simulating an ACK or permanent NACK downstream)
        contexts.set_outbound_all_transient_errors(outbound_key, false);

        let inbound = contexts.clear_outbound(outbound_key).unwrap();
        assert!(
            !inbound.outbound_all_transient_errors,
            "flag should be false after being explicitly set to false"
        );
    }

    /// Scenario: set_outbound_all_transient_errors is called with a key from a different
    /// Contexts instance.
    /// Guarantees: The call does not panic when the outbound key is not found.
    #[test]
    fn test_set_outbound_all_transient_errors_with_invalid_key() {
        let mut contexts = new_contexts();

        let invalid_key = {
            let mut temp_contexts = new_contexts();
            let ctx = create_context_with_subscribers();
            let inbound_key = temp_contexts.insert_inbound(ctx, None, None).unwrap();
            temp_contexts.insert_outbound(inbound_key).unwrap()
        };

        // should not panic
        contexts.set_outbound_all_transient_errors(invalid_key, false);
    }

    /// Scenario: An inbound batch is inserted with an associated payload.
    /// Guarantees: The payload is preserved and returned when the inbound is completed
    /// via clear_outbound.
    #[test]
    fn test_insert_inbound_with_payload() {
        let mut contexts = new_contexts();
        let pdata = create_test_pdata();
        let (context, payload) = pdata.into_parts();

        // subscribe so context needs completion tracking
        let pdata = crate::pdata::OtapPdata::new(context, payload).test_subscribe_to(
            otel_arrow_dfe_engine::Interests::ACKS,
            smallvec::smallvec![otel_arrow_dfe_engine::control::Context8u8::from(1u64)],
            1,
        );
        let (context, payload) = pdata.into_parts();

        let inbound_key = contexts
            .insert_inbound(context, Some(payload.clone()), None)
            .unwrap();
        assert!(!inbound_key.is_null());

        let outbound_key = contexts.insert_outbound(inbound_key).unwrap();
        let inbound = contexts.clear_outbound(outbound_key).unwrap();

        assert!(
            inbound.payload.is_some(),
            "payload should be preserved in inbound"
        );
        assert_eq!(
            inbound.payload.unwrap().signal_type(),
            payload.signal_type(),
            "returned payload should match the original"
        );
    }

    /// Scenario: Multiple outbounds share one inbound, and set_outbound_all_transient_errors
    /// is called with false on only one of them.
    /// Guarantees: The inbound's flag is false after all outbounds are cleared, because
    /// the flag can only transition from true to false (never back).
    #[test]
    fn test_multiple_outbounds_transient_errors_flag_set_false_on_one() {
        let mut contexts = new_contexts();
        let context = create_context_with_subscribers();
        let inbound_key = contexts.insert_inbound(context, None, None).unwrap();

        let outbound_key1 = contexts.insert_outbound(inbound_key).unwrap();
        let outbound_key2 = contexts.insert_outbound(inbound_key).unwrap();
        let outbound_key3 = contexts.insert_outbound(inbound_key).unwrap();

        // only set false on one outbound (e.g. it was ACK'd)
        contexts.set_outbound_all_transient_errors(outbound_key2, false);

        // clear all outbounds
        assert!(contexts.clear_outbound(outbound_key1).is_none());
        assert!(contexts.clear_outbound(outbound_key2).is_none());

        let inbound = contexts.clear_outbound(outbound_key3).unwrap();
        assert!(
            !inbound.outbound_all_transient_errors,
            "flag should be false because at least one outbound set it to false"
        );
    }
}
