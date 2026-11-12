// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Borrowed native-log fields and reusable attribute indexes for codec encoders.
//!
//! These helpers preserve native hierarchy and attribute-row order without
//! constructing the generic view's per-parent attribute maps. Callers keep
//! indexes in their encoder and reset them after both successful and failed batches.

use arrow::array::{
    Array, BooleanArray, Float64Array, Int64Array, StringArray, StructArray, UInt8Array,
    UInt16Array,
};
use arrow::datatypes::{DataType, Fields};
use otel_arrow_dfe_pdata::OtapArrowRecords;
use otel_arrow_dfe_pdata::arrays::{
    ByteArrayAccessor, MaybeDictArrayAccessor, NullableArrayAccessor,
    get_required_array_from_struct_array,
};
use otel_arrow_dfe_pdata::error::Error;
use otel_arrow_dfe_pdata::otlp::attributes::AttributeValueType;
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_dfe_pdata::schema::consts;
use otel_arrow_dfe_pdata::views::otap::OtapLogsView;
use otel_arrow_dfe_pdata::views::otap::common::OtapAnyValueView;

/// Reusable lookup of attribute rows by a decoded 16-bit parent ID.
#[derive(Default)]
pub struct AttributeIndex {
    rows: Vec<(u16, usize)>,
}

impl AttributeIndex {
    /// Rebuilds the index, preserving original row order within each parent.
    ///
    /// Null parents are excluded. Passing `None` clears the previous batch.
    pub fn rebuild(&mut self, columns: Option<&AttributeColumns<'_>>) {
        self.rows.clear();
        if let Some(columns) = columns {
            self.rows.extend(
                (0..columns.types.len())
                    .filter_map(|row| columns.parents.value_at(row).map(|id| (id, row))),
            );
            // Including the row preserves duplicate-key precedence for scattered parents.
            self.rows.sort_unstable();
        }
    }

    /// Returns `(parent ID, original row)` entries in original row order.
    #[must_use]
    pub fn rows_for(&self, parent: u16) -> &[(u16, usize)] {
        let start = self.rows.partition_point(|(id, _)| *id < parent);
        let end = start + self.rows[start..].partition_point(|(id, _)| *id == parent);
        &self.rows[start..end]
    }

    /// Clears batch state, releasing storage above the supplied byte limit.
    ///
    /// This limits retained scratch, not temporary growth while indexing input.
    pub fn reset(&mut self, max_retained_bytes: usize) {
        self.rows.clear();
        if self.rows.capacity() > max_retained_bytes / size_of::<(u16, usize)>() {
            self.rows = Vec::new();
        }
    }
}

/// Cached borrowed accessors for a native 16-bit-parent attribute batch.
pub struct AttributeColumns<'a> {
    parents: MaybeDictArrayAccessor<'a, UInt16Array>,
    keys: MaybeDictArrayAccessor<'a, StringArray>,
    types: &'a UInt8Array,
    strings: Option<MaybeDictArrayAccessor<'a, StringArray>>,
    integers: Option<MaybeDictArrayAccessor<'a, Int64Array>>,
    doubles: Option<&'a Float64Array>,
    booleans: Option<&'a BooleanArray>,
    bytes: Option<ByteArrayAccessor<'a>>,
    serialized: Option<ByteArrayAccessor<'a>>,
}

fn column<'a, T: Array + 'static>(
    columns: &'a StructArray,
    name: &str,
    expected: DataType,
) -> Result<Option<&'a T>, Error> {
    columns
        .column_by_name(name)
        .map(|array| {
            array
                .as_any()
                .downcast_ref::<T>()
                .ok_or_else(|| Error::ColumnDataTypeMismatch {
                    name: name.into(),
                    expect: expected,
                    actual: array.data_type().clone(),
                })
        })
        .transpose()
}

