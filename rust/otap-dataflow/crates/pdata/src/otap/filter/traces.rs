// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module focuses on taking a filter definition for Traces and building a filter
//! as a BooleanArray for the Spans, ResourceAttr, SpansAttr, SpanEvents, SpanEventAttrs, and SpanLinkAttrs OTAP Record Batches
//!

use crate::arrays::{get_required_array, get_required_array_from_struct_array_from_record_batch};
use crate::otap::OtapArrowRecords;
use crate::otap::error::{Error, Result};
use crate::otap::filter::{
    KeyValue, MatchType, NO_RECORD_BATCH_FILTER_SIZE, apply_filter, default_match_type,
    get_attr_filter, get_resource_attr_filter, new_child_record_batch_filter,
    new_parent_record_batch_filter, nulls_to_false, regex_match_column,
    update_child_record_batch_filter, update_parent_record_batch_filter,
};
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use crate::schema::consts;
use arrow::array::{BooleanArray, StringArray};
use arrow::buffer::BooleanBuffer;
use serde::Deserialize;
use std::collections::HashMap;

/// struct that describes the overall requirements to use in order to filter traces
#[derive(Debug, Clone, Deserialize)]
pub struct TraceFilter {
    // Include match properties describe traces that should be included in the Collector Service pipeline,
    // all other traces should be dropped from further processing.
    // If both Include and Exclude are specified, Include filtering occurs first.
    include: Option<TraceMatchProperties>,
    // Exclude match properties describe traces that should be excluded from the Collector Service pipeline,
    // all other traces should be included.
    // If both Include and Exclude are specified, Include filtering occurs first.
    exclude: Option<TraceMatchProperties>,
    // ToDo: Add ottl support -> see golang version https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/8558294afa723b9ed917ed5d6bb6c656bb096a49/processor/filterprocessor/config.go#L90
}

