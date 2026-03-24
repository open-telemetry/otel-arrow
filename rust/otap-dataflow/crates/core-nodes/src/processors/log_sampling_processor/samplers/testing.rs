// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared test helpers for sampler unit tests.

use otap_df_pdata::encode::encode_logs_otap_batch;
use otap_df_pdata::otap::OtapArrowRecords;
use otap_df_pdata::testing::fixtures::logs_with_varying_attributes_and_properties;

/// Create an [`OtapArrowRecords`] containing `n` log records.
pub fn make_log_records(n: usize) -> OtapArrowRecords {
    let logs_data = logs_with_varying_attributes_and_properties(n);
    encode_logs_otap_batch(&logs_data).expect("encode")
}
