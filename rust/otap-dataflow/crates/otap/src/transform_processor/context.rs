// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! utilities for managing context of inbound and outbound requests
//! produced by transform processor.

use std::num::NonZeroUsize;

use otap_df_engine::{control::CallData, error::Error};

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
        // TODO maybe we only want to insert this if there is call data?
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
        // TODO remove the unwraps here and probably only do the insert if there is an inbound key

        // incr inbound
        let inbound = self.inbound.get_mut(inbound_key).unwrap();
        inbound.num_outbound += 1;

        // insert outbound
        let outbound = Outbound { inbound_key };
        let (outbound_key, _) = self.outbound.allocate(|| (outbound, ())).unwrap();
        Ok(outbound_key)
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
            let outbound = self.outbound.get(outbound_key).unwrap();
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
    #[test]
    fn test_context_push_pop() {}
}
