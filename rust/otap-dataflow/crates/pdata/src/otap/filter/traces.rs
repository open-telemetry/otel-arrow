// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module focuses on taking a filter definition for Traces and building a filter
//! as a BooleanArray for the Traces, ResourceAttr, and LogsAttr OTAP Record Batches
//!

use crate::arrays::{get_required_array, get_required_array_from_struct_array_from_record_batch};
use crate::otap::OtapArrowRecords;
use crate::otap::error::{Error, Result};
use crate::otap::filter::{
    AnyValue, KeyValue, MatchType, NO_RECORD_BATCH_FILTER_SIZE, apply_filter,
    build_uint16_id_filter, default_match_type, get_uint16_ids, new_filter, nulls_to_false,
    regex_match_column, update_filter, update_primary_filter,
};
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use crate::schema::consts;
use arrow::array::{Array, BooleanArray, Float64Array, Int64Array, StringArray, UInt16Array};
use arrow::buffer::BooleanBuffer;
use serde::Deserialize;
use std::collections::HashMap;

/// struct that describes the overall requirements to use in order to filter logs
#[derive(Debug, Clone, Deserialize)]
pub struct TraceFilter {
    // Include match properties describe logs that should be included in the Collector Service pipeline,
    // all other logs should be dropped from further processing.
    // If both Include and Exclude are specified, Include filtering occurs first.
    include: Option<TraceMatchProperties>,
    // Exclude match properties describe logs that should be excluded from the Collector Service pipeline,
    // all other logs should be included.
    // If both Include and Exclude are specified, Include filtering occurs first.
    exclude: Option<TraceMatchProperties>,
    // ToDo: Add ottl support -> see golang version https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/8558294afa723b9ed917ed5d6bb6c656bb096a49/processor/filterprocessor/config.go#L90
}

/// LogMatchProperties specifies the set of properties in a log to match against and the type of string pattern matching to use.
#[derive(Debug, Clone, Deserialize)]
pub struct TraceMatchProperties {
    // MatchType specifies the type of matching desired
    #[serde(default = "default_match_type")]
    match_type: MatchType,

    // ResourceAttributes defines a list of possible resource attributes to match traces against.
    // A match occurs if any resource attribute matches all expressions in this given list.
    #[serde(default)]
    resource_attributes: Vec<KeyValue>,

    // SpanAttributes defines a list of possible record attributes to match traces against.
    // A match occurs if any record attribute matches at least one expression in this given list.
    #[serde(default)]
    span_attributes: Vec<KeyValue>,

    // SpanNames is a list of span names that the span's names field must match against.
    span_names: Vec<String>,

    // EventNames is a list of event names that the span's events field must match against.
    #[serde(default)]
    event_names: Vec<String>,

    // EventsAttributes is a list of possible event attributes to match span events against.
    // A match occurs of any event attribute matches at least one expression in this given list.
    #[serde(default)]
    event_attributes: Vec<KeyValue>,

    // LinkAttributes is a list of possible link attributes to match span links against.
    // A match occurs of any link attribute matches at least one expression in this given list.
    #[serde(default)]
    link_attributes: Vec<KeyValue>,
}

impl TraceFilter {
    /// create a new log filter
    #[must_use]
    pub fn new(
        include: Option<TraceMatchProperties>,
        exclude: Option<TraceMatchProperties>,
    ) -> Self {
        Self { include, exclude }
    }

