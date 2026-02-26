// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Joining different data domains
//!
//! TODO
//! - better module documentation
use std::rc::Rc;
use std::sync::Arc;

use arrow::array::{Array, ArrayRef, Int32Array, RecordBatch, StructArray, UInt16Array};
use arrow::compute::take;
use arrow::datatypes::{DataType, Field, Fields, Schema};
use datafusion::logical_expr::ColumnarValue;
use otap_df_pdata::OtapArrowRecords;
use otap_df_pdata::arrays::get_required_struct_array;
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otap_df_pdata::schema::consts;

use crate::error::{Error, Result};
use crate::pipeline::expr::{
    DataDomainId, LEFT_COLUMN_NAME, PhysicalExprEvalResult, RIGHT_COLUMN_NAME,
};
use crate::pipeline::planner::AttributesIdentifier;

/// Two-level lookup structure for joining u16 IDs.
///
///
/// TODO the comments about memory efficiency aren't right here. This is efficient for
/// dense ranges that don't use the full ID range
///
/// This structure provides O(1) lookups while being memory-efficient for sparse ID ranges.
/// The u16 space (0-65535) is divided into 64 pages of 1024 entries each.
/// Pages are only allocated when IDs in that range are actually used.
///
/// # Memory Layout
/// - Outer array: 64 entries (top 6 bits of u16)
/// - Each page: 1024 entries (bottom 10 bits of u16)
/// - Each page is ~8KB (Option<usize> is 8 bytes on 64-bit systems)
///
/// # Example
/// For ID 5120 (binary: 00010100_00000000):
/// - Outer index: 5120 >> 10 = 5
/// - Inner index: 5120 & 0x3FF = 0
const PAGE_SIZE: usize = 1024;
const PAGE_BITS: u16 = 10;
const PAGE_MASK: u16 = 0x3FF; // Bottom 10 bits
const NUM_PAGES: usize = 64; // 2^16 / 2^10 = 2^6 = 64

struct IdJoinLookup {
    /// Two-level lookup: outer array indexed by top 6 bits, inner pages indexed by bottom 10 bits.
    /// Each page maps parent_id -> row index in the right-side batch.
    lookup: Vec<Option<Box<[Option<usize>; PAGE_SIZE]>>>,
}

// TODO eventually this will need to support u32 IDs
impl IdJoinLookup {
    /// Creates a new IdJoin from a slice of u16 parent IDs.
    ///
    /// # Arguments
    /// * `parent_ids` - The parent_id values from the right side of the join
    ///
    /// # Returns
    /// A lookup structure mapping parent_id -> row index
    fn new(parent_ids: &[u16]) -> Self {
        let mut lookup: Vec<Option<Box<[Option<usize>; PAGE_SIZE]>>> = vec![None; NUM_PAGES];

        for (row_idx, &parent_id) in parent_ids.iter().enumerate() {
            let outer = (parent_id >> PAGE_BITS) as usize;
            let inner = (parent_id & PAGE_MASK) as usize;

            // Allocate page if needed
            if lookup[outer].is_none() {
                lookup[outer] = Some(Box::new([None; PAGE_SIZE]));
            }

            // Store the mapping
            lookup[outer].as_mut().unwrap()[inner] = Some(row_idx);
        }

        Self { lookup }
    }

    /// Looks up a left-side ID and returns the corresponding right-side row index.
    ///
    /// # Arguments
    /// * `left_id` - The ID value from the left side to look up
    ///
    /// # Returns
    /// * `Some(row_idx)` - The row index in the right batch if a match exists
    /// * `None` - No matching parent_id found
    #[inline]
    fn lookup(&self, left_id: u16) -> Option<usize> {
        let outer = (left_id >> PAGE_BITS) as usize;
        let inner = (left_id & PAGE_MASK) as usize;

        self.lookup[outer].as_ref().and_then(|page| page[inner])
    }
}

