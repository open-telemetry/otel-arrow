// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module focuses on taking a filter definition for Logs and building a filter
//! as a BooleanArray for the Logs, ResourceAttr, and LogsAttr OTAP Record Batches
//!

use crate::arrays::{
    get_required_array, get_required_array_from_struct_array,
    get_required_array_from_struct_array_from_record_batch, get_required_struct_array,
};
use crate::otap::OtapArrowRecords;
use crate::otap::error::{self, Result};
use crate::otap::filter::{AnyValue, KeyValue, MatchType, nulls_to_false};
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use crate::schema::consts;
use arrow::array::{
    Array, BooleanArray, Float64Array, Int32Array, Int64Array, StringArray, UInt16Array,
};
use arrow::datatypes::DataType;
use serde::Deserialize;
use snafu::OptionExt;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

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

/// LogMatchProperties specifies the set of properties in a log to match against and the type of string pattern matching to use.
#[derive(Debug, Clone, Deserialize)]
pub struct LogMatchProperties {
    // MatchType specifies the type of matching desired
    match_type: MatchType,

    // ResourceAttributes defines a list of possible resource attributes to match logs against.
    // A match occurs if any resource attribute matches all expressions in this given list.
    resource_attributes: Vec<KeyValue>,

    // RecordAttributes defines a list of possible record attributes to match logs against.
    // A match occurs if any record attribute matches at least one expression in this given list.
    record_attributes: Vec<KeyValue>,

    // SeverityTexts is a list of strings that the LogRecord's severity text field must match
    // against.
    severity_texts: Vec<String>,

    // SeverityNumberProperties defines how to match against a log record's SeverityNumber, if defined.
    severity_number: Option<LogSeverityNumberMatchProperties>,

    // LogBodies is a list of values that the LogRecord's body field must match
    // against.
    bodies: Vec<AnyValue>,
}

/// LogSeverityNumberMatchProperties specifies the requirements needed to match on the log severity field
#[derive(Debug, Clone, Deserialize)]
pub struct LogSeverityNumberMatchProperties {
    // Min is the minimum severity needed for the log record to match.
    // This corresponds to the short names specified here:
    // https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/logs/data-model.md#displaying-severity
    // this field is case-insensitive ("INFO" == "info")
    min: i32,
    // MatchUndefined lets logs records with "unknown" severity match.
    // If MinSeverity is not set, this field is ignored, as fields are not matched based on severity.
    match_undefined: bool,
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
        let resource_attr_filter = arrow::compute::and_kleene(
            &include_resource_attr_filter,
            &exclude_resource_attr_filter,
        )
        .expect("can combine two boolean arrays with equal length");
        let log_record_filter =
            arrow::compute::and_kleene(&include_log_record_filter, &exclude_log_record_filter)
                .expect("can combine two boolean arrays with equal length");
        let log_attr_filter =
            arrow::compute::and_kleene(&include_log_attr_filter, &exclude_log_attr_filter)
                .expect("can combine two boolean arrays with equal length");

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

        // apply filters to the logs
        let filtered_resource_attrs =
            arrow::compute::filter_record_batch(resource_attrs, &resource_attr_filter)
                .expect("can apply predicate on record batch with equal row length");
        let filtered_log_records =
            arrow::compute::filter_record_batch(log_records, &log_record_filter)
                .expect("can apply predicate on record batch with equal row length");
        let filtered_log_attrs = arrow::compute::filter_record_batch(log_attrs, &log_attr_filter)
            .expect("can apply predicate on record batch with equal row length");

        logs_payload.set(ArrowPayloadType::ResourceAttrs, filtered_resource_attrs);
        logs_payload.set(ArrowPayloadType::Logs, filtered_log_records);
        logs_payload.set(ArrowPayloadType::LogAttrs, filtered_log_attrs);

        if let Some(filter) = scope_attr_filter {
            let scope_attrs = logs_payload
                .get(ArrowPayloadType::ScopeAttrs)
                .context(error::LogRecordNotFoundSnafu)?;
            let filtered_scope_attrs = arrow::compute::filter_record_batch(scope_attrs, &filter)
                .expect("can apply predicate on record batch with equal row length");
            logs_payload.set(ArrowPayloadType::ScopeAttrs, filtered_scope_attrs);
        }

