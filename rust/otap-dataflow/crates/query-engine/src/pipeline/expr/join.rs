// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Joining different data domains
//!
//! TODO
//! - better module documentation
use std::sync::Arc;

use arrow::array::{Array, ArrayRef, Int32Array, RecordBatch, StructArray, UInt16Array};
use arrow::compute::take;
use arrow::datatypes::{DataType, Field, Fields, Schema};
use otap_df_pdata::arrays::get_required_struct_array;
use otap_df_pdata::proto::opentelemetry::arrow::v1::{ArrowPayload, ArrowPayloadType};
use otap_df_pdata::schema::consts;

use crate::error::{Error, Result};
use crate::pipeline::expr::{CHILD_COLUMN_NAME, DataDomainId, PhysicalExprEvalResult};
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

pub fn join(
    left: &RecordBatch,
    left_domain: &DataDomainId,
    right: &PhysicalExprEvalResult,
    right_domain: &DataDomainId,
) -> Result<(RecordBatch, DataDomainId)> {
    // TODO - find a way to avoid cloning all the data domains here
    match (left_domain, right_domain) {
        (DataDomainId::Attributes(attrs_id, _), DataDomainId::Attributes(attrs_id2, _)) => {
            if attrs_id == attrs_id2 {
                let join_exec = AttributeToSameAttributeJoin {};
                let join_result = join_exec.join(left, right)?;
                Ok((join_result, left_domain.clone()))
            } else {
                todo!()
            }
        }
        (DataDomainId::Root, DataDomainId::Attributes(attr_id, _)) => {
            // TODO do something with attr ID
            let join_exec = RootToAttribtuesJoin {
                // TODO - should the type just be copy?
                // TODO - constructor instead?
                attrs_id: attr_id.clone(),
            };
            let join_result = join_exec.join(left, right)?;
            Ok((join_result, left_domain.clone()))
        }
        (DataDomainId::Attributes(attr_id, _), DataDomainId::Root) => {
            match attr_id {
                AttributesIdentifier::Root => {
                    let join_exec = RootAttrsToRootJoin {};
                    let join_result = join_exec.join(left, right).unwrap();
                    Ok((join_result, left_domain.clone()))
                }
                AttributesIdentifier::NonRoot(payload_type) => {
                    let join_exec = NonRootAttrsToRootReverseJoin {
                        attrs_payload_type: *payload_type,
                    };
                    let join_result = join_exec.join(left, right).unwrap();
                    Ok((join_result, right_domain.clone()))
                }
            }
            // TODO do something with attr ID
            // let join_exec = RootAttrsToRootJoin {};
            // join_exec.join(left, right)
            // todo!()
        }
        _ => {
            todo!()
        }
    }
}

fn insert_joined_column(left: &RecordBatch, col: ArrayRef) -> RecordBatch {
    let mut fields = left.schema().fields().to_vec();
    fields.push(Arc::new(Field::new(
        CHILD_COLUMN_NAME,
        col.data_type().clone(),
        true,
    )));

    let mut columns = left.columns().to_vec();
    columns.push(col);

    // TODO expect instead of unwrap
    return RecordBatch::try_new(Arc::new(Schema::new(fields)), columns).unwrap();
}

trait JoinExec {
    fn join(&self, left: &RecordBatch, right: &PhysicalExprEvalResult) -> Result<RecordBatch>;
}

// TODO this is almost xactly the same as AttributeToSameAttributeJoin
struct RootToAttribtuesJoin {
    attrs_id: AttributesIdentifier,
}

// TODO - is there a world where we should join from the smaller side?
impl JoinExec for RootToAttribtuesJoin {
    fn join(&self, left: &RecordBatch, right: &PhysicalExprEvalResult) -> Result<RecordBatch> {
        let right_parent_ids = right
            .parent_ids
            .as_ref()
            .unwrap() // TODO no unwrap - return err
            .as_any()
            .downcast_ref::<UInt16Array>()
            .unwrap(); // TODO no unwrap - need return error if unexpected type
        let right_lookup = IdJoinLookup::new(right_parent_ids.values());

        // TODO need to inspect the attr ID to figure out the right column to join on
        let left_id_col = match self.attrs_id {
            AttributesIdentifier::Root => left.column_by_name(consts::ID),
            AttributesIdentifier::NonRoot(payload_type) => match payload_type {
                ArrowPayloadType::ResourceAttrs => {
                    // TODO - handle case this isn't present
                    let resource_col = get_required_struct_array(left, consts::RESOURCE).unwrap();
                    resource_col.column_by_name(consts::ID)
                }
                ArrowPayloadType::ScopeAttrs => {
                    // TODO - handle case this isn't present
                    let scope_col = get_required_struct_array(left, consts::SCOPE).unwrap();
                    scope_col.column_by_name(consts::ID)
                }
                _ => {
                    todo!()
                }
            },
        };
        let left_parent_ids = left_id_col
            .unwrap() // TODO - handle case column missing
            .as_any()
            .downcast_ref::<UInt16Array>()
            .unwrap(); // TODO no unwrap - need return err if unexpected type

        let mut to_take = Int32Array::builder(left_parent_ids.len());

        left_parent_ids.iter().for_each(|id| {
            // TODO test if this null handling works ...
            if id.is_none() {
                to_take.append_null();
            } else {
                // TODO crappy option handling
                let left_id = id.unwrap();
                let right_index = right_lookup.lookup(left_id).map(|i| i as i32);
                to_take.append_option(right_index);
            }
        });

        // TODO no unwrap
        let right_values = right.values.to_array(right_parent_ids.len()).unwrap();
        // TODO no unwrap
        let joined_arr = take(&right_values, &to_take.finish(), None).unwrap();

        Ok(insert_joined_column(left, joined_arr))
    }
}