/// TraceMatchProperties specifies the set of properties in a trace to match against and the type of string pattern matching to use.
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
    /// create a new trace filter
    #[must_use]
    pub fn new(
        include: Option<TraceMatchProperties>,
        exclude: Option<TraceMatchProperties>,
    ) -> Self {
        Self { include, exclude }
    }

    /// take a traces payload and return the filtered result
    pub fn filter(
        &self,
        mut traces_payload: OtapArrowRecords,
    ) -> Result<(OtapArrowRecords, u64, u64)> {
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
            let num_rows = traces_payload
                .get(ArrowPayloadType::Spans)
                .ok_or_else(|| Error::RecordBatchNotFound {
                    payload_type: ArrowPayloadType::Spans,
                })?
                .num_rows() as u64;
            return Ok((traces_payload, num_rows, num_rows));
        };

        let (span_filter, child_record_batch_filters) = self.sync_up_filters(
            &traces_payload,
            resource_attr_filter,
            span_filter,
            span_attr_filter,
            span_event_filter,
            span_event_attr_filter,
            span_link_attr_filter,
        )?;

        let (span_rows_before, span_rows_removed) =
            apply_filter(&mut traces_payload, ArrowPayloadType::Spans, &span_filter)?;

        for (payload_type, filter) in child_record_batch_filters {
            let (_, _) = apply_filter(&mut traces_payload, payload_type, &filter)?;
        }

        Ok((traces_payload, span_rows_before, span_rows_removed))
    }

    /// this function takes the filters for each record batch and makes sure that incomplete
    /// returns the cleaned up filters that can be immediately applied on the record batches
    fn sync_up_filters(
        &self,
        traces_payload: &OtapArrowRecords,
        resource_attr_filter: BooleanArray,
        mut span_filter: BooleanArray,
        span_attr_filter: BooleanArray,
        mut span_event_filter: BooleanArray,
        span_event_attr_filter: BooleanArray,
        span_link_attr_filter: BooleanArray,
    ) -> Result<(BooleanArray, HashMap<ArrowPayloadType, BooleanArray>)> {
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

        // use the event and link attrs to update the event and link record batches (their parent record batches)
        // and then we use those to update the span record batch
        match span_event_attrs {
            Some(span_event_attrs_record_batch) => {
                // get event id column
                if let Some(span_events_record_batch) = span_events {
                    let span_event_ids_column =
                        get_required_array(span_events_record_batch, consts::ID)?;
                    span_event_filter = update_parent_record_batch_filter(
                        span_event_attrs_record_batch,
                        span_event_ids_column,
                        &span_event_attr_filter,
                        &span_event_filter,
                    )?;
                } else {
                    return Err(Error::UnexpectedRecordBatchState { reason: "Span Event Attribute Record Batch found without Span Event Record Batch".to_string() });
                }
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

        // if we have span link attributes then we can create the span link filter
        let mut span_link_filter = match span_link_attrs {
            Some(span_link_attrs_record_batch) => {
                // get event id column
                if let Some(span_links_record_batch) = span_links {
                    let span_link_ids_column =
                        get_required_array(span_links_record_batch, consts::ID)?;
                    Some(new_parent_record_batch_filter(
                        span_link_attrs_record_batch,
                        span_link_ids_column,
                        &span_link_attr_filter,
                    )?)
                } else {
                    return Err(Error::UnexpectedRecordBatchState { reason: "Span Event Attribute Record Batch found without Span Event Record Batch".to_string() });
                }
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
                None
            }
        };

        // optional record batch
        match resource_attrs {
            Some(resource_attrs_record_batch) => {
                span_filter = update_parent_record_batch_filter(
                    resource_attrs_record_batch,
                    span_resource_ids_column,
                    &resource_attr_filter,
                    &span_filter,
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
                span_filter = update_parent_record_batch_filter(
                    span_attrs_record_batch,
                    span_ids_column,
                    &span_attr_filter,
                    &span_filter,
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
                span_filter = update_parent_record_batch_filter(
                    span_events_record_batch,
                    span_ids_column,
                    &span_event_filter,
                    &span_filter,
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

        if let Some(link_filter) = &span_link_filter {
            match span_links {
                Some(span_links_record_batch) => {
                    span_filter = update_parent_record_batch_filter(
                        span_links_record_batch,
                        span_ids_column,
                        link_filter,
                        &span_filter,
                    )?;
                }
                None => {
                    return Err(Error::UnexpectedRecordBatchState { reason: "Span Link Filter created from Span Link Attribute Record Batch but no Span Link Record Batch found".to_string() });
                }
            }
        }

        // now using the updated span_filter we need to update the rest of the filers

        // use hashmap to map filters to their payload types to return,
        // only record batches that exist will have their filter added to this hashmap
        let mut child_record_batch_filters = HashMap::new();

        if let Some(span_attrs_record_batch) = span_attrs {
            _ = child_record_batch_filters.insert(
                ArrowPayloadType::SpanAttrs,
                update_child_record_batch_filter(
                    span_attrs_record_batch,
                    span_ids_column,
                    &span_attr_filter,
                    &span_filter,
                )?,
            );
        }

        if let Some(span_events_record_batch) = span_events {
            // we update the span event filter because we still need it to
            // update its child
            span_event_filter = update_child_record_batch_filter(
                span_events_record_batch,
                span_ids_column,
                &span_event_filter,
                &span_filter,
            )?;
            _ = child_record_batch_filters
                .insert(ArrowPayloadType::SpanEvents, span_event_filter.clone());
        }

        span_link_filter = if let Some(span_links_record_batch) = span_links {
            if let Some(link_filter) = &span_link_filter {
                let updated_filter = update_child_record_batch_filter(
                    span_links_record_batch,
                    span_ids_column,
                    link_filter,
                    &span_filter,
                )?;

                _ = child_record_batch_filters
                    .insert(ArrowPayloadType::SpanLinks, updated_filter.clone());
                Some(updated_filter)
            } else {
                let new_filter = new_child_record_batch_filter(
                    span_links_record_batch,
                    span_ids_column,
                    &span_filter,
                )?;
                _ = child_record_batch_filters
                    .insert(ArrowPayloadType::SpanLinks, new_filter.clone());
                Some(new_filter)
            }
        } else {
            None
        };

        if let Some(resource_attrs_record_batch) = resource_attrs {
            _ = child_record_batch_filters.insert(
                ArrowPayloadType::ResourceAttrs,
                update_child_record_batch_filter(
                    resource_attrs_record_batch,
                    span_resource_ids_column,
                    &resource_attr_filter,
                    &span_filter,
                )?,
            );
        }

        if let Some(scope_attrs_record_batch) = scope_attrs {
            _ = child_record_batch_filters.insert(
                ArrowPayloadType::ScopeAttrs,
                new_child_record_batch_filter(
                    scope_attrs_record_batch,
                    span_scope_ids_column,
                    &span_filter,
                )?,
            );
        }

        if let Some(span_event_attrs_record_batch) = span_event_attrs {
            let span_event_ids_column = if let Some(span_events_record_batch) = span_events {
                get_required_array(span_events_record_batch, consts::ID)?
            } else {
                return Err(Error::UnexpectedRecordBatchState {
                    reason:
                        "Span Event Attribute Record Batch found without Span Event Record Batch"
                            .to_string(),
                });
            };
            _ = child_record_batch_filters.insert(
                ArrowPayloadType::SpanEventAttrs,
                update_child_record_batch_filter(
                    span_event_attrs_record_batch,
                    span_event_ids_column,
                    &span_event_attr_filter,
                    &span_event_filter,
                )?,
            );
        }

        if let Some(span_link_attrs_record_batch) = span_link_attrs {
            if let Some(link_filter) = &span_link_filter {
                let span_link_ids_column = if let Some(span_links_record_batch) = span_links {
                    get_required_array(span_links_record_batch, consts::ID)?
                } else {
                    return Err(Error::UnexpectedRecordBatchState {
                        reason:
                            "Span Link Attribute Record Batch found without Span Link Record Batch"
                                .to_string(),
                    });
                };
                _ = child_record_batch_filters.insert(
                    ArrowPayloadType::SpanLinkAttrs,
                    update_child_record_batch_filter(
                        span_link_attrs_record_batch,
                        span_link_ids_column,
                        &span_link_attr_filter,
                        link_filter,
                    )?,
                );
            }
        }

        Ok((span_filter, child_record_batch_filters))
    }
}

impl TraceMatchProperties {
    /// create a new TraceMatchProperties
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
            get_resource_attr_filter(traces_payload, &self.resource_attributes, &self.match_type)?,
            self.get_span_filter(traces_payload)?,
            get_attr_filter(
                traces_payload,
                &self.span_attributes,
                &self.match_type,
                ArrowPayloadType::SpanAttrs,
            )?,
            self.get_span_event_filter(traces_payload)?,
            get_attr_filter(
                traces_payload,
                &self.event_attributes,
                &self.match_type,
                ArrowPayloadType::SpanEventAttrs,
            )?,
            get_attr_filter(
                traces_payload,
                &self.link_attributes,
                &self.match_type,
                ArrowPayloadType::SpanLinkAttrs,
            )?,
            // self.get_span_event_attr_filter(traces_payload)?,
            // self.get_span_link_attr_filter(traces_payload)?,
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

    /// Creates a booleanarray that will filter a span record batch based on the
    /// span name. A span should have one of the defined span names
    fn get_span_filter(&self, traces_payload: &OtapArrowRecords) -> Result<BooleanArray> {
        let spans = traces_payload
            .get(ArrowPayloadType::Spans)
            .ok_or_else(|| Error::SpanRecordNotFound)?;
        let num_rows = spans.num_rows();

        if !&self.span_names.is_empty() {
            let mut filter: BooleanArray = BooleanArray::from(BooleanBuffer::new_unset(num_rows));
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

            Ok(nulls_to_false(&filter))
        } else {
            Ok(BooleanArray::from(BooleanBuffer::new_set(num_rows)))
        }
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
        let mut filter: BooleanArray = BooleanArray::from(BooleanBuffer::new_unset(num_rows));

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
}
