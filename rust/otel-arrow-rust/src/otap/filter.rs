// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module focuses on taking a logs payload and filtering it based on the provided specifications
//!

use self::logs::LogMatchProperties;
use crate::arrays::{get_required_array, get_required_array_from_struct_array_from_record_batch};
use crate::otap::OtapArrowRecords;
use crate::otap::error::{self, Result};
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use crate::schema::consts;
use arrow::array::{Array, BooleanArray, UInt16Array};
use arrow::datatypes::DataType;
use serde::Deserialize;
use snafu::OptionExt;
use std::collections::HashSet;
use std::sync::Arc;

pub mod logs;

/// struct that describes the overall requirements to use in order to filter logs
#[derive(Debug, Clone, Deserialize)]
pub struct LogFilter {
    // Include match properties describe logs that should be included in the Collector Service pipeline,
    // all other logs should be dropped from further processing.
    // If both Include and Exclude are specified, Include filtering occurs first.
    include: LogMatchProperties,
    // Exclude match properties describe logs that should be excluded from the Collector Service pipeline,
    // all other logs should be included.
    // If both Include and Exclude are specified, Include filtering occurs first.
    exclude: LogMatchProperties,

    // LogConditions is a list of OTTL conditions for an ottllog context.
    // If any condition resolves to true, the log event will be dropped.
    // Supports `and`, `or`, and `()`
    #[allow(dead_code)]
    log_record: Vec<String>,
}

/// enum that allows a field to have any type
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum AnyValue {
    /// string type
    String(String),
    /// int type
    Int(i64),
    /// double type
    Double(f64),
    /// boolean type
    Boolean(bool),
    /// array of any type
    Array(Vec<AnyValue>),
    /// keyvalue type
    KeyValue(Vec<KeyValue>),
}

/// struct that represents attributes and other key value pairs
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct KeyValue {
    key: String,
    value: AnyValue,
}

impl LogFilter {
    /// create a new log filter
    #[must_use]
    pub fn new(
        include: LogMatchProperties,
        exclude: LogMatchProperties,
        log_record: Vec<String>,
    ) -> Self {
        Self {
            include,
            exclude,
            log_record,
        }
    }

    /// take a logs payload and return the filtered result
    pub fn filter(&self, mut logs_payload: OtapArrowRecords) -> Result<OtapArrowRecords> {
        // we get the filters
        let (include_resource_attr_filter, include_log_record_filter, include_log_attr_filter) =
            self.include.create_filters(&logs_payload, false)?;
        let (exclude_resource_attr_filter, exclude_log_record_filter, exclude_log_attr_filter) =
            self.exclude.create_filters(&logs_payload, true)?;

        // combine the include and exclude filters
        let resource_attr_filter =
            arrow::compute::and(&include_resource_attr_filter, &exclude_resource_attr_filter)
                .expect("boolean arrays should have equal length");
        let log_record_filter =
            arrow::compute::and(&include_log_record_filter, &exclude_log_record_filter)
                .expect("boolean arrays should have equal length");
        let log_attr_filter =
            arrow::compute::and(&include_log_attr_filter, &exclude_log_attr_filter)
                .expect("boolean arrays should have equal length");

        let (resource_attr_filter, scope_attr_filter, log_record_filter, log_attr_filter) = self
            .sync_up_filters(
                &logs_payload,
                resource_attr_filter,
                log_record_filter,
                log_attr_filter,
            )?;

        // get the record batches we are going to filter
        let resource_attrs = logs_payload
            .get(ArrowPayloadType::ResourceAttrs)
            .context(error::LogRecordNotFoundSnafu)?;
        let log_records = logs_payload
            .get(ArrowPayloadType::Logs)
            .context(error::LogRecordNotFoundSnafu)?;
        let log_attrs = logs_payload
            .get(ArrowPayloadType::LogAttrs)
            .context(error::LogRecordNotFoundSnafu)?;
        let scope_attrs = logs_payload
            .get(ArrowPayloadType::ScopeAttrs)
            .context(error::LogRecordNotFoundSnafu)?;

        // apply filters to the logs
        let filtered_resource_attrs =
            arrow::compute::filter_record_batch(resource_attrs, &resource_attr_filter)
                .expect("columns should have equal length");
        let filtered_log_records =
            arrow::compute::filter_record_batch(log_records, &log_record_filter)
                .expect("columns should have equal length");
        let filtered_log_attrs = arrow::compute::filter_record_batch(log_attrs, &log_attr_filter)
            .expect("columns should have equal length");
        let filtered_scope_attrs =
            arrow::compute::filter_record_batch(scope_attrs, &scope_attr_filter)
                .expect("columns should have equal length");

        logs_payload.set(ArrowPayloadType::ResourceAttrs, filtered_resource_attrs);
        logs_payload.set(ArrowPayloadType::Logs, filtered_log_records);
        logs_payload.set(ArrowPayloadType::LogAttrs, filtered_log_attrs);
        logs_payload.set(ArrowPayloadType::ScopeAttrs, filtered_scope_attrs);

        Ok(logs_payload)
    }

