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
    AnyValue, KeyValue, MatchType, NO_RECORD_BATCH_FILTER_SIZE, apply_filter,
    build_uint16_id_filter, default_match_type, get_uint16_ids, nulls_to_false, regex_match_column,
};
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use crate::schema::consts;
use arrow::array::{
    Array, BooleanArray, Float64Array, Int32Array, Int64Array, StringArray, UInt16Array,
};
use arrow::buffer::BooleanBuffer;
use serde::Deserialize;
use std::collections::HashMap;

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
        .map_err(|e| Error::ColumnLengthMismatch { source: e })?;
        let log_record_filter =
            arrow::compute::and_kleene(&include_log_record_filter, &exclude_log_record_filter)
                .map_err(|e| Error::ColumnLengthMismatch { source: e })?;
        let log_attr_filter =
            arrow::compute::and_kleene(&include_log_attr_filter, &exclude_log_attr_filter)
                .map_err(|e| Error::ColumnLengthMismatch { source: e })?;

        let (resource_attr_filter, scope_attr_filter, log_record_filter, log_attr_filter) = self
            .sync_up_filters(
                &logs_payload,
                resource_attr_filter,
                log_record_filter,
                log_attr_filter,
            )?;

        apply_filter(
            &mut logs_payload,
            ArrowPayloadType::Logs,
            &log_record_filter,
        )?;

        if let Some(filter) = resource_attr_filter {
            apply_filter(&mut logs_payload, ArrowPayloadType::ResourceAttrs, &filter)?;
        }

        if let Some(filter) = scope_attr_filter {
            apply_filter(&mut logs_payload, ArrowPayloadType::ScopeAttrs, &filter)?;
        }

        if let Some(filter) = log_attr_filter {
            apply_filter(&mut logs_payload, ArrowPayloadType::LogAttrs, &filter)?;
        }

        Ok(logs_payload)
    }

    /// this function takes the filters for each record batch and makes sure that incomplete
    /// returns the cleaned up filters that can be immediately applied on the record batches
    /// ToDo: RecordBatches that are optional and not present will result in the corresponding filter being returned as None
    /// ToDo: Handle edge case where LogRecords don't have attributes are getting through when we want to filter on attributes as well.
    fn sync_up_filters(
        &self,
        logs_payload: &OtapArrowRecords,
        resource_attr_filter: BooleanArray,
        mut log_record_filter: BooleanArray,
        log_attr_filter: BooleanArray,
    ) -> Result<(
        Option<BooleanArray>,
        Option<BooleanArray>,
        BooleanArray,
        Option<BooleanArray>,
    )> {
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
                // starting with the resource_attr
                // -> get ids of filtered attributes
                // -> map ids to resource_ids in log_record
                // -> create filter to require these resource_ids
                // -> update log_record filter
                let resource_attr_parent_ids_column =
                    get_required_array(resource_attrs_record_batch, consts::PARENT_ID)?;

                let resource_attr_parent_ids_filtered = get_uint16_ids(
                    resource_attr_parent_ids_column,
                    &resource_attr_filter,
                    consts::PARENT_ID,
                )?;

                // create filter to remove these ids from log_record
                let log_record_resource_ids_filter = build_uint16_id_filter(
                    log_record_resource_ids_column,
                    resource_attr_parent_ids_filtered,
                )?;

                // update the log_record_filter
                log_record_filter =
                    arrow::compute::and_kleene(&log_record_filter, &log_record_resource_ids_filter)
                        .map_err(|e| Error::ColumnLengthMismatch { source: e })?;
                // apply current logic
            }
            None => {
                if resource_attr_filter.true_count() == 0 {
                    // the configuration required certain resource_attributes but found none so we can return early
                    // remove all elements as nothing matches
                    return Ok((
                        None,
                        None,
                        BooleanArray::from(BooleanBuffer::new_unset(log_record_filter.len())),
                        None,
                    ));
                }
            }
        }

        match log_attrs {
            Some(log_attrs_record_batch) => {
                let log_attr_parent_ids_column =
                    get_required_array(log_attrs_record_batch, consts::PARENT_ID)?;

                // repeat with ids from log_attrs
                let log_attr_parent_ids_filtered = get_uint16_ids(
                    log_attr_parent_ids_column,
                    &log_attr_filter,
                    consts::PARENT_ID,
                )?;
                let log_record_ids_filter =
                    build_uint16_id_filter(log_record_ids_column, log_attr_parent_ids_filtered)?;
                log_record_filter =
                    arrow::compute::and_kleene(&log_record_filter, &log_record_ids_filter)
                        .map_err(|e| Error::ColumnLengthMismatch { source: e })?;
            }
            None => {
                if log_attr_filter.true_count() == 0 {
                    // the configuration required certain resource_attributes but found none so we can return early
                    // remove all elements as nothing matches
                    return Ok((
                        None,
                        None,
                        BooleanArray::from(BooleanBuffer::new_unset(log_record_filter.len())),
                        None,
                    ));
                }
            }
        }

        // now using the updated log_record_filter we need to update the rest of the filers
        let updated_log_attr_filter = if let Some(log_attrs_record_batch) = log_attrs {
            let log_attr_parent_ids_column =
                get_required_array(log_attrs_record_batch, consts::PARENT_ID)?;

            let log_record_ids_filtered =
                get_uint16_ids(log_record_ids_column, &log_record_filter, consts::ID)?;
            let log_attr_parent_ids_filter =
                build_uint16_id_filter(log_attr_parent_ids_column, log_record_ids_filtered)?;

            Some(
                arrow::compute::and_kleene(&log_attr_filter, &log_attr_parent_ids_filter)
                    .map_err(|e| Error::ColumnLengthMismatch { source: e })?,
            )
        } else {
            None
        };

        // part 4: clean up resource attrs

        let updated_resource_attr_filter = if let Some(resource_attrs_record_batch) = resource_attrs
        {
            let resource_attr_parent_ids_column =
                get_required_array(resource_attrs_record_batch, consts::PARENT_ID)?;
            let log_record_resource_ids_filtered = get_uint16_ids(
                log_record_resource_ids_column,
                &log_record_filter,
                consts::ID,
            )?;

            let resource_attr_parent_ids_filter = build_uint16_id_filter(
                resource_attr_parent_ids_column,
                log_record_resource_ids_filtered,
            )?;
            Some(
                arrow::compute::and_kleene(&resource_attr_filter, &resource_attr_parent_ids_filter)
                    .map_err(|e| Error::ColumnLengthMismatch { source: e })?,
            )
        } else {
            None
        };

        let scope_attr_filter = if let Some(scope_attrs_record_batch) = scope_attrs {
            let scope_attr_parent_ids_column =
                get_required_array(scope_attrs_record_batch, consts::PARENT_ID)?;
            let log_record_scope_ids_filtered =
                get_uint16_ids(log_record_scope_ids_column, &log_record_filter, consts::ID)?;
            Some(build_uint16_id_filter(
                scope_attr_parent_ids_column,
                log_record_scope_ids_filtered,
            )?)
        } else {
            None
        };

        Ok((
            updated_resource_attr_filter,
            scope_attr_filter,
            log_record_filter,
            updated_log_attr_filter,
        ))
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

        // invert flag depending on whether we are excluding or including
        if invert {
            resource_attr_filter =
                arrow::compute::not(&resource_attr_filter).expect("not doesn't fail");

            log_record_filter = arrow::compute::not(&log_record_filter).expect("not doesn't fail");

            log_attr_filter = arrow::compute::not(&log_attr_filter).expect("not doesn't fail");
        }

        Ok((resource_attr_filter, log_record_filter, log_attr_filter))
    }

    /// Creates a booleanarray that will filter a resource_attribute record batch based on the
    /// defined resource attributes we want to match
    fn get_resource_attr_filter(&self, logs_payload: &OtapArrowRecords) -> Result<BooleanArray> {
        // get resource_attrs record batch
        let resource_attrs = match logs_payload.get(ArrowPayloadType::ResourceAttrs) {
            Some(record_batch) => {
                if self.resource_attributes.is_empty() {
                    return Ok(BooleanArray::from(BooleanBuffer::new_set(
                        record_batch.num_rows(),
                    )));
                }
                record_batch
            }
            None => {
                // if there is no record batch then
                // if we didn't plan to match any resource attributes -> allow all values through
                if self.resource_attributes.is_empty() {
                    return Ok(BooleanArray::from(BooleanBuffer::new_set(
                        NO_RECORD_BATCH_FILTER_SIZE,
                    )));
                } else {
                    // if we did match on resource attributes then there are no attributes to match
                    return Ok(BooleanArray::from(BooleanBuffer::new_unset(
                        NO_RECORD_BATCH_FILTER_SIZE,
                    )));
                }
            }
        };

        let num_rows = resource_attrs.num_rows();
        let mut attributes_filter = BooleanArray::new_null(num_rows);
        let key_column = get_required_array(resource_attrs, consts::ATTRIBUTE_KEY)?;

        // generate the filter for this record_batch
        for attribute in &self.resource_attributes {
            // match on key
            let key_scalar = StringArray::new_scalar(attribute.key.clone());
            // since we use a scalar here we don't have to worry a column length mismatch when we compare
            let key_filter = arrow::compute::kernels::cmp::eq(&key_column, &key_scalar)
                .expect("can compare string key column to string scalar");
            // and match on value
            let value_filter = match &attribute.value {
                AnyValue::String(value) => {
                    // get string column
                    let string_column = get_required_array(resource_attrs, consts::ATTRIBUTE_STR)?;
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
                    let int_column = resource_attrs.column_by_name(consts::ATTRIBUTE_INT);

                    // check if column exists if not then there is no resource that has this attribute so we can return a all false boolean array
                    match int_column {
                        Some(column) => {
                            let value_scalar = Int64Array::new_scalar(*value);
                            // since we use a scalar here we don't have to worry a column length mismatch when we compare

                            arrow::compute::kernels::cmp::eq(&column, &value_scalar)
                                .expect("can compare i64 value column to i64 scalar")
                        }
                        None => {
                            return Ok(BooleanArray::from(BooleanBuffer::new_unset(num_rows)));
                        }
                    }
                }
                AnyValue::Double(value) => {
                    let double_column = resource_attrs.column_by_name(consts::ATTRIBUTE_DOUBLE);
                    match double_column {
                        Some(column) => {
                            let value_scalar = Float64Array::new_scalar(*value);
                            // since we use a scalar here we don't have to worry a column length mismatch when we compare

                            arrow::compute::kernels::cmp::eq(&column, &value_scalar)
                                .expect("can compare f64 value column to f64 scalar")
                        }
                        None => {
                            return Ok(BooleanArray::from(BooleanBuffer::new_unset(num_rows)));
                        }
                    }
                }
                AnyValue::Boolean(value) => {
                    let bool_column = resource_attrs.column_by_name(consts::ATTRIBUTE_BOOL);
                    match bool_column {
                        Some(column) => {
                            let value_scalar = BooleanArray::new_scalar(*value);
                            // since we use a scalar here we don't have to worry a column length mismatch when we compare

                            arrow::compute::kernels::cmp::eq(&column, &value_scalar)
                                .expect("can compare bool value column to bool scalar")
                        }
                        None => {
                            return Ok(BooleanArray::from(BooleanBuffer::new_unset(num_rows)));
                        }
                    }
                }
                _ => {
                    // ToDo add keyvalue, array, and bytes
                    return Ok(BooleanArray::from(BooleanBuffer::new_unset(num_rows)));
                }
            };
            // build filter that checks for both matching key and value filter
            let attribute_filter = arrow::compute::and_kleene(&key_filter, &value_filter)
                .map_err(|e| Error::ColumnLengthMismatch { source: e })?;
            // combine with overrall filter
            attributes_filter = arrow::compute::or_kleene(&attributes_filter, &attribute_filter)
                .map_err(|e| Error::ColumnLengthMismatch { source: e })?;
        }

        // using the attribute filter we need to get the ids of the rows that match and use that to build our final filter
        // this is to make sure we don't drop attributes that belong to a resource that matched the resource_attributes that
        // were defined

        // we get the id column and apply filter to get the ids we should keep
        let parent_id_column = get_required_array(resource_attrs, consts::PARENT_ID)?;
        // the ids should show up self.resource_attr.len() times otherwise they don't have all the required attributes
        let ids = arrow::compute::filter(&parent_id_column, &attributes_filter)
            .map_err(|e| Error::ColumnLengthMismatch { source: e })?;
        // extract correct ids
        let ids = ids
            .as_any()
            .downcast_ref::<UInt16Array>()
            .expect("array can be downcast to UInt16Array");
        // remove null values
        let mut ids_counted: HashMap<u16, usize> = HashMap::with_capacity(ids.len());
        // since we require that all the resource attributes match we use the count of the ids extracted to determine a full match
        // a full match should meant that the amount of times a id appears is equal the number of resource attributes we want to match on
        for id in ids.iter().flatten() {
            *ids_counted.entry(id).or_default() += 1;
        }

        let required_ids_count = self.resource_attributes.len();
        // filter out ids that don't fully match
        ids_counted.retain(|_key, value| *value >= required_ids_count);

        // return filter built with the ids
        build_uint16_id_filter(parent_id_column, ids_counted.into_keys().collect())
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

    /// Creates a booleanarray that will filter a log_attribute record batch based on the
    /// defined log record attributes we want to match
    fn get_log_attr_filter(&self, logs_payload: &OtapArrowRecords) -> Result<BooleanArray> {
        // get log_attrs record batch
        let log_attrs = match logs_payload.get(ArrowPayloadType::LogAttrs) {
            Some(record_batch) => {
                if self.record_attributes.is_empty() {
                    return Ok(BooleanArray::from(BooleanBuffer::new_set(
                        record_batch.num_rows(),
                    )));
                }
                record_batch
            }
            None => {
                // if there is no record batch then
                // if we didn't plan to match any record attributes -> allow all values through
                if self.record_attributes.is_empty() {
                    return Ok(BooleanArray::from(BooleanBuffer::new_set(
                        NO_RECORD_BATCH_FILTER_SIZE,
                    )));
                } else {
                    // if we did match on record attributes then there are no attributes to match
                    return Ok(BooleanArray::from(BooleanBuffer::new_unset(
                        NO_RECORD_BATCH_FILTER_SIZE,
                    )));
                }
            }
        };

        let num_rows = log_attrs.num_rows();
        // if there is nothing to filter we return all true
        let mut attributes_filter = BooleanArray::new_null(num_rows);

        let key_column = get_required_array(log_attrs, consts::ATTRIBUTE_KEY)?;

        // generate the filter for this record_batch
        for attribute in &self.record_attributes {
            // match on key
            let key_scalar = StringArray::new_scalar(attribute.key.clone());
            // since we use a scalar here we don't have to worry a column length mismatch when we compare

            let key_filter = arrow::compute::kernels::cmp::eq(&key_column, &key_scalar)
                .expect("can compare string key column to string scalar");
            // match on value
            let value_filter = match &attribute.value {
                AnyValue::String(value) => {
                    // get string column
                    let string_column = get_required_array(log_attrs, consts::ATTRIBUTE_STR)?;

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
                    let int_column = log_attrs.column_by_name(consts::ATTRIBUTE_INT);
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
                    let double_column = log_attrs.column_by_name(consts::ATTRIBUTE_DOUBLE);
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
                    let bool_column = log_attrs.column_by_name(consts::ATTRIBUTE_BOOL);
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
            // build filter that checks for both matching key and value filter
            let attribute_filter = arrow::compute::and_kleene(&key_filter, &value_filter)
                .map_err(|e| Error::ColumnLengthMismatch { source: e })?;
            // combine with rest of filters
            attributes_filter = arrow::compute::or_kleene(&attributes_filter, &attribute_filter)
                .map_err(|e| Error::ColumnLengthMismatch { source: e })?;
        }

        // now we get ids of filtered attributes to make sure we don't drop any attributes that belong to the log record
        let parent_id_column = get_required_array(log_attrs, consts::PARENT_ID)?;

        let ids = get_uint16_ids(parent_id_column, &attributes_filter, consts::PARENT_ID)?;
        // build filter around the ids and return the filter
        build_uint16_id_filter(parent_id_column, ids)
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
