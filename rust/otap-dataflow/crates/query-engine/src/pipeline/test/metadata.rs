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
use otel_arrow_dfe_pdata::testing::round_trip::{otlp_to_otap, to_logs_data};
use otel_arrow_dfe_pdata::{OtapArrowRecords, OtapPayloadHelpers};
use otel_arrow_dfe_query_engine_languages::opl::parser::OplParser;

use crate::parser::default_parser_options;
use crate::pipeline::state::ExecutionState;
use crate::pipeline::{Pipeline, PipelineOptions};

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
    let mut pipeline_options = PipelineOptions::default();
    pipeline_options.node_bindings = Arc::new(node_bindings_for_transport_header(header_name));
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
    let mut pipeline_options = PipelineOptions::default();
    pipeline_options.node_bindings = Arc::new(node_bindings_for_transport_header(header_name));
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
    let mut pipeline_options = PipelineOptions::default();
    pipeline_options.node_bindings = Arc::new(node_bindings_for_transport_header(header_name));
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

/// Scenario: use the value returned by the metadata access function (has_transport_header) in the
/// predicate in the if/else construct's condition to dynamically route batches based on transport
/// header presence
/// Guarantees: this type of expression correctly routes batches
#[tokio::test]
#[ignore]
async fn test_conditionally_route_batches_by_transport_header_presence() {
    todo!()
}

/// Scenario: Assign the value of the metadata access function (has_transport_header) invocation to
/// an attribute
/// Guarantees: this type of expression correctly assigns the attribute
#[tokio::test]
#[ignore]
async fn test_simple_assign_transport_header_presence_as_property() {
    todo!()
}

/// Scenario: Assign the value of the metadata access function (has_transport_header) invocation to
/// an attribute using some complex logic to transform the result
/// Guarantees: this type of expression correctly assigns the attribute
#[tokio::test]
#[ignore]
async fn test_complex_assign_transport_header_presence_as_property() {
    todo!()
}