    /// take a logs payload and return the filtered result
    pub fn filter(&self, mut traces_payload: OtapArrowRecords) -> Result<OtapArrowRecords> {
        let (
            resource_attr_filter,
            span_filter,
            span_attr_filter,
            span_event_filter,
            span_event_attr_filter,
            span_link_attr_filter,
        ) = if let Some(include_config) = &self.include
            && let Some(exclude_config) = &self.exclude
        {
            let (
                include_resource_attr_filter,
                include_span_filter,
                include_span_attr_filter,
                include_span_event_filter,
                include_span_event_attr_filter,
                include_span_link_attr_filter,
            ) = include_config.create_filters(&traces_payload, false)?;
            let (
                exclude_resource_attr_filter,
                exclude_span_filter,
                exclude_span_attr_filter,
                exclude_span_event_filter,
                exclude_span_event_attr_filter,
                exclude_span_link_attr_filter,
            ) = exclude_config.create_filters(&traces_payload, true)?;

            // combine the include and exclude filters
            let resource_attr_filter = arrow::compute::and_kleene(
                &include_resource_attr_filter,
                &exclude_resource_attr_filter,
            )
            .map_err(|e| Error::ColumnLengthMismatch { source: e })?;
            let span_filter =
                arrow::compute::and_kleene(&include_span_filter, &exclude_span_filter).map_err(
                    |e: arrow_schema::ArrowError| Error::ColumnLengthMismatch { source: e },
                )?;
            let span_attr_filter =
                arrow::compute::and_kleene(&include_span_attr_filter, &exclude_span_attr_filter)
                    .map_err(|e| Error::ColumnLengthMismatch { source: e })?;

            let span_event_filter =
                arrow::compute::and_kleene(&include_span_event_filter, &exclude_span_event_filter)
                    .map_err(|e| Error::ColumnLengthMismatch { source: e })?;

            let span_event_attr_filter = arrow::compute::and_kleene(
                &include_span_event_attr_filter,
                &exclude_span_event_attr_filter,
            )
            .map_err(|e| Error::ColumnLengthMismatch { source: e })?;

            let span_link_attr_filter = arrow::compute::and_kleene(
                &include_span_link_attr_filter,
                &exclude_span_link_attr_filter,
            )
            .map_err(|e| Error::ColumnLengthMismatch { source: e })?;
            (
                resource_attr_filter,
                span_filter,
                span_attr_filter,
                span_event_filter,
                span_event_attr_filter,
                span_link_attr_filter,
            )
        } else if self.include.is_none()
            && let Some(exclude_config) = &self.exclude
        {
            exclude_config.create_filters(&traces_payload, true)?
        } else if let Some(include_config) = &self.include
            && self.exclude.is_none()
        {
            include_config.create_filters(&traces_payload, false)?
        } else {
            // both include and exclude is none
            return Ok(traces_payload);
        };

        let (span_filter, optional_record_batch_filters) = self.sync_up_filters(
            &traces_payload,
            resource_attr_filter,
            span_filter,
            span_attr_filter,
            span_event_filter,
            span_event_attr_filter,
            span_link_attr_filter,
        )?;

        apply_filter(&mut traces_payload, ArrowPayloadType::Spans, &span_filter)?;

        for (payload_type, filter) in optional_record_batch_filters {
            apply_filter(&mut traces_payload, payload_type, &filter)?;
        }

        Ok(traces_payload)
    }

