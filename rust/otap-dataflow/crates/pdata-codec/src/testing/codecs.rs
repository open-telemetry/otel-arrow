// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Registered OTLP-compatible test codecs for generic pdata paths.

use otel_arrow_dfe_config::SignalType;

use crate::codecs::otlp::{OtlpBatcher, OtlpEncoder, create_decoder};
use crate::{
    BatchSettings, BatchSizer, BatchingSupport, CodecBatcherRegistration, CodecMetadata,
    CodecRegistration, PdataEncoding,
};

/// Registered encoding with OTAP fallback batching.
pub const TEST_ENCODING: PdataEncoding = PdataEncoding::new("test-otlp-codec");
/// Registered input encoding without an output encoder.
pub const DECODE_ONLY_ENCODING: PdataEncoding = PdataEncoding::new("test-otlp-decode-only");
/// Codec with native byte batching, independent of the built-in identity.
pub const NATIVE_ENCODING: PdataEncoding = PdataEncoding::new("test-native-otlp");
/// Output-only codec that must never be admitted into the pipeline.
pub const ENCODE_ONLY_ENCODING: PdataEncoding = PdataEncoding::new("test-otlp-encode-only");

static SIGNALS: &[SignalType] = &[SignalType::Logs, SignalType::Metrics, SignalType::Traces];

static FALLBACK_METADATA: CodecMetadata = CodecMetadata::new(TEST_ENCODING, SIGNALS);
static DECODE_METADATA: CodecMetadata = CodecMetadata::new(DECODE_ONLY_ENCODING, SIGNALS);
static NATIVE_METADATA: CodecMetadata = CodecMetadata::new(NATIVE_ENCODING, SIGNALS);
static ENCODE_METADATA: CodecMetadata =
    CodecMetadata::new(ENCODE_ONLY_ENCODING, &[SignalType::Logs]);
static NATIVE_BATCHING: BatchingSupport = BatchingSupport {
    sizers: &[BatchSizer::Bytes],
    default_settings: BatchSettings::otlp(),
};

crate::register_pdata_codec!(
    FALLBACK,
    CodecRegistration::new(&FALLBACK_METADATA)
        .with_decoder(create_decoder)
        .with_encoder(|policy| Ok(Box::new(OtlpEncoder::new(policy))))
        .with_item_counter(|signal, bytes| Some(otel_arrow_dfe_pdata::count_otlp_items(
            signal, bytes
        ))),
);

crate::register_pdata_codec!(
    DECODE,
    CodecRegistration::new(&DECODE_METADATA)
        .with_decoder(create_decoder)
        .with_item_counter(|signal, bytes| Some(otel_arrow_dfe_pdata::count_otlp_items(
            signal, bytes
        ))),
);

crate::register_pdata_codec!(
    NATIVE,
    CodecRegistration::new(&NATIVE_METADATA)
        .with_decoder(create_decoder)
        .with_encoder(|policy| Ok(Box::new(OtlpEncoder::new(policy))))
        .with_batcher(CodecBatcherRegistration::new(&NATIVE_BATCHING, || {
            Box::new(OtlpBatcher)
        }))
        .with_item_counter(|signal, bytes| Some(otel_arrow_dfe_pdata::count_otlp_items(
            signal, bytes
        ))),
);

crate::register_pdata_codec!(
    ENCODE,
    CodecRegistration::new(&ENCODE_METADATA)
        .with_encoder(|policy| Ok(Box::new(OtlpEncoder::new(policy)))),
);
