// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This test suite contains tests exercising the function library that accesses batch metadata.
//!
//! Note that many of the tests in this module are manually creating both the `Metadata` (required
//! to be populated on the execution state) and the `NodeBindings` (required to be populated on
//! `PipelineOptions` for planning). Because of this, these test cases only cover the evaluation of
//! the pipeline. Some Additional test coverage must exist on the DFE node implementations to
//! ensure that these are properly populated when the query-engine is embedded within some
//! pipeline component.
//!

use std::sync::Arc;

use otel_arrow_contrib_data_engine_kql_parser::Parser;
use otel_arrow_dfe_config::ContextEntryName;
use otel_arrow_dfe_config::context::transport_headers::{TransportHeader, ValueKind};
use otel_arrow_dfe_engine::pdata_context::{
    ContextDomain, ContextEntrySelector, ContextEntrySelectorForm, ContextEntryTake,
    ContextEntryTarget, ContextMaterializationPlan, ContextNameMatching, Metadata, NodeBindings,
    ValueRead,
};
use otel_arrow_dfe_pdata::proto::OtlpProtoMessage;
use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::*;
use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::*;
use otel_arrow_dfe_pdata::testing::round_trip::{otap_to_otlp, otlp_to_otap, to_logs_data};
use otel_arrow_dfe_pdata::{OtapArrowRecords, OtapPayloadHelpers};
use otel_arrow_dfe_query_engine_languages::opl::parser::OplParser;

use crate::parser::default_parser_options;
use crate::pipeline::state::ExecutionState;
use crate::pipeline::{Pipeline, PipelineOptions, SignalKind};

/// create a simple logs batch
fn create_logs_batch(attributes: Vec<KeyValue>) -> OtapArrowRecords {
    otlp_to_otap(&OtlpProtoMessage::Logs(to_logs_data(vec![
        LogRecord::build()
            .event_name("hello.world")
            .attributes(attributes)
            .finish(),
    ])))
}

/// create node bindings with a selector for the given header name
fn node_bindings_for_transport_header(header_name: &str) -> NodeBindings {
    let context_entry_selector = ContextEntrySelector {
        target: ContextEntryTarget::Primitive {
            domain: ContextDomain::TransportHeader,
            name: ContextEntryName::try_from(header_name).unwrap(),
            matching: ContextNameMatching::Exact,
        },
        form: ContextEntrySelectorForm::StoredKeyValue,
        take: ContextEntryTake::First,
    };

    let context_materialization_plan = ContextMaterializationPlan::default();
    let value_read =
        ValueRead::compile_for_test(&context_entry_selector, &context_materialization_plan)
            .unwrap();
    let mut node_bindings = NodeBindings::default();
    node_bindings.set_value_reads(vec![value_read]);

    node_bindings
}

/// insert a new transport header into the metadata
///
/// note: does not replace existing with same name, so duplicates may be created.
fn add_transport_header(metadata: &mut Metadata, header_name: &str, header_value: &str) {
    let transport_header = TransportHeader::new(
        ContextEntryName::try_from(header_name).unwrap(),
        ValueKind::Text,
        header_value.as_bytes().to_vec().into_boxed_slice(),
    );

    let mut transport_headers = metadata.take_transport_headers().unwrap_or_default();
    transport_headers.push(transport_header);
    metadata.set_transport_headers(transport_headers);
}

async fn execute_pipeline(
    query: &str,
    pipeline_options: PipelineOptions,
    batch: OtapArrowRecords,
    metadata: Metadata,
) -> OtapArrowRecords {
    let mut exec_state = ExecutionState::new();
    exec_state.set_metadata(metadata);

    let parser_result = OplParser::parse_with_options(query, default_parser_options()).unwrap();
    let mut pipeline = Pipeline::new_with_options(parser_result.pipeline, pipeline_options);
    pipeline
        .execute_with_state(batch, &mut exec_state)
        .await
        .unwrap()
}

/// Scenario: a metadata access function is used as the predicate in the filter operator call to
/// keep/discard batches based on presence of a metadata transport header
/// Guarantees: the predicate correctly filters the result batches
#[tokio::test]
async fn test_simple_filter_batches_having_transport_header() {
    let header_name = "foo";
    let header_val = "bar";
    let pipeline_options = PipelineOptions {
        node_bindings: Arc::new(node_bindings_for_transport_header(header_name)),
        ..Default::default()
    };
    let mut metadata = Metadata::default();
    add_transport_header(&mut metadata, header_name, header_val);

    let input_batch = create_logs_batch(Default::default());
    let query = "signals | where has_transport_header(\"foo\")";
    let result = execute_pipeline(
        query,
        pipeline_options.clone(),
        input_batch.clone(),
        metadata,
    )
    .await;

    // assert we kept all the rows:
    assert_eq!(result.num_items(), 1);

    // assert we return an empty batch (all rows filtered out) when the header isn't present
    let result_empty =
        execute_pipeline(query, pipeline_options, input_batch, Metadata::default()).await;
    assert!(result_empty.is_empty())
}

