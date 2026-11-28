// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Compiled context fixtures for node tests.
//!
//! Tests form request context through a compiled arrival binding and read it
//! back as plain values, so they do not depend on how context is stored.
//! Reads and sinks come from the same compile as the formed context.
//!
//! A fixture pipeline always contains an arrival receiver named
//! [`ARRIVAL_NODE`]. Tests add the nodes under test, either from a real
//! factory or as compile-only consumer and sink nodes.

use crate::PipelineFactory;
use crate::capability::auth::ClaimValue;
use crate::context::PipelineContext;
use crate::pdata_context::{
    Arrival, ArrivalBinding, ArrivalHeaders, ContextConsumerSelector, ContextDeclaration,
    ContextDeclarationProvider, ContextDomain, ContextEntrySelector, MaterializedContextValues,
    Metadata, NodeBindings, NodeContextDeclarations,
};
use otel_arrow_dfe_config::ContextEntryName;
use otel_arrow_dfe_config::context::authorized_identity::AuthorizedIdentityPolicy;
use otel_arrow_dfe_config::context::transport_headers::{
    TransportHeaders, TransportHeadersData, ValueKind,
};
use otel_arrow_dfe_config::context::transport_policy::HeaderCapturePolicy;
use otel_arrow_dfe_config::error::Error as ConfigError;
use otel_arrow_dfe_config::node::NodeUserConfig;
use otel_arrow_dfe_config::pipeline::PipelineConfig;
use serde_json::{Map, Value, json};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, OnceLock, PoisonError};

/// Node id of the arrival receiver in every fixture pipeline.
pub const ARRIVAL_NODE: &str = "arrival";
/// Compile-only receiver that forms context at arrival.
pub const ARRIVAL_RECEIVER_URN: &str = "urn:test:receiver:arrival";
/// Compile-only processor whose declarations are supplied by the fixture.
pub const DECLARING_PROCESSOR_URN: &str = "urn:test:processor:declaring";
/// Compile-only exporter that only carries a propagation policy.
pub const SINK_EXPORTER_URN: &str = "urn:test:exporter:sink";

/// Node id of the sink added to retain original header names.
const RETAIN_NODE: &str = "retain_original_names";

/// Declarations registered for compile-only processors.
///
/// Factory declaration callbacks are plain functions, so each processor node
/// names its registered declarations by index. The registry only grows.
static PROCESSOR_DECLARATIONS: Mutex<Vec<Box<[ContextDeclaration]>>> = Mutex::new(Vec::new());

/// Configuration of a compile-only processor.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct DeclaringProcessorConfig {
    /// Index into [`PROCESSOR_DECLARATIONS`].
    declarations: usize,
}

/// Registers processor declarations and returns their node configuration.
fn register_declarations(declarations: impl IntoIterator<Item = ContextDeclaration>) -> Value {
    let mut registry = PROCESSOR_DECLARATIONS
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    registry.push(declarations.into_iter().collect());
    json!({ "declarations": registry.len() - 1 })
}

/// Returns the declarations registered for a compile-only processor.
fn declaring_processor_declarations(
    config: &Value,
) -> Result<NodeContextDeclarations, ConfigError> {
    let config: DeclaringProcessorConfig =
        serde_json::from_value(config.clone()).map_err(|error| ConfigError::InvalidUserConfig {
            error: format!("invalid context fixture processor config: {error}"),
        })?;
    let registry = PROCESSOR_DECLARATIONS
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    let declarations =
        registry
            .get(config.declarations)
            .ok_or_else(|| ConfigError::InvalidUserConfig {
                error: format!(
                    "unknown context fixture declarations {}",
                    config.declarations
                ),
            })?;
    Ok(declarations.iter().cloned().collect())
}

/// Accepts any configuration for compile-only nodes.
fn accept_any_config(_: &Value) -> Result<(), ConfigError> {
    Ok(())
}

