// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use bytes::Bytes;
use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_pdata::OtapArrowRecords;

use crate::CodecError;

/// Reusable decoder owned by one pipeline runtime.
pub trait PdataDecoder: Send {
    /// Decodes one complete batch while the caller retains the original bytes.
    fn decode(&mut self, signal: SignalType, bytes: &Bytes)
    -> Result<OtapArrowRecords, CodecError>;
}