// helper function for determining join order. In most of our join implementations, we build a
// lookup table for the right side, then scan the left. However, if the left:right
// relationship is one:many, we need do the join backwards to avoid ambiguity about which row on
// in the lookup corresponds with the row from the side we're scanning.
//
// one:many relationships include:
// - Resource -> Scope
// - Resource -> Log/Trace/Metric,
// - Scope -> Log/Trace/Metric
//
pub fn is_one_to_many(
    left_attrs_id: &AttributesIdentifier,
    right_attrs_id: &AttributesIdentifier,
) -> bool {
    match (left_attrs_id, right_attrs_id) {
        (AttributesIdentifier::Root, _) => false,
        (AttributesIdentifier::NonRoot(_), AttributesIdentifier::Root) => true,
        (AttributesIdentifier::NonRoot(left), AttributesIdentifier::NonRoot(right)) => {
            *left == ArrowPayloadType::ResourceAttrs && *right == ArrowPayloadType::ScopeAttrs
        }
    }
}

pub fn missing_column_err(column_name: &str) -> Error {
    Error::ExecutionError {
        cause: format!("Invalid record batch: missing required column {column_name}"),
    }
}

pub fn invalid_column_type_error(data_type: &DataType) -> Error {
    Error::ExecutionError {
        cause: format!("Invalid record batch. Expected u16 ID column, found {data_type:?}"),
    }
}

pub fn join<'a>(
    left: &'a PhysicalExprEvalResult,
    right: &'a PhysicalExprEvalResult,
    otap_batch: &'a OtapArrowRecords,
) -> Result<(RecordBatch, Rc<DataDomainId>)> {
    match (left.data_domain.as_ref(), right.data_domain.as_ref()) {
        (
            DataDomainId::Attributes(left_attrs_id, _),
            DataDomainId::Attributes(right_attrs_id, _),
        ) => {
            if left_attrs_id == right_attrs_id {
                let join_exec = AttributeToSameAttributeJoin {};
                let join_result = join_exec.join(left, right, otap_batch)?;
                Ok((join_result, left.data_domain.clone()))
            } else {
                if is_one_to_many(left_attrs_id, right_attrs_id) {
                    let join_exec = AttributeToDifferentAttributeReverseJoin {
                        left: left_attrs_id.clone(),
                        right: right_attrs_id.clone(),
                    };
                    let join_result = join_exec.join(left, right, otap_batch)?;
                    Ok((join_result, right.data_domain.clone()))
                } else {
                    let join_exec = AttributeToDifferentAttributeJoin {
                        left: left_attrs_id.clone(),
                        right: right_attrs_id.clone(),
                    };
                    let join_result = join_exec.join(left, right, otap_batch)?;
                    Ok((join_result, left.data_domain.clone()))
                }
            }
        }
        (DataDomainId::Root, DataDomainId::Attributes(attr_id, _)) => {
            let join_exec = RootToAttributesJoin {
                attrs_id: attr_id.clone(),
            };
            let join_result = join_exec.join(left, right, otap_batch)?;
            Ok((join_result, left.data_domain.clone()))
        }
        (DataDomainId::Attributes(attr_id, _), DataDomainId::Root) => match attr_id {
            AttributesIdentifier::Root => {
                let join_exec = RootAttrsToRootJoin {};
                let join_result = join_exec.join(left, right, otap_batch)?;
                Ok((join_result, left.data_domain.clone()))
            }
            AttributesIdentifier::NonRoot(payload_type) => {
                let join_exec = NonRootAttrsToRootReverseJoin {
                    attrs_payload_type: *payload_type,
                };
                let join_result = join_exec.join(left, right, otap_batch)?;
                Ok((join_result, right.data_domain.clone()))
            }
        },
        _ => {
            todo!()
        }
    }
}