/// Returns the error reported when a compile-only node is built.
fn compile_only(node: &NodeUserConfig) -> ConfigError {
    ConfigError::InvalidUserConfig {
        error: format!("context fixture node `{}` is compile-only", node.r#type),
    }
}

/// Rejects construction of a compile-only receiver.
fn compile_only_receiver<PData>(
    _: PipelineContext,
    _: crate::node::NodeId,
    node: Arc<NodeUserConfig>,
    _: &crate::config::ReceiverConfig,
    _: &crate::capability::registry::Capabilities,
) -> Result<crate::receiver::ReceiverWrapper<PData>, ConfigError> {
    Err(compile_only(&node))
}

/// Rejects construction of a compile-only processor.
fn compile_only_processor<PData>(
    _: PipelineContext,
    _: crate::node::NodeId,
    node: Arc<NodeUserConfig>,
    _: &crate::config::ProcessorConfig,
    _: &crate::capability::registry::Capabilities,
) -> Result<crate::processor::ProcessorWrapper<PData>, ConfigError> {
    Err(compile_only(&node))
}

/// Rejects construction of a compile-only exporter.
fn compile_only_exporter<PData>(
    _: PipelineContext,
    _: crate::node::NodeId,
    node: Arc<NodeUserConfig>,
    _: &crate::config::ExporterConfig,
    _: &crate::capability::registry::Capabilities,
) -> Result<crate::exporter::ExporterWrapper<PData>, ConfigError> {
    Err(compile_only(&node))
}

/// Leaks a factory slice; fixture factories live for the test process.
fn leak<T>(factories: Vec<T>) -> &'static [T] {
    Box::leak(factories.into_boxed_slice())
}

/// Returns `base` plus the compile-only fixture factories.
///
/// The fixture factories are never registered in a distributed slice, so
/// component inventories do not see them.
fn with_fixture_factories<PData: 'static + Clone>(
    base: &PipelineFactory<PData>,
) -> PipelineFactory<PData> {
    let mut receivers = base.receiver_factories.to_vec();
    receivers.push(crate::ReceiverFactory {
        name: ARRIVAL_RECEIVER_URN,
        create: compile_only_receiver::<PData>,
        context_declarations: Some(ContextDeclarationProvider::arriving()),
        wiring_contract: crate::wiring_contract::WiringContract::UNRESTRICTED,
        validate_config: accept_any_config,
    });
    let mut processors = base.processor_factories.to_vec();
    processors.push(crate::ProcessorFactory {
        name: DECLARING_PROCESSOR_URN,
        create: compile_only_processor::<PData>,
        context_declarations: Some(ContextDeclarationProvider {
            declarations: declaring_processor_declarations,
        }),
        wiring_contract: crate::wiring_contract::WiringContract::UNRESTRICTED,
        validate_config: accept_any_config,
    });
    let mut exporters = base.exporter_factories.to_vec();
    exporters.push(crate::ExporterFactory {
        name: SINK_EXPORTER_URN,
        create: compile_only_exporter::<PData>,
        context_declarations: None,
        wiring_contract: crate::wiring_contract::WiringContract::UNRESTRICTED,
        validate_config: accept_any_config,
    });
    PipelineFactory {
        receiver_factory_map: OnceLock::new(),
        processor_factory_map: OnceLock::new(),
        exporter_factory_map: OnceLock::new(),
        extension_factory_map: OnceLock::new(),
        receiver_factories: leak(receivers),
        processor_factories: leak(processors),
        exporter_factories: leak(exporters),
        extension_factories: base.extension_factories,
        context_materializer: base.context_materializer,
    }
}

/// Builds a [`ContextFixture`].
#[derive(Default)]
#[must_use]
pub struct ContextFixtureBuilder {
    policies: Map<String, Value>,
    arrival: Map<String, Value>,
    nodes: Map<String, Value>,
    pipeline_ctx: Option<PipelineContext>,
}

