// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! utilities for managing context of inbound and outbound requests
//! produced by transform processor.

use otap_df_engine::error::Error;
use slotmap::Key as _;
use std::num::NonZeroUsize;

use crate::{
    accessory::slots::{Key, State},
    pdata::Context,
};

struct Inbound {
    context: Context,
    error_reason: Option<String>, //
    num_outbound: usize,
}

struct Outbound {
    inbound_key: Key,
}

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

    pub fn insert_inbound(
        &mut self,
        context: Context,
        error_reason: Option<String>,
    ) -> Result<Key, Error> {
        if !context.has_subscribers() {
            // no point in managing the inbound/outbound the context if there are no subscribers
            return Ok(Key::null());
        }

        let inbound = Inbound {
            context,
            num_outbound: 0,
            error_reason,
        };
        // TODO don't unwrap these here ...
        let (inbound_key, _) = self.inbound.allocate(|| (inbound, ())).unwrap();
        Ok(inbound_key)
    }

    pub fn insert_outbound(&mut self, inbound_key: Key) -> Result<Key, Error> {
        // incr inbound
        if let Some(inbound) = self.inbound.get_mut(inbound_key) {
            inbound.num_outbound += 1;

            // insert outbound
            let outbound = Outbound { inbound_key };
            let (outbound_key, _) = self.outbound.allocate(|| (outbound, ())).unwrap();
            Ok(outbound_key)
        } else {
            Ok(Key::null())
        }
    }

    pub fn set_failed(&mut self, outbound_key: Key, error_reason: String) {
        if let Some(inbound) = self.inbound.get_mut(outbound_key) {
            // Only keep the original error
            // TODO - maybe we should concatenate the errors together ...
            if inbound.error_reason.is_none() {
                inbound.error_reason = Some(error_reason)
            }
        }
    }

    pub fn clear_outbound(&mut self, outbound_key: Key) -> Option<(Context, Option<String>)> {
        let inbound_key = {
            let outbound = self.outbound.get(outbound_key)?;
            outbound.inbound_key.clone()
        };

        let num_outbound = {
            let inbound = self.inbound.get_mut(inbound_key.clone())?;
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

        let outbound_key = contexts.insert_outbound(inbound_key.clone()).unwrap();
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
}