        Ok(logs_payload)
    }

    fn sync_up_filters(
        &self,
        logs_payload: &OtapArrowRecords,
        mut resource_attr_filter: BooleanArray,
        mut log_record_filter: BooleanArray,
        mut log_attr_filter: BooleanArray,
    ) -> Result<(
        BooleanArray,
        Option<BooleanArray>,
        BooleanArray,
        BooleanArray,
    )> {
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
        let scope_attrs = logs_payload.get(ArrowPayloadType::ScopeAttrs);

        // get the id columns from each record batch
        let resource_attr_parent_ids_column =
            get_required_array(resource_attrs, consts::PARENT_ID)?;
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
        let resource_attr_parent_ids_removed = self.get_ids(
            resource_attr_parent_ids_column,
            &inverse_resource_attr_filter,
            consts::PARENT_ID,
        )?;

        // create filter to remove these ids from log_record
        let log_record_resource_ids_filter = self.build_id_filter(
            log_record_resource_ids_column,
            resource_attr_parent_ids_removed,
            true,
        );

        // update the log_record_filter
        log_record_filter =
            arrow::compute::and_kleene(&log_record_filter, &log_record_resource_ids_filter)
                .expect("can combine two boolean arrays with equal length");

        // repeat with ids from log_attrs
        let inverse_log_attr_filter =
            arrow::compute::not(&log_attr_filter).expect("not doesn't fail");
        let log_attr_parent_ids_removed = self.get_ids(
            log_attr_parent_ids_column,
            &inverse_log_attr_filter,
            consts::PARENT_ID,
        )?;
        let log_record_ids_filter =
            self.build_id_filter(log_record_ids_column, log_attr_parent_ids_removed, true);
        log_record_filter = arrow::compute::and_kleene(&log_record_filter, &log_record_ids_filter)
            .expect("can combine two boolean arrays with equal length");

        // now using the updated log_record_filter we need to update the rest of the filers
        let inverse_log_record_filter =
            arrow::compute::not(&log_record_filter).expect("not doesn't fail");
        let log_record_ids_removed = self.get_ids(
            log_record_ids_column,
            &inverse_log_record_filter,
            consts::ID,
        )?;
        let log_attr_parent_ids_filter =
            self.build_id_filter(log_attr_parent_ids_column, log_record_ids_removed, true);
        log_attr_filter = arrow::compute::and_kleene(&log_attr_filter, &log_attr_parent_ids_filter)
            .expect("can combine two boolean arrays with equal length");

        // part 4: clean up resource attrs

        let log_record_resource_ids_kept = self.get_ids(
            log_record_resource_ids_column,
            &log_record_filter,
            consts::ID,
        )?;
        let resource_attr_parent_ids_filter = self.build_id_filter(
            resource_attr_parent_ids_column,
            log_record_resource_ids_kept,
            false,
        );
        resource_attr_filter =
            arrow::compute::and_kleene(&resource_attr_filter, &resource_attr_parent_ids_filter)
                .expect("can combine two boolean arrays with equal length");

        let scope_attr_filter = if let Some(scope_attrs_record_batch) = scope_attrs {
            let scope_attr_parent_ids_column =
                get_required_array(scope_attrs_record_batch, consts::PARENT_ID)?;
            let log_record_scope_ids_kept =
                self.get_ids(log_record_scope_ids_column, &log_record_filter, consts::ID)?;
            Some(self.build_id_filter(
                scope_attr_parent_ids_column,
                log_record_scope_ids_kept,
                false,
            ))
        } else {
            None
        };

        Ok((
            resource_attr_filter,
            scope_attr_filter,
            log_record_filter,
            log_attr_filter,
        ))
    }

    fn get_ids(
        &self,
        id_column: &Arc<dyn Array>,
        filter: &BooleanArray,
        column_type: &str,
    ) -> Result<HashSet<u16>> {
        // get ids being removed
        // error out herre
        let filtered_ids = arrow::compute::filter(id_column, filter)
            .expect("can apply predicate on column with equal length");

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
        id_set: HashSet<u16>,
        match_id: bool,
    ) -> BooleanArray {
        let mut combined_id_filter = BooleanArray::new_null(id_column.len());
        for id in id_set {
            let id_scalar = UInt16Array::new_scalar(id);
            let id_filter = if match_id {
                arrow::compute::kernels::cmp::eq(id_column, &id_scalar)
                    .expect("can compare uint16 id column with uint16 scalar")
            } else {
                arrow::compute::kernels::cmp::neq(id_column, &id_scalar)
                    .expect("can compare uint16 id column with uint16 scalar")
            };
            combined_id_filter = arrow::compute::or_kleene(&combined_id_filter, &id_filter)
                .expect("can combine two boolean arrays with equal length");
        }
        combined_id_filter = nulls_to_false(&combined_id_filter);
        // inverse because these are the ids we want to remove
        arrow::compute::not(&combined_id_filter).expect("not doesn't fail")
    }
}