impl ContextFixtureBuilder {
    /// Sets one pipeline policy, such as `transport_headers`,
    /// `authorized_identity` or `context`, from its configuration form.
    ///
    /// # Panics
    ///
    /// Panics if the policy does not serialize.
    pub fn policy(mut self, name: &str, policy: impl serde::Serialize) -> Self {
        _ = self.policies.insert(
            name.to_owned(),
            serde_json::to_value(policy).expect("serializable policy"),
        );
        self
    }

    /// Sets the pipeline authorized identity policy.
    pub fn authorized_identity(self, policy: &AuthorizedIdentityPolicy) -> Self {
        self.policy("authorized_identity", policy)
    }

    /// Sets the arrival receiver's own header capture policy.
    ///
    /// # Panics
    ///
    /// Panics if the policy does not serialize.
    pub fn header_capture(mut self, policy: &HeaderCapturePolicy) -> Self {
        _ = self.arrival.insert(
            "header_capture".to_owned(),
            serde_json::to_value(policy).expect("serializable capture policy"),
        );
        self
    }

    /// Captures exactly the given headers at arrival.
    ///
    /// Derives one capture rule per wire name that stores the header under
    /// its name and kind, with limits large enough for the input. Retains
    /// original wire names when any header has one. Use
    /// [`ContextFixture::form_captured`] to form context from the headers.
    ///
    /// # Panics
    ///
    /// Panics if one wire name maps to two stored names or kinds.
    pub fn capture(mut self, headers: &TransportHeaders) -> Self {
        let mut rules: Vec<(String, String, ValueKind)> = Vec::new();
        let mut max_name_bytes = 128;
        let mut max_value_bytes = 4096;
        for header in headers.iter() {
            let wire = header.wire_name();
            max_name_bytes = max_name_bytes.max(wire.len());
            max_value_bytes = max_value_bytes.max(header.value.bytes.len());
            let stored = header.name.as_str();
            let kind = header.value.value_kind;
            match rules
                .iter()
                .find(|(existing, _, _)| existing.eq_ignore_ascii_case(wire))
            {
                Some((_, existing_stored, existing_kind)) => assert!(
                    existing_stored == stored && *existing_kind == kind,
                    "wire name `{wire}` is captured under two names or kinds"
                ),
                None => rules.push((wire.to_owned(), stored.to_owned(), kind)),
            }
        }
        let rules: Vec<Value> = rules
            .into_iter()
            .map(|(wire, stored, kind)| {
                json!({
                    "match_names": [wire],
                    "store_as": stored,
                    "value_kind": match kind {
                        ValueKind::Text => "text",
                        ValueKind::Binary => "binary",
                    },
                })
            })
            .collect();
        _ = self.arrival.insert(
            "header_capture".to_owned(),
            json!({
                "defaults": {
                    "max_entries": headers.len().max(32),
                    "max_name_bytes": max_name_bytes,
                    "max_value_bytes": max_value_bytes,
                },
                "headers": rules,
            }),
        );
        if headers
            .iter()
            .any(|header| header.value.original_name.is_some())
        {
            self = self.sink(
                RETAIN_NODE,
                json!({ "default": { "selector": { "type": "all_captured" } } }),
            );
        }
        self
    }

    /// Adds a node from its configuration form, for example
    /// `{"type": "urn:...", "config": {...}}`.
    pub fn node(mut self, id: &str, config: Value) -> Self {
        _ = self.nodes.insert(id.to_owned(), config);
        self
    }

    /// Adds a node from its configuration.
    ///
    /// # Panics
    ///
    /// Panics if the configuration does not serialize.
    pub fn node_config(self, id: &str, config: &NodeUserConfig) -> Self {
        let config = serde_json::to_value(config).expect("serializable node configuration");
        self.node(id, config)
    }

    /// Adds a compile-only processor with the given declarations.
    pub fn declaring(
        self,
        id: &str,
        declarations: impl IntoIterator<Item = ContextDeclaration>,
    ) -> Self {
        let config = register_declarations(declarations);
        self.node(
            id,
            json!({ "type": DECLARING_PROCESSOR_URN, "config": config }),
        )
    }