    /// this function takes the filters for each record batch and makes sure that incomplete
    /// returns the cleaned up filters that can be immediately applied on the record batches
    fn sync_up_filters(
        &self,
        traces_payload: &OtapArrowRecords,
        resource_attr_filter: BooleanArray,
        mut span_filter: BooleanArray,
        span_attr_filter: BooleanArray,
        span_event_filter: BooleanArray,
        span_event_attr_filter: BooleanArray,
        span_link_attr_filter: BooleanArray,
    ) -> Result<((BooleanArray, HashMap<ArrowPayloadType, BooleanArray>))> {
        // get the record batches we are going to filter
        let resource_attrs = traces_payload.get(ArrowPayloadType::ResourceAttrs);
        let spans = traces_payload
            .get(ArrowPayloadType::Spans)
            .ok_or_else(|| Error::SpanRecordNotFound {})?;
        let span_attrs = traces_payload.get(ArrowPayloadType::SpanAttrs);
        let scope_attrs = traces_payload.get(ArrowPayloadType::ScopeAttrs);
        let span_events = traces_payload.get(ArrowPayloadType::SpanEvents);
        let span_event_attrs = traces_payload.get(ArrowPayloadType::SpanEventAttrs);
        let span_links = traces_payload.get(ArrowPayloadType::SpanLinks);
        let span_link_attrs = traces_payload.get(ArrowPayloadType::SpanLinkAttrs);

        // get the id columns from record batch
        let span_ids_column = get_required_array(spans, consts::ID)?;
        let span_resource_ids_column = get_required_array_from_struct_array_from_record_batch(
            spans,
            consts::RESOURCE,
            consts::ID,
        )?;
        let span_scope_ids_column = get_required_array_from_struct_array_from_record_batch(
            spans,
            consts::SCOPE,
            consts::ID,
        )?;

        // optional record batch
        match resource_attrs {
            Some(resource_attrs_record_batch) => {
                span_filter = update_primary_filter(
                    record_batch,
                    id_column,
                    filter_to_update,
                    primary_filter,
                )?;
                // apply current logic
            }
            None => {
                if resource_attr_filter.true_count() == 0 {
                    // the configuration required certain resource_attributes but found none so we can return early
                    // remove all elements as nothing matches
                    return Ok((
                        BooleanArray::from(BooleanBuffer::new_unset(span_filter.len())),
                        HashMap::new(),
                    ));
                }
            }
        }

        match span_attrs {
            Some(span_attrs_record_batch) => {
                span_filter = update_primary_filter(
                    record_batch,
                    id_column,
                    filter_to_update,
                    primary_filter,
                )?;
            }
            None => {
                if span_attr_filter.true_count() == 0 {
                    // the configuration required certain resource_attributes but found none so we can return early
                    // remove all elements as nothing matches
                    return Ok((
                        BooleanArray::from(BooleanBuffer::new_unset(span_filter.len())),
                        HashMap::new(),
                    ));
                }
            }
        }

        match span_events {
            Some(span_events_record_batch) => {
                span_filter = update_primary_filter(
                    record_batch,
                    id_column,
                    filter_to_update,
                    primary_filter,
                )?;
            }
            None => {
                if span_event_filter.true_count() == 0 {
                    // the configuration required certain resource_attributes but found none so we can return early
                    // remove all elements as nothing matches
                    return Ok((
                        BooleanArray::from(BooleanBuffer::new_unset(span_filter.len())),
                        HashMap::new(),
                    ));
                }
            }
        }

        match span_event_attrs {
            Some(span_event_attrs_record_batch) => {
                span_filter = update_primary_filter(
                    record_batch,
                    id_column,
                    filter_to_update,
                    primary_filter,
                )?;
            }
            None => {
                if span_event_attr_filter.true_count() == 0 {
                    // the configuration required certain resource_attributes but found none so we can return early
                    // remove all elements as nothing matches
                    return Ok((
                        BooleanArray::from(BooleanBuffer::new_unset(span_filter.len())),
                        HashMap::new(),
                    ));
                }
            }
        }

        match span_link_attrs {
            Some(span_link_attrs_record_batch) => {
                span_filter = update_primary_filter(
                    record_batch,
                    id_column,
                    filter_to_update,
                    primary_filter,
                )?;
            }
            None => {
                if span_link_attr_filter.true_count() == 0 {
                    // the configuration required certain resource_attributes but found none so we can return early
                    // remove all elements as nothing matches
                    return Ok((
                        BooleanArray::from(BooleanBuffer::new_unset(span_filter.len())),
                        HashMap::new(),
                    ));
                }
            }
        }

        // now using the updated span_filter we need to update the rest of the filers

        // use hashmap to map filters to their payload types to return,
        // only record batches that exist will have their filter added to this hashmap
        let mut optional_record_batch_filters = HashMap::new();

        if let Some(span_attrs_record_batch) = span_attrs {
            _ = optional_record_batch_filters.insert(
                ArrowPayloadType::SpanAttrs,
                update_filter(
                    span_attrs_record_batch,
                    span_ids_column,
                    &span_attr_filter,
                    &span_filter,
                )?,
            );
        }

        if let Some(span_events_record_batch) = span_events {
            _ = optional_record_batch_filters.insert(
                ArrowPayloadType::SpanEvents,
                update_filter(
                    span_events_record_batch,
                    span_ids_column,
                    &span_event_filter,
                    &span_filter,
                )?,
            );
        }

        if let Some(span_event_attrs_record_batch) = span_event_attrs {
            _ = optional_record_batch_filters.insert(
                ArrowPayloadType::SpanEventAttrs,
                update_filter(
                    span_event_attrs_record_batch,
                    span_ids_column,
                    &span_event_attr_filter,
                    &span_filter,
                )?,
            );
        }

        if let Some(span_link_attrs_record_batch) = span_link_attrs {
            _ = optional_record_batch_filters.insert(
                ArrowPayloadType::SpanLinkAttrs,
                update_filter(
                    span_link_attrs_record_batch,
                    span_ids_column,
                    &span_link_attr_filter,
                    &span_filter,
                )?,
            );
        }

        // part 4: clean up resource attrs

        if let Some(resource_attrs_record_batch) = resource_attrs {
            _ = optional_record_batch_filters.insert(
                ArrowPayloadType::ResourceAttrs,
                update_filter(
                    resource_attrs_record_batch,
                    span_ids_column,
                    &resource_attr_filter,
                    &span_filter,
                )?,
            );
        }

        if let Some(scope_attrs_record_batch) = scope_attrs {
            _ = optional_record_batch_filters.insert(
                ArrowPayloadType::ScopeAttrs,
                new_filter(
                    scope_attrs_record_batch,
                    span_scope_ids_column,
                    &span_filter,
                )?,
            );
        }

        if let Some(span_links_record_batch) = span_links {
            _ = optional_record_batch_filters.insert(
                ArrowPayloadType::SpanLinks,
                new_filter(span_links_record_batch, span_ids_column, &span_filter)?,
            );
        }

        Ok((span_filter, optional_record_batch_filters))
    }
}