/// Scenario: a metadata access function is used as the predicate in the filter operator call to
/// keep/discard batches based on presence of a metadata transport header, and then the predicate
/// is inverted using "not".
/// Guarantees: the predicate correctly filters the result batches
#[tokio::test]
async fn test_filter_batches_not_having_transport_header() {
    let header_name = "foo";
    let header_val = "bar";
    let pipeline_options = PipelineOptions {
        node_bindings: Arc::new(node_bindings_for_transport_header(header_name)),
        ..Default::default()
    };
    let mut metadata = Metadata::default();
    add_transport_header(&mut metadata, header_name, header_val);

    let input_batch = create_logs_batch(Default::default());
    let query = "signals | where not(has_transport_header(\"foo\"))";
    let result = execute_pipeline(
        query,
        pipeline_options.clone(),
        input_batch.clone(),
        metadata,
    )
    .await;

    // assert we kept all the rows:
    assert!(result.is_empty());

    // assert we return an empty batch (all rows filtered out) when the header isn't present
    let result_empty =
        execute_pipeline(query, pipeline_options, input_batch, Metadata::default()).await;
    assert_eq!(result_empty.num_items(), 1);
}

/// Scenario: a complex predicate is evaluated combining invocation of metadata-oriented function
/// and other logical expressions
/// Guarantees: the predicate correctly filters the result batches
#[tokio::test]
async fn test_complex_filter_batches_by_metadata() {
    let header_name = "foo";
    let header_val = "bar";
    let pipeline_options = PipelineOptions {
        node_bindings: Arc::new(node_bindings_for_transport_header(header_name)),
        ..Default::default()
    };
    let query =
        "signals | where attributes[\"attr.1\"] == \"val\" or has_transport_header(\"foo\")";

    // this should match on the metadata
    let mut metadata = Metadata::default();
    add_transport_header(&mut metadata, header_name, header_val);
    let result = execute_pipeline(
        query,
        pipeline_options.clone(),
        create_logs_batch(vec![KeyValue::new("attr.1", AnyValue::new_string("no"))]),
        metadata,
    )
    .await;
    assert_eq!(result.num_items(), 1);

    // this should match on the attribute, but not the header
    let metadata = Metadata::default();
    let result = execute_pipeline(
        query,
        pipeline_options.clone(),
        create_logs_batch(vec![KeyValue::new("attr.1", AnyValue::new_string("val"))]),
        metadata,
    )
    .await;
    assert_eq!(result.num_items(), 1);

    // this should not match because it has neither attr nor header
    let metadata = Metadata::default();
    let result = execute_pipeline(
        query,
        pipeline_options.clone(),
        create_logs_batch(vec![KeyValue::new("attr.1", AnyValue::new_string("no"))]),
        metadata,
    )
    .await;
    assert!(result.is_empty())
}

/// Scenario: Assign the value of the metadata access function (has_transport_header) invocation to
/// an attribute
/// Guarantees: this type of expression correctly assigns the attribute
#[tokio::test]
async fn test_simple_assign_transport_header_presence_as_property() {
    let header_name = "foo";
    let header_val = "bar";
    let pipeline_options = PipelineOptions {
        node_bindings: Arc::new(node_bindings_for_transport_header(header_name)),
        ..Default::default()
    };

    // assert it assigns "true" when it has the header
    let mut metadata = Metadata::default();
    add_transport_header(&mut metadata, header_name, header_val);
    let result = execute_pipeline(
        "signals | set attributes[\"has_header\"] = has_transport_header(\"foo\")",
        pipeline_options.clone(),
        create_logs_batch(Default::default()),
        metadata,
    )
    .await;

    let OtlpProtoMessage::Logs(result) = otap_to_otlp(&result) else {
        panic!("wrong signal type")
    };
    let attr = &result.resource_logs[0].scope_logs[0].log_records[0].attributes[0];
    assert_eq!(attr, &KeyValue::new("has_header", AnyValue::new_bool(true)));

    // assert it returns false when we don't have the header
    let result = execute_pipeline(
        "signals | set attributes[\"has_header\"] = has_transport_header(\"foo\")",
        pipeline_options,
        create_logs_batch(Default::default()),
        Metadata::default(),
    )
    .await;

    let OtlpProtoMessage::Logs(result) = otap_to_otlp(&result) else {
        panic!("wrong signal type")
    };
    let attr = &result.resource_logs[0].scope_logs[0].log_records[0].attributes[0];
    assert_eq!(
        attr,
        &KeyValue::new("has_header", AnyValue::new_bool(false))
    );
}

#[tokio::test]
async fn test_batch_metadata_function_result_used_as_function_argument() {
    let header_name = "foo";
    let header_val = "bar";
    let pipeline_options = PipelineOptions {
        node_bindings: Arc::new(node_bindings_for_transport_header(header_name)),
        signal_context: crate::pipeline::SignalContext::Single(SignalKind::Logs),
        ..Default::default()
    };

    // this should return a non-null "true", the result from coalesce will not be "false"
    let query = "logs | where coalesce(has_transport_header(\"foo\"), false)";

    let mut metadata = Metadata::default();
    add_transport_header(&mut metadata, header_name, header_val);
    let result = execute_pipeline(
        query,
        pipeline_options.clone(),
        create_logs_batch(Default::default()),
        metadata,
    )
    .await;

    // assert we kept all the rows:
    assert_eq!(result.num_items(), 1);

    // do the same test, this time with more args... body should be null on the input batch,
    // so the first arg will resolve to null and the from coalesce will be the "true" from the
    // call to has_transport_header
    let query = "logs | where coalesce(body, has_transport_header(\"foo\"), false)";

    let mut metadata = Metadata::default();
    add_transport_header(&mut metadata, header_name, header_val);
    let result = execute_pipeline(
        query,
        pipeline_options.clone(),
        create_logs_batch(Default::default()),
        metadata,
    )
    .await;

    // assert we kept all the rows:
    assert_eq!(result.num_items(), 1);
}