impl LogMatchProperties {
    /// create a new LogMatchProperties
    #[must_use]
    pub fn new(
        match_type: MatchType,
        resource_attributes: Vec<KeyValue>,
        record_attributes: Vec<KeyValue>,
        severity_texts: Vec<String>,
        severity_number: Option<LogSeverityNumberMatchProperties>,
        bodies: Vec<AnyValue>,
    ) -> Self {
        Self {
            match_type,
            resource_attributes,
            record_attributes,
            severity_texts,
            severity_number,
            bodies,
        }
    }

    /// create filter takes a logs_payload and returns the filters for each of the record batches, also takes a invert flag to determine if the filters will be inverted
    pub fn create_filters(
        &self,
        logs_payload: &OtapArrowRecords,
        invert: bool,
    ) -> Result<(BooleanArray, BooleanArray, BooleanArray)> {
        let (mut resource_attr_filter, mut log_record_filter, mut log_attr_filter) = (
            self.get_resource_attr_filter(logs_payload)?,
            self.get_log_record_filter(logs_payload)?,
            self.get_log_attr_filter(logs_payload)?,
        );

        if invert {
            resource_attr_filter =
                arrow::compute::not(&resource_attr_filter).expect("not doesn't fail");

            log_record_filter = arrow::compute::not(&log_record_filter).expect("not doesn't fail");

            log_attr_filter = arrow::compute::not(&log_attr_filter).expect("not doesn't fail");
        }

        Ok((resource_attr_filter, log_record_filter, log_attr_filter))
    }

