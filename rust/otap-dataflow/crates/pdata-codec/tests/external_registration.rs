// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Verifies that downstream crates can register codecs through the public macro.

use bytes::Bytes;
use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_pdata::OtapArrowRecords;
use otel_arrow_dfe_pdata_codec::{
    BatchSettings, BatchSizer, BatchingSupport, CodecBatcherRegistration, CodecBatches, CodecError,
    CodecMetadata, CodecRegistration, CodecRegistry, EncodedPdataBatch, PdataBatcher, PdataDecoder,
    PdataEncoding, register_pdata_codec,
};

const EXTERNAL_ENCODING: PdataEncoding = PdataEncoding::new("external-fixture-v1-zstd");

static EXTERNAL_METADATA: CodecMetadata =
    CodecMetadata::new(EXTERNAL_ENCODING, &[SignalType::Logs])
        .with_format_version("1")
        .with_compression("zstd");

struct ExternalDecoder;

impl PdataDecoder for ExternalDecoder {
    fn decode(
        &mut self,
        _signal: SignalType,
        _bytes: &Bytes,
    ) -> Result<OtapArrowRecords, CodecError> {
        panic!("the registration hygiene test does not decode payloads")
    }
}

impl PdataBatcher for ExternalDecoder {
    fn batch(
        &mut self,
        _signal: SignalType,
        _settings: &BatchSettings,
        _inputs: Vec<Bytes>,
    ) -> Result<CodecBatches, CodecError> {
        panic!("the registration hygiene test does not batch payloads")
    }
}

static EXTERNAL_BATCHING: BatchingSupport = BatchingSupport {
    sizers: &[BatchSizer::Bytes],
    default_settings: BatchSettings::otlp(),
};

register_pdata_codec!(
    EXTERNAL_CODEC,
    CodecRegistration::new(&EXTERNAL_METADATA)
        .with_decoder(|_| panic!("admission must not create a decoder"))
        .with_item_counter(|_, bytes| {
            assert!(bytes.is_empty(), "constructed batches must bypass counting");
            Some(0)
        })
        .with_batcher(CodecBatcherRegistration::new(&EXTERNAL_BATCHING, || {
            Box::new(ExternalDecoder)
        })),
);

// A codec extension keeps these fields private and creates this value only
// through its builder. This fixture uses bytes whose contents are irrelevant
// to admission, to expose accidental decoder or counter invocation.
struct BuiltBatch {
    bytes: Bytes,
    count: usize,
    signal: SignalType,
}

impl EncodedPdataBatch for BuiltBatch {
    const ENCODING: PdataEncoding = EXTERNAL_ENCODING;

    fn signal_type(&self) -> SignalType {
        self.signal
    }

    fn into_parts(self) -> (Bytes, usize) {
        (self.bytes, self.count)
    }
}

/// Scenario: An external codec constructs a batch whose item count is already known.
/// Guarantees: Admission and cached reads preserve bytes, signal, identity, and known zero
/// without invoking the counter or creating a decoder, including after shallow cloning.
#[test]
fn built_admission_preserves_counts_without_scanning() {
    let codec = CodecRegistry::global()
        .unwrap()
        .resolve(&EXTERNAL_ENCODING)
        .unwrap();
    for count in [0, 3] {
        let bytes = Bytes::from(vec![1, 2, 3]);
        let pointer = bytes.as_ptr();
        let mut payload = codec
            .admit_built(BuiltBatch {
                bytes,
                count,
                signal: SignalType::Logs,
            })
            .unwrap();
        assert_eq!(payload.encoding(), Some(&EXTERNAL_ENCODING));
        assert_eq!(payload.signal_type(), SignalType::Logs);
        assert_eq!(payload.encoded_bytes().unwrap().as_ptr(), pointer);
        assert_eq!(payload.known_item_count(), Some(count));
        assert_eq!(payload.num_items(), count);
        let mut clone = payload.clone();
        assert_eq!(clone.num_items(), count);
        assert_eq!(clone.encoded_bytes().unwrap().as_ptr(), pointer);
    }
}

/// Scenario: A constructed batch has mismatched identity, unsupported signal, or an uncacheable count.
/// Guarantees: Admission rejects invalid metadata and missing decode support without invoking codec state.
#[test]
fn built_admission_rejects_invalid_metadata() {
    let registry = CodecRegistry::global().unwrap();
    let codec = registry.resolve(&EXTERNAL_ENCODING).unwrap();
    let batch = |signal, count| BuiltBatch {
        bytes: Bytes::from_static(b"not telemetry"),
        count,
        signal,
    };
    let wrong_codec = registry.resolve(&PdataEncoding::OTLP).unwrap();
    assert!(matches!(
        wrong_codec.admit_built(batch(SignalType::Logs, 1)),
        Err(CodecError::InvalidConstruction { .. })
    ));
    assert!(matches!(
        codec.admit_built(batch(SignalType::Metrics, 1)),
        Err(CodecError::Unsupported { .. })
    ));
    assert!(matches!(
        codec.admit_built(batch(SignalType::Logs, usize::MAX)),
        Err(CodecError::InvalidConstruction { .. })
    ));

    static ENCODE_ONLY: [CodecRegistration; 1] = [CodecRegistration::new(&EXTERNAL_METADATA)
        .with_encoder(|_| panic!("construction must not create an encoder"))];
    let encode_only = CodecRegistry::validate(&ENCODE_ONLY)
        .unwrap()
        .resolve(&EXTERNAL_ENCODING)
        .unwrap();
    assert!(matches!(
        encode_only.admit_built(batch(SignalType::Logs, 1)),
        Err(CodecError::Unsupported { .. })
    ));
    // Failure does not affect later admission through the same immutable handle.
    assert_eq!(
        codec
            .admit_built(batch(SignalType::Logs, 1))
            .unwrap()
            .known_item_count(),
        Some(1)
    );
}

/// Scenario: A downstream crate registers a counted decoder and native batcher without importing linkme.
/// Guarantees: The public macro is hygienic and the metadata builders preserve all declarations.
#[test]
fn downstream_registration_needs_no_linkme_import() {
    let registry = CodecRegistry::global().expect("linked codec registrations must be valid");
    let codec = registry
        .resolve(&EXTERNAL_ENCODING)
        .expect("the macro must add the downstream codec to the registry");

    assert!(codec.can_decode());
    assert!(!codec.can_encode());
    assert!(codec.can_batch());
    assert_eq!(
        codec.batching_support().unwrap().sizers,
        &[BatchSizer::Bytes]
    );
    assert_eq!(codec.count_items(SignalType::Logs, &[]), Some(0));
    assert_eq!(codec.metadata().encoding(), &EXTERNAL_ENCODING);
    assert_eq!(codec.metadata().signals(), &[SignalType::Logs]);
    assert_eq!(codec.metadata().format_version(), Some("1"));
    assert_eq!(codec.metadata().compression(), Some("zstd"));
}

/// Scenario: A downstream crate enables codec helpers without opting into fixture registrations.
/// Guarantees: The testing feature does not add sample codec identities to the process registry.
#[cfg(not(feature = "test-codecs"))]
#[test]
fn testing_helpers_do_not_register_fixture_codecs() {
    let registry = CodecRegistry::global().expect("linked codec registrations must be valid");
    for name in [
        "test-otlp-codec",
        "test-otlp-decode-only",
        "test-native-otlp",
        "test-otlp-encode-only",
    ] {
        assert!(
            registry.resolve(&PdataEncoding::new(name)).is_err(),
            "{name}"
        );
    }
}
