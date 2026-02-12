// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Signal drop validation.
//!
//! Compare two slices of messages (typically control vs. system-under-validation)
//! and verify that the latter contains fewer items.

use otap_df_pdata::proto::OtlpProtoMessage;

/// Validate that the `after` messages contain strictly fewer items than `before`.
pub fn check_signal_drop(before: &[OtlpProtoMessage], after: &[OtlpProtoMessage]) -> bool {
    let before_total: usize = before.iter().map(OtlpProtoMessage::num_items).sum();
    let after_total: usize = after.iter().map(OtlpProtoMessage::num_items).sum();

    after_total < before_total
}
