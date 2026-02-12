// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Batch validation utilities.
//!
//! Validates that each message in a collection meets a minimum batch size. This
//! is useful for ensuring the batch processor emitted groups as expected.

use otap_df_pdata::proto::OtlpProtoMessage;

/// Ensure every message has at least `min_items` items.
/// returns false if any message has fewer items than `min_items`.
/// if min_items is set to 0 will always return true
pub fn check_min_batch_size(messages: &[OtlpProtoMessage], min_items: usize) -> bool {
    if messages.is_empty() {
        return false;
    }

    if min_items == 0 {
        return true;
    }

    messages.iter().all(|msg| msg.num_items() >= min_items)
}
