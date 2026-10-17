// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Verifies that downstream crates can register codecs through the public macro.

use bytes::Bytes;
use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_pdata::OtapArrowRecords;
use otel_arrow_dfe_pdata_codec::{
    BatchSettings, BatchSizer, BatchingSupport, CodecBatcherRegistration, CodecBatches, CodecError,
    CodecMetadata, CodecRegistration, CodecRegistry, PdataBatcher, PdataDecoder, PdataEncoding,
    register_pdata_codec,
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
        .with_decoder(|_| Box::new(ExternalDecoder))
        .with_item_counter(|_, _| Some(0))
        .with_batcher(CodecBatcherRegistration::new(&EXTERNAL_BATCHING, || {
            Box::new(ExternalDecoder)
        })),
);

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
