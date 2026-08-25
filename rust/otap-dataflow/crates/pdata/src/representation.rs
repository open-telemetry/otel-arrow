// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Named and versioned pluggable pdata representations.

use crate::otap::memory;
use arrow::array::RecordBatch;
use bytes::Bytes;
use otel_arrow_dfe_config::SignalType;
use std::collections::BTreeMap;

/// Version of a pluggable representation contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RepresentationVersion {
    /// Incompatible contract version.
    pub major: u16,
    /// Backward-compatible contract version.
    pub minor: u16,
}

impl RepresentationVersion {
    /// Creates a representation version.
    #[must_use]
    pub const fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }
}

/// Error constructing a pluggable representation.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum RepresentationError {
    /// The format ID is empty.
    #[error("pluggable representation format ID must not be empty")]
    EmptyFormatId,
    /// The media type is empty.
    #[error("pluggable byte representation media type must not be empty")]
    EmptyMediaType,
    /// A format-local Arrow table ID occurs more than once.
    #[error("duplicate pluggable Arrow table ID {table_id}")]
    DuplicateTableId {
        /// Duplicate format-local table ID.
        table_id: u32,
    },
}

/// One format-local table in a pluggable Arrow representation.
#[derive(Clone, Debug, PartialEq)]
pub struct PDataArrowTable {
    /// Identifier interpreted only by the named representation contract.
    pub table_id: u32,
    /// Arrow batch for this table.
    pub batch: RecordBatch,
}

/// A named, versioned set of format-local Arrow tables.
#[derive(Clone, Debug, PartialEq)]
pub struct PDataArrowRecordSet {
    format_id: String,
    version: RepresentationVersion,
    signal: SignalType,
    root_items: usize,
    tables: BTreeMap<u32, RecordBatch>,
}

impl PDataArrowRecordSet {
    /// Creates a validated pluggable Arrow record set.
    pub fn new(
        format_id: impl Into<String>,
        version: RepresentationVersion,
        signal: SignalType,
        root_items: usize,
        tables: impl IntoIterator<Item = PDataArrowTable>,
    ) -> Result<Self, RepresentationError> {
        let format_id = format_id.into();
        if format_id.is_empty() {
            return Err(RepresentationError::EmptyFormatId);
        }

        let mut table_map = BTreeMap::new();
        for table in tables {
            if table_map.insert(table.table_id, table.batch).is_some() {
                return Err(RepresentationError::DuplicateTableId {
                    table_id: table.table_id,
                });
            }
        }

        Ok(Self {
            format_id,
            version,
            signal,
            root_items,
            tables: table_map,
        })
    }

    /// Stable representation contract ID.
    #[must_use]
    pub fn format_id(&self) -> &str {
        &self.format_id
    }

    /// Representation contract version.
    #[must_use]
    pub const fn version(&self) -> RepresentationVersion {
        self.version
    }

    /// Semantic signal covered by this initial single-signal carrier.
    #[must_use]
    pub const fn signal_type(&self) -> SignalType {
        self.signal
    }

    /// Format-defined number of root items.
    #[must_use]
    pub const fn num_items(&self) -> usize {
        self.root_items
    }

    /// Returns a format-local table.
    #[must_use]
    pub fn table(&self, table_id: u32) -> Option<&RecordBatch> {
        self.tables.get(&table_id)
    }

    /// Iterates over format-local tables in table-ID order.
    #[must_use]
    pub fn tables(&self) -> impl ExactSizeIterator<Item = (u32, &RecordBatch)> {
        self.tables.iter().map(|(id, batch)| (*id, batch))
    }

    /// Logical Arrow bytes across all tables.
    pub fn logical_arrow_bytes(&self) -> crate::error::Result<usize> {
        self.tables.values().try_fold(0usize, |total, batch| {
            let batch_bytes = memory::record_batch_logical_bytes(batch)
                .map_err(|source| crate::error::Error::LogicalArrowSize { source })?;
            total
                .checked_add(batch_bytes)
                .ok_or_else(|| crate::error::Error::LogicalArrowSize {
                    source: arrow::error::ArrowError::ComputeError(
                        "integer overflow computing pluggable Arrow byte size".to_owned(),
                    ),
                })
        })
    }

    /// Deduplicated retained Arrow buffer capacity across all tables.
    #[must_use]
    pub fn retained_memory_bytes(&self) -> usize {
        let mut seen = memory::CountedAllocations::default();
        self.tables
            .values()
            .map(|batch| memory::record_batch_pinned_bytes(batch, &mut seen))
            .sum()
    }

