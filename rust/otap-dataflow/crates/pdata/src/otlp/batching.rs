// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::OtlpProtoBytes;
use crate::error::{Error, Result};
use otap_df_config::SignalType;
use std::num::NonZeroU64;

/// Combines OTLP content by concatenation of bytes. Because we have a
/// top-level repeated field, this is precisely correct.
pub fn make_bytes_batches(
    _signal: SignalType,
    _max_bytes: Option<NonZeroU64>,
    _records: Vec<OtlpProtoBytes>,
) -> Result<Vec<OtlpProtoBytes>> {
    // @@@
    Err(Error::EmptyBatch)
}