    /// Adds a compile-only processor that reads the given entries, in order.
    pub fn consumer(
        self,
        id: &str,
        entries: impl IntoIterator<Item = ContextEntrySelector>,
    ) -> Self {
        self.declaring(
            id,
            [ContextDeclaration::Consumes {
                selector: ContextConsumerSelector::Entries {
                    entries: entries.into_iter().collect(),
                },
            }],
        )
    }

    /// Adds a compile-only processor that reads every captured header.
    pub fn headers_consumer(self, id: &str) -> Self {
        self.declaring(
            id,
            [ContextDeclaration::Consumes {
                selector: ContextConsumerSelector::AllCaptured {
                    domain: ContextDomain::TransportHeader,
                },
            }],
        )
    }

    /// Adds a compile-only exporter with the given header propagation policy
    /// in its configuration form.
    pub fn sink(self, id: &str, header_propagation: Value) -> Self {
        self.node(
            id,
            json!({ "type": SINK_EXPORTER_URN, "header_propagation": header_propagation }),
        )
    }

    /// Uses `pipeline_ctx` instead of a fresh test pipeline context.
    pub fn pipeline_context(mut self, pipeline_ctx: PipelineContext) -> Self {
        self.pipeline_ctx = Some(pipeline_ctx);
        self
    }

    /// Compiles the fixture pipeline with `factory`'s components.
    ///
    /// # Panics
    ///
    /// Panics if the pipeline does not compile.
    #[must_use]
    pub fn compile<PData: 'static + Clone + std::fmt::Debug>(
        self,
        factory: &PipelineFactory<PData>,
    ) -> ContextFixture {
        self.try_compile(factory)
            .expect("context fixture pipeline compiles")
    }

    /// Compiles the fixture pipeline with only compile-only components.
    ///
    /// # Panics
    ///
    /// Panics if the pipeline does not compile.
    #[must_use]
    pub fn compile_standalone(self) -> ContextFixture {
        self.compile(&PipelineFactory::<()>::new(&[], &[], &[], &[]))
    }

    /// Compiles the fixture pipeline with `factory`'s components.
    ///
    /// # Errors
    ///
    /// Returns the configuration or declaration error of the pipeline.
    pub fn try_compile<PData: 'static + Clone + std::fmt::Debug>(
        self,
        factory: &PipelineFactory<PData>,
    ) -> Result<ContextFixture, crate::error::Error> {
        let Self {
            policies,
            mut arrival,
            mut nodes,
            pipeline_ctx,
        } = self;
        _ = arrival.insert("type".to_owned(), json!(ARRIVAL_RECEIVER_URN));
        _ = nodes.insert(ARRIVAL_NODE.to_owned(), Value::Object(arrival));
        let invalid = |error: serde_json::Error| {
            crate::error::Error::ConfigError(Box::new(ConfigError::InvalidUserConfig {
                error: format!("invalid context fixture pipeline: {error}"),
            }))
        };
        // `Policies::resolve` ignores `context`; the engine resolver merges it
        // separately, so the fixture passes it through explicitly.
        let context = policies
            .get("context")
            .cloned()
            .map(serde_json::from_value)
            .transpose()
            .map_err(invalid)?;
        let pipeline: PipelineConfig = serde_json::from_value(json!({
            "policies": policies,
            "nodes": nodes,
        }))
        .map_err(invalid)?;
        let nodes = pipeline
            .node_iter()
            .map(|(id, node)| (id.to_string(), Arc::clone(node)))
            .collect();
        let mut pipeline_ctx = pipeline_ctx.unwrap_or_else(|| super::test_pipeline_ctx().0);
        super::install_bindings_with_context_policy(
            &mut pipeline_ctx,
            &with_fixture_factories(factory),
            pipeline,
            context,
        )?;
        Ok(ContextFixture {
            pipeline_ctx,
            nodes,
        })
    }
}

/// A compiled fixture pipeline.
pub struct ContextFixture {
    pipeline_ctx: PipelineContext,
    nodes: HashMap<String, Arc<NodeUserConfig>>,
}

