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
use crate::otap::error::{Error, Result};
use crate::otap::filter::{
    AnyValue, KeyValue, MatchType, apply_filter, default_match_type, get_attr_filter,
    get_resource_attr_filter, new_child_record_batch_filter, nulls_to_false, regex_match_column,
    update_child_record_batch_filter, update_parent_record_batch_filter,
};
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use crate::schema::consts;
use arrow::array::{BooleanArray, Float64Array, Int32Array, Int64Array, StringArray};
use arrow::buffer::BooleanBuffer;
use serde::Deserialize;
use std::collections::HashMap;

/// struct that describes the overall requirements to use in order to filter logs
#[derive(Debug, Clone, Deserialize)]
pub struct LogFilter {
    // Include match properties describe logs that should be included in the Collector Service pipeline,
    // all other logs should be dropped from further processing.
    // If both Include and Exclude are specified, Include filtering occurs first.
    include: Option<LogMatchProperties>,
    // Exclude match properties describe logs that should be excluded from the Collector Service pipeline,
    // all other logs should be included.
    // If both Include and Exclude are specified, Include filtering occurs first.
    exclude: Option<LogMatchProperties>,

    // LogConditions is a list of OTTL conditions for an ottllog context.
    // If any condition resolves to true, the log event will be dropped.
    // Supports `and`, `or`, and `()`
    #[allow(dead_code)]
    // ToDo: Add OTTL Support and accept OTTL expressions
    log_record: Vec<String>,
}

/// LogMatchProperties specifies the set of properties in a log to match against and the type of string pattern matching to use.
#[derive(Debug, Clone, Deserialize)]
pub struct LogMatchProperties {
    // MatchType specifies the type of matching desired
    #[serde(default = "default_match_type")]
    match_type: MatchType,

    // ResourceAttributes defines a list of possible resource attributes to match logs against.
    // A match occurs if any resource attribute matches all expressions in this given list.
    #[serde(default)]
    resource_attributes: Vec<KeyValue>,

    // RecordAttributes defines a list of possible record attributes to match logs against.
    // A match occurs if any record attribute matches at least one expression in this given list.
    #[serde(default)]
    record_attributes: Vec<KeyValue>,

    // SeverityTexts is a list of strings that the LogRecord's severity text field must match
    // against.
    #[serde(default)]
    severity_texts: Vec<String>,

    // SeverityNumberProperties defines how to match against a log record's SeverityNumber, if defined.
    severity_number: Option<LogSeverityNumberMatchProperties>,