impl<'a> AttributeColumns<'a> {
    /// Validates column types and caches native or dictionary-backed accessors.
    pub fn new(array: &'a StructArray) -> Result<Self, Error> {
        Ok(Self {
            parents: MaybeDictArrayAccessor::<UInt16Array>::try_new(
                get_required_array_from_struct_array(array, consts::PARENT_ID)?,
            )?,
            keys: MaybeDictArrayAccessor::<StringArray>::try_new(
                get_required_array_from_struct_array(array, consts::ATTRIBUTE_KEY)?,
            )?,
            types: column(array, consts::ATTRIBUTE_TYPE, DataType::UInt8)?.ok_or_else(|| {
                Error::ColumnNotFound {
                    name: consts::ATTRIBUTE_TYPE.into(),
                }
            })?,
            strings: array
                .column_by_name(consts::ATTRIBUTE_STR)
                .map(MaybeDictArrayAccessor::<StringArray>::try_new)
                .transpose()?,
            integers: array
                .column_by_name(consts::ATTRIBUTE_INT)
                .map(MaybeDictArrayAccessor::<Int64Array>::try_new)
                .transpose()?,
            doubles: column(array, consts::ATTRIBUTE_DOUBLE, DataType::Float64)?,
            booleans: column(array, consts::ATTRIBUTE_BOOL, DataType::Boolean)?,
            bytes: array
                .column_by_name(consts::ATTRIBUTE_BYTES)
                .map(ByteArrayAccessor::try_new)
                .transpose()?,
            serialized: array
                .column_by_name(consts::ATTRIBUTE_SER)
                .map(ByteArrayAccessor::try_new)
                .transpose()?,
        })
    }

    fn value(&self, row: usize) -> OtapAnyValueView<'_> {
        match self
            .types
            .value_at(row)
            .and_then(|kind| AttributeValueType::try_from(kind).ok())
        {
            Some(AttributeValueType::Str) => OtapAnyValueView::Str(
                self.strings
                    .as_ref()
                    .and_then(|col| col.str_at(row))
                    .unwrap_or("")
                    .as_bytes(),
            ),
            Some(AttributeValueType::Int) => OtapAnyValueView::Int(
                self.integers
                    .as_ref()
                    .and_then(|col| col.value_at(row))
                    .unwrap_or(0),
            ),
            Some(AttributeValueType::Double) => OtapAnyValueView::Double(
                self.doubles
                    .and_then(|col| col.value_at(row))
                    .unwrap_or(0.0),
            ),
            Some(AttributeValueType::Bool) => OtapAnyValueView::Bool(
                self.booleans
                    .and_then(|col| col.value_at(row))
                    .unwrap_or(false),
            ),
            Some(AttributeValueType::Bytes) => OtapAnyValueView::Bytes(
                self.bytes
                    .as_ref()
                    .and_then(|col| col.slice_at(row))
                    .unwrap_or(b""),
            ),
            Some(AttributeValueType::Map | AttributeValueType::Slice) => self
                .serialized
                .as_ref()
                .and_then(|col| col.slice_at(row))
                .map_or(OtapAnyValueView::Empty, OtapAnyValueView::Serialized),
            _ => OtapAnyValueView::Empty,
        }
    }

    /// Borrows keys and values for rows returned by the matching attribute index.
    ///
    /// Null keys are omitted. Missing/null optional values use the same defaults
    /// as the canonical native view. References borrow Arrow storage, not copies.
    pub fn attributes<'b>(
        &'b self,
        rows: &'b [(u16, usize)],
    ) -> impl Iterator<Item = (&'b [u8], OtapAnyValueView<'b>)> {
        rows.iter().filter_map(|(_, row)| {
            self.keys
                .str_at(*row)
                .map(|key| (key.as_bytes(), self.value(*row)))
        })
    }
}

/// Native logs with separate attribute columns and an attribute-free root view.
///
/// The canonical view supplies root/body validation and hierarchy traversal.
/// Use the separate attribute accessors with [`AttributeIndex`] instead of the
/// root view's attribute iterators, which are intentionally empty.
pub struct NativeLogs {
    roots: OtapArrowRecords,
    log_attributes: Option<StructArray>,
    resource_attributes: Option<StructArray>,
    scope_attributes: Option<StructArray>,
    ids: Option<UInt16Array>,
    resource_ids: Option<UInt16Array>,
    scope_ids: Option<UInt16Array>,
    count: usize,
}

