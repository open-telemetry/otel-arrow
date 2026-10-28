// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Reusable conformance checks for pdata codec implementations.
//!
//! The helpers exercise the representation-independent guarantees expected by
//! the engine: signal preservation, stateless item counting, successful decode,
//! strict malformed-input failures, recovery on the same decoder instance, and
//! decoder-state reuse. A codec should run these checks for every supported
//! signal with representative valid and malformed batches. When a malformed
//! sample is supplied, the service must use [`crate::DecodeValidation::Strict`].
//!
//! Conformance checks do not replace codec-specific coverage. Implementations
//! remain responsible for format edge cases, resource and decompression limits,
//! conversion fidelity, output compatibility, and any native batching contract
//! they advertise.
//!
//! Enabling `testing` exposes helpers without registering any additional codecs.
//! Fixture registrations require the separate `test-codecs` feature, which
//! downstream tests must enable only through a dev-dependency.

#[cfg(any(test, feature = "test-codecs"))]
mod codecs;

#[cfg(any(test, feature = "test-codecs"))]
pub use codecs::{DECODE_ONLY_ENCODING, ENCODE_ONLY_ENCODING, NATIVE_ENCODING, TEST_ENCODING};

use bytes::Bytes;
use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_pdata::OtapPayloadHelpers;

use crate::{CodecService, EncodedPdata};

/// Injects a cache value solely to exercise failure recovery in downstream tests.
/// Production code must obtain counts from the codec rather than asserting them.
#[must_use]
pub fn with_cached_item_count(payload: crate::PdataPayload, count: usize) -> crate::PdataPayload {
    payload.with_item_count(count)
}

/// One encoded sample used by the decoder conformance checks.
///
/// Supply a batch with a known primary-signal item count. When `malformed` is
/// present, admission must remain lazy and decoding must reject it repeatedly.
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
        assert_eq!(
            service.decode_policy().validation(),
            crate::DecodeValidation::Strict,
            "malformed-input conformance requires strict decode validation"
        );
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