impl ContextFixture {
    /// Starts a fixture pipeline that contains only the arrival receiver.
    pub fn builder() -> ContextFixtureBuilder {
        ContextFixtureBuilder::default()
    }

    /// Returns the pipeline context holding the compiled bindings.
    #[must_use]
    pub fn pipeline_context(&self) -> &PipelineContext {
        &self.pipeline_ctx
    }

    /// Returns the context for constructing node `id` from its factory.
    ///
    /// # Panics
    ///
    /// Panics if the pipeline has no node `id`.
    #[must_use]
    pub fn node_context(&self, id: &str) -> PipelineContext {
        let node = self.node(id);
        self.pipeline_ctx.with_node_context(
            id.to_owned().into(),
            node.r#type.clone(),
            node.kind(),
            HashMap::new(),
        )
    }

    /// Returns the compiled bindings of node `id`.
    ///
    /// # Panics
    ///
    /// Panics if the pipeline has no node `id`.
    #[must_use]
    pub fn node_bindings(&self, id: &str) -> Arc<NodeBindings> {
        _ = self.node(id);
        self.pipeline_ctx
            .compiled_context_bindings()
            .node_bindings(&self.pipeline_ctx.pipeline_key(), &id.to_owned().into())
            .cloned()
            .unwrap_or_else(|| panic!("node `{id}` has compiled bindings"))
    }

    /// Returns the arrival receiver's binding.
    ///
    /// # Panics
    ///
    /// Panics if the arrival receiver has no arrival binding.
    #[must_use]
    pub fn arrival(&self) -> ArrivalBinding {
        self.node_bindings(ARRIVAL_NODE)
            .arrival()
            .cloned()
            .expect("arrival receiver has an arrival binding")
    }

    /// Forms context for one arrival.
    ///
    /// # Panics
    ///
    /// Panics if forming fails.
    #[must_use]
    pub fn form(&self, arrival: Arrival<'_>) -> Metadata {
        self.arrival()
            .form(arrival)
            .expect("arrival forms context")
            .0
    }

    /// Forms context from header pairs, in order.
    #[must_use]
    pub fn form_headers(&self, headers: &[(&str, &[u8])]) -> Metadata {
        self.form(Arrival {
            headers: ArrivalHeaders::Pairs(headers),
            identity: None,
            peer: None,
        })
    }

    /// Forms context by sending `headers` under their wire names.
    ///
    /// With [`ContextFixtureBuilder::capture`] for the same headers, the
    /// formed context captures exactly `headers`.
    #[must_use]
    pub fn form_captured(&self, headers: &TransportHeaders, peer: Option<SocketAddr>) -> Metadata {
        let pairs: Vec<(&str, &[u8])> = headers
            .iter()
            .map(|header| (header.wire_name(), header.value.bytes))
            .collect();
        self.form(Arrival {
            headers: ArrivalHeaders::Pairs(&pairs),
            identity: None,
            peer,
        })
    }

    /// Forms context with no request inputs.
    #[must_use]
    pub fn form_empty(&self) -> Metadata {
        self.form_headers(&[])
    }

    /// Returns the configuration of node `id`.
    fn node(&self, id: &str) -> &NodeUserConfig {
        self.nodes
            .get(id)
            .unwrap_or_else(|| panic!("context fixture has node `{id}`"))
    }
}

/// Returns context that holds exactly `headers`, as captured at arrival.
#[must_use]
pub fn headers_metadata(headers: &TransportHeaders) -> Metadata {
    ContextFixture::builder()
        .capture(headers)
        .compile_standalone()
        .form_captured(headers, None)
}

/// Returns context that holds only the peer observed at arrival.
///
/// # Panics
///
/// Panics if forming fails.
#[must_use]
pub fn peer_metadata(peer: SocketAddr) -> Metadata {
    ArrivalBinding::default()
        .form(Arrival {
            headers: ArrivalHeaders::Pairs(&[]),
            identity: None,
            peer: Some(peer),
        })
        .expect("arrival forms context")
        .0
}

