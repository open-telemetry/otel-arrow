// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of policy engine

use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_pdata::{OtapArrowRecords, OtapPayloadHelpers};
use policy_rs::{CompiledMatchers, LogSignal, PolicyError, PolicySnapshot, Signal};

/// Policy evaluation engine
pub struct PolicyEngine {}

impl PolicyEngine {
    /// Create a new instance.
    pub fn new() -> Self {
        Self {}
    }

    /// Evaluate the policies in the snapshot on all the rows within the otap batch.
    pub fn evaluate_and_transform(
        &self,
        snapshot: &PolicySnapshot,
        otap_batch: OtapArrowRecords,
    ) -> Result<OtapArrowRecords, PolicyError> {
        // TODO
        // - volume tracking
        // - other signal type handling

        match otap_batch.signal_type() {
            SignalType::Logs => {
                let Some(compiled_matchers) = LogSignal::compiled_matchers(snapshot) else {
                    todo!()
                };
                let policy_matches =
                    find_matching_policies::<LogSignal>(compiled_matchers, &otap_batch)?;

                
            }
            _ => {
                // TODO support other signal types
                todo!()
            }
        }

        Ok(otap_batch)
    }
}

#[derive(Clone)]
enum PolicyMatch {
    All,
    None,
    Some(Vec<u8>), // bitmap
}

/// Returns a [`Vec`] of [`PolicyMatch`] indicating which rows matched the policies at some index.
fn find_matching_policies<S: Signal>(
    compiled: &CompiledMatchers<S>,
    otap_batch: &OtapArrowRecords,
) -> Result<Vec<PolicyMatch>, PolicyError> {
    let policy_count = compiled.policies.len();
    if policy_count == 0 {
        return Ok(Vec::new())
    }

    let matches = vec![PolicyMatch::All; policy_count];

    // TODO - all the stuff

    Ok(matches)
}
