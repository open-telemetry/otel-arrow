// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//!
//!

use crate::arrays::get_required_array;
use crate::otap::OtapArrowRecords;
use crate::otap::error::{self, Result};
use crate::otap::filter::{AnyValue, KeyValue};
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use crate::schema::consts;
use arrow::array::{BooleanArray, Float64Array, Int32Array, Int64Array, StringArray, UInt16Array};
use serde::Deserialize;
use snafu::OptionExt;
use std::collections::{HashMap, HashSet};

/// LogMatchProperties specifies the set of properties in a log to match against and the type of string pattern matching to use.
#[derive(Debug, Clone, Deserialize)]
pub struct LogMatchProperties {
    // LogMatchType specifies the type of matching desired
    match_type: LogMatchType,

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
    severity_number: Option<LogServerityNumberMatchProperties>,

    // LogBodies is a list of values that the LogRecord's body field must match
    // against.
    bodies: Vec<AnyValue>,
}

/// LogSeverityNumberMatchProperties specifies the requirements needed to match on the log severity field
#[derive(Debug, Clone, Deserialize)]
pub struct LogServerityNumberMatchProperties {
    // Min is the minimum severity needed for the log record to match.
    // This corresponds to the short names specified here:
    // https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/logs/data-model.md#displaying-severity
    // this field is case-insensitive ("INFO" == "info")
    min: i32,
    // MatchUndefined lets logs records with "unknown" severity match.
    // If MinSeverity is not set, this field is ignored, as fields are not matched based on severity.
    match_undefined: bool,
}

/// LogMatchType describes how we should match the String values provided
#[derive(Debug, Clone, Deserialize)]
pub enum LogMatchType {
    /// match on the string values exactly how they are defined
    Strict,
    /// apply string values as a regexp
    Regexp,
}

impl LogMatchProperties {
    /// create a new LogMatchProperties
    pub fn new(
        match_type: LogMatchType,
        resource_attributes: Vec<KeyValue>,
        record_attributes: Vec<KeyValue>,
        severity_texts: Vec<String>,
        severity_number: Option<LogServerityNumberMatchProperties>,
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

        let mut attributes_filter = BooleanArray::new_null(num_rows);
        let key_column = get_required_array(resource_attrs, consts::ATTRIBUTE_KEY)?;
        // generate the filter for this record_batch
        for attribute in &self.resource_attributes {
            let key_scalar = StringArray::new_scalar(attribute.key.clone());

            let key_filter = arrow::compute::kernels::cmp::eq(&key_column, &key_scalar)
                .expect("columns should have equal length");
            let value_filter = match &attribute.value {
                AnyValue::String(value) => {
                    // get string column
                    let string_column = get_required_array(resource_attrs, consts::ATTRIBUTE_STR)?;
                    match self.match_type {
                        LogMatchType::Regexp => {
                            let string_column = string_column
                                .as_any()
                                .downcast_ref::<StringArray>()
                                .expect("array can be downcast to StringArray");
                            arrow::compute::regexp_is_match_scalar(string_column, &value, None)
                                .expect("columns should have equal length")
                        }
                        LogMatchType::Strict => {
                            let value_scalar = StringArray::new_scalar(value);
                            arrow::compute::kernels::cmp::eq(&string_column, &value_scalar)
                                .expect("columns should have equal length")
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
                                .expect("columns should have equal length")
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
                                .expect("columns should have equal length")
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
                                .expect("columns should have equal length")
                        }
                        None => return Ok(vec![false; num_rows].into()),
                    }
                }
                _ => {
                    // ToDo add keyvalue, array, and bytes
                    return Ok(vec![false; num_rows].into());
                }
            };
            // build filter that checks for both matching key and value filter
            let attribute_filter = arrow::compute::and(&key_filter, &value_filter)
                .expect("boolean arrays should have equal length");
            // combine with rest of filters
            attributes_filter = arrow::compute::and(&attributes_filter, &attribute_filter)
                .expect("boolean arrays should have equal length");
        }

        // ToDo optimize the logic below where we build the final filter based on the ids
        // now we get ids of resource_attrs
        let parent_id_column = get_required_array(resource_attrs, consts::PARENT_ID)?;
        // the ids should show up self.resource_attr.len() times otherwise they don't have all the required attributes
        let ids = arrow::compute::filter(&parent_id_column, &attributes_filter)
            .expect("columns should have equal length");
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
                .expect("columns should have equal length");
            filter = arrow::compute::or_kleene(&filter, &id_filter)
                .expect("boolean arrays should have equal length");
        }