fn to_join_result(left: &PhysicalExprEvalResult, right_col: ArrayRef) -> RecordBatch {
    // TODO preallocate
    let mut columns = Vec::new();
    let mut fields = Vec::new();

    match &left.values {
        ColumnarValue::Array(arr) => {
            fields.push(Field::new(LEFT_COLUMN_NAME, arr.data_type().clone(), true));
            columns.push(arr.clone());
        }
        _ => {
            // TODO - not sure this is reachable?
            todo!("scalar val in result conversion")
        }
    }

    fields.push(Field::new(
        RIGHT_COLUMN_NAME,
        right_col.data_type().clone(),
        true,
    ));
    columns.push(right_col);

    if let Some(ids) = &left.ids {
        fields.push(Field::new(consts::ID, ids.data_type().clone(), true));
        columns.push(ids.clone());
    }

    if let Some(parent_ids) = &left.parent_ids {
        fields.push(Field::new(
            consts::PARENT_ID,
            parent_ids.data_type().clone(),
            false,
        ));
        columns.push(parent_ids.clone());
    }

    if let Some(col) = &left.resource_ids {
        let struct_arr = StructArray::new(
            Fields::from(vec![Field::new(consts::ID, col.data_type().clone(), true)]),
            vec![col.clone()],
            None,
        );
        fields.push(Field::new(
            consts::RESOURCE,
            struct_arr.data_type().clone(),
            true,
        ));
        columns.push(Arc::new(struct_arr));
    }

    if let Some(col) = &left.scope_ids {
        let struct_arr = StructArray::new(
            Fields::from(vec![Field::new(consts::ID, col.data_type().clone(), true)]),
            vec![col.clone()],
            None,
        );
        fields.push(Field::new(
            consts::SCOPE,
            struct_arr.data_type().clone(),
            true,
        ));
        columns.push(Arc::new(struct_arr));
    }
    RecordBatch::try_new(Arc::new(Schema::new(fields)), columns)
        .map_err(|e| Error::ExecutionError {
            cause: format!("Failed to create record batch: {e}"),
        })
        .expect("Failed to create join result record batch")
}

trait JoinExec {
    fn join(
        &self,
        left: &PhysicalExprEvalResult,
        right: &PhysicalExprEvalResult,
        otap_batch: &OtapArrowRecords,
    ) -> Result<RecordBatch>;
}

// TODO this is almost xactly the same as AttributeToSameAttributeJoin
struct RootToAttributesJoin {
    attrs_id: AttributesIdentifier,
}

impl JoinExec for RootToAttributesJoin {
    fn join(
        &self,
        left: &PhysicalExprEvalResult,
        right: &PhysicalExprEvalResult,
        _otap_batch: &OtapArrowRecords,
    ) -> Result<RecordBatch> {
        let right_parent_ids = right
            .parent_ids
            .as_ref()
            .ok_or_else(|| missing_column_err(consts::PARENT_ID))?;
        let right_parent_ids = right_parent_ids
            .as_any()
            .downcast_ref::<UInt16Array>()
            .ok_or_else(|| invalid_column_type_error(right_parent_ids.data_type()))?;
        let right_lookup = IdJoinLookup::new(right_parent_ids.values());

        // TODO need to inspect the attr ID to figure out the right column to join on
        let left_id_col = match self.attrs_id {
            AttributesIdentifier::Root => &left.ids,
            AttributesIdentifier::NonRoot(payload_type) => match payload_type {
                ArrowPayloadType::ResourceAttrs => &left.resource_ids,
                ArrowPayloadType::ScopeAttrs => &left.scope_ids,
                _ => {
                    todo!()
                }
            },
        };
        let left_parent_ids = left_id_col
            .as_ref()
            .ok_or_else(|| missing_column_err(consts::ID))?;
        let left_parent_ids = left_parent_ids
            .as_any()
            .downcast_ref::<UInt16Array>()
            .ok_or_else(|| invalid_column_type_error(left_parent_ids.data_type()))?;

        let mut to_take = Int32Array::builder(left_parent_ids.len());

        left_parent_ids.iter().for_each(|id| {
            // TODO test if this null handling works ...
            if id.is_none() {
                to_take.append_null();
            } else {
                let left_id = id.expect("Checked for None above");
                let right_index = right_lookup.lookup(left_id).map(|i| i as i32);
                to_take.append_option(right_index);
            }
        });

        let right_values = right.values.to_array(right_parent_ids.len())?;
        let joined_arr = take(&right_values, &to_take.finish(), None)?;

        // TODO this is an innefficent way to do this
        Ok(to_join_result(&left, joined_arr))
    }
}

struct RootAttrsToRootJoin {}