    fn get_resource_attr_filter(&self, logs_payload: &OtapArrowRecords) -> Result<BooleanArray> {
        // get resource_attrs record batch
        let resource_attrs = logs_payload
            .get(ArrowPayloadType::ResourceAttrs)
            .context(error::LogRecordNotFoundSnafu)?;

        let num_rows = resource_attrs.num_rows();
        if self.resource_attributes.is_empty() {
            return Ok(vec![true; num_rows].into());
        }

        let mut attributes_filter = BooleanArray::new_null(num_rows);
        let key_column = get_required_array(resource_attrs, consts::ATTRIBUTE_KEY)?;
        // generate the filter for this record_batch
        for attribute in &self.resource_attributes {
            let key_scalar = StringArray::new_scalar(attribute.key.clone());

            let key_filter = arrow::compute::kernels::cmp::eq(&key_column, &key_scalar)
                .expect("can compare string key column to string scalar");
            let value_filter = match &attribute.value {
                AnyValue::String(value) => {
                    // get string column
                    let string_column = get_required_array(resource_attrs, consts::ATTRIBUTE_STR)?;
                    match self.match_type {
                        MatchType::Regexp => {
                            let string_column = string_column
                                .as_any()
                                .downcast_ref::<StringArray>()
                                .expect("array can be downcast to StringArray");
                            arrow::compute::regexp_is_match_scalar(string_column, value, None)
                                .expect("can apply string column to regexp scalar")
                        }
                        MatchType::Strict => {
                            let value_scalar = StringArray::new_scalar(value);
                            arrow::compute::kernels::cmp::eq(&string_column, &value_scalar)
                                .expect("can compare string value column to string scalar")
                        }
                    }
                }
                AnyValue::Int(value) => {
                    let int_column = resource_attrs.column_by_name(consts::ATTRIBUTE_INT);
                    // check if column exists if not then there is no resource that has this attribute so we can return a all false boolean array
                    match int_column {
                        Some(column) => {
                            let value_scalar = Int64Array::new_scalar(*value);
                            arrow::compute::kernels::cmp::eq(&column, &value_scalar)
                                .expect("can compare i64 value column to i64 scalar")
                        }
                        None => {
                            return Ok(vec![false; num_rows].into());
                        }
                    }
                }
                AnyValue::Double(value) => {
                    let double_column = resource_attrs.column_by_name(consts::ATTRIBUTE_DOUBLE);
                    match double_column {
                        Some(column) => {
                            let value_scalar = Float64Array::new_scalar(*value);
                            arrow::compute::kernels::cmp::eq(&column, &value_scalar)
                                .expect("can compare f64 value column to f64 scalar")
                        }
                        None => {
                            return Ok(vec![false; num_rows].into());
                        }
                    }
                }
                AnyValue::Boolean(value) => {
                    let bool_column = resource_attrs.column_by_name(consts::ATTRIBUTE_BOOL);
                    match bool_column {
                        Some(column) => {
                            let value_scalar = BooleanArray::new_scalar(*value);
                            arrow::compute::kernels::cmp::eq(&column, &value_scalar)
                                .expect("can compare bool value column to bool scalar")
                        }
                        None => {
                            return Ok(vec![false; num_rows].into());
                        }
                    }
                }
                _ => {
                    // ToDo add keyvalue, array, and bytes
                    return Ok(vec![false; num_rows].into());
                }
            };
            // build filter that checks for both matching key and value filter
            let attribute_filter = arrow::compute::and_kleene(&key_filter, &value_filter)
                .expect("can combine two boolean arrays with equal length");
            attributes_filter = arrow::compute::or_kleene(&attributes_filter, &attribute_filter)
                .expect("can combine two boolean arrays with equal length");
        }

        // ToDo optimize the logic below where we build the final filter based on the ids
        // now we get ids of resource_attrs
        let parent_id_column = get_required_array(resource_attrs, consts::PARENT_ID)?;
        // the ids should show up self.resource_attr.len() times otherwise they don't have all the required attributes
        let ids = arrow::compute::filter(&parent_id_column, &attributes_filter)
            .expect("can apply predicate on column with equal length");
        // extract correct ids
        let ids = ids
            .as_any()
            .downcast_ref::<UInt16Array>()
            .expect("array can be downcast to UInt16Array");
        // remove null values
        let ids: Vec<u16> = ids.iter().flatten().collect();
        let mut ids_counted: HashMap<u16, usize> = HashMap::new();
        for &id in &ids {
            *ids_counted.entry(id).or_insert(0) += 1;
        }

        let required_ids_count = self.resource_attributes.len();

        ids_counted.retain(|_key, value| *value >= required_ids_count);

        // build filter around the ids
        let mut filter = BooleanArray::new_null(num_rows);
        for (id, _) in ids_counted {
            let id_scalar = UInt16Array::new_scalar(id);
            let id_filter = arrow::compute::kernels::cmp::eq(&parent_id_column, &id_scalar)
                .expect("can compare uint16 id column to uint16 scalar");
            filter = arrow::compute::or_kleene(&filter, &id_filter)
                .expect("can combine two boolean arrays with equal length");
        }
        Ok(nulls_to_false(&filter))
    }

