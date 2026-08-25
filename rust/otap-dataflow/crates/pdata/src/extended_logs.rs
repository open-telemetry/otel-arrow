// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Compact stacktrace extension for canonical OTAP log records.

use crate::otap::raw_batch_store::RawLogsStore;
use crate::otap::{Logs, OtapArrowRecords};
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use crate::representation::{PDataArrowRecordSet, RepresentationVersion};
use arrow::array::{Array, StringArray, UInt16Array, UInt32Array, UInt64Array};
use otel_arrow_dfe_config::SignalType;
use std::collections::BTreeMap;

/// Stable format ID for canonical OTAP logs with compact stacktrace tables.
pub const EXTENDED_LOGS_FORMAT_ID: &str = "otap.logs.extended";
/// Current extended-logs representation version.
pub const EXTENDED_LOGS_VERSION: RepresentationVersion = RepresentationVersion::new(1, 0);

/// Format-local table containing log-to-stack relations.
pub const LOG_STACKS_TABLE_ID: u32 = 100;
/// Format-local table containing ordered stack frames.
pub const STACK_FRAMES_TABLE_ID: u32 = 101;
/// Format-local table containing unique instruction-pointer locations.
pub const LOCATIONS_TABLE_ID: u32 = 102;
/// Format-local table containing resolved symbols.
pub const SYMBOLS_TABLE_ID: u32 = 103;

/// One resolved stack frame associated with an OTAP log row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtendedLogStackFrame {
    /// Instruction pointer.
    pub address: u64,
    /// Resolved function name.
    pub function_name: Option<String>,
    /// Resolved source filename.
    pub filename: Option<String>,
    /// Resolved source line.
    pub line: Option<u32>,
}

/// Validated view of an extended-log representation.
pub struct ExtendedLogsView<'a> {
    records: &'a PDataArrowRecordSet,
}

impl<'a> TryFrom<&'a PDataArrowRecordSet> for ExtendedLogsView<'a> {
    type Error = crate::error::Error;

    fn try_from(records: &'a PDataArrowRecordSet) -> Result<Self, Self::Error> {
        if records.format_id() != EXTENDED_LOGS_FORMAT_ID
            || records.version().major != EXTENDED_LOGS_VERSION.major
            || records.signal_type() != SignalType::Logs
        {
            return Err(crate::error::Error::UnsupportedRepresentation {
                format_id: records.format_id().to_owned(),
            });
        }
        Ok(Self { records })
    }
}