impl NativeLogs {
    /// Decodes transport IDs in place and cheaply clones the native containers.
    ///
    /// Original attribute batches remain in `records`. Column/accessor failures
    /// are returned without a codec identity so each caller can attach its own.
    pub fn new(records: &mut OtapArrowRecords) -> Result<Self, Error> {
        records.decode_transport_optimized_ids()?;
        let mut roots = records.clone();
        let log_attributes = roots
            .remove(ArrowPayloadType::LogAttrs)
            .map(StructArray::from);
        let resource_attributes = roots
            .remove(ArrowPayloadType::ResourceAttrs)
            .map(StructArray::from);
        let scope_attributes = roots
            .remove(ArrowPayloadType::ScopeAttrs)
            .map(StructArray::from);
        let root = roots
            .get(ArrowPayloadType::Logs)
            .ok_or(Error::RecordBatchNotFound {
                payload_type: ArrowPayloadType::Logs,
            })?;
        let columns = StructArray::from(root.clone());
        let resource: &StructArray = column(
            &columns,
            consts::RESOURCE,
            DataType::Struct(Fields::empty()),
        )?
        .ok_or_else(|| Error::ColumnNotFound {
            name: consts::RESOURCE.into(),
        })?;
        let scope: &StructArray =
            column(&columns, consts::SCOPE, DataType::Struct(Fields::empty()))?.ok_or_else(
                || Error::ColumnNotFound {
                    name: consts::SCOPE.into(),
                },
            )?;
        let ids = column::<UInt16Array>(&columns, consts::ID, DataType::UInt16)?.cloned();
        let resource_ids = column::<UInt16Array>(resource, consts::ID, DataType::UInt16)?.cloned();
        let scope_ids = column::<UInt16Array>(scope, consts::ID, DataType::UInt16)?.cloned();
        let count = root.num_rows();
        Ok(Self {
            roots,
            log_attributes,
            resource_attributes,
            scope_attributes,
            ids,
            resource_ids,
            scope_ids,
            count,
        })
    }