impl JoinExec for RootAttrsToRootJoin {
    fn join(
        &self,
        left: &PhysicalExprEvalResult,
        right: &PhysicalExprEvalResult,
        _otap_batch: &OtapArrowRecords,
    ) -> Result<RecordBatch> {
        let right_ids = right
            .ids
            .as_ref()
            .ok_or_else(|| missing_column_err(consts::ID))?;
        let right_ids = right_ids
            .as_any()
            .downcast_ref::<UInt16Array>()
            .ok_or_else(|| invalid_column_type_error(right_ids.data_type()))?;

        // TODO this might not be right if there are null IDs!!
        let right_lookup = IdJoinLookup::new(right_ids.values());

        let left_parent_ids = left
            .parent_ids
            .as_ref()
            .ok_or_else(|| missing_column_err(consts::PARENT_ID))?;
        let left_parent_ids = left_parent_ids
            .as_any()
            .downcast_ref::<UInt16Array>()
            .ok_or_else(|| invalid_column_type_error(left_parent_ids.data_type()))?;

        let mut to_take = Int32Array::builder(left_parent_ids.len());

        left_parent_ids.iter().for_each(|parent_id| {
            if parent_id.is_none() {
                to_take.append_null();
            } else {
                let parent_id = parent_id.expect("Checked for None above");
                let right_index = right_lookup.lookup(parent_id).map(|i| i as i32);
                to_take.append_option(right_index);
            }
        });

        let right_values = right.values.to_array(right_ids.len())?;
        let joined_arr = take(&right_values, &to_take.finish(), None)?;

        Ok(to_join_result(left, joined_arr))
    }
}

struct NonRootAttrsToRootReverseJoin {
    attrs_payload_type: ArrowPayloadType,
}

impl JoinExec for NonRootAttrsToRootReverseJoin {
    fn join(
        &self,
        left: &PhysicalExprEvalResult,
        right: &PhysicalExprEvalResult,
        _otap_batch: &OtapArrowRecords,
    ) -> Result<RecordBatch> {
        let left_parent_ids = left
            .parent_ids
            .as_ref()
            .ok_or_else(|| missing_column_err(consts::PARENT_ID))?;
        let left_parent_ids = left_parent_ids
            .as_any()
            .downcast_ref::<UInt16Array>()
            .ok_or_else(|| invalid_column_type_error(left_parent_ids.data_type()))?;
        let left_lookup = IdJoinLookup::new(left_parent_ids.values());

        let right_ids = match self.attrs_payload_type {
            ArrowPayloadType::ResourceAttrs => right.resource_ids.as_ref(),
            ArrowPayloadType::ScopeAttrs => right.scope_ids.as_ref(),
            _ => {
                todo!()
            }
        };

        let right_ids = right_ids.ok_or_else(|| missing_column_err(consts::ID))?;
        let right_ids = right_ids
            .as_any()
            .downcast_ref::<UInt16Array>()
            .ok_or_else(|| invalid_column_type_error(right_ids.data_type()))?;

        // TODO - this could be computed by calling from_iter
        let mut to_take = Int32Array::builder(right_ids.len());
        right_ids.iter().for_each(|id| {
            if id.is_none() {
                to_take.append_null();
            } else {
                let right_id = id.expect("Checked for None above");
                let right_index = left_lookup.lookup(right_id).map(|i| i as i32);
                to_take.append_option(right_index);
            }
        });
        let to_take = to_take.finish();

        // TODO - rename
        // TODO - preallocate
        let mut new_fields = Vec::new();
        let mut new_columns = Vec::new();

        if let Some(col) = &right.ids {
            new_fields.push(Field::new(consts::ID, col.data_type().clone(), true));
            new_columns.push(col.clone());
        }

        if let Some(col) = &right.resource_ids {
            let struct_arr = StructArray::new(
                Fields::from(vec![Field::new(consts::ID, col.data_type().clone(), true)]),
                vec![col.clone()],
                None,
            );
            new_fields.push(Field::new(
                consts::RESOURCE,
                struct_arr.data_type().clone(),
                true,
            ));
            new_columns.push(Arc::new(struct_arr));
        }

        if let Some(col) = &right.scope_ids {
            let struct_arr = StructArray::new(
                Fields::from(vec![Field::new(consts::ID, col.data_type().clone(), true)]),
                vec![col.clone()],
                None,
            );
            new_fields.push(Field::new(
                consts::SCOPE,
                struct_arr.data_type().clone(),
                true,
            ));
            new_columns.push(Arc::new(struct_arr));
        }

        let left_values = left.values.to_array(100)?;
        let joined_vals = take(&left_values, &to_take, None)?;
        new_fields.push(Field::new(
            LEFT_COLUMN_NAME,
            joined_vals.data_type().clone(),
            true,
        ));
        new_columns.push(joined_vals);

        // TODO have a match, assert right.values is an array, and use the array instead of
        // callling to_array wiht a random length (which will be ignored b/c we should know
        // that this is an array at this point)
        let child_col = right.values.to_array(100)?;
        new_fields.push(Field::new(
            RIGHT_COLUMN_NAME,
            child_col.data_type().clone(),
            true,
        ));
        new_columns.push(child_col);

        RecordBatch::try_new(Arc::new(Schema::new(new_fields)), new_columns).map_err(|e| {
            Error::ExecutionError {
                cause: format!("Failed to create record batch: {e}"),
            }
        })
    }
}