impl ExtendedLogsView<'_> {
    /// Reconstructs the canonical four-table OTAP logs representation.
    pub fn standard_logs(&self) -> crate::error::Result<OtapArrowRecords> {
        let mut raw = RawLogsStore::new();
        for payload_type in [
            ArrowPayloadType::ResourceAttrs,
            ArrowPayloadType::ScopeAttrs,
            ArrowPayloadType::Logs,
            ArrowPayloadType::LogAttrs,
        ] {
            if let Some(batch) = self.records.table(payload_type as u32) {
                raw.set(payload_type, batch.clone());
            }
        }
        Ok(Logs::try_from(raw)?.into())
    }

    /// Resolves compact extension tables into stacks keyed by zero-based row
    /// position in the canonical OTAP Logs table.
    pub fn stacks(&self) -> crate::error::Result<BTreeMap<u16, Vec<ExtendedLogStackFrame>>> {
        let Some(log_stacks) = self.records.table(LOG_STACKS_TABLE_ID) else {
            return Ok(BTreeMap::new());
        };
        let stack_frames = self.required_table(STACK_FRAMES_TABLE_ID, "stack frames")?;
        let locations = self.required_table(LOCATIONS_TABLE_ID, "locations")?;

        let log_ids = required_u16(log_stacks, "log_id")?;
        let stack_ids = required_u32(log_stacks, "stack_id")?;
        let frame_stack_ids = required_u32(stack_frames, "stack_id")?;
        let ordinals = required_u16(stack_frames, "ordinal")?;
        let location_ids = required_u32(stack_frames, "location_id")?;
        let ids = required_u32(locations, "id")?;
        let addresses = required_u64(locations, "address")?;

        let mut location_by_id = BTreeMap::new();
        for row in 0..locations.num_rows() {
            let _ = location_by_id.insert(ids.value(row), addresses.value(row));
        }

        let mut symbols_by_location = BTreeMap::new();
        if let Some(symbols) = self.records.table(SYMBOLS_TABLE_ID) {
            let symbol_location_ids = required_u32(symbols, "location_id")?;
            let function_names = optional_string(symbols, "function_name")?;
            let filenames = optional_string(symbols, "filename")?;
            let lines = optional_u32(symbols, "line")?;
            for row in 0..symbols.num_rows() {
                let _ = symbols_by_location.insert(
                    symbol_location_ids.value(row),
                    (
                        string_at(function_names, row),
                        string_at(filenames, row),
                        lines.and_then(|array| (!array.is_null(row)).then(|| array.value(row))),
                    ),
                );
            }
        }

        let mut frames_by_stack: BTreeMap<u32, Vec<(u16, ExtendedLogStackFrame)>> = BTreeMap::new();
        for row in 0..stack_frames.num_rows() {
            let location_id = location_ids.value(row);
            let (function_name, filename, line) = symbols_by_location
                .get(&location_id)
                .cloned()
                .unwrap_or_default();
            frames_by_stack
                .entry(frame_stack_ids.value(row))
                .or_default()
                .push((
                    ordinals.value(row),
                    ExtendedLogStackFrame {
                        address: *location_by_id.get(&location_id).ok_or_else(|| {
                            crate::error::Error::UnexpectedRecordBatchState {
                                reason: format!(
                                    "stack frame references missing location {location_id}"
                                ),
                            }
                        })?,
                        function_name,
                        filename,
                        line,
                    },
                ));
        }
        for frames in frames_by_stack.values_mut() {
            frames.sort_by_key(|(ordinal, _)| *ordinal);
        }

        let mut result = BTreeMap::new();
        for row in 0..log_stacks.num_rows() {
            let frames = frames_by_stack
                .get(&stack_ids.value(row))
                .cloned()
                .unwrap_or_default();
            let _ = result.insert(
                log_ids.value(row),
                frames.into_iter().map(|(_, frame)| frame).collect(),
            );
        }
        Ok(result)
    }

    fn required_table(
        &self,
        table_id: u32,
        table_name: &str,
    ) -> crate::error::Result<&arrow::array::RecordBatch> {
        self.records.table(table_id).ok_or_else(|| {
            crate::error::Error::UnexpectedRecordBatchState {
                reason: format!(
                    "extended logs with log stacks require the {table_name} table ({table_id})"
                ),
            }
        })
    }
}

fn required_u16<'a>(
    batch: &'a arrow::array::RecordBatch,
    name: &str,
) -> crate::error::Result<&'a UInt16Array> {
    batch
        .column_by_name(name)
        .and_then(|column| column.as_any().downcast_ref())
        .ok_or_else(|| crate::error::Error::UnexpectedRecordBatchState {
            reason: format!("extended logs table requires UInt16 column `{name}`"),
        })
}

fn required_u32<'a>(
    batch: &'a arrow::array::RecordBatch,
    name: &str,
) -> crate::error::Result<&'a UInt32Array> {
    batch
        .column_by_name(name)
        .and_then(|column| column.as_any().downcast_ref())
        .ok_or_else(|| crate::error::Error::UnexpectedRecordBatchState {
            reason: format!("extended logs table requires UInt32 column `{name}`"),
        })
}

fn required_u64<'a>(
    batch: &'a arrow::array::RecordBatch,
    name: &str,
) -> crate::error::Result<&'a UInt64Array> {
    batch
        .column_by_name(name)
        .and_then(|column| column.as_any().downcast_ref())
        .ok_or_else(|| crate::error::Error::UnexpectedRecordBatchState {
            reason: format!("extended logs table requires UInt64 column `{name}`"),
        })
}

fn optional_u32<'a>(
    batch: &'a arrow::array::RecordBatch,
    name: &str,
) -> crate::error::Result<Option<&'a UInt32Array>> {
    batch
        .column_by_name(name)
        .map(|column| {
            column.as_any().downcast_ref().ok_or_else(|| {
                crate::error::Error::UnexpectedRecordBatchState {
                    reason: format!("extended logs table requires UInt32 column `{name}`"),
                }
            })
        })
        .transpose()
}

