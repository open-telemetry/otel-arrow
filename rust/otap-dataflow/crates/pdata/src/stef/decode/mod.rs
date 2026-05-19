// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! STEF metrics stream decoding into direct OTAP Arrow records.

mod builder;
mod entities;
mod record_builders;

use crate::OtapArrowRecords;

use self::builder::DirectOtapMetricsBuilder;
use self::entities::{
    DecPoint, DecodedAttribute, DirectDecMetric, DirectDecResource, DirectDecScope,
    DirectDecoderState, DirectMetricsFrameDecoder,
};
use super::Error;
use super::wire::{
    BitReader, FrameDecoder, SliceReader, decode_metrics_column_tree, read_decode_column_data,
    read_decode_column_sizes, read_fixed_header, read_var_header,
};

/// Decodes a complete STEF metrics byte stream directly into OTAP metrics Arrow records.
pub fn decode_metrics_otap(bytes: &[u8]) -> Result<OtapArrowRecords, Error> {
    decode_metrics_otap_with_count(bytes).map(|(records, _)| records)
}

/// Decodes a complete STEF metrics byte stream directly into OTAP records and a STEF record count.
pub fn decode_metrics_otap_with_count(bytes: &[u8]) -> Result<(OtapArrowRecords, u64), Error> {
    MetricsOtapStreamDecoder::decode(bytes)
}

/// Incrementally decodes a STEF metrics stream transported as gRPC chunks.
///
/// The Go STEF client sends the fixed header, the variable header frame, and each data frame as
/// separate gRPC chunks. This decoder preserves dictionaries and modified-record state across
/// those chunks while returning OTAP records only for chunks that contain completed data frames.
#[derive(Default)]
pub struct MetricsStreamDecoder {
    core: MetricsDecoderCore,
    frames: Option<FrameDecoder>,
    pending: Vec<u8>,
    phase: StreamPhase,
}

impl MetricsStreamDecoder {
    /// Decodes the next STEF/gRPC chunk.
    ///
    /// Returns `None` for header-only chunks or incomplete chunks. When data frames are decoded,
    /// the returned count is the number of STEF metric records decoded from this call.
    pub fn decode_chunk(&mut self, bytes: &[u8]) -> Result<Option<(OtapArrowRecords, u64)>, Error> {
        self.pending.extend_from_slice(bytes);
        let mut builder = DirectOtapMetricsBuilder::default();
        let mut decoded_count = 0_u64;

        loop {
            match self.phase {
                StreamPhase::FixedHeader => {
                    let Some((compression, consumed)) = try_read_fixed_header(&self.pending)?
                    else {
                        break;
                    };
                    self.frames = Some(FrameDecoder::new(compression)?);
                    drop(self.pending.drain(..consumed));
                    self.phase = StreamPhase::VarHeader;
                }
                StreamPhase::VarHeader => {
                    let frames = self
                        .frames
                        .as_mut()
                        .ok_or(Error::InvalidHeader("missing fixed header"))?;
                    let Some((_, consumed)) = try_read_var_header_frame(frames, &self.pending)?
                    else {
                        break;
                    };
                    drop(self.pending.drain(..consumed));
                    self.phase = StreamPhase::Data;
                }
                StreamPhase::Data => {
                    let frames = self
                        .frames
                        .as_mut()
                        .ok_or(Error::InvalidHeader("missing fixed header"))?;
                    let Some((flags, frame, consumed)) = try_read_frame(frames, &self.pending)?
                    else {
                        break;
                    };
                    decoded_count = decoded_count.saturating_add(self.core.decode_frame(
                        flags,
                        frame.as_ref(),
                        &mut builder,
                    )?);
                    drop(self.pending.drain(..consumed));
                }
            }
        }

        if decoded_count == 0 {
            Ok(None)
        } else {
            Ok(Some((builder.finish()?, decoded_count)))
        }
    }
}

#[derive(Clone, Copy, Default)]
enum StreamPhase {
    #[default]
    FixedHeader,
    VarHeader,
    Data,
}

#[derive(Default)]
struct MetricsOtapStreamDecoder;