impl TraceMatchProperties {
    /// create a new LogMatchProperties
    #[must_use]
    pub fn new(
        match_type: MatchType,
        resource_attributes: Vec<KeyValue>,
        span_attributes: Vec<KeyValue>,
        span_names: Vec<String>,
        event_names: Vec<String>,
        event_attributes: Vec<KeyValue>,
        link_attributes: Vec<KeyValue>,
    ) -> Self {
        Self {
            match_type,
            resource_attributes,
            span_attributes,
            span_names,
            event_names,
            event_attributes,
            link_attributes,
        }
    }

    /// create filter takes a traces_payload and returns the filters for each of the record batches, also takes a invert flag to determine if the filters will be inverted
    pub fn create_filters(
        &self,
        traces_payload: &OtapArrowRecords,
        invert: bool,
    ) -> Result<(
        BooleanArray,
        BooleanArray,
        BooleanArray,
        BooleanArray,
        BooleanArray,
        BooleanArray,
    )> {
        let (
            mut resource_attr_filter,
            mut span_filter,
            mut span_attr_filter,
            mut span_event_filter,
            mut span_event_attr_filter,
            mut span_link_attr_filter,
        ) = (
            self.get_resource_attr_filter(traces_payload)?,
            self.get_span_filter(traces_payload)?,
            self.get_span_attr_filter(traces_payload)?,
            self.get_span_event_filter(traces_payload)?,
            self.get_span_event_attr_filter(traces_payload)?,
            self.get_span_link_attr_filter(traces_payload)?,
        );

        // invert flag depending on whether we are excluding or including
        if invert {
            resource_attr_filter =
                arrow::compute::not(&resource_attr_filter).expect("not doesn't fail");

            span_filter = arrow::compute::not(&span_filter).expect("not doesn't fail");

            span_attr_filter = arrow::compute::not(&span_attr_filter).expect("not doesn't fail");

            span_event_filter = arrow::compute::not(&span_event_filter).expect("not doesn't fail");

            span_event_attr_filter =
                arrow::compute::not(&span_event_attr_filter).expect("not doesn't fail");

            span_link_attr_filter =
                arrow::compute::not(&span_link_attr_filter).expect("not doesn't fail");
        }

        Ok((
            resource_attr_filter,
            span_filter,
            span_attr_filter,
            span_event_filter,
            span_event_attr_filter,
            span_link_attr_filter,
        ))
    }

