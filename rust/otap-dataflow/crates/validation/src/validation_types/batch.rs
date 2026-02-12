// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Batch validation utilities.
//!
//! Validates that each message in a collection meets configured batch size bounds.

use otap_df_pdata::proto::OtlpProtoMessage;

/// Ensure every message size is within `[min_items, max_items]` (if provided).
/// Returns false if the slice is empty or any message violates the bounds.
pub fn check_batch_size(
    messages: &[OtlpProtoMessage],
    min_items: Option<usize>,
    max_items: Option<usize>,
) -> bool {
    if messages.is_empty() {
        return false;
    }

    messages.iter().all(|msg| {
        let batch_size = msg.num_items();
        if let Some(min) = min_items {
            if batch_size < min {
                return false;
            }
        }
        if let Some(max) = max_items {
            if batch_size > max {
                return false;
            }
        }
        true
    })
}
