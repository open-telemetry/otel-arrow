// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Registered OTLP-compatible test codecs for exercising generic pdata paths.

use crate::batching::CodecBatches;
use crate::batching::{BatchProfile, BatchSizer, BatchingSupport};
use crate::codec::{
    OtlpCodec, PDATA_CODEC_FACTORIES, PdataCodec, PdataCodecMetadata, PdataCodecRegistration,
    PdataEncoding,
};
use crate::{OtapArrowRecords, error::Error};
use bytes::Bytes;
use otel_arrow_dfe_config::{EncodeOptions, SignalType};

// The native OTLP conversion uses trusted byte views. Test extensions validate
// their input so malformed test messages exercise the generic codec error path.
#[derive(Default)]
struct TestCodec(OtlpCodec);

impl PdataCodec for TestCodec {
    fn decode(
        &mut self,
        signal: SignalType,
        bytes: &Bytes,
    ) -> Result<OtapArrowRecords, crate::encode::Error> {
        use crate::views::otlp::bytes::{
            logs::RawLogsData, metrics::RawMetricsData, traces::RawTraceData,
        };
        match signal {
            SignalType::Logs => RawLogsData::try_new(bytes).map(|_| ()),
            SignalType::Metrics => RawMetricsData::try_new(bytes).map(|_| ()),
            SignalType::Traces => RawTraceData::try_new(bytes).map(|_| ()),
        }
        .map_err(|error| Error::Format {
            error: error.to_string(),
        })?;
        self.0.decode(signal, bytes)
    }

    fn encode(
        &mut self,
        records: OtapArrowRecords,
        options: EncodeOptions,
    ) -> Result<Bytes, Error> {
        self.0.encode(records, options)
    }

    fn batch(
        &mut self,
        signal: SignalType,
        profile: &BatchProfile,
        inputs: Vec<Bytes>,
    ) -> Result<CodecBatches, Error> {
        self.0.batch(signal, profile, inputs)
    }
}

/// A registered encoding with OTAP fallback batching.
pub const TEST_ENCODING: PdataEncoding = PdataEncoding::new("test-otlp-codec");
/// A registered input encoding without an output encoder.
pub const DECODE_ONLY_ENCODING: PdataEncoding = PdataEncoding::new("test-otlp-decode-only");
/// A codec with native byte batching, independent of the built-in identity.
pub const NATIVE_ENCODING: PdataEncoding = PdataEncoding::new("test-native-otlp");
/// An output-only codec that must never be admitted into the pipeline.
pub const ENCODE_ONLY_ENCODING: PdataEncoding = PdataEncoding::new("test-otlp-encode-only");

static FALLBACK_METADATA: PdataCodecMetadata = PdataCodecMetadata {
    encoding: TEST_ENCODING,
    signals: &[SignalType::Logs, SignalType::Metrics, SignalType::Traces],
    format_version: None,
    compression: None,
    can_decode: true,
    can_encode: true,
    batching: None,
};
static DECODE_METADATA: PdataCodecMetadata = PdataCodecMetadata {
    encoding: DECODE_ONLY_ENCODING,
    signals: &[SignalType::Logs, SignalType::Metrics, SignalType::Traces],
    format_version: None,
    compression: None,
    can_decode: true,
    can_encode: false,
    batching: None,
};
static NATIVE_METADATA: PdataCodecMetadata = PdataCodecMetadata {
    encoding: NATIVE_ENCODING,
    signals: &[SignalType::Logs, SignalType::Metrics, SignalType::Traces],
    format_version: None,
    compression: None,
    can_decode: true,
    can_encode: true,
    batching: Some(BatchingSupport {
        sizers: &[BatchSizer::Bytes],
        default_profile: BatchProfile::otlp(),
    }),
};
static ENCODE_METADATA: PdataCodecMetadata = PdataCodecMetadata {
    encoding: ENCODE_ONLY_ENCODING,
    signals: &[SignalType::Logs],
    format_version: None,
    compression: None,
    can_decode: false,
    can_encode: true,
    batching: None,
};

#[allow(unsafe_code)]
#[linkme::distributed_slice(PDATA_CODEC_FACTORIES)]
static FALLBACK: PdataCodecRegistration = PdataCodecRegistration {
    count_items: None,
    metadata: &FALLBACK_METADATA,
    create: || Box::<TestCodec>::default(),
};
#[allow(unsafe_code)]
#[linkme::distributed_slice(PDATA_CODEC_FACTORIES)]
static DECODE: PdataCodecRegistration = PdataCodecRegistration {
    count_items: None,
    metadata: &DECODE_METADATA,
    create: || Box::<TestCodec>::default(),
};
#[allow(unsafe_code)]
#[linkme::distributed_slice(PDATA_CODEC_FACTORIES)]
static NATIVE: PdataCodecRegistration = PdataCodecRegistration {
    count_items: None,
    metadata: &NATIVE_METADATA,
    create: || Box::<TestCodec>::default(),
};
#[allow(unsafe_code)]
#[linkme::distributed_slice(PDATA_CODEC_FACTORIES)]
static ENCODE: PdataCodecRegistration = PdataCodecRegistration {
    count_items: None,
    metadata: &ENCODE_METADATA,
    create: || Box::<TestCodec>::default(),
};