    /// Creates a booleanarray that will filter a resource_attribute record batch based on the
    /// defined resource attributes we want to match
    fn get_resource_attr_filter(&self, traces_payload: &OtapArrowRecords) -> Result<BooleanArray> {
        // get resource_attrs record batch
        let resource_attrs = match traces_payload.get(ArrowPayloadType::ResourceAttrs) {
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
    fn get_span_filter(&self, traces_payload: &OtapArrowRecords) -> Result<BooleanArray> {
        let spans = traces_payload
            .get(ArrowPayloadType::Spans)
            .ok_or_else(|| Error::SpanRecordNotFound)?;
        let num_rows = spans.num_rows();
        // create filter for span names
        let mut filter: BooleanArray = BooleanArray::from(BooleanBuffer::new_set(num_rows));

        if !&self.span_names.is_empty() {
            // create filter for span names
            let names_column = get_required_array(spans, consts::NAME)?;
            for name in &self.span_names {
                // match on body value
                let name_filter = match self.match_type {
                    MatchType::Regexp => regex_match_column(names_column, name)?,
                    MatchType::Strict => {
                        let value_scalar = StringArray::new_scalar(name);
                        // since we use a scalar here we don't have to worry a column length mismatch when we compare

                        arrow::compute::kernels::cmp::eq(&names_column, &value_scalar)
                            .expect("can compare string value column to string scalar")
                    }
                };
                filter = arrow::compute::or_kleene(&name_filter, &filter)
                    .map_err(|e| Error::ColumnLengthMismatch { source: e })?;
                // combine the filters
            }
        }

        Ok(nulls_to_false(&filter))
    }

    /// Creates a booleanarray that will filter a span_attribute record batch based on the
    /// defined span attributes we want to match
    fn get_span_attr_filter(&self, traces_payload: &OtapArrowRecords) -> Result<BooleanArray> {
        // get span_attrs record batch
        let span_attrs = match traces_payload.get(ArrowPayloadType::SpanAttrs) {
            Some(record_batch) => {
                if self.span_attributes.is_empty() {
                    return Ok(BooleanArray::from(BooleanBuffer::new_set(
                        record_batch.num_rows(),
                    )));
                }
                record_batch
            }
            None => {
                // if there is no record batch then
                // if we didn't plan to match any record attributes -> allow all values through
                if self.span_attributes.is_empty() {
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

        let num_rows = span_attrs.num_rows();
        // if there is nothing to filter we return all true
        let mut attributes_filter = BooleanArray::new_null(num_rows);

        let key_column = get_required_array(span_attrs, consts::ATTRIBUTE_KEY)?;

        // generate the filter for this record_batch
        for attribute in &self.span_attributes {
            // match on key
            let key_scalar = StringArray::new_scalar(attribute.key.clone());
            // since we use a scalar here we don't have to worry a column length mismatch when we compare

            let key_filter = arrow::compute::kernels::cmp::eq(&key_column, &key_scalar)
                .expect("can compare string key column to string scalar");
            // match on value
            let value_filter = match &attribute.value {
                AnyValue::String(value) => {
                    // get string column
                    let string_column = get_required_array(span_attrs, consts::ATTRIBUTE_STR)?;

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
                    let int_column = span_attrs.column_by_name(consts::ATTRIBUTE_INT);
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
                    let double_column = span_attrs.column_by_name(consts::ATTRIBUTE_DOUBLE);
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
                    let bool_column = span_attrs.column_by_name(consts::ATTRIBUTE_BOOL);
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
        let parent_id_column = get_required_array(span_attrs, consts::PARENT_ID)?;

        let ids = get_uint16_ids(parent_id_column, &attributes_filter, consts::PARENT_ID)?;
        // build filter around the ids and return the filter
        build_uint16_id_filter(parent_id_column, ids)
    }

    /// Creates a booleanarray that will filter a span_event record batch based on the
    /// defined event names we want to match
    fn get_span_event_filter(&self, traces_payload: &OtapArrowRecords) -> Result<BooleanArray> {
        let span_events = match traces_payload.get(ArrowPayloadType::SpanEvents) {
            Some(record_batch) => {
                if self.event_names.is_empty() {
                    return Ok(BooleanArray::from(BooleanBuffer::new_set(
                        record_batch.num_rows(),
                    )));
                }
                record_batch
            }
            None => {
                // if there is no record batch then
                // if we didn't plan to match any record attributes -> allow all values through
                if self.event_names.is_empty() {
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

        let num_rows = span_events.num_rows();
        // create filter for span names
        let mut filter: BooleanArray = BooleanArray::from(BooleanBuffer::new_set(num_rows));

        if !&self.event_names.is_empty() {
            // create filter for span names
            let names_column = get_required_array(span_events, consts::NAME)?;
            for name in &self.event_names {
                // match on body value
                let name_filter = match self.match_type {
                    MatchType::Regexp => regex_match_column(names_column, name)?,
                    MatchType::Strict => {
                        let value_scalar = StringArray::new_scalar(name);
                        // since we use a scalar here we don't have to worry a column length mismatch when we compare

                        arrow::compute::kernels::cmp::eq(&names_column, &value_scalar)
                            .expect("can compare string value column to string scalar")
                    }
                };
                filter = arrow::compute::or_kleene(&name_filter, &filter)
                    .map_err(|e| Error::ColumnLengthMismatch { source: e })?;
                // combine the filters
            }
        }

        Ok(nulls_to_false(&filter))
    }

    /// Creates a booleanarray that will filter a span_event_attribute record batch based on the
    /// defined span event attributes we want to match
    fn get_span_event_attr_filter(
        &self,
        traces_payload: &OtapArrowRecords,
    ) -> Result<BooleanArray> {
        let span_event_attrs = match traces_payload.get(ArrowPayloadType::SpanEventAttrs) {
            Some(record_batch) => {
                if self.event_names.is_empty() {
                    return Ok(BooleanArray::from(BooleanBuffer::new_set(
                        record_batch.num_rows(),
                    )));
                }
                record_batch
            }
            None => {
                // if there is no record batch then
                // if we didn't plan to match any record attributes -> allow all values through
                if self.event_names.is_empty() {
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

        let num_rows = span_event_attrs.num_rows();
        // if there is nothing to filter we return all true
        let mut attributes_filter = BooleanArray::new_null(num_rows);

        let key_column = get_required_array(span_event_attrs, consts::ATTRIBUTE_KEY)?;

        // generate the filter for this record_batch
        for attribute in &self.event_attributes {
            // match on key
            let key_scalar = StringArray::new_scalar(attribute.key.clone());
            // since we use a scalar here we don't have to worry a column length mismatch when we compare

            let key_filter = arrow::compute::kernels::cmp::eq(&key_column, &key_scalar)
                .expect("can compare string key column to string scalar");
            // match on value
            let value_filter = match &attribute.value {
                AnyValue::String(value) => {
                    // get string column
                    let string_column =
                        get_required_array(span_event_attrs, consts::ATTRIBUTE_STR)?;

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
                    let int_column = span_event_attrs.column_by_name(consts::ATTRIBUTE_INT);
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
                    let double_column = span_event_attrs.column_by_name(consts::ATTRIBUTE_DOUBLE);
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
                    let bool_column = span_event_attrs.column_by_name(consts::ATTRIBUTE_BOOL);
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
        let parent_id_column = get_required_array(span_event_attrs, consts::PARENT_ID)?;

        let ids = get_uint16_ids(parent_id_column, &attributes_filter, consts::PARENT_ID)?;
        // build filter around the ids and return the filter
        build_uint16_id_filter(parent_id_column, ids)
    }

    /// Creates a booleanarray that will filter a span_link_attribute record batch based on the
    /// defined span link attributes we want to match
    fn get_span_link_attr_filter(&self, traces_payload: &OtapArrowRecords) -> Result<BooleanArray> {
        let span_link_attrs = match traces_payload.get(ArrowPayloadType::SpanLinkAttrs) {
            Some(record_batch) => {
                if self.event_names.is_empty() {
                    return Ok(BooleanArray::from(BooleanBuffer::new_set(
                        record_batch.num_rows(),
                    )));
                }
                record_batch
            }
            None => {
                // if there is no record batch then
                // if we didn't plan to match any record attributes -> allow all values through
                if self.event_names.is_empty() {
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

        let num_rows = span_link_attrs.num_rows();
        // if there is nothing to filter we return all true
        let mut attributes_filter = BooleanArray::new_null(num_rows);

        let key_column = get_required_array(span_link_attrs, consts::ATTRIBUTE_KEY)?;

        // generate the filter for this record_batch
        for attribute in &self.link_attributes {
            // match on key
            let key_scalar = StringArray::new_scalar(attribute.key.clone());
            // since we use a scalar here we don't have to worry a column length mismatch when we compare

            let key_filter = arrow::compute::kernels::cmp::eq(&key_column, &key_scalar)
                .expect("can compare string key column to string scalar");
            // match on value
            let value_filter = match &attribute.value {
                AnyValue::String(value) => {
                    // get string column
                    let string_column = get_required_array(span_link_attrs, consts::ATTRIBUTE_STR)?;

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
                    let int_column = span_link_attrs.column_by_name(consts::ATTRIBUTE_INT);
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
                    let double_column = span_link_attrs.column_by_name(consts::ATTRIBUTE_DOUBLE);
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
                    let bool_column = span_link_attrs.column_by_name(consts::ATTRIBUTE_BOOL);
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
        let parent_id_column = get_required_array(span_link_attrs, consts::PARENT_ID)?;

        let ids = get_uint16_ids(parent_id_column, &attributes_filter, consts::PARENT_ID)?;
        // build filter around the ids and return the filter
        build_uint16_id_filter(parent_id_column, ids)
    }
}
