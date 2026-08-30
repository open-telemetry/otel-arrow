// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Behavioral checks that third-party codec registrations can reuse.

use bytes::Bytes;
use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_pdata::OtapPayloadHelpers;

use crate::{CodecService, EncodedPdata};

/// One encoded sample used by the decoder conformance checks.
pub struct DecodeConformanceCase {
    /// Admitted valid batch.
    pub valid: EncodedPdata,
    /// Malformed bytes that strict decoders must reject.
    pub malformed: Option<Bytes>,
    /// Expected signal.
    pub signal: SignalType,
    /// Expected primary-signal item count.
    pub expected_items: usize,
}

/// Checks signal preservation, state reuse, repeated failures, and recovery.
pub fn assert_decode_conformance(service: &CodecService, case: DecodeConformanceCase) {
    let codec = case.valid.codec();
    assert_eq!(case.valid.signal_type(), case.signal);
    assert_eq!(
        codec.count_items(case.signal, case.valid.bytes()),
        Some(case.expected_items)
    );
    let records = service
        .decode(&case.valid)
        .expect("valid codec bytes must decode");
    assert_eq!(records.signal_type(), case.signal);
    assert_eq!(records.num_items(), case.expected_items);

    if let Some(malformed) = case.malformed {
        let malformed = codec
            .admit(case.signal, malformed)
            .expect("admission must remain lazy");
        for _ in 0..2 {
            assert!(service.decode(&malformed).is_err());
        }
        assert_eq!(
            service
                .decode(&case.valid)
                .expect("valid decode must recover after repeated failures")
                .num_items(),
            case.expected_items
        );
    }
}