struct AttributeToSameAttributeJoin {}

impl JoinExec for AttributeToSameAttributeJoin {
    fn join(
        &self,
        left: &PhysicalExprEvalResult,
        right: &PhysicalExprEvalResult,
        _otap_batch: &OtapArrowRecords,
    ) -> Result<RecordBatch> {
        // build the right join table

        let right_parent_ids = right
            .parent_ids
            .as_ref()
            .ok_or_else(|| missing_column_err(consts::PARENT_ID))?;
        let right_parent_ids = right_parent_ids
            .as_any()
            .downcast_ref::<UInt16Array>()
            .ok_or_else(|| invalid_column_type_error(right_parent_ids.data_type()))?;
        let right_lookup = IdJoinLookup::new(right_parent_ids.values());

        let left_parent_ids = left
            .parent_ids
            .as_ref()
            .ok_or_else(|| missing_column_err(consts::PARENT_ID))?;
        let left_parent_ids = left_parent_ids
            .as_any()
            .downcast_ref::<UInt16Array>()
            .ok_or_else(|| invalid_column_type_error(left_parent_ids.data_type()))?;

        let mut to_take = Int32Array::builder(left_parent_ids.len());

        left_parent_ids.iter().for_each(|id| {
            if let Some(left_id) = id {
                let right_index = right_lookup.lookup(left_id).map(|i| i as i32);
                to_take.append_option(right_index);
            } else {
                to_take.append_null();
            }
        });

        let right_values = right.values.to_array(right_parent_ids.len())?;
        let joined_arr = take(&right_values, &to_take.finish(), None)?;

        Ok(to_join_result(left, joined_arr))
    }
}

// Helper function to extract id column from root batch based on AttributesIdentifier
// Returns owned Vec<u16> to avoid lifetime issues with temporary struct arrays
fn get_attrs_id_values<'a>(
    root_batch: &'a RecordBatch,
    attrs_id: &'a AttributesIdentifier,
) -> Result<&'a UInt16Array> {
    match attrs_id {
        AttributesIdentifier::Root => {
            let id_col = root_batch
                .column_by_name(consts::ID)
                .ok_or_else(|| missing_column_err(consts::ID))?;
            Ok(id_col
                .as_any()
                .downcast_ref::<UInt16Array>()
                .ok_or_else(|| invalid_column_type_error(id_col.data_type()))?)
        }
        AttributesIdentifier::NonRoot(payload_type) => {
            match payload_type {
                ArrowPayloadType::ResourceAttrs => {
                    let resource_struct = get_required_struct_array(root_batch, consts::RESOURCE)
                        .map_err(|e| Error::ExecutionError {
                        cause: format!("Failed to get resource struct: {e}"),
                    })?;
                    let id_col = resource_struct
                        .column_by_name(consts::ID)
                        .ok_or_else(|| missing_column_err(consts::ID))?;
                    Ok(id_col
                        .as_any()
                        .downcast_ref::<UInt16Array>()
                        .ok_or_else(|| invalid_column_type_error(id_col.data_type()))?)
                }
                ArrowPayloadType::ScopeAttrs => {
                    let scope_struct = get_required_struct_array(root_batch, consts::SCOPE)
                        .map_err(|e| Error::ExecutionError {
                            cause: format!("Failed to get scope struct: {e}"),
                        })?;
                    let id_col = scope_struct
                        .column_by_name(consts::ID)
                        .ok_or_else(|| missing_column_err(consts::ID))?;
                    Ok(id_col
                        .as_any()
                        .downcast_ref::<UInt16Array>()
                        .ok_or_else(|| invalid_column_type_error(id_col.data_type()))?)
                }
                _ => Err(Error::ExecutionError {
                    cause: "Unsupported attribute type".to_string(),
                }),
            }
        }
    }
}