    pub(crate) fn take_payload(&mut self) -> Self {
        Self {
            format_id: self.format_id.clone(),
            version: self.version,
            signal: self.signal,
            root_items: std::mem::take(&mut self.root_items),
            tables: std::mem::take(&mut self.tables),
        }
    }
}

/// A named, versioned byte representation carried without interpretation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PDataBytes {
    format_id: String,
    version: RepresentationVersion,
    media_type: String,
    signal: SignalType,
    num_items: usize,
    bytes: Bytes,
}

impl PDataBytes {
    /// Creates a validated pluggable byte representation.
    pub fn new(
        format_id: impl Into<String>,
        version: RepresentationVersion,
        media_type: impl Into<String>,
        signal: SignalType,
        num_items: usize,
        bytes: Bytes,
    ) -> Result<Self, RepresentationError> {
        let format_id = format_id.into();
        if format_id.is_empty() {
            return Err(RepresentationError::EmptyFormatId);
        }
        let media_type = media_type.into();
        if media_type.is_empty() {
            return Err(RepresentationError::EmptyMediaType);
        }
        Ok(Self {
            format_id,
            version,
            media_type,
            signal,
            num_items,
            bytes,
        })
    }

    /// Stable representation contract ID.
    #[must_use]
    pub fn format_id(&self) -> &str {
        &self.format_id
    }

    /// Representation contract version.
    #[must_use]
    pub const fn version(&self) -> RepresentationVersion {
        self.version
    }

    /// Media type of the encoded bytes.
    #[must_use]
    pub fn media_type(&self) -> &str {
        &self.media_type
    }

    /// Semantic signal covered by this initial single-signal carrier.
    #[must_use]
    pub const fn signal_type(&self) -> SignalType {
        self.signal
    }

    /// Format-defined item count.
    #[must_use]
    pub const fn num_items(&self) -> usize {
        self.num_items
    }

    /// Encoded representation bytes.
    #[must_use]
    pub const fn bytes(&self) -> &Bytes {
        &self.bytes
    }

    pub(crate) fn take_payload(&mut self) -> Self {
        Self {
            format_id: self.format_id.clone(),
            version: self.version,
            media_type: self.media_type.clone(),
            signal: self.signal,
            num_items: std::mem::take(&mut self.num_items),
            bytes: std::mem::take(&mut self.bytes),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::UInt32Array;
    use arrow::datatypes::{DataType, Field, Schema};
    use std::sync::Arc;

    fn batch(values: Vec<u32>) -> RecordBatch {
        RecordBatch::try_new(
            Arc::new(Schema::new(vec![Field::new(
                "value",
                DataType::UInt32,
                false,
            )])),
            vec![Arc::new(UInt32Array::from(values))],
        )
        .unwrap()
    }

    /// Scenario: a pluggable Arrow record set contains two distinct local table IDs.
    /// Guarantees: table lookup, root-item accounting, and memory accounting cover both tables.
    #[test]
    fn arrow_record_set_accounts_for_all_tables() {
        let records = PDataArrowRecordSet::new(
            "example.logs",
            RepresentationVersion::new(1, 0),
            SignalType::Logs,
            3,
            [
                PDataArrowTable {
                    table_id: 7,
                    batch: batch(vec![1, 2, 3]),
                },
                PDataArrowTable {
                    table_id: 42,
                    batch: batch(vec![4]),
                },
            ],
        )
        .unwrap();

        assert_eq!(records.format_id(), "example.logs");
        assert_eq!(records.num_items(), 3);
        assert_eq!(records.tables().len(), 2);
        assert_eq!(records.table(7).unwrap().num_rows(), 3);
        assert!(records.logical_arrow_bytes().unwrap() > 0);
        assert!(records.retained_memory_bytes() > 0);
    }

    /// Scenario: two Arrow tables use the same format-local table ID.
    /// Guarantees: construction fails instead of replacing one table silently.
    #[test]
    fn arrow_record_set_rejects_duplicate_table_ids() {
        let result = PDataArrowRecordSet::new(
            "example.logs",
            RepresentationVersion::new(1, 0),
            SignalType::Logs,
            1,
            [
                PDataArrowTable {
                    table_id: 7,
                    batch: batch(vec![1]),
                },
                PDataArrowTable {
                    table_id: 7,
                    batch: batch(vec![2]),
                },
            ],
        );

        assert_eq!(
            result.unwrap_err(),
            RepresentationError::DuplicateTableId { table_id: 7 }
        );
    }
}