    fn get_log_record_filter(&self, logs_payload: &OtapArrowRecords) -> Result<BooleanArray> {
        let log_records = logs_payload
            .get(ArrowPayloadType::Logs)
            .context(error::LogRecordNotFoundSnafu)?;
        let num_rows = log_records.num_rows();
        // create filter for severity texts

        let severity_texts_column = match log_records.column_by_name(consts::SEVERITY_TEXT) {
            Some(column) => column,
            None => {
                return Ok(vec![false; num_rows].into());
            }
        };
        let mut filter: BooleanArray = vec![true; num_rows].into();
        if !&self.severity_texts.is_empty() {
            let mut severity_texts_filter = BooleanArray::new_null(num_rows);
            for severity_text in &self.severity_texts {
                let severity_text_scalar = StringArray::new_scalar(severity_text);
                let severity_text_filter =
                    arrow::compute::kernels::cmp::eq(&severity_texts_column, &severity_text_scalar)
                        .expect("can compare string severity text column to string scalar");
                severity_texts_filter =
                    arrow::compute::or_kleene(&severity_texts_filter, &severity_text_filter)
                        .expect("can combine two boolean arrays with equal length");
            }
            filter = arrow::compute::and_kleene(&filter, &severity_texts_filter)
                .expect("can combine two boolean arrays with equal length");
        }

        if !&self.bodies.is_empty() {
            // create filter for log bodies
            let mut bodies_filter = BooleanArray::new_null(num_rows);
            let bodies_column = get_required_struct_array(log_records, consts::BODY)?;
            for body in &self.bodies {
                let body_filter = match body {
                    AnyValue::String(value) => {
                        // get string column
                        let string_column = get_required_array_from_struct_array(
                            bodies_column,
                            consts::ATTRIBUTE_STR,
                        )?;
                        match self.match_type {
                            MatchType::Regexp => {
                                let string_column = string_column
                                    .as_any()
                                    .downcast_ref::<StringArray>()
                                    .expect("array can be downcast to StringArray");

                                arrow::compute::regexp_is_match_scalar(string_column, value, None)
                                    .expect("columns should have equal length")
                            }
                            MatchType::Strict => {
                                let value_scalar = StringArray::new_scalar(value);
                                arrow::compute::kernels::cmp::eq(&string_column, &value_scalar)
                                    .expect("can compare string value column to string scalar")
                            }
                        }
                    }
                    AnyValue::Int(value) => {
                        let int_column = bodies_column.column_by_name(consts::ATTRIBUTE_INT);
                        match int_column {
                            Some(column) => {
                                let value_scalar = Int64Array::new_scalar(*value);
                                arrow::compute::kernels::cmp::eq(&column, &value_scalar)
                                    .expect("can compare i64 value column to i64 scalar")
                            }
                            None => {
                                continue;
                            }
                        }
                    }
                    AnyValue::Double(value) => {
                        let double_column = bodies_column.column_by_name(consts::ATTRIBUTE_DOUBLE);
                        match double_column {
                            Some(column) => {
                                let value_scalar = Float64Array::new_scalar(*value);
                                arrow::compute::kernels::cmp::eq(&column, &value_scalar)
                                    .expect("can compare f64 value column to f64 scalar")
                            }
                            None => {
                                continue;
                            }
                        }
                    }
                    AnyValue::Boolean(value) => {
                        let bool_column = bodies_column.column_by_name(consts::ATTRIBUTE_BOOL);
                        match bool_column {
                            Some(column) => {
                                let value_scalar = BooleanArray::new_scalar(*value);
                                arrow::compute::kernels::cmp::eq(&column, &value_scalar)
                                    .expect("can compare bool value column to bool scalar")
                            }
                            None => {
                                continue;
                            }
                        }
                    }
                    _ => {
                        // ToDo add keyvalue, array, and bytes
                        continue;
                    }
                };
                bodies_filter = arrow::compute::or_kleene(&body_filter, &bodies_filter)
                    .expect("can combine two boolean arrays with equal length");
                // combine the filters
            }
            filter = arrow::compute::and_kleene(&filter, &bodies_filter)
                .expect("can combine two boolean arrays with equal length");
        }

        // if the severity_number field is defined then we create the severity_number filter
        if let Some(severity_number_properties) = &self.severity_number {
            let severity_number_column = match log_records.column_by_name(consts::SEVERITY_NUMBER) {
                Some(column) => column,
                None => {
                    return Ok(vec![false; num_rows].into());
                }
            };

            // TODO make min a string that contains the severity number type and map to the int instead
            let min_severity_number = severity_number_properties.min;
            let min_severity_scalar = Int32Array::new_scalar(min_severity_number);
            let mut severity_numbers_filter =
                arrow::compute::kernels::cmp::gt_eq(&severity_number_column, &min_severity_scalar)
                    .expect("can compare i32 severity number column to i32 scalar");
            // update the filter if we allow unknown
            if severity_number_properties.match_undefined {
                let unknown_severity_scalar = Int32Array::new_scalar(0);
                let unknown_severity_number_filter = arrow::compute::kernels::cmp::eq(
                    &severity_number_column,
                    &unknown_severity_scalar,
                )
                .expect("can compare i32 severity number column to i32 scalar");
                severity_numbers_filter = arrow::compute::or_kleene(
                    &severity_numbers_filter,
                    &unknown_severity_number_filter,
                )
                .expect("can combine two boolean arrays with equal length");
            }
            // combine severity number filter to the log record filter
            filter = arrow::compute::and_kleene(&filter, &severity_numbers_filter)
                .expect("can combine two boolean arrays with equal length");
        }
        Ok(nulls_to_false(&filter))
    }