struct AttributeToDifferentAttributeJoin {
    left: AttributesIdentifier,
    right: AttributesIdentifier,
}

impl JoinExec for AttributeToDifferentAttributeJoin {
    fn join(
        &self,
        left: &PhysicalExprEvalResult,
        right: &PhysicalExprEvalResult,
        otap_batch: &OtapArrowRecords,
    ) -> Result<RecordBatch> {
        // Two-hop join through root batch
        // Example: scope.attributes["x"] + resource.attributes["y"]
        // Path: left.parent_id (scope id) -> log.scope.id -> log.resource.id -> right.parent_id (resource id)

        // Step 1: Build lookup from right side (resource attributes)
        let right_parent_ids = right
            .parent_ids
            .as_ref()
            .ok_or_else(|| missing_column_err(consts::PARENT_ID))?;
        let right_parent_ids = right_parent_ids
            .as_any()
            .downcast_ref::<UInt16Array>()
            .ok_or_else(|| invalid_column_type_error(right_parent_ids.data_type()))?;
        let right_lookup = IdJoinLookup::new(right_parent_ids.values());

        // Step 2: Get root batch and extract the id columns we need
        let root_batch = otap_batch
            .root_record_batch()
            .ok_or_else(|| Error::ExecutionError {
                cause: "Missing root record batch".to_string(),
            })?;

        // Step 3: Get the left and right id columns from root batch based on attribute identifiers
        // TODO not sure I love the method name here ...
        let left_root_ids = get_attrs_id_values(&root_batch, &self.left)?;
        let right_root_ids = get_attrs_id_values(&root_batch, &self.right)?;

        // // Step 4: Build mapping from left id -> right id using root batch as bridge
        // // Collect right_ids indexed by position, then build IdJoinLookup with left_ids
        let inter_join_lookup = IdJoinLookup::new(left_root_ids.values());

        // Step 5: For each left row, find corresponding right row
        let left_parent_ids = left
            .parent_ids
            .as_ref()
            .ok_or_else(|| missing_column_err(consts::PARENT_ID))?;
        let left_parent_ids = left_parent_ids
            .as_any()
            .downcast_ref::<UInt16Array>()
            .ok_or_else(|| invalid_column_type_error(left_parent_ids.data_type()))?;

        let mut to_take = Int32Array::builder(left_parent_ids.len());

        left_parent_ids.iter().for_each(|left_parent_id_opt| {
            // TODO - could be re-written in a way where there's not so much crappy null handling
            // TODO - we should have unit tests covering all these append_null cases
            if let Some(left_parent_id) = left_parent_id_opt {
                if let Some(root_index) = inter_join_lookup.lookup(left_parent_id) {
                    if right_root_ids.is_valid(root_index) {
                        let right_id = right_root_ids.value(root_index);
                        if let Some(right_index) = right_lookup.lookup(right_id) {
                            to_take.append_value(right_index as i32);
                        } else {
                            to_take.append_null();
                        }
                    } else {
                        to_take.append_null();
                    }
                } else {
                    to_take.append_null();
                }
            } else {
                to_take.append_null();
            }
        });

        // Step 6: Take from right values
        let right_values = right.values.to_array(right_parent_ids.len())?;
        let joined_arr = take(&right_values, &to_take.finish(), None)?;

        Ok(to_join_result(left, joined_arr))
    }
}

struct AttributeToDifferentAttributeReverseJoin {
    left: AttributesIdentifier,
    right: AttributesIdentifier,
}

