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
    BitReader, SliceReader, decode_metrics_column_tree, read_decode_column_data,
    read_decode_column_sizes, read_fixed_header, read_frame, read_var_header,
};

/// Decodes a complete STEF metrics byte stream directly into OTAP metrics Arrow records.
pub fn decode_metrics_otap(bytes: &[u8]) -> Result<OtapArrowRecords, Error> {
    decode_metrics_otap_with_count(bytes).map(|(records, _)| records)
}

/// Decodes a complete STEF metrics byte stream directly into OTAP records and a STEF record count.
pub fn decode_metrics_otap_with_count(bytes: &[u8]) -> Result<(OtapArrowRecords, u64), Error> {
    MetricsOtapStreamDecoder::decode(bytes)
}

#[derive(Default)]
struct MetricsOtapStreamDecoder<'a> {
    state: DirectDecoderState<'a>,
    builder: DirectOtapMetricsBuilder<'a>,
    current_resource: DirectDecResource<'a>,
    current_scope: DirectDecScope<'a>,
    current_metric: DirectDecMetric<'a>,
    current_attrs: Vec<DecodedAttribute<'a>>,
    current_point: DecPoint,
}

impl<'a> MetricsOtapStreamDecoder<'a> {
    fn decode(bytes: &'a [u8]) -> Result<(OtapArrowRecords, u64), Error> {
        let mut input = SliceReader::new(bytes);
        read_fixed_header(&mut input)?;
        let (_, var_header) = read_frame(&mut input)?;
        read_var_header(var_header)?;

        let mut decoder = Self::default();
        let mut record_count = 0_u64;
        while !input.is_empty() {
            let (flags, frame) = read_frame(&mut input)?;
            if flags & !0b111 != 0 {
                return Err(Error::InvalidFrame("unknown frame flags"));
            }
            if flags & 0b001 != 0 {
                decoder.state.reset_dictionaries();
            }
            if flags & 0b100 != 0 {
                decoder.state.reset_codecs();
            }
            record_count = record_count.saturating_add(decoder.decode_data_frame(frame)?);
        }

        let records = decoder.builder.finish()?;
        Ok((records, record_count))
    }

    fn decode_data_frame(&mut self, frame: &'a [u8]) -> Result<u64, Error> {
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
        self.builder.reserve_number_points(
            usize::try_from(record_count).map_err(|_| Error::ValueOutOfRange("record count"))?,
        );

        let mut frame_decoder = DirectMetricsFrameDecoder::new(&columns);
        for _ in 0..record_count {
            frame_decoder.decode_record(
                &mut self.builder,
                &mut self.current_resource,
                &mut self.current_scope,
                &mut self.current_metric,
                &mut self.current_attrs,
                &mut self.current_point,
                &mut self.state,
            )?;
        }
        Ok(record_count)
    }
}