    // LogBodies is a list of values that the LogRecord's body field must match
    // against.
    #[serde(default)]
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
        include: Option<LogMatchProperties>,
        exclude: Option<LogMatchProperties>,
        log_record: Vec<String>,
    ) -> Self {
        Self {
            include,
            exclude,
            log_record,
        }
    }

    /// take a logs payload and return the filtered result
    pub fn filter(
        &self,
        mut logs_payload: OtapArrowRecords,
    ) -> Result<(OtapArrowRecords, u64, u64)> {
        let (resource_attr_filter, log_record_filter, log_attr_filter) = if let Some(include_config) =
            &self.include
            && let Some(exclude_config) = &self.exclude
        {
            let (include_resource_attr_filter, include_log_record_filter, include_log_attr_filter) =
                include_config.create_filters(&logs_payload, false)?;
            let (exclude_resource_attr_filter, exclude_log_record_filter, exclude_log_attr_filter) =
                exclude_config.create_filters(&logs_payload, true)?;

            // combine the include and exclude filters
            let resource_attr_filter = arrow::compute::and_kleene(
                &include_resource_attr_filter,
                &exclude_resource_attr_filter,
            )
            .map_err(|e| Error::ColumnLengthMismatch { source: e })?;
            let log_record_filter =
                arrow::compute::and_kleene(&include_log_record_filter, &exclude_log_record_filter)
                    .map_err(|e| Error::ColumnLengthMismatch { source: e })?;
            let log_attr_filter =
                arrow::compute::and_kleene(&include_log_attr_filter, &exclude_log_attr_filter)
                    .map_err(|e| Error::ColumnLengthMismatch { source: e })?;
            (resource_attr_filter, log_record_filter, log_attr_filter)
        } else if self.include.is_none()
            && let Some(exclude_config) = &self.exclude
        {
            exclude_config.create_filters(&logs_payload, true)?
        } else if let Some(include_config) = &self.include
            && self.exclude.is_none()
        {
            include_config.create_filters(&logs_payload, false)?
        } else {
            // both include and exclude is none
            let num_rows = logs_payload
                .get(ArrowPayloadType::Logs)
                .ok_or_else(|| Error::RecordBatchNotFound {
                    payload_type: ArrowPayloadType::Logs,
                })?
                .num_rows() as u64;
            return Ok((logs_payload, num_rows, num_rows));
        };

        let (log_record_filter, child_record_batch_filters) = self.sync_up_filters(
            &logs_payload,
            resource_attr_filter,
            log_record_filter,
            log_attr_filter,
        )?;

        let (log_rows_before, log_rows_removed) = apply_filter(
            &mut logs_payload,
            ArrowPayloadType::Logs,
            &log_record_filter,
        )?;

        for (payload_type, filter) in child_record_batch_filters {
            let (_, _) = apply_filter(&mut logs_payload, payload_type, &filter)?;
        }

        Ok((logs_payload, log_rows_before, log_rows_removed))
    }

    /// this function takes the filters for each record batch and makes sure that incomplete
    /// returns the cleaned up filters that can be immediately applied on the record batches
    fn sync_up_filters(
        &self,
        logs_payload: &OtapArrowRecords,
        resource_attr_filter: BooleanArray,
        mut log_record_filter: BooleanArray,
        log_attr_filter: BooleanArray,
    ) -> Result<(BooleanArray, HashMap<ArrowPayloadType, BooleanArray>)> {
        // get the record batches we are going to filter
        let resource_attrs = logs_payload.get(ArrowPayloadType::ResourceAttrs);
        let log_records = logs_payload
            .get(ArrowPayloadType::Logs)
            .ok_or_else(|| Error::LogRecordNotFound {})?;
        let log_attrs = logs_payload.get(ArrowPayloadType::LogAttrs);
        let scope_attrs = logs_payload.get(ArrowPayloadType::ScopeAttrs);

        // get the id columns from record batch
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

        // optional record batch
        match resource_attrs {
            Some(resource_attrs_record_batch) => {
                log_record_filter = update_parent_record_batch_filter(
                    resource_attrs_record_batch,
                    log_record_resource_ids_column,
                    &resource_attr_filter,
                    &log_record_filter,
                )?;
            }
            None => {
                if resource_attr_filter.true_count() == 0 {
                    // the configuration required certain resource_attributes but found none so we can return early
                    // remove all elements as nothing matches
                    return Ok((
                        BooleanArray::from(BooleanBuffer::new_unset(log_record_filter.len())),
                        HashMap::new(),
                    ));
                }
            }
        }

        match log_attrs {
            Some(log_attrs_record_batch) => {
                log_record_filter = update_parent_record_batch_filter(
                    log_attrs_record_batch,
                    log_record_ids_column,
                    &log_attr_filter,
                    &log_record_filter,
                )?;
            }
            None => {
                if log_attr_filter.true_count() == 0 {
                    // the configuration required certain resource_attributes but found none so we can return early
                    // remove all elements as nothing matches
                    return Ok((
                        BooleanArray::from(BooleanBuffer::new_unset(log_record_filter.len())),
                        HashMap::new(),
                    ));
                }
            }
        }

        // now using the updated log_record_filter we need to update the rest of the filers

        // use hashmap to map filters to their payload types to return,
        // only record batches that exist will have their filter added to this hashmap
        let mut child_record_batch_filters = HashMap::new();

        if let Some(log_attrs_record_batch) = log_attrs {
            _ = child_record_batch_filters.insert(
                ArrowPayloadType::LogAttrs,
                update_child_record_batch_filter(
                    log_attrs_record_batch,
                    log_record_ids_column,
                    &log_attr_filter,
                    &log_record_filter,
                )?,
            );
        }

        if let Some(resource_attrs_record_batch) = resource_attrs {
            _ = child_record_batch_filters.insert(
                ArrowPayloadType::ResourceAttrs,
                update_child_record_batch_filter(
                    resource_attrs_record_batch,
                    log_record_resource_ids_column,
                    &resource_attr_filter,
                    &log_record_filter,
                )?,
            );
        }

        if let Some(scope_attrs_record_batch) = scope_attrs {
            _ = child_record_batch_filters.insert(
                ArrowPayloadType::ScopeAttrs,
                new_child_record_batch_filter(
                    scope_attrs_record_batch,
                    log_record_scope_ids_column,
                    &log_record_filter,
                )?,
            );
        }

        Ok((log_record_filter, child_record_batch_filters))
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
            get_resource_attr_filter(logs_payload, &self.resource_attributes, &self.match_type)?,
            self.get_log_record_filter(logs_payload)?,
            get_attr_filter(
                logs_payload,
                &self.record_attributes,
                &self.match_type,
                ArrowPayloadType::LogAttrs,
            )?,
        );

        // invert flag depending on whether we are excluding or including
        if invert {
            resource_attr_filter =
                arrow::compute::not(&resource_attr_filter).expect("not doesn't fail");

            log_record_filter = arrow::compute::not(&log_record_filter).expect("not doesn't fail");

            log_attr_filter = arrow::compute::not(&log_attr_filter).expect("not doesn't fail");
        }

        Ok((resource_attr_filter, log_record_filter, log_attr_filter))
    }

    /// Creates a booleanarray that will filter a log record record batch based on the
    /// defined severity_text, log body, and severity_number (if definied). A log record should match all
    /// defined requirements.
    fn get_log_record_filter(&self, logs_payload: &OtapArrowRecords) -> Result<BooleanArray> {
        let log_records = logs_payload
            .get(ArrowPayloadType::Logs)
            .ok_or_else(|| Error::LogRecordNotFound)?;
        let num_rows = log_records.num_rows();
        // create filter for severity texts
        let mut filter: BooleanArray = BooleanArray::from(BooleanBuffer::new_set(num_rows));
        if !&self.severity_texts.is_empty() {
            // get he severity text column
            let severity_texts_column = match log_records.column_by_name(consts::SEVERITY_TEXT) {
                Some(column) => column,
                None => {
                    // any columns that don't exist means we can't match so we default to all false boolean array
                    return Ok(BooleanArray::from(BooleanBuffer::new_unset(num_rows)));
                }
            };
            let mut severity_texts_filter = BooleanArray::new_null(num_rows);
            for severity_text in &self.severity_texts {
                let severity_text_filter = match self.match_type {
                    MatchType::Regexp => regex_match_column(severity_texts_column, severity_text)?,
                    MatchType::Strict => {
                        let severity_text_scalar = StringArray::new_scalar(severity_text);
                        // since we use a scalar here we don't have to worry a column length mismatch when we compare

                        arrow::compute::kernels::cmp::eq(
                            &severity_texts_column,
                            &severity_text_scalar,
                        )
                        .expect("can compare string severity text column to string scalar")
                    }
                };
                severity_texts_filter =
                    arrow::compute::or_kleene(&severity_texts_filter, &severity_text_filter)
                        .map_err(|e| Error::ColumnLengthMismatch { source: e })?;
            }
            filter = arrow::compute::and_kleene(&filter, &severity_texts_filter)
                .map_err(|e| Error::ColumnLengthMismatch { source: e })?;
        }

        if !&self.bodies.is_empty() {
            // create filter for log bodies
            let mut bodies_filter = BooleanArray::new_null(num_rows);
            let bodies_column = get_required_struct_array(log_records, consts::BODY)?;
            for body in &self.bodies {
                // match on body value
                let body_filter = match body {
                    AnyValue::String(value) => {
                        // get string column
                        let string_column = get_required_array_from_struct_array(
                            bodies_column,
                            consts::ATTRIBUTE_STR,
                        )?;
                        match self.match_type {
                            MatchType::Regexp => regex_match_column(string_column, value)?,
                            MatchType::Strict => {
                                let value_scalar = StringArray::new_scalar(value);
                                // since we use a scalar here we don't have to worry a column length mismatch when we compare

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
                                // since we use a scalar here we don't have to worry a column length mismatch when we compare

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
                                // since we use a scalar here we don't have to worry a column length mismatch when we compare

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
                                // since we use a scalar here we don't have to worry a column length mismatch when we compare

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
                    .map_err(|e| Error::ColumnLengthMismatch { source: e })?;
                // combine the filters
            }
            filter = arrow::compute::and_kleene(&filter, &bodies_filter)
                .map_err(|e| Error::ColumnLengthMismatch { source: e })?;
        }

        // if the severity_number field is defined then we create the severity_number filter
        if let Some(severity_number_properties) = &self.severity_number {
            let severity_number_column = match log_records.column_by_name(consts::SEVERITY_NUMBER) {
                Some(column) => column,
                None => {
                    return Ok(BooleanArray::from(BooleanBuffer::new_unset(num_rows)));
                }
            };

            // TODO make min a string that contains the severity number type and map to the int instead
            let min_severity_number = severity_number_properties.min;
            let min_severity_scalar = Int32Array::new_scalar(min_severity_number);
            let mut severity_numbers_filter =
                        // since we use a scalar here we don't have to worry a column length mismatch when we compare

                arrow::compute::kernels::cmp::gt_eq(&severity_number_column, &min_severity_scalar)
                    .expect("can compare i32 severity number column to i32 scalar");
            // update the filter if we allow unknown
            if severity_number_properties.match_undefined {
                let unknown_severity_scalar = Int32Array::new_scalar(0);
                // since we use a scalar here we don't have to worry a column length mismatch when we compare

                let unknown_severity_number_filter = arrow::compute::kernels::cmp::eq(
                    &severity_number_column,
                    &unknown_severity_scalar,
                )
                .expect("can compare i32 severity number column to i32 scalar");
                severity_numbers_filter = arrow::compute::or_kleene(
                    &severity_numbers_filter,
                    &unknown_severity_number_filter,
                )
                .map_err(|e| Error::ColumnLengthMismatch { source: e })?;
            }
            // combine severity number filter to the log record filter
            filter = arrow::compute::and_kleene(&filter, &severity_numbers_filter)
                .map_err(|e| Error::ColumnLengthMismatch { source: e })?;
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
