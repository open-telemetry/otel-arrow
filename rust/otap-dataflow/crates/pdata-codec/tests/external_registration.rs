// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Verifies that downstream crates can register codecs through the public macro.

use bytes::Bytes;
use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_pdata::OtapArrowRecords;
use otel_arrow_dfe_pdata_codec::{
    CodecError, CodecMetadata, CodecRegistration, CodecRegistry, PdataDecoder, PdataEncoding,
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

register_pdata_codec!(
    EXTERNAL_CODEC,
    CodecRegistration::new(&EXTERNAL_METADATA).with_decoder(|| Box::new(ExternalDecoder)),
);

/// Scenario: A downstream crate registers a decode-only codec without importing linkme.
/// Guarantees: The public macro is hygienic and the metadata builders preserve all declarations.
#[test]
fn downstream_registration_needs_no_linkme_import() {
    let registry = CodecRegistry::global().expect("linked codec registrations must be valid");
    let codec = registry
        .resolve(&EXTERNAL_ENCODING)
        .expect("the macro must add the downstream codec to the registry");

    assert!(codec.can_decode());
    assert!(!codec.can_encode());
    assert_eq!(codec.metadata().encoding(), &EXTERNAL_ENCODING);
    assert_eq!(codec.metadata().signals(), &[SignalType::Logs]);
    assert_eq!(codec.metadata().format_version(), Some("1"));
    assert_eq!(codec.metadata().compression(), Some("zstd"));
}