    /// Returns the canonical hierarchy/root/body view without attribute maps.
    pub fn view(&self) -> Result<OtapLogsView<'_>, Error> {
        OtapLogsView::try_from(&self.roots)
    }

    /// Returns borrowed log attributes, validating their column types.
    pub fn log_attributes(&self) -> Result<Option<AttributeColumns<'_>>, Error> {
        self.log_attributes
            .as_ref()
            .map(AttributeColumns::new)
            .transpose()
    }

    /// Returns borrowed resource attributes, validating their column types.
    pub fn resource_attributes(&self) -> Result<Option<AttributeColumns<'_>>, Error> {
        self.resource_attributes
            .as_ref()
            .map(AttributeColumns::new)
            .transpose()
    }

    /// Returns borrowed scope attributes, validating their column types.
    pub fn scope_attributes(&self) -> Result<Option<AttributeColumns<'_>>, Error> {
        self.scope_attributes
            .as_ref()
            .map(AttributeColumns::new)
            .transpose()
    }

    /// Looks up the attribute parent ID for an original physical log row.
    #[must_use]
    pub fn log_id(&self, row: usize) -> Option<u16> {
        self.ids.value_at(row)
    }

    /// Rebuilds reusable `(resource ID, scope ID, physical row)` traversal order.
    ///
    /// Matches the canonical root view: resource ID, scope ID, then original row.
    /// Rows with missing/null resource or scope IDs are omitted by both paths.
    pub fn order_rows(&self, rows: &mut Vec<(u16, u16, usize)>) {
        rows.clear();
        rows.extend((0..self.count).filter_map(|row| {
            Some((
                self.resource_ids.value_at(row)?,
                self.scope_ids.value_at(row)?,
                row,
            ))
        }));
        rows.sort_unstable();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::{ArrayRef, DictionaryArray};
    use arrow::datatypes::{Field, UInt8Type, UInt16Type};
    use std::sync::Arc;

    fn attributes(parents: ArrayRef, keys: ArrayRef, values: ArrayRef) -> StructArray {
        let types = Arc::new(UInt8Array::from(vec![
            AttributeValueType::Str as u8;
            parents.len()
        ])) as ArrayRef;
        StructArray::from(
            [
                (consts::PARENT_ID, parents),
                (consts::ATTRIBUTE_KEY, keys),
                (consts::ATTRIBUTE_TYPE, types),
                (consts::ATTRIBUTE_STR, values),
            ]
            .into_iter()
            .map(|(name, array)| {
                (
                    Arc::new(Field::new(name, array.data_type().clone(), true)),
                    array,
                )
            })
            .collect::<Vec<_>>(),
        )
    }

    /// Scenario: Attribute parents are scattered, dictionary encoded, and separated by null keys or values.
    /// Guarantees: Both dictionary widths preserve duplicate row order while excluding null parents and keys; small indexes reuse storage.
    #[test]
    fn attribute_index_handles_dictionary_nulls_and_scattered_rows() {
        let parents = UInt16Array::from(vec![Some(9), Some(2), None, Some(9), Some(2), Some(9)]);
        let keys = StringArray::from(vec![
            Some("message"),
            Some("message"),
            Some("ignored"),
            None,
            Some("message"),
            Some("message"),
        ]);
        let values = StringArray::from(vec![
            Some("nine-first"),
            Some("two-first"),
            Some("ignored"),
            Some("ignored"),
            Some("two-last"),
            Some("nine-last"),
        ]);
        for key_type in [DataType::UInt8, DataType::UInt16] {
            let parent_values = Arc::new(UInt16Array::from(vec![Some(9), Some(2), None]));
            let parents: ArrayRef = if key_type == DataType::UInt8 {
                Arc::new(DictionaryArray::<UInt8Type>::new(
                    UInt8Array::from(vec![
                        Some(0),
                        Some(1),
                        None,
                        Some(0),
                        Some(1),
                        Some(0),
                        Some(2),
                    ]),
                    parent_values,
                ))
            } else {
                Arc::new(DictionaryArray::<UInt16Type>::new(
                    UInt16Array::from(vec![
                        Some(0),
                        Some(1),
                        None,
                        Some(0),
                        Some(1),
                        Some(0),
                        Some(2),
                    ]),
                    parent_values,
                ))
            };
            // The final row references a null dictionary value, not a null dictionary key.
            let keys = arrow::compute::cast(
                &StringArray::from_iter(keys.iter().chain([Some("ignored")])),
                &DataType::Dictionary(Box::new(key_type.clone()), Box::new(DataType::Utf8)),
            )
            .unwrap();
            let values = arrow::compute::cast(
                &StringArray::from_iter(values.iter().chain([Some("ignored")])),
                &DataType::Dictionary(Box::new(key_type), Box::new(DataType::Utf8)),
            )
            .unwrap();
            let array = attributes(parents, keys, values);
            let columns = AttributeColumns::new(&array).unwrap();
            let mut index = AttributeIndex::default();
            index.rebuild(Some(&columns));
            for (id, expected) in [
                (2, [b"two-first".as_slice(), b"two-last".as_slice()]),
                (9, [b"nine-first".as_slice(), b"nine-last".as_slice()]),
            ] {
                let values: Vec<_> = columns
                    .attributes(index.rows_for(id))
                    .map(|(key, value)| {
                        assert_eq!(key, b"message");
                        let OtapAnyValueView::Str(value) = value else {
                            panic!("expected borrowed string");
                        };
                        value
                    })
                    .collect();
                assert_eq!(values, expected);
            }
            assert_eq!(index.rows_for(9), &[(9, 0), (9, 3), (9, 5)]);
            assert_eq!(index.rows.len(), 5);
            assert!(index.rows_for(0).is_empty());
        }
        let array = attributes(Arc::new(parents), Arc::new(keys), Arc::new(values));
        let columns = AttributeColumns::new(&array).unwrap();
        let mut index = AttributeIndex::default();
        index.rebuild(Some(&columns));
        assert_eq!(index.rows_for(2), &[(2, 1), (2, 4)]);
        let pointer = index.rows.as_ptr();
        index.reset(256);
        index.rebuild(Some(&columns));
        assert_eq!(index.rows.as_ptr(), pointer);
        index.rebuild(None);
        assert!(index.rows_for(2).is_empty());
    }

    /// Scenario: An attribute-heavy batch grows index scratch beyond a caller's retention limit.
    /// Guarantees: Oversized scratch is released and no parent matches survive a reset.
    #[test]
    fn attribute_index_bounds_retained_scratch() {
        let limit = 256;
        let mut index = AttributeIndex {
            rows: vec![(7, 0); limit / size_of::<(u16, usize)>() + 1],
        };
        assert!(!index.rows_for(7).is_empty());
        index.reset(limit);
        assert_eq!(index.rows.capacity(), 0);
        assert!(index.rows_for(7).is_empty());
    }
}
