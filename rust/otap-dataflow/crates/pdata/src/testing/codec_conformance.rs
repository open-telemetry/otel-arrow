// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Reusable behavioral checks for pdata codecs before payload integration.

use bytes::Bytes;
use otel_arrow_dfe_config::SignalType;

use crate::OtapPayloadHelpers;
use crate::codec::{CodecState, EncodingPlan, ResolvedCodec};

/// One codec and byte sample used by the decode conformance checks.
pub struct DecodeConformanceCase {
    /// Resolved codec under test.
    pub codec: ResolvedCodec,
    /// Signal carried outside the encoded bytes.
    pub signal: SignalType,
    /// Complete independently decodable batch.
    pub valid: Bytes,
    /// Malformed bytes that strict decoders must reject.
    pub malformed: Option<Bytes>,
    /// Expected primary-signal item count.
    pub expected_items: usize,
}

/// Checks signal preservation, borrowed views, stateless counts, state reuse,
/// repeated malformed input, and independent output decoding.
pub fn assert_decode_conformance(case: DecodeConformanceCase) {
    let pointer = case.valid.as_ptr();
    assert_eq!(
        case.codec.count_items(case.signal, &case.valid),
        Some(case.expected_items)
    );

    let mut state = CodecState::default();
    let view = state
        .view(case.codec, case.signal, &case.valid)
        .expect("valid codec bytes must produce a view");
    assert_eq!(view.signal_type(), case.signal);
    assert_eq!(case.valid.as_ptr(), pointer);

    let records = state
        .decode(case.codec, case.signal, &case.valid)
        .expect("valid codec bytes must decode");
    assert_eq!(records.signal_type(), case.signal);
    assert_eq!(records.num_items(), case.expected_items);
    assert_eq!(state.test_instance_count(), 1);

    if let Some(malformed) = case.malformed {
        let malformed_pointer = malformed.as_ptr();
        for _ in 0..2 {
            assert!(state.decode(case.codec, case.signal, &malformed).is_err());
            assert_eq!(malformed.as_ptr(), malformed_pointer);
        }
        let recovered = state
            .decode(case.codec, case.signal, &case.valid)
            .expect("valid decode must recover after repeated failures");
        assert_eq!(recovered.num_items(), case.expected_items);
    }

    if case.codec.metadata().can_encode() {
        let plan = EncodingPlan::new(case.codec, Default::default())
            .expect("conforming output codec must resolve");
        let mut records = records;
        let output = state
            .prepare_encode(&mut records, &plan)
            .expect("conforming codec must encode")
            .into_bytes();
        let independently_decoded = CodecState::default()
            .decode(case.codec, case.signal, &output)
            .expect("each encoded output must decode independently");
        assert_eq!(independently_decoded.signal_type(), case.signal);
        assert_eq!(independently_decoded.num_items(), case.expected_items);
    }
}
