// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::OtlpProtoBytes;
use crate::error::{Error, Result};
use crate::proto::consts::wire_types;
use crate::views::otlp::bytes::decode::{read_len_delim, read_varint};
use otap_df_config::SignalType;
use std::num::NonZeroU64;

/// Position within the inputs (input index and byte offset)
#[derive(Copy, Clone, PartialEq)]
struct Position {
    input_idx: usize,
    offset: usize,
}

impl Position {
    fn new(input_idx: usize, offset: usize) -> Self {
        Self { input_idx, offset }
    }
}

/// Definition of the batch size in bytes
fn batch_bytes(start_pos: &Position, end_pos: &Position, inputs: &[OtlpProtoBytes]) -> usize {
    if start_pos.input_idx == end_pos.input_idx {
        end_pos.offset - start_pos.offset
    } else {
        (inputs[start_pos.input_idx].byte_size() - start_pos.offset)
            + (start_pos.input_idx + 1..end_pos.input_idx)
                .map(|i| inputs[i].byte_size())
                .sum::<usize>()
            + end_pos.offset
    }
}

/// Helper to finalize batch by copying bytes from start to end positions
fn finalize_batch(
    signal: SignalType,
    inputs: &[OtlpProtoBytes],
    start_pos: &Position,
    end_pos: &Position,
    outputs: &mut Vec<OtlpProtoBytes>,
) {
    // Calculate exact size needed
    let size = batch_bytes(&start_pos, &end_pos, inputs);

    if size == 0 {
        return;
    }

    let mut batch = Vec::with_capacity(size);

    if start_pos.input_idx == end_pos.input_idx {
        // Single input
        let buf = inputs[start_pos.input_idx].as_bytes();
        batch.extend_from_slice(&buf[start_pos.offset..end_pos.offset]);
    } else {
        // Multiple inputs
        let buf = inputs[start_pos.input_idx].as_bytes();
        batch.extend_from_slice(&buf[start_pos.offset..]);

        for idx in (start_pos.input_idx + 1)..end_pos.input_idx {
            batch.extend_from_slice(inputs[idx].as_bytes());
        }

        let buf = inputs[end_pos.input_idx].as_bytes();
        batch.extend_from_slice(&buf[..end_pos.offset]);
    }

    outputs.push(OtlpProtoBytes::new_from_bytes(signal, batch))
}

/// Combines OTLP content by concatenation of bytes. Because we have a
/// top-level repeated field, this is precisely correct.
pub fn make_bytes_batches(
    signal: SignalType,
    max_bytes: Option<NonZeroU64>,
    inputs: Vec<OtlpProtoBytes>,
) -> Result<Vec<OtlpProtoBytes>> {
    if inputs.is_empty() {
        return Err(Error::EmptyBatch);
    }
    let total_size: usize = inputs.iter().map(|i| i.byte_size()).sum();
    if total_size == 0 {
        return Err(Error::EmptyBatch);
    }
    if inputs.len() == 1 {
        return Ok(inputs);
    }
    match max_bytes {
        None => Ok(vec![OtlpProtoBytes::new_from_bytes(
            signal,
            inputs
                .into_iter()
                .fold(Vec::with_capacity(total_size), |mut acc, record| {
                    acc.extend_from_slice(record.as_bytes());
                    acc
                }),
        )]),
        Some(max_nz) => {
            let max_size = max_nz.get() as usize;
            let mut batches = Vec::new();

            // Track current position and batch boundaries
            let mut batch_start = Position::new(0, 0);
            let mut current_size = 0;
            let mut current_pos = Position::new(0, 0);

            while current_pos.input_idx < inputs.len() {
                // Step through top-level fields in the current input
                loop {
                    // If we're at end-of-file, the read_varint will error, which
                    // gives the invalid wire type, causes skipping to EOF.
                    let field_start = current_pos;
                    let (field_end_offset, input_size) = {
                        let buf = inputs[current_pos.input_idx].as_bytes();

                        // All cases that encounter an error skip to EOF.
                        let input_size = buf.len();

                        let (tag, next_pos) = read_varint(buf, current_pos.offset)
                            .unwrap_or((wire_types::INVALID, input_size));

                        let wire_type = tag & wire_types::PROTOBUF_TAG_BITMASK;

                        // Try to skip field value to find field end
                        let field_end = match wire_type {
                            wire_types::VARINT => {
                                if let Some((_, pos)) = read_varint(buf, next_pos) {
                                    pos
                                } else {
                                    input_size
                                }
                            }
                            wire_types::LEN => {
                                if let Some((_, pos)) = read_len_delim(buf, next_pos) {
                                    pos
                                } else {
                                    input_size
                                }
                            }
                            wire_types::FIXED64 => {
                                if next_pos + 8 <= input_size {
                                    next_pos + 8
                                } else {
                                    input_size
                                }
                            }
                            wire_types::FIXED32 => {
                                if next_pos + 4 <= input_size {
                                    next_pos + 4
                                } else {
                                    input_size
                                }
                            }
                            _ => input_size,
                        };

                        (field_end, input_size)
                    };
                    let field_size = field_end_offset - field_start.offset;
                    debug_assert_ne!(field_size, 0);

                    let field_end = Position::new(current_pos.input_idx, field_end_offset);

                    // Advance the current position
                    current_pos = field_end;
                    if current_pos.offset == input_size {
                        current_pos.offset = 0;
                        current_pos.input_idx += 1;
                    }

                    if field_size + current_size <= max_size {
                        // If this field fits without becoming
                        // over-full, batch_start is unchanged.
                    } else if current_size != 0 {
                        // Flush the pending batch b/c the new field
                        // does not fit.
                        finalize_batch(signal, &inputs, &batch_start, &field_start, &mut batches);
                        batch_start = field_start;
                        current_size = 0;
                    }

                    // Increment size counter
                    current_size += field_size;
                }
            }

            // Finish last batch if exists
            if current_pos != batch_start {
                finalize_batch(signal, &inputs, &batch_start, &current_pos, &mut batches);
            }

            Ok(batches)
        }
    }
}
