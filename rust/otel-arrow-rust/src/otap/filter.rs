// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//!
//!

use self::logs::LogMatchProperties;
use crate::otap::OtapArrowRecords;
use crate::otap::error::{self, Result};
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use crate::schema::consts;
use arrow::array::UInt16Array;
use serde::Deserialize;
use snafu::OptionExt;

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

        // get
        let resource_attrs = logs_payload.get(ArrowPayloadType::ResourceAttrs).context(error::LogRecordNotFoundSnafu)?;
        let log_records = logs_payload.get(ArrowPayloadType::Logs).context(error::LogRecordNotFoundSnafu)?;
        let log_attrs = logs_payload.get(ArrowPayloadType::LogAttrs).context(error::LogRecordNotFoundSnafu)?;
        let scope_attrs = logs_payload.get(ArrowPayloadType::ScopeAttrs).context(error::LogRecordNotFoundSnafu)?;

        // HERE WE CLEAN UP THE TABLE WE EXTRACT THE IDS OF RECORDS WE ARE REMOVING AND MAKE SURE TO SYNC THESE REMOVALS WITH THE OTHER TABLES

        // we start with the resource_attr we get the id of the resource_attr that are being removed and use that to update the log_records filter
        // we want to remove the log_records the matching parent_id as the resource_attr that have been removed

        // create filters that we will use to get the rows that are getting removed
        // use these filters to get the ids of rows that are getting removed from the resource_attr
        let inverse_resource_attr_filter =
            arrow::compute::not(&resource_attr_filter).expect("not doesn't fail");
        let resource_id_column = resource_attrs.column_by_name(consts::PARENT_ID).context(
            error::ColumnNotFoundSnafu {
                name: consts::PARENT_ID,
            },
        )?;
        // we have a have a array contain all the parent_ids that will be removed

        // get ids being removed
        let resource_ids_filtered =
            arrow::compute::filter(&resource_id_column, &inverse_resource_attr_filter)
                .expect("columns should have equal length");

        // get column of resource ids from log records
        // let resource_struct_array = get_required_array(log_records, consts::RESOURCE)?;
        // let resource_struct_array = struct_array
        //     .as_any()
        //     .downcast_ref::<StructArray>()
        //     .with_context(|| error::ColumnDataTypeMismatchSnafu {
        //         name: consts::RESOURCE,
        //         actual: struct_array.data_type().clone(),
        //         expect: Self::data_type().clone(),
        //     })?;
        // let resource_ids: Vec<u16> = resource_struct_array
        //     .iter()
        //     .flatten()
        //     .map(|resource_struct| resource_struct.id)
        //     .collect();

        // // init booleanarray here
        // let num_rows = resource_attrs.num_rows();
        // let log_record_resouce_id_filter = BooleanArray::new_null(num_rows);

        // // build filter
        // for id in resource_ids {
        //     let id_scalar = UInt16Array::new_scalar(id);
        //     let id_filter =
        //         arrow::compute::kernels::cmp::eq(&log_record_resource_id_column, &id_scalar);
        //     log_record_resource_id_filter =
        //         arrow::compute::or_kleene(&log_record_resource_id_filter, &id_filter);
        // }
        // // inverse because these are the ids we want to remove
        // log_record_resource_id_filter = arrow::compute::not(&log_record_resource_id_filter);

        // // combine filter with log record so now it will remove the log_records that shouldn't belong
        // log_record_filter = arrow::compute::and(&log_record_filter, &log_record_resource_id_filter);

        // // NOW WE CLEAN UP LOG_ATTR AND SCOPE_ATTR

        // // invert filter to get all the removed rows
        // let id_log_records_filter = arrow::compute::not(&log_record_filter)?;

        // let log_record_id_column = log_records.column_by_name("id")?;

        // // these are the ids we need to remove from the log_attr table
        // let ids = arrow::compute::filter(&log_record_id_column, &id_log_records_filter)?;
        // let log_attr_id_column = log_attrs.column_by_name("parent_id");
        // let log_attr_parent_id_filter;
        // for id in ids {
        //     let id_scalar = UInt16Array::new_scalar(id);
        //     let id_filter = arrow::compute::kernels::cmp::eq(&log_attr_id_column, &id_scalar)?;
        //     log_attr_parent_id_filter =
        //         arrow::compute::or_kleene(&log_attr_parent_id_filter, &id_filter)?;
        // }
        // log_attr_parent_id_filter = arrow::compute::not(&log_attr_parent_id_filter)?;
        // log_attr_filter = arrow::compute::and(&log_attr_filter, &log_attr_parent_id_filter)?;

        // let log_record_scope_id_column = log_records.column_by_name("scope_id")?;

        // // here we need to also get the inverse and get the set difference to deal with overlapping scope_ids that are removed and kept
        // let scope_ids =
        //     arrow::compute::filter(&log_record_scope_id_column, &id_log_records_filter)?;

        // let scope_attr_id_column = scope_attrs.column_by_name("parent_id");
        // let scope_attr_filter;
        // for id in scope_ids {
        //     let id_scalar = UInt16Array::new_scalar(id);
        //     let id_filter = arrow::compute::kernels::cmp::eq(&scope_attr_id_column, &id_scalar)?;
        //     scope_attr_filter = arrow::compute::or_kleene(&scope_attr_filter, &id_filter)?;
        // }
        // scope_attr_filter = arrow::compute::not(&scope_attr_filter)?;

        // apply filters to the logs
        let filtered_resource_attrs =
            arrow::compute::filter_record_batch(resource_attrs, &resource_attr_filter)
                .expect("columns should have equal length");
        let filtered_log_records =
            arrow::compute::filter_record_batch(log_records, &log_record_filter)
                .expect("columns should have equal length");
        let filtered_log_attrs = arrow::compute::filter_record_batch(log_attrs, &log_attr_filter)
            .expect("columns should have equal length");
        // let filtered_scope_attrs =
        //     arrow::compute::filter_record_batch(scope_attrs, &scope_attr_filter)
        //         .expect("columns should have equal length");

        logs_payload.set(ArrowPayloadType::ResourceAttrs, filtered_resource_attrs);
        logs_payload.set(ArrowPayloadType::Logs, filtered_log_records);
        logs_payload.set(ArrowPayloadType::LogAttrs, filtered_log_attrs);
        // logs_payload.set(ArrowPayloadType::ScopeAttrs, filtered_scope_attrs);

        Ok(logs_payload)
    }
}