        Ok(filter)
    }

    fn get_log_record_filter(&self, logs_payload: &OtapArrowRecords) -> Result<BooleanArray> {
        let log_records = logs_payload
            .get(ArrowPayloadType::Logs)
            .context(error::LogRecordNotFoundSnafu)?;
        let num_rows = log_records.num_rows();
        // create filter for severity texts
        let severity_texts_column = log_records.column_by_name(consts::SEVERITY_TEXT).context(
            error::ColumnNotFoundSnafu {
                name: consts::SEVERITY_TEXT,
            },
        )?;

        let mut severity_texts_filter = BooleanArray::new_null(num_rows);
        for severity_text in &self.severity_texts {
            let severity_text_scalar = StringArray::new_scalar(severity_text);
            let severity_text_filter =
                arrow::compute::kernels::cmp::eq(&severity_texts_column, &severity_text_scalar)
                    .expect("columns should have equal length");
            severity_texts_filter =
                arrow::compute::or_kleene(&severity_texts_filter, &severity_text_filter)
                    .expect("boolean arrays should have equal length");
        }

        // create filter for log bodies
        let mut bodies_filter = BooleanArray::new_null(num_rows);
        for body in &self.bodies {
            let body_filter = match body {
                AnyValue::String(value) => {
                    // get string column
                    let string_column = get_required_array(log_records, consts::BODY_STR)?;
                    match self.match_type {
                        LogMatchType::Regexp => {
                            let string_column = string_column
                                .as_any()
                                .downcast_ref::<StringArray>()
                                .expect("array can be downcast to StringArray");

                            arrow::compute::regexp_is_match_scalar(string_column, &value, None)
                                .expect("columns should have equal length")
                        }
                        LogMatchType::Strict => {
                            let value_scalar = StringArray::new_scalar(value);
                            arrow::compute::kernels::cmp::eq(&string_column, &value_scalar)
                                .expect("columns should have equal length")
                        }
                    }
                }
                AnyValue::Int(value) => {
                    let int_column = log_records.column_by_name(consts::BODY_INT);
                    match int_column {
                        Some(column) => {
                            let value_scalar = Int64Array::new_scalar(*value);
                            arrow::compute::kernels::cmp::eq(&column, &value_scalar)
                                .expect("columns should have equal length")
                        }
                        None => {
                            continue;
                        }
                    }
                }
                AnyValue::Double(value) => {
                    let double_column = log_records.column_by_name(consts::BODY_DOUBLE);
                    match double_column {
                        Some(column) => {
                            let value_scalar = Float64Array::new_scalar(*value);
                            arrow::compute::kernels::cmp::eq(&column, &value_scalar)
                                .expect("columns should have equal length")
                        }
                        None => {
                            continue;
                        }
                    }
                }
                AnyValue::Boolean(value) => {
                    let bool_column = log_records.column_by_name(consts::BODY_BOOL);
                    match bool_column {
                        Some(column) => {
                            let value_scalar = BooleanArray::new_scalar(*value);
                            arrow::compute::kernels::cmp::eq(&column, &value_scalar)
                                .expect("columns should have equal length")
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
                .expect("boolean arrays should have equal length");
        }

        // combine the filters
        let mut filter = arrow::compute::and(&bodies_filter, &severity_texts_filter)
            .expect("boolean arrays should have equal length");

        // if the severity_number field is defined then we create the severity_number filter
        if let Some(severity_number_properties) = &self.severity_number {
            let severity_number_column = get_required_array(log_records, consts::SEVERITY_NUMBER)?;

            // TODO make min a string that contains the severity number type and map to the int instead
            let min_severity_number = severity_number_properties.min;
            let min_severity_scalar = Int32Array::new_scalar(min_severity_number);
            let mut severity_numbers_filter =
                arrow::compute::kernels::cmp::gt_eq(&severity_number_column, &min_severity_scalar)
                    .expect("columns should have equal length");
            // update the filter if we allow unknown
            if severity_number_properties.match_undefined {
                let unknown_severity_scalar = Int32Array::new_scalar(0);
                let unknown_severity_number_filter = arrow::compute::kernels::cmp::eq(
                    &severity_number_column,
                    &unknown_severity_scalar,
                )
                .expect("columns should have equal length");
                severity_numbers_filter = arrow::compute::or_kleene(
                    &severity_numbers_filter,
                    &unknown_severity_number_filter,
                )
                .expect("boolean arrays should have equal length");
            }
            // combine severity number filter to the log record filter
            filter = arrow::compute::and(&filter, &severity_numbers_filter)
                .expect("boolean arrays should have equal length");
        }

        Ok(filter)
    }

    fn get_log_attr_filter(&self, logs_payload: &OtapArrowRecords) -> Result<BooleanArray> {
        // get log_attrs record batch
        let log_attrs = logs_payload
            .get(ArrowPayloadType::LogAttrs)
            .context(error::LogRecordNotFoundSnafu)?;

        let num_rows = log_attrs.num_rows();
        let mut attributes_filter = BooleanArray::new_null(num_rows);

        let key_column = get_required_array(log_attrs, consts::ATTRIBUTE_KEY)?;

        // generate the filter for this record_batch
        for attribute in &self.record_attributes {
            let key_scalar = StringArray::new_scalar(attribute.key.clone());
            let key_filter = arrow::compute::kernels::cmp::eq(&key_column, &key_scalar)
                .expect("columns should have equal length");
            let value_filter = match &attribute.value {
                AnyValue::String(value) => {
                    // get string column
                    let string_column = get_required_array(log_attrs, consts::ATTRIBUTE_STR)?;

                    match self.match_type {
                        LogMatchType::Regexp => {
                            let string_column = string_column
                                .as_any()
                                .downcast_ref::<StringArray>()
                                .expect("array can be downcast to StringArray");

                            arrow::compute::regexp_is_match_scalar(string_column, &value, None)
                                .expect("columns should have equal length")
                        }
                        LogMatchType::Strict => {
                            let value_scalar = StringArray::new_scalar(value);
                            arrow::compute::kernels::cmp::eq(&string_column, &value_scalar)
                                .expect("columns should have equal length")
                        }
                    }
                }
                AnyValue::Int(value) => {
                    let int_column = log_attrs.column_by_name(consts::ATTRIBUTE_INT);
                    match int_column {
                        Some(column) => {
                            let value_scalar = Int64Array::new_scalar(*value);
                            arrow::compute::kernels::cmp::eq(&column, &value_scalar)
                                .expect("columns should have equal length")
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
                                .expect("columns should have equal length")
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
                                .expect("columns should have equal length")
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
            let attribute_filter = arrow::compute::and(&key_filter, &value_filter)
                .expect("boolean arrays should have equal length");
            // combine with rest of filters
            attributes_filter = arrow::compute::or_kleene(&attributes_filter, &attribute_filter)
                .expect("boolean arrays should have equal length");
        }

        // now we get ids of
        let parent_id_column = get_required_array(log_attrs, consts::PARENT_ID)?;

        let ids = arrow::compute::filter(&parent_id_column, &attributes_filter)
            .expect("columns should have equal length");
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
                .expect("columns should have equal length");
            filter = arrow::compute::or_kleene(&filter, &id_filter)
                .expect("boolean arrays should have equal length");
        }

        Ok(filter)
    }
}
