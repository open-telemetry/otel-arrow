// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::{TOPIC_EXPORTER, TOPIC_EXPORTER_URN};
use crate::receivers::topic_receiver::TOPIC_RECEIVER;
use otel_arrow_dfe_config::TopicName;
use otel_arrow_dfe_config::pipeline::{PipelineConfigBuilder, PipelineType};
use otel_arrow_dfe_config::topic::TopicAckPropagationMode::{self, Auto, Disabled};
use otel_arrow_dfe_engine::error::Error;
use otel_arrow_dfe_engine::testing::{
    exporter::create_test_pipeline_context, install_test_context_bindings,
};
use otel_arrow_dfe_engine::topic::{PipelineTopicBinding, TopicBroker, TopicOptions, TopicSet};
use otel_arrow_dfe_engine::wiring_contract::{TopicRequirements, WiringContract};
use otel_arrow_dfe_engine::{ExporterFactory, PipelineFactory, ReceiverFactory};
use otel_arrow_dfe_otap::pdata::OtapPdata;
use serde_json::json;

const ACK_SOURCE: &str = "urn:otel:receiver:test_ack";
const SINGLE_SOURCE: &str = "urn:otel:receiver:test_single";
const BALANCED: TopicOptions = TopicOptions::BalancedOnly { capacity: 1 };

const fn source(
    name: &'static str,
    topic_requirements: TopicRequirements,
) -> ReceiverFactory<OtapPdata> {
    ReceiverFactory {
        name,
        wiring_contract: WiringContract {
            topic_requirements,
            ..WiringContract::UNRESTRICTED
        },
        ..TOPIC_RECEIVER
    }
}

static SOURCES: [ReceiverFactory<OtapPdata>; 2] = [
    source(ACK_SOURCE, TopicRequirements::SUBSCRIBER_ACK),
    source(SINGLE_SOURCE, TopicRequirements::SINGLE_SUBSCRIBER),
];

fn build_route(
    sources: &[&'static str],
    exporter: &'static ExporterFactory<OtapPdata>,
    mode: TopicAckPropagationMode,
    options: TopicOptions,
) -> Result<(), Error> {
    let factory = PipelineFactory::new(&SOURCES, &[], std::slice::from_ref(exporter), &[]);
    let broker = TopicBroker::new();
    let topics = TopicSet::<OtapPdata>::new("test");
    for (name, options) in [("incoming", BALANCED), ("outgoing", options)] {
        let name = TopicName::parse(name).unwrap();
        let handle = broker
            .create_in_memory_topic(name.clone(), options)
            .unwrap();
        let _ = topics.insert(
            name,
            PipelineTopicBinding::new(handle).with_default_ack_propagation_mode(mode),
        );
    }
    let mut ctx = create_test_pipeline_context();
    ctx.set_topic_set(topics);
    let mut builder = PipelineConfigBuilder::new().add_exporter(
        "sink",
        TOPIC_EXPORTER_URN,
        Some(json!({"topic": "outgoing"})),
    );
    for (index, urn) in sources.iter().enumerate() {
        let name = format!("source{index}");
        let config = json!({
            "topic": "incoming",
            "subscription": {"mode": "balanced", "group": "readers"}
        });
        builder = builder
            .add_receiver(name.clone(), *urn, Some(config))
            .one_of(name, ["sink"]);
    }
    let config = builder
        .build(PipelineType::Otap, "test_grp", "test_pipeline")
        .unwrap();
    install_test_context_bindings(&mut ctx, &factory, config.clone())?;
    factory
        .build(
            ctx,
            config,
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            None,
            None,
        )
        .map(drop)
}

/// Scenario: A direct source requests subscriber acknowledgements with no delivery fanout limit.
/// Guarantees: Queue admission alone is rejected, and automatic propagation works on broadcast-capable topics.
#[test]
fn subscriber_ack_rejects_queue_admission_only() {
    let error = build_route(&[ACK_SOURCE], &TOPIC_EXPORTER, Disabled, BALANCED)
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("policies.ack_propagation.mode")
            && error.contains("outgoing")
            && error.contains("sink"),
        "{error}"
    );
    build_route(
        &[ACK_SOURCE],
        &TOPIC_EXPORTER,
        Auto,
        TopicOptions::default(),
    )
    .unwrap();
}

/// Scenario: A source limits delivery to one subscriber per publication without requiring a subscriber Ack.
/// Guarantees: A balanced topic accepts queue admission, while a broadcast-capable topic fails even with propagation.
#[test]
fn single_subscriber_is_independent_of_ack_propagation() {
    build_route(&[SINGLE_SOURCE], &TOPIC_EXPORTER, Disabled, BALANCED).unwrap();
    let error = build_route(
        &[SINGLE_SOURCE],
        &TOPIC_EXPORTER,
        Auto,
        TopicOptions::default(),
    )
    .unwrap_err()
    .to_string();
    for detail in [
        "sink",
        "outgoing",
        "at most one subscriber",
        "impl_selection: auto",
        "subscription.mode: balanced",
        "same `subscription.group`",
    ] {
        assert!(error.contains(detail), "{error}");
    }
}

/// Scenario: Two sources with different requirements feed the same topic exporter.
/// Guarantees: The route must satisfy both requirements, regardless of which source the engine visits first.
#[test]
fn direct_sources_combine_topic_requirements() {
    static SINK: ExporterFactory<OtapPdata> = ExporterFactory {
        create: |ctx, node, config, exporter_config, capabilities| {
            assert_eq!(
                ctx.input_topic_requirements(),
                TopicRequirements::SUBSCRIBER_ACK | TopicRequirements::SINGLE_SUBSCRIBER
            );
            (TOPIC_EXPORTER.create)(ctx, node, config, exporter_config, capabilities)
        },
        ..TOPIC_EXPORTER
    };

    build_route(&[ACK_SOURCE, SINGLE_SOURCE], &SINK, Auto, BALANCED).unwrap();
}