struct RootAttrsToRootJoin {}

impl JoinExec for RootAttrsToRootJoin {
    fn join(&self, left: &RecordBatch, right: &PhysicalExprEvalResult) -> Result<RecordBatch> {
        let right_ids = right.ids.as_ref().unwrap(); // TODO no unwrap - return err

        // right.parent_ids actually contains the id values for root domain
        // TODO: this naming is confusing - maybe need to refactor PhysicalExprEvalResult
        let right_id_array = right_ids.as_any().downcast_ref::<UInt16Array>().unwrap(); // TODO no unwrap - need return error if unexpected type
        // TODO this might not be right if there are null IDs!!
        let right_lookup = IdJoinLookup::new(right_id_array.values());

        // Left is attributes batch, scan its parent_id column
        let left_parent_ids = left
            .column_by_name(consts::PARENT_ID)
            .unwrap() // TODO no unwrap - return err
            .as_any()
            .downcast_ref::<UInt16Array>()
            .unwrap(); // TODO no unwrap - need return err if unexpected type

        let mut to_take = Int32Array::builder(left_parent_ids.len());

        left_parent_ids.iter().for_each(|parent_id| {
            // TODO this one shouldn't need the null handling actually
            // TODO test if this null handling works ...
            if parent_id.is_none() {
                to_take.append_null();
            } else {
                // TODO crappy option handling
                let parent_id = parent_id.unwrap();
                let right_index = right_lookup.lookup(parent_id).map(|i| i as i32);
                to_take.append_option(right_index);
            }
        });

        // TODO no unwrap
        let right_values = right.values.to_array(right_id_array.len()).unwrap();
        // TODO no unwrap
        let joined_arr = take(&right_values, &to_take.finish(), None).unwrap();

        Ok(insert_joined_column(left, joined_arr))
    }
}

// TODO comments
struct NonRootAttrsToRootReverseJoin {
    attrs_payload_type: ArrowPayloadType,
}

impl JoinExec for NonRootAttrsToRootReverseJoin {
    fn join(&self, left: &RecordBatch, right: &PhysicalExprEvalResult) -> Result<RecordBatch> {
        let left_parent_ids = left
            .column_by_name(consts::PARENT_ID)
            .unwrap()
            .as_any()
            .downcast_ref::<UInt16Array>()
            .unwrap();
        let left_lookup = IdJoinLookup::new(left_parent_ids.values());

        let right_ids = match self.attrs_payload_type {
            ArrowPayloadType::ResourceAttrs => right.resource_ids.as_ref(),
            ArrowPayloadType::ScopeAttrs => right.scope_ids.as_ref(),
            _ => {
                todo!()
            }
        };

        let right_ids = right_ids
            // TODO no unwrap, handle case where ID column missing
            // although, if this were None, that means there would be no attributes
            // which would mean that left recordbatch shouldn't exist ...
            .unwrap()
            .as_any()
            .downcast_ref::<UInt16Array>()
            .unwrap(); // TODO handle invalid batch

        // TODO - this could be computed by calling from_iter
        let mut to_take = Int32Array::builder(right_ids.len());
        right_ids.iter().for_each(|id| {
            if id.is_none() {
                to_take.append_null();
            } else {
                // TODO crappy option handling
                let right_id = id.unwrap();
                let right_index = left_lookup.lookup(right_id).map(|i| i as i32);
                to_take.append_option(right_index);
            }
        });
        let to_take = to_take.finish();

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

        // TODO no unwrap
        let joined_vals = take(left.column_by_name("value").unwrap(), &to_take, None).unwrap();
        new_fields.push(Field::new("value", joined_vals.data_type().clone(), true));
        new_columns.push(joined_vals);

        // TODO have a match, assert right.values is an array, and use the array instead of
        // callling to_array wiht a random length (which will be ignored b/c we should know
        // that this is an array at this point)
        let child_col = right.values.to_array(100).unwrap();
        new_fields.push(Field::new(
            CHILD_COLUMN_NAME,
            child_col.data_type().clone(),
            true,
        ));
        new_columns.push(child_col);

        println!("new cols = {new_columns:?}");
        println!("new_fields = {new_fields:?}");

        Ok(RecordBatch::try_new(Arc::new(Schema::new(new_fields)), new_columns).unwrap())
    }
}

struct AttributeToSameAttributeJoin {}

impl JoinExec for AttributeToSameAttributeJoin {
    fn join(&self, left: &RecordBatch, right: &PhysicalExprEvalResult) -> Result<RecordBatch> {
        // build the right join table

        let right_parent_ids = right
            .parent_ids
            .as_ref()
            .unwrap() // TODO no unwrap - return err
            .as_any()
            .downcast_ref::<UInt16Array>()
            .unwrap(); // TODO no unwrap - need return error if unexpected type
        let right_lookup = IdJoinLookup::new(right_parent_ids.values());

        let left_parent_ids = left
            .column_by_name(consts::PARENT_ID)
            .unwrap() // TODO no unwrap - return err
            .as_any()
            .downcast_ref::<UInt16Array>()
            .unwrap(); // TODO no unwrap - need return err if unexpected type

        let mut to_take = Int32Array::builder(left_parent_ids.len());

        left_parent_ids.iter().for_each(|id| {
            // TODO no unwrap, handle nulls even though they're not supposed to be here
            let left_id = id.unwrap();
            let right_index = right_lookup.lookup(left_id).map(|i| i as i32);
            to_take.append_option(right_index);
        });

        // TODO no unwrap
        let right_values = right.values.to_array(right_parent_ids.len()).unwrap();
        // TODO no unwrap
        let joined_arr = take(&right_values, &to_take.finish(), None).unwrap();

        Ok(insert_joined_column(left, joined_arr))
    }
}