impl MetricsOtapStreamDecoder {
    fn decode(bytes: &[u8]) -> Result<(OtapArrowRecords, u64), Error> {
        let mut input = SliceReader::new(bytes);
        let compression = read_fixed_header(&mut input)?;
        let mut frames = FrameDecoder::new(compression)?;
        let (_, var_header) = frames.read_frame(&mut input)?;
        read_var_header(var_header.as_ref())?;

        let mut core = MetricsDecoderCore::default();
        let mut builder = DirectOtapMetricsBuilder::default();
        let mut record_count = 0_u64;
        while !input.is_empty() {
            let (flags, frame) = frames.read_frame(&mut input)?;
            record_count = record_count.saturating_add(core.decode_frame(
                flags,
                frame.as_ref(),
                &mut builder,
            )?);
        }

        let records = builder.finish()?;
        Ok((records, record_count))
    }
}

#[derive(Default)]
struct MetricsDecoderCore {
    state: DirectDecoderState,
    current_resource: DirectDecResource,
    current_scope: DirectDecScope,
    current_metric: DirectDecMetric,
    current_attrs: Vec<DecodedAttribute>,
    current_point: DecPoint,
}

impl MetricsDecoderCore {
    fn decode_frame(
        &mut self,
        flags: u8,
        frame: &[u8],
        builder: &mut DirectOtapMetricsBuilder,
    ) -> Result<u64, Error> {
        if flags & !0b111 != 0 {
            return Err(Error::InvalidFrame("unknown frame flags"));
        }
        if flags & 0b001 != 0 {
            self.state.reset_dictionaries();
        }
        if flags & 0b100 != 0 {
            self.state.reset_codecs();
        }
        self.decode_data_frame(frame, builder)
    }

    fn decode_data_frame(
        &mut self,
        frame: &[u8],
        builder: &mut DirectOtapMetricsBuilder,
    ) -> Result<u64, Error> {
        let mut reader = SliceReader::new(frame);
        let record_count = reader.read_uvarint()?;
        let size_len = usize::try_from(reader.read_uvarint()?)
            .map_err(|_| Error::ValueOutOfRange("column size buffer"))?;
        let size_bytes = reader.read_bytes(size_len)?;
        let mut columns = decode_metrics_column_tree();
        let mut size_reader = BitReader::new(size_bytes);
        read_decode_column_sizes(
            &mut columns,
            &mut size_reader,
            reader.remaining_len() as u64,
        )?;
        read_decode_column_data(&mut columns, &mut reader)?;
        builder.reserve_number_points(
            usize::try_from(record_count).map_err(|_| Error::ValueOutOfRange("record count"))?,
        );

        let mut frame_decoder = DirectMetricsFrameDecoder::new(&columns, &self.state.codecs);
        for _ in 0..record_count {
            frame_decoder.decode_record(
                builder,
                &mut self.current_resource,
                &mut self.current_scope,
                &mut self.current_metric,
                &mut self.current_attrs,
                &mut self.current_point,
                &mut self.state,
            )?;
        }
        frame_decoder.save_codecs(&mut self.state.codecs);
        Ok(record_count)
    }
}

fn try_read_fixed_header(bytes: &[u8]) -> Result<Option<(super::StefCompression, usize)>, Error> {
    let mut reader = SliceReader::new(bytes);
    match read_fixed_header(&mut reader) {
        Ok(compression) => Ok(Some((compression, reader.position()))),
        Err(Error::UnexpectedEof) => Ok(None),
        Err(e) => Err(e),
    }
}

fn try_read_var_header_frame(
    frames: &mut FrameDecoder,
    bytes: &[u8],
) -> Result<Option<(u8, usize)>, Error> {
    let mut reader = SliceReader::new(bytes);
    match frames.read_frame(&mut reader) {
        Ok((flags, var_header)) => {
            read_var_header(var_header.as_ref())?;
            Ok(Some((flags, reader.position())))
        }
        Err(Error::UnexpectedEof) => Ok(None),
        Err(e) => Err(e),
    }
}

fn try_read_frame<'a>(
    frames: &mut FrameDecoder,
    bytes: &'a [u8],
) -> Result<Option<(u8, std::borrow::Cow<'a, [u8]>, usize)>, Error> {
    let mut reader = SliceReader::new(bytes);
    match frames.read_frame(&mut reader) {
        Ok((flags, frame)) => Ok(Some((flags, frame, reader.position()))),
        Err(Error::UnexpectedEof) => Ok(None),
        Err(e) => Err(e),
    }
}