    fn get_log_attr_filter(&self, logs_payload: &OtapArrowRecords) -> Result<BooleanArray> {
        // get log_attrs record batch
        let log_attrs = logs_payload
            .get(ArrowPayloadType::LogAttrs)
            .context(error::LogRecordNotFoundSnafu)?;

        let num_rows = log_attrs.num_rows();
        // if there is nothing to filter we return all true
        if self.record_attributes.is_empty() {
            return Ok(vec![true; num_rows].into());
        }
        let mut attributes_filter = BooleanArray::new_null(num_rows);

        let key_column = get_required_array(log_attrs, consts::ATTRIBUTE_KEY)?;

        // generate the filter for this record_batch
        for attribute in &self.record_attributes {
            let key_scalar = StringArray::new_scalar(attribute.key.clone());
            let key_filter = arrow::compute::kernels::cmp::eq(&key_column, &key_scalar)
                .expect("can compare string key column to string scalar");
            let value_filter = match &attribute.value {
                AnyValue::String(value) => {
                    // get string column
                    let string_column = get_required_array(log_attrs, consts::ATTRIBUTE_STR)?;

                    match self.match_type {
                        MatchType::Regexp => {
                            let string_column = string_column
                                .as_any()
                                .downcast_ref::<StringArray>()
                                .expect("array can be downcast to StringArray");

                            arrow::compute::regexp_is_match_scalar(string_column, value, None)
                                .expect("can apply match string column with regexp scalar")
                        }
                        MatchType::Strict => {
                            let value_scalar = StringArray::new_scalar(value);
                            arrow::compute::kernels::cmp::eq(&string_column, &value_scalar)
                                .expect("can compare string value column to string scalar")
                        }
                    }
                }
                AnyValue::Int(value) => {
                    let int_column = log_attrs.column_by_name(consts::ATTRIBUTE_INT);
                    match int_column {
                        Some(column) => {
                            let value_scalar = Int64Array::new_scalar(*value);
                            arrow::compute::kernels::cmp::eq(&column, &value_scalar)
                                .expect("can compare i64 value column to i64 scalar")
                        }
                        None => {
                            continue;
                        }
                    }
                }
                AnyValue::Double(value) => {
                    let double_column = log_attrs.column_by_name(consts::ATTRIBUTE_DOUBLE);
                    match double_column {
                        Some(column) => {
                            let value_scalar = Float64Array::new_scalar(*value);
                            arrow::compute::kernels::cmp::eq(&column, &value_scalar)
                                .expect("can compare f64 value column to f64 scalar")
                        }
                        None => {
                            continue;
                        }
                    }
                }
                AnyValue::Boolean(value) => {
                    let bool_column = log_attrs.column_by_name(consts::ATTRIBUTE_BOOL);
                    match bool_column {
                        Some(column) => {
                            let value_scalar = BooleanArray::new_scalar(*value);
                            arrow::compute::kernels::cmp::eq(&column, &value_scalar)
                                .expect("can compare bool value column to bool scalar")
                        }
                        None => {
                            continue;
                        }
                    }
                }
                _ => {
                    // ToDo add keyvalue, array, and bytes
                    continue;
                }
            };
            // build filter that checks for both matching key and value filter
            let attribute_filter = arrow::compute::and_kleene(&key_filter, &value_filter)
                .expect("can combine two boolean arrays with equal length");
            // combine with rest of filters
            attributes_filter = arrow::compute::or_kleene(&attributes_filter, &attribute_filter)
                .expect("can combine two boolean arrays with equal length");
        }

        // now we get ids of
        let parent_id_column = get_required_array(log_attrs, consts::PARENT_ID)?;

        let ids = arrow::compute::filter(&parent_id_column, &attributes_filter)
            .expect("can apply predicate on column with equal length");
        let ids = ids
            .as_any()
            .downcast_ref::<UInt16Array>()
            .expect("array can be downcast into UInt16Array");
        let ids: HashSet<u16> = ids.iter().flatten().collect();
        // build filter around the ids
        let mut filter = BooleanArray::new_null(num_rows);
        for id in ids {
            let id_scalar = UInt16Array::new_scalar(id);
            let id_filter = arrow::compute::kernels::cmp::eq(&parent_id_column, &id_scalar)
                .expect("can compare uint16 id column to uint16 scalar");
            filter = arrow::compute::or_kleene(&filter, &id_filter)
                .expect("can combine two boolean arrays with equal length");
        }
        Ok(nulls_to_false(&filter))
    }
}

impl LogSeverityNumberMatchProperties {
    /// create a new LogSeverityNumberMatchProperties
    #[must_use]
    pub fn new(min: i32, match_undefined: bool) -> Self {
        Self {
            min,
            match_undefined,
        }
    }
}