    fn sync_up_filters(
        &self,
        logs_payload: &OtapArrowRecords,
        mut resource_attr_filter: BooleanArray,
        mut log_record_filter: BooleanArray,
        mut log_attr_filter: BooleanArray,
    ) -> Result<(BooleanArray, BooleanArray, BooleanArray, BooleanArray)> {
        // get the record batches we are going to filter
        let resource_attrs = logs_payload
            .get(ArrowPayloadType::ResourceAttrs)
            .context(error::LogRecordNotFoundSnafu)?;
        let log_records = logs_payload
            .get(ArrowPayloadType::Logs)
            .context(error::LogRecordNotFoundSnafu)?;
        let log_attrs = logs_payload
            .get(ArrowPayloadType::LogAttrs)
            .context(error::LogRecordNotFoundSnafu)?;
        let scope_attrs = logs_payload
            .get(ArrowPayloadType::ScopeAttrs)
            .context(error::LogRecordNotFoundSnafu)?;

        // get the id columns from each record batch
        let resource_attr_parent_ids_column =
            get_required_array(resource_attrs, consts::PARENT_ID)?;
        let scope_attr_parent_ids_column = get_required_array(scope_attrs, consts::PARENT_ID)?;
        let log_attr_parent_ids_column = get_required_array(log_attrs, consts::PARENT_ID)?;
        let log_record_ids_column = get_required_array(log_records, consts::ID)?;
        let log_record_resource_ids_column =
            get_required_array_from_struct_array_from_record_batch(
                log_records,
                consts::RESOURCE,
                consts::ID,
            )?;
        let log_record_scope_ids_column = get_required_array_from_struct_array_from_record_batch(
            log_records,
            consts::SCOPE,
            consts::ID,
        )?;

        // starting with the resource_attr
        // -> get ids being removed
        // -> map ids to resource_ids in log_record
        // -> create filter to remove those resource_ids
        // -> update log_record filter
        let inverse_resource_attr_filter =
            arrow::compute::not(&resource_attr_filter).expect("not doesn't fail");

        // get set of ids that are being removed from resource_attr
        let resource_attr_parent_ids_removed = self.get_removed_ids(
            resource_attr_parent_ids_column,
            &inverse_resource_attr_filter,
            consts::PARENT_ID,
        )?;
        // create filter to remove these ids from log_record
        let log_record_resource_ids_filter = self.build_id_filter(
            log_record_resource_ids_column,
            resource_attr_parent_ids_removed,
        );
        // update the log_record_filter
        log_record_filter =
            arrow::compute::and(&log_record_filter, &log_record_resource_ids_filter)
                .expect("boolean arrays should have equal length");

        // repeat with ids from log_attrs
        let inverse_log_attr_filter =
            arrow::compute::not(&log_attr_filter).expect("not doesn't fail");
        let log_attr_parent_ids_removed = self.get_removed_ids(
            log_attr_parent_ids_column,
            &inverse_log_attr_filter,
            consts::PARENT_ID,
        )?;
        let log_record_ids_filter =
            self.build_id_filter(log_record_ids_column, log_attr_parent_ids_removed);
        log_record_filter = arrow::compute::and(&log_record_filter, &log_record_ids_filter)
            .expect("boolean arrays should have equal length");

        // now using the updated log_record_filter we need to update the rest of the filers
        let inverse_log_record_filter =
            arrow::compute::not(&log_record_filter).expect("not doesn't fail");
        let log_record_ids_removed = self.get_removed_ids(
            log_record_ids_column,
            &inverse_log_record_filter,
            consts::ID,
        )?;
        let log_attr_parent_ids_filter =
            self.build_id_filter(log_attr_parent_ids_column, log_record_ids_removed);
        log_attr_filter = arrow::compute::and(&log_attr_filter, &log_attr_parent_ids_filter)
            .expect("boolean arrays should have equal length");

        // part 4: clean up resource attrs

        let log_record_resource_ids_removed = self.get_removed_ids(
            log_record_resource_ids_column,
            &inverse_log_record_filter,
            consts::ID,
        )?;
        let resource_attr_parent_ids_filter = self.build_id_filter(
            resource_attr_parent_ids_column,
            log_record_resource_ids_removed,
        );
        resource_attr_filter =
            arrow::compute::and(&resource_attr_filter, &resource_attr_parent_ids_filter)
                .expect("boolean arrays should have equal length");

        let log_record_scope_ids_removed = self.get_removed_ids(
            log_record_scope_ids_column,
            &inverse_log_record_filter,
            consts::ID,
        )?;
        let scope_attr_filter =
            self.build_id_filter(scope_attr_parent_ids_column, log_record_scope_ids_removed);

        Ok((
            resource_attr_filter,
            scope_attr_filter,
            log_record_filter,
            log_attr_filter,
        ))
    }

    fn get_removed_ids(
        &self,
        id_column: &Arc<dyn Array>,
        filter: &BooleanArray,
        column_type: &str,
    ) -> Result<HashSet<u16>> {
        // get ids being removed
        let filtered_ids =
            arrow::compute::filter(id_column, filter).expect("columns should have equal length");

        // downcast id and get unique values
        let filtered_ids = filtered_ids
            .as_any()
            .downcast_ref::<UInt16Array>()
            .with_context(|| error::ColumnDataTypeMismatchSnafu {
                name: column_type,
                actual: filtered_ids.data_type().clone(),
                expect: DataType::UInt16,
            })?;
        Ok(filtered_ids.iter().flatten().collect())
    }

    fn build_id_filter(
        &self,
        id_column: &Arc<dyn Array>,
        ids_to_remove: HashSet<u16>,
    ) -> BooleanArray {
        let mut combined_id_filter = BooleanArray::new_null(id_column.len());
        for id in ids_to_remove {
            let id_scalar = UInt16Array::new_scalar(id);
            let id_filter = arrow::compute::kernels::cmp::eq(id_column, &id_scalar)
                .expect("columns should have equal length");
            combined_id_filter = arrow::compute::or_kleene(&combined_id_filter, &id_filter)
                .expect("boolean arrays should have equal length");
        }
        // inverse because these are the ids we want to remove
        arrow::compute::not(&combined_id_filter).expect("not doesn't fail")
    }
}

impl KeyValue {
    /// create a new key value pair
    #[must_use]
    pub fn new(key: String, value: AnyValue) -> Self {
        Self { key, value }
    }
}