/// Returns the captured transport headers, in capture order.
#[must_use]
pub fn captured_headers(metadata: &Metadata) -> Option<TransportHeaders> {
    metadata
        .transport_headers()
        .map(TransportHeadersData::to_transport_headers)
}

/// Returns the peer observed at arrival.
#[must_use]
pub fn peer(metadata: &Metadata) -> Option<SocketAddr> {
    metadata.peer_addr()
}

/// Returns the authorized identity claim stored under `name`.
#[must_use]
pub fn identity_claim(metadata: &Metadata, name: &str) -> Option<ClaimValue> {
    let value = metadata.authorized_identity_entries()?.get(name)?.value();
    Some(if value.is_many() {
        ClaimValue::Many(value.values().map(str::to_owned).collect())
    } else {
        ClaimValue::One(value.as_str()?.to_owned())
    })
}

/// Returns the names under which authorized identity claims are stored, in
/// policy order.
#[must_use]
pub fn identity_claim_names(metadata: &Metadata) -> Vec<String> {
    metadata
        .authorized_identity_entries()
        .into_iter()
        .flat_map(|entries| entries.iter().map(|entry| entry.name().to_owned()))
        .collect()
}

/// Returns the materialized values of one composite member.
///
/// # Panics
///
/// Panics if `composite` or `member` is not a valid context entry name.
#[must_use]
pub fn materialized_member(
    metadata: &Metadata,
    composite: &str,
    member: &str,
) -> Option<MaterializedContextValues> {
    let composite = ContextEntryName::try_from(composite).expect("valid composite name");
    let member = ContextEntryName::try_from(member).expect("valid member name");
    metadata
        .materialized_context()?
        .entry(&composite)?
        .member(&member)
        .map(|member| member.values().clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::auth::AuthorizedIdentity;
    use crate::pdata_context::{
        ContextEntrySelectorForm, ContextEntryTake, ContextEntryTarget, ContextNameMatching,
        PropagatedValue,
    };
    use otel_arrow_dfe_config::context::transport_headers::TransportHeader;

    /// Parses a context entry name.
    fn name(value: &str) -> ContextEntryName {
        value.try_into().expect("valid context entry name")
    }

    /// Scenario: headers with stored names, original wire names and binary
    /// values are captured through a derived capture policy.
    /// Guarantees: the formed context holds exactly the given headers, in
    /// order, with their kinds and original names.
    #[test]
    fn captured_headers_round_trip() {
        let mut headers = TransportHeaders::new();
        headers.push(TransportHeader::captured(
            name("tenant"),
            "X-Tenant-Id",
            true,
            ValueKind::Text,
            b"acme".to_vec(),
        ));
        headers.push(TransportHeader::binary(name("trace-bin"), [0_u8, 1, 2]));
        headers.push(TransportHeader::captured(
            name("tenant"),
            "X-Tenant-Id",
            true,
            ValueKind::Text,
            b"other".to_vec(),
        ));
        headers.push(TransportHeader::text(name("x-plain"), "value"));

        let metadata = headers_metadata(&headers);

        assert_eq!(captured_headers(&metadata), Some(headers));
        assert_eq!(peer(&metadata), None);
    }

    /// Scenario: a fixture forms context from no inputs and from a peer only.
    /// Guarantees: no headers are reported, and the peer is reported only
    /// when supplied.
    #[test]
    fn empty_and_peer_context() {
        let addr: SocketAddr = "127.0.0.1:4317".parse().expect("socket address");

        assert_eq!(captured_headers(&peer_metadata(addr)), None);
        assert_eq!(peer(&peer_metadata(addr)), Some(addr));
        let empty = ContextFixture::builder().compile_standalone().form_empty();
        assert_eq!(peer(&empty), None);
        assert_eq!(identity_claim_names(&empty), Vec::<String>::new());
    }

    /// Scenario: the pipeline authorized identity policy selects scalar and
    /// multi-valued claims from a verified identity at arrival.
    /// Guarantees: claims are reported under their stored names with their
    /// cardinality, and unselected claims are absent.
    #[test]
    fn identity_claims_round_trip() {
        let policy: AuthorizedIdentityPolicy = serde_json::from_value(json!([
            {"claim": "sub", "store_as": "customer_id"},
            {"claim": "groups", "store_as": "groups"},
        ]))
        .expect("valid identity policy");
        let fixture = ContextFixture::builder()
            .authorized_identity(&policy)
            .compile_standalone();
        let identity = AuthorizedIdentity::new()
            .with_subject("alice")
            .with_claim_values("groups", ["a", "b"])
            .with_claim_str("ignored", "x");

        let metadata = fixture.form(Arrival {
            headers: ArrivalHeaders::Pairs(&[]),
            identity: Some(&identity),
            peer: None,
        });

        assert_eq!(
            identity_claim(&metadata, "customer_id"),
            Some(ClaimValue::one("alice"))
        );
        assert_eq!(
            identity_claim(&metadata, "groups"),
            Some(ClaimValue::many(["a", "b"]))
        );
        assert_eq!(identity_claim(&metadata, "ignored"), None);
        assert_eq!(
            identity_claim_names(&metadata),
            vec!["customer_id".to_owned(), "groups".to_owned()]
        );
    }

    /// Scenario: one fixture compiles a consumer read and a sink alongside the
    /// arrival receiver, then forms context from captured headers.
    /// Guarantees: the read and the sink accept context formed by the same
    /// compile and observe the captured values.
    #[test]
    fn reads_and_sinks_share_the_compile() {
        let mut headers = TransportHeaders::new();
        headers.push(TransportHeader::text(name("x-tenant"), "acme"));
        let selector = ContextEntrySelector {
            target: ContextEntryTarget::Primitive {
                domain: ContextDomain::TransportHeader,
                name: name("x-tenant"),
                matching: ContextNameMatching::Exact,
            },
            form: ContextEntrySelectorForm::Value,
            take: ContextEntryTake::First,
        };
        let fixture = ContextFixture::builder()
            .capture(&headers)
            .consumer("reader", [selector.clone()])
            .sink(
                "out",
                json!({ "default": { "selector": { "type": "all_captured" } } }),
            )
            .compile_standalone();
        let metadata = fixture.form_captured(&headers, None);

        let reader = fixture.node_bindings("reader");
        let read = reader.value(&selector).expect("compiled read");
        let mut values = Vec::new();
        read.read(&metadata)
            .expect("read matches context")
            .expect("header is present")
            .visit_members(|member| {
                values.extend(member.values().map(|value| value.bytes.to_vec()));
            });
        assert_eq!(values, vec![b"acme".to_vec()]);

        let mut propagated = Vec::new();
        fixture
            .node_bindings("out")
            .sink()
            .expect("sink binding")
            .propagate(&metadata, |value: PropagatedValue<'_>| {
                propagated.push((value.wire_name.to_owned(), value.bytes.to_vec()));
            })
            .expect("sink matches context");
        assert_eq!(propagated, vec![("x-tenant".to_owned(), b"acme".to_vec())]);
    }

    /// Scenario: a pipeline context policy defines a constant composite member.
    /// Guarantees: the member is materialized at arrival and reported with its
    /// value.
    #[test]
    fn materialized_member_is_reported() {
        let fixture = ContextFixture::builder()
            .policy(
                "context",
                json!({ "entries": { "tenant": [
                    { "type": "constant", "name": "route", "value": "dedicated" }
                ] } }),
            )
            .sink(
                "out",
                json!({ "default": { "selector": { "type": "named", "named": ["tenant:route"] } } }),
            )
            .compile_standalone();

        let metadata = fixture.form_empty();

        let route = materialized_member(&metadata, "tenant", "route").expect("route member");
        assert_eq!(route.values().len(), 1);
        assert_eq!(route.values()[0].as_str(), Some("dedicated"));
        assert_eq!(materialized_member(&metadata, "tenant", "missing"), None);
    }
}