impl JoinExec for AttributeToDifferentAttributeReverseJoin {
    fn join(
        &self,
        left: &PhysicalExprEvalResult,
        right: &PhysicalExprEvalResult,
        otap_batch: &OtapArrowRecords,
    ) -> Result<RecordBatch> {
        // TODO these comments are copied from somewhere else so might not be super relevant
        // Two-hop join through root batch
        // Example: scope.attributes["x"] + resource.attributes["y"]
        // Path: left.parent_id (scope id) -> log.scope.id -> log.resource.id -> right.parent_id (resource id)

        let left_parent_ids = left
            .parent_ids
            .as_ref()
            .ok_or_else(|| missing_column_err(consts::PARENT_ID))?;
        let left_parent_ids = left_parent_ids
            .as_any()
            .downcast_ref::<UInt16Array>()
            .ok_or_else(|| invalid_column_type_error(left_parent_ids.data_type()))?;
        let left_lookup = IdJoinLookup::new(left_parent_ids.values());

        // Step 2: Get root batch and extract the id columns we need
        let root_batch = otap_batch
            .root_record_batch()
            .ok_or_else(|| Error::ExecutionError {
                cause: "Missing root record batch".to_string(),
            })?;

        // Step 3: Get the left and right id columns from root batch based on attribute identifiers
        // TODO not sure I love the method name here ...
        // For reverse join, we swap the sides when looking up root IDs
        let left_root_ids = get_attrs_id_values(&root_batch, &self.left)?;
        let right_root_ids = get_attrs_id_values(&root_batch, &self.right)?;

        // Step 4: For reverse join, iterate through right side
        let right_parent_ids = right
            .parent_ids
            .as_ref()
            .ok_or_else(|| missing_column_err(consts::PARENT_ID))?;
        let right_parent_ids = right_parent_ids
            .as_any()
            .downcast_ref::<UInt16Array>()
            .ok_or_else(|| invalid_column_type_error(right_parent_ids.data_type()))?;

        let mut to_take = Int32Array::builder(left_parent_ids.len());

        // Build intermediate lookup for reverse join
        let inter_join_lookup = IdJoinLookup::new(right_root_ids.values());

        right_parent_ids.iter().for_each(|right_parent_id_opt| {
            // TODO - could be re-written in a way where there's not so much crappy null handling
            // TODO - we should have unit tests covering all these append_null cases
            if let Some(right_parent_id) = right_parent_id_opt {
                if let Some(root_index) = inter_join_lookup.lookup(right_parent_id) {
                    if left_root_ids.is_valid(root_index) {
                        let left_id = left_root_ids.value(root_index);
                        if let Some(left_index) = left_lookup.lookup(left_id) {
                            to_take.append_value(left_index as i32);
                        } else {
                            to_take.append_null();
                        }
                    } else {
                        to_take.append_null();
                    }
                } else {
                    to_take.append_null();
                }
            } else {
                to_take.append_null();
            }
        });
        let to_take = to_take.finish();

        let mut fields = Vec::with_capacity(3);
        let mut columns = Vec::with_capacity(3);

        fields.push(Field::new(consts::PARENT_ID, DataType::UInt16, false));
        columns.push(
            right
                .parent_ids
                .as_ref()
                .ok_or_else(|| missing_column_err(consts::PARENT_ID))?
                .clone(),
        );

        let left_values = left.values.to_array(100)?;
        let joined_vals = take(&left_values, &to_take, None)?;
        fields.push(Field::new(
            LEFT_COLUMN_NAME,
            joined_vals.data_type().clone(),
            true,
        ));
        columns.push(joined_vals);

        // TODO have a match, assert right.values is an array, and use the array instead of
        // callling to_array wiht a random length (which will be ignored b/c we should know
        // that this is an array at this point)
        let child_col = right.values.to_array(100)?;
        fields.push(Field::new(
            RIGHT_COLUMN_NAME,
            child_col.data_type().clone(),
            true,
        ));
        columns.push(child_col);

        RecordBatch::try_new(Arc::new(Schema::new(fields)), columns).map_err(|e| {
            Error::ExecutionError {
                cause: format!("Failed to create record batch: {e}"),
            }
        })
    }
}