fn optional_string<'a>(
    batch: &'a arrow::array::RecordBatch,
    name: &str,
) -> crate::error::Result<Option<&'a StringArray>> {
    batch
        .column_by_name(name)
        .map(|column| {
            column.as_any().downcast_ref().ok_or_else(|| {
                crate::error::Error::UnexpectedRecordBatchState {
                    reason: format!("extended logs table requires Utf8 column `{name}`"),
                }
            })
        })
        .transpose()
}

fn string_at(array: Option<&StringArray>, row: usize) -> Option<String> {
    array.and_then(|array| (!array.is_null(row)).then(|| array.value(row).to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::representation::PDataArrowTable;
    use arrow::array::RecordBatch;
    use arrow::datatypes::{DataType, Field, Schema};
    use std::sync::Arc;

    fn batch(fields: Vec<Field>, columns: Vec<arrow::array::ArrayRef>) -> RecordBatch {
        RecordBatch::try_new(Arc::new(Schema::new(fields)), columns).unwrap()
    }

    fn records(tables: Vec<PDataArrowTable>) -> PDataArrowRecordSet {
        PDataArrowRecordSet::new(
            EXTENDED_LOGS_FORMAT_ID,
            EXTENDED_LOGS_VERSION,
            SignalType::Logs,
            2,
            tables,
        )
        .unwrap()
    }

    /// Scenario: two log rows refer to the same compact stack ID.
    /// Guarantees: decoding preserves the complete ordered stack for both logs.
    #[test]
    fn shared_stack_is_available_to_every_log() {
        let records = records(vec![
            PDataArrowTable {
                table_id: LOG_STACKS_TABLE_ID,
                batch: batch(
                    vec![
                        Field::new("log_id", DataType::UInt16, false),
                        Field::new("stack_id", DataType::UInt32, false),
                    ],
                    vec![
                        Arc::new(UInt16Array::from(vec![4, 9])),
                        Arc::new(UInt32Array::from(vec![7, 7])),
                    ],
                ),
            },
            PDataArrowTable {
                table_id: STACK_FRAMES_TABLE_ID,
                batch: batch(
                    vec![
                        Field::new("stack_id", DataType::UInt32, false),
                        Field::new("ordinal", DataType::UInt16, false),
                        Field::new("location_id", DataType::UInt32, false),
                    ],
                    vec![
                        Arc::new(UInt32Array::from(vec![7, 7])),
                        Arc::new(UInt16Array::from(vec![1, 0])),
                        Arc::new(UInt32Array::from(vec![2, 1])),
                    ],
                ),
            },
            PDataArrowTable {
                table_id: LOCATIONS_TABLE_ID,
                batch: batch(
                    vec![
                        Field::new("id", DataType::UInt32, false),
                        Field::new("address", DataType::UInt64, false),
                    ],
                    vec![
                        Arc::new(UInt32Array::from(vec![1, 2])),
                        Arc::new(UInt64Array::from(vec![0x10, 0x20])),
                    ],
                ),
            },
        ]);

        let stacks = ExtendedLogsView::try_from(&records)
            .unwrap()
            .stacks()
            .unwrap();
        assert_eq!(
            stacks[&4]
                .iter()
                .map(|frame| frame.address)
                .collect::<Vec<_>>(),
            vec![0x10, 0x20]
        );
        assert_eq!(stacks[&4], stacks[&9]);
    }

    /// Scenario: a representation declares log-to-stack rows without frame tables.
    /// Guarantees: malformed stack extensions return an error instead of dropping stacks.
    #[test]
    fn partial_stack_extension_is_rejected() {
        let records = records(vec![PDataArrowTable {
            table_id: LOG_STACKS_TABLE_ID,
            batch: batch(
                vec![
                    Field::new("log_id", DataType::UInt16, false),
                    Field::new("stack_id", DataType::UInt32, false),
                ],
                vec![
                    Arc::new(UInt16Array::from(vec![0])),
                    Arc::new(UInt32Array::from(vec![0])),
                ],
            ),
        }]);

        let error = ExtendedLogsView::try_from(&records)
            .unwrap()
            .stacks()
            .unwrap_err();
        assert!(error.to_string().contains("stack frames table"));
    }
}
