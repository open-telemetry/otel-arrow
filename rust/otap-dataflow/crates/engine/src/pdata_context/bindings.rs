// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::{
    AuthorizedIdentityRead, CompiledHeaderPropagationPolicy, ContextMaterializationPlan,
    HeadersRead, Metadata, ValueRead,
};
use crate::capability::auth::{AuthorizedIdentity, ClaimValue};
use http::HeaderMap;
use otel_arrow_dfe_config::context::authorized_identity::AuthorizedIdentityPolicy;
use otel_arrow_dfe_config::context::transport_headers::{TransportHeaders, ValueKind};
use otel_arrow_dfe_config::context::transport_policy::{CaptureStats, CompiledHeaderCapturePolicy};
use std::net::SocketAddr;

/// A borrowed verified claim, retaining scalar or multi-valued cardinality.
pub type ClaimRef<'a> = &'a ClaimValue;

/// Verified identity input used to capture only policy-selected claims.
///
/// Lookups must remain stable for the duration of one arrival operation.
pub trait Claims {
    /// Returns a verified claim by its exact name.
    fn claim(&self, name: &str) -> Option<ClaimRef<'_>>;
}

impl Claims for AuthorizedIdentity {
    fn claim(&self, name: &str) -> Option<ClaimRef<'_>> {
        self.claim(name)
    }
}

/// Transport input without a temporary HTTP-to-pairs conversion.
pub enum ArrivalHeaders<'a> {
    /// Protocol-decoded header values; binary values are already decoded.
    Pairs(&'a [(&'a str, &'a [u8])]),
    /// Native HTTP headers, captured through the compiled HTTP index.
    Http(&'a HeaderMap),
}

/// All request-scoped inputs supplied by a receiver.
pub struct Arrival<'a> {
    /// Inbound transport headers.
    pub headers: ArrivalHeaders<'a>,
    /// Verified identity, if authorization produced one.
    pub identity: Option<&'a dyn Claims>,
    /// Peer observed at request acceptance.
    pub peer: Option<SocketAddr>,
}

/// Compiled capture and materialization operations for a receiver.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ArrivalBinding {
    pub(super) header_capture: Option<CompiledHeaderCapturePolicy>,
    pub(super) authorized_identity_capture: Option<AuthorizedIdentityPolicy>,
    pub(super) materialization: ContextMaterializationPlan,
}

impl ArrivalBinding {
    /// Captures headers and identity, records the peer, then materializes composites.
    #[must_use]
    pub fn form(&self, arrival: &Arrival<'_>) -> (Metadata, Option<CaptureStats>) {
        Self::form_with(
            self.header_capture.as_ref(),
            self.authorized_identity_capture.as_ref(),
            Some(&self.materialization),
            arrival,
        )
    }

    pub(crate) fn form_with(
        capture: Option<&CompiledHeaderCapturePolicy>,
        identity_policy: Option<&AuthorizedIdentityPolicy>,
        materialization: Option<&ContextMaterializationPlan>,
        arrival: &Arrival<'_>,
    ) -> (Metadata, Option<CaptureStats>) {
        let mut metadata = Metadata::default();
        let stats = capture.and_then(|policy| {
            let mut headers = TransportHeaders::new();
            let stats = match &arrival.headers {
                ArrivalHeaders::Pairs(pairs) => {
                    policy.capture_from_pairs(pairs.iter().copied(), &mut headers)
                }
                ArrivalHeaders::Http(input) => {
                    policy.capture_from_http_headers(input, &mut headers)
                }
            };
            metadata.set_transport_headers(headers);
            stats
        });
        if let (Some(policy), Some(identity)) = (identity_policy, arrival.identity) {
            metadata.capture_authorized_identity(policy, identity);
        }
        if let Some(peer) = arrival.peer {
            metadata.set_peer_addr(peer);
        }
        if let Some(plan) = materialization {
            metadata.materialize_context(plan);
        }
        (metadata, stats)
    }
}

/// A borrowed outbound value with its transport kind and wire name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PropagatedValue<'a> {
    /// Name selected by the compiled propagation policy.
    pub wire_name: &'a str,
    /// Text or binary representation; bytes are not re-encoded.
    pub kind: ValueKind,
    /// Raw value bytes.
    pub bytes: &'a [u8],
}

/// Compiled primitive and materialized propagation for an exporter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SinkBinding {
    pub(super) policy: CompiledHeaderPropagationPolicy,
}

impl SinkBinding {
    pub(crate) fn new(policy: CompiledHeaderPropagationPolicy) -> Self {
        Self { policy }
    }

    pub(crate) fn policy(&self) -> &CompiledHeaderPropagationPolicy {
        &self.policy
    }

    /// Emits the existing primitive and materialized propagation results in order.
    pub fn propagate<'a>(
        &'a self,
        metadata: &'a Metadata,
        mut emit: impl FnMut(&'a str, PropagatedValue<'a>),
    ) {
        for header in metadata
            .transport_headers()
            .into_iter()
            .flat_map(|headers| self.policy.propagate(headers))
            .chain(
                self.policy
                    .materialized_headers(metadata.materialized_context()),
            )
        {
            emit(
                header.header_name,
                PropagatedValue {
                    wire_name: header.header_name,
                    kind: header.value_kind,
                    bytes: header.value,
                },
            );
        }
    }
}

/// Immutable runtime context policies compiled for one node.
///
/// Compiler declaration-validation state is kept separately. Policy types stay
/// private as node-facing binding operations are introduced incrementally.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NodeBindings {
    pub(super) arrival: Option<ArrivalBinding>,
    pub(super) sink: Option<SinkBinding>,
    pub(super) values: Box<[ValueRead]>,
    pub(super) headers: Option<HeadersRead>,
    pub(super) authorized_identity: Option<AuthorizedIdentityRead>,
}

impl NodeBindings {
    /// Returns the receiver's compiled arrival operation.
    #[must_use]
    pub fn arrival(&self) -> Option<&ArrivalBinding> {
        self.arrival.as_ref()
    }

    /// Returns the exporter's compiled propagation operation, if configured.
    #[must_use]
    pub fn sink(&self) -> Option<&SinkBinding> {
        self.sink.as_ref()
    }

    /// Returns declared named reads for selection once during node construction.
    #[must_use]
    pub fn values(&self) -> &[ValueRead] {
        &self.values
    }

    /// Returns the all-stored header read, if declared.
    #[must_use]
    pub fn headers(&self) -> Option<&HeadersRead> {
        self.headers.as_ref()
    }

    /// Returns the all-stored authorized-identity read, if declared.
    #[must_use]
    pub fn authorized_identity(&self) -> Option<&AuthorizedIdentityRead> {
        self.authorized_identity.as_ref()
    }

    pub(crate) fn header_capture_policy(&self) -> Option<&CompiledHeaderCapturePolicy> {
        self.arrival.as_ref()?.header_capture.as_ref()
    }

    pub(crate) fn authorized_identity_policy(&self) -> Option<&AuthorizedIdentityPolicy> {
        self.arrival.as_ref()?.authorized_identity_capture.as_ref()
    }

    pub(crate) fn header_propagation_policy(&self) -> Option<&CompiledHeaderPropagationPolicy> {
        Some(&self.sink.as_ref()?.policy)
    }

    pub(crate) fn set_header_capture_policy(
        &mut self,
        policy: Option<CompiledHeaderCapturePolicy>,
    ) {
        self.arrival
            .get_or_insert_with(Default::default)
            .header_capture = policy;
    }

    pub(crate) fn set_authorized_identity_policy(
        &mut self,
        policy: Option<AuthorizedIdentityPolicy>,
    ) {
        self.arrival
            .get_or_insert_with(Default::default)
            .authorized_identity_capture = policy;
    }

    pub(crate) fn set_materialization_plan(&mut self, plan: ContextMaterializationPlan) {
        self.arrival
            .get_or_insert_with(Default::default)
            .materialization = plan;
    }

    pub(crate) fn set_header_propagation_policy(
        &mut self,
        policy: Option<CompiledHeaderPropagationPolicy>,
    ) {
        self.sink = policy.map(|policy| SinkBinding { policy });
    }

    pub(super) fn is_empty(&self) -> bool {
        self.header_capture_policy().is_none()
            && self.authorized_identity_policy().is_none()
            && self.header_propagation_policy().is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_config::context::ContextEntryName;
    use otel_arrow_dfe_config::context::policy::{
        ContextEntryDeclaration, ContextEntryDefinition, ContextEntryPart, ContextRandomnessKind,
        ContextScope,
    };
    use otel_arrow_dfe_config::context::transport_policy::{
        HeaderCapturePolicy, HeaderPropagationPolicy,
    };

    fn name(value: &str) -> ContextEntryName {
        value.try_into().expect("valid context name")
    }

    fn declaration(parts: Vec<ContextEntryPart>) -> ContextEntryDeclaration {
        ContextEntryDeclaration {
            scope: ContextScope::Pipeline("group".into(), "pipeline".into()),
            name: name("request"),
            definition: ContextEntryDefinition(parts),
        }
    }

    fn capture_policy() -> CompiledHeaderCapturePolicy {
        serde_json::from_value::<HeaderCapturePolicy>(serde_json::json!({
            "defaults": {"max_entries": 4, "max_value_bytes": 8},
            "headers": [
                {"match_names": ["x-tenant"], "store_as": "tenant"},
                {"match_names": ["x-token-bin"], "store_as": "token"},
                {"match_names": ["x-limit"]}
            ]
        }))
        .expect("capture policy")
        .compile(|_| true)
    }

    fn propagation_policy(selector: serde_json::Value, naming: &str) -> HeaderPropagationPolicy {
        serde_json::from_value(serde_json::json!({
            "default": {"selector": selector, "name": naming}
        }))
        .expect("propagation policy")
    }

    /// Scenario: arrival receives repeated, binary, unmatched, and oversized transport headers.
    /// Guarantees: pairs and native HTTP capture preserve legacy values, limits, peer, and constants.
    #[test]
    fn arrival_matches_pairs_and_http_capture() {
        let binding = ArrivalBinding {
            header_capture: Some(capture_policy()),
            materialization: ContextMaterializationPlan::compile(&[declaration(vec![
                ContextEntryPart::TransportHeader {
                    name: name("tenant"),
                    store_as: None,
                },
                ContextEntryPart::Constant {
                    name: name("route"),
                    value: "fixed".to_owned(),
                },
            ])])
            .expect("materialization plan"),
            ..Default::default()
        };
        let pairs = [
            ("x-tenant", b"first".as_slice()),
            ("x-tenant", b"second".as_slice()),
            ("x-token-bin", b"\x80\xff".as_slice()),
            ("x-limit", b"too-long-value".as_slice()),
            ("unselected", b"ignored".as_slice()),
        ];
        let mut http = HeaderMap::new();
        for (key, value) in pairs {
            let _ = http.append(
                http::HeaderName::from_bytes(key.as_bytes()).expect("header name"),
                http::HeaderValue::from_bytes(value).expect("header value"),
            );
        }
        let peer = "127.0.0.1:4318".parse().expect("peer address");
        for input in [ArrivalHeaders::Pairs(&pairs), ArrivalHeaders::Http(&http)] {
            let mut headers = TransportHeaders::new();
            let policy = binding.header_capture.as_ref().expect("capture policy");
            let expected_stats = match &input {
                ArrivalHeaders::Pairs(input) => {
                    policy.capture_from_pairs(input.iter().copied(), &mut headers)
                }
                ArrivalHeaders::Http(input) => {
                    policy.capture_from_http_headers(input, &mut headers)
                }
            };
            let mut expected = Metadata::default();
            expected.set_transport_headers(headers);
            expected.set_peer_addr(peer);
            expected.materialize_context(&binding.materialization);

            let (actual, stats) = binding.form(&Arrival {
                headers: input,
                identity: None,
                peer: Some(peer),
            });
            assert_eq!(actual, expected);
            assert_eq!(stats, expected_stats);
            assert_eq!(
                stats
                    .expect("capture limit exceeded")
                    .skipped_value_too_long,
                1
            );
            assert_eq!(actual.transport_headers().expect("headers").len(), 3);
            assert!(
                actual
                    .materialized_context()
                    .expect("context")
                    .entry(&name("request"))
                    .is_some()
            );
        }
    }

    /// Scenario: arrival projects scalar, empty-many, and one-element-many verified claims.
    /// Guarantees: only selected claims are queried, cardinality survives, and identity precedes materialization.
    #[test]
    fn arrival_matches_selected_identity_capture() {
        struct SelectedClaims(AuthorizedIdentity);
        impl Claims for SelectedClaims {
            fn claim(&self, name: &str) -> Option<ClaimRef<'_>> {
                assert!(matches!(name, "sub" | "groups" | "empty" | "missing"));
                self.0.claim(name)
            }
        }
        let identity = SelectedClaims(
            AuthorizedIdentity::new()
                .with_subject("verified")
                .with_claim_values("groups", ["reader"])
                .with_claim_values("empty", [])
                .with_claim_str("unselected", "private"),
        );
        let policy: AuthorizedIdentityPolicy = serde_json::from_value(serde_json::json!([
            {"claim": "sub", "store_as": "tenant"},
            {"claim": "groups", "store_as": "groups"},
            {"claim": "empty", "store_as": "empty"},
            {"claim": "missing", "store_as": "missing"}
        ]))
        .expect("identity policy");
        let binding = ArrivalBinding {
            authorized_identity_capture: Some(policy.clone()),
            materialization: ContextMaterializationPlan::compile(&[declaration(vec![
                ContextEntryPart::AuthorizedIdentity {
                    name: name("tenant"),
                    store_as: None,
                },
                ContextEntryPart::Constant {
                    name: name("route"),
                    value: "authorized".to_owned(),
                },
            ])])
            .expect("materialization plan"),
            ..Default::default()
        };
        for identity in [Some(&identity as &dyn Claims), None] {
            let (actual, stats) = binding.form(&Arrival {
                headers: ArrivalHeaders::Pairs(&[("tenant", b"untrusted")]),
                identity,
                peer: None,
            });
            let mut expected = Metadata::default();
            if let Some(identity) = identity {
                expected.capture_authorized_identity(&policy, identity);
            }
            expected.materialize_context(&binding.materialization);
            assert_eq!(actual, expected);
            assert!(stats.is_none());
            assert!(actual.transport_headers().is_none());
            if identity.is_some() {
                let entries = actual.authorized_identity_entries().expect("claims");
                assert_eq!(entries.len(), 3);
                assert_eq!(
                    entries.get("tenant").expect("tenant").value().as_str(),
                    Some("verified")
                );
                assert!(entries.get("groups").expect("groups").value().is_many());
                assert!(entries.get("empty").expect("empty").value().is_empty());
                assert!(
                    actual
                        .materialized_context()
                        .expect("context")
                        .entry(&name("request"))
                        .is_some()
                );
            } else {
                assert!(actual.authorized_identity_entries().is_none());
                assert!(actual.materialized_context().is_none());
            }
        }
    }

    /// Scenario: an arrival containing constants and randomness reaches the legacy send-time materializer.
    /// Guarantees: the hook retains the exact generated value and metadata without reevaluating inputs.
    #[test]
    fn arrival_materialization_is_idempotent_at_send_time() {
        let binding = ArrivalBinding {
            materialization: ContextMaterializationPlan::compile(&[declaration(vec![
                ContextEntryPart::Constant {
                    name: name("route"),
                    value: "fixed".to_owned(),
                },
                ContextEntryPart::Randomness {
                    name: name("id"),
                    value: ContextRandomnessKind::Uuid7,
                },
            ])])
            .expect("materialization plan"),
            ..Default::default()
        };
        let (mut metadata, stats) = binding.form(&Arrival {
            headers: ArrivalHeaders::Pairs(&[]),
            identity: None,
            peer: None,
        });
        assert!(stats.is_none());
        let entry = metadata
            .materialized_context()
            .expect("context")
            .entry(&name("request"))
            .expect("request");
        let id = entry.member(&name("id")).expect("id").values().values()[0].bytes();
        assert_eq!(
            uuid::Uuid::try_parse(std::str::from_utf8(id).expect("UUID text"))
                .expect("UUID")
                .get_version_num(),
            7
        );
        let before = metadata.clone();
        let materialize = crate::receiver::ReceiverContextMaterialization::new(
            binding.materialization.clone(),
            Metadata::materialize_context,
        );
        materialize.apply(&mut metadata);
        assert_eq!(metadata, before);
    }

    /// Scenario: sink propagation selects primitive or composite headers with either naming strategy.
    /// Guarantees: order, duplicates, binary kind, raw bytes, and wire names match the legacy paths.
    #[test]
    fn sink_matches_primitive_and_materialized_propagation() {
        let declarations = [declaration(vec![
            ContextEntryPart::TransportHeader {
                name: name("token"),
                store_as: Some(name("binary")),
            },
            ContextEntryPart::Constant {
                name: name("route"),
                value: "fixed".to_owned(),
            },
        ])];
        for selector in [
            serde_json::json!({"type": "all_captured"}),
            serde_json::json!({"type": "named", "named": ["tenant", "request:binary", "request:route"]}),
        ] {
            for naming in ["preserve", "stored_name"] {
                let policy = CompiledHeaderPropagationPolicy::compile_standalone(
                    propagation_policy(selector.clone(), naming),
                    &declarations,
                )
                .expect("propagation compiles");
                let arrival = ArrivalBinding {
                    header_capture: Some(capture_policy()),
                    materialization: policy.materialization_plan().clone(),
                    ..Default::default()
                };
                let (metadata, _) = arrival.form(&Arrival {
                    headers: ArrivalHeaders::Pairs(&[
                        ("X-Tenant", b"first"),
                        ("x-tenant", b"second"),
                        ("X-Token-bin", b"\x00\xff"),
                    ]),
                    identity: None,
                    peer: None,
                });
                let expected = policy
                    .propagate_materialized(
                        metadata.transport_headers().expect("headers"),
                        metadata.materialized_context(),
                    )
                    .map(|header| {
                        (
                            header.header_name.to_owned(),
                            header.value_kind,
                            header.value.to_vec(),
                        )
                    })
                    .collect::<Vec<_>>();
                let sink = SinkBinding::new(policy);
                let mut actual = Vec::new();
                sink.propagate(&metadata, |name, value| {
                    assert_eq!(name, value.wire_name);
                    actual.push((name.to_owned(), value.kind, value.bytes.to_vec()));
                });
                assert_eq!(actual, expected);
                assert!(
                    actual
                        .iter()
                        .any(|(_, kind, bytes)| *kind == ValueKind::Binary && bytes == b"\x00\xff")
                );
            }
        }
    }

    /// Scenario: local and shared exporter handlers receive compiled bindings and are cloned.
    /// Guarantees: legacy getters borrow the sink's policy, clones share bindings, and setters remain isolated.
    #[test]
    fn exporter_handlers_share_sink_bindings() {
        use crate::runtime_services::PipelineRuntimeServices;
        use crate::testing::test_node;
        use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
        use std::sync::Arc;

        let services = PipelineRuntimeServices::new(Default::default()).expect("runtime services");
        let (_metrics_rx, reporter) = MetricsReporter::create_new_and_receiver(1);
        let policy = CompiledHeaderPropagationPolicy::compile_standalone(
            propagation_policy(serde_json::json!({"type": "all_captured"}), "preserve"),
            &[],
        )
        .expect("propagation compiles");
        let bindings = Arc::new(NodeBindings {
            arrival: None,
            sink: Some(SinkBinding::new(policy.clone())),
            ..Default::default()
        });
        macro_rules! check_handler {
            ($handler:expr) => {{
                let mut handler = $handler;
                handler.set_propagation_policy(Some(policy.clone()));
                assert!(handler.node_bindings().is_none());
                assert!(std::ptr::eq(
                    handler.propagation_policy().expect("legacy policy"),
                    handler.sink().expect("legacy sink").policy()
                ));
                handler.set_node_bindings(Arc::clone(&bindings));
                let cloned = handler.clone();
                assert!(Arc::ptr_eq(
                    cloned.node_bindings().expect("cloned bindings"),
                    &bindings
                ));
                assert!(std::ptr::eq(
                    handler.propagation_policy().expect("policy"),
                    bindings
                        .header_propagation_policy()
                        .expect("compiled policy")
                ));
                handler.set_propagation_policy(None);
                assert!(handler.sink().is_none());
                assert!(handler.propagation_policy().is_none());
                assert!(cloned.sink().is_some());
            }};
        }
        check_handler!(crate::local::exporter::EffectHandler::<()>::new(
            test_node("local"),
            reporter.clone(),
            services.clone(),
        ));
        check_handler!(crate::shared::exporter::EffectHandler::<()>::new(
            test_node("shared"),
            reporter,
            services,
        ));
    }

    /// Scenario: local and shared receivers use legacy setters, then install and clone compiled bindings.
    /// Guarantees: both formation paths agree, getters borrow compiled policies, and later setters do not mutate clones.
    #[test]
    fn receiver_handlers_preserve_legacy_and_compiled_formation() {
        use crate::runtime_services::PipelineRuntimeServices;
        use crate::testing::test_node;
        use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
        use std::sync::Arc;

        let services = PipelineRuntimeServices::new(Default::default()).expect("runtime services");
        let (_metrics_rx, reporter) = MetricsReporter::create_new_and_receiver(1);
        let (control, _control_rx) = crate::control::runtime_ctrl_msg_channel(1);
        let binding = ArrivalBinding {
            header_capture: Some(capture_policy()),
            authorized_identity_capture: Some(
                serde_json::from_value(serde_json::json!([
                    {"claim": "sub", "store_as": "tenant"}
                ]))
                .expect("identity policy"),
            ),
            materialization: ContextMaterializationPlan::compile(&[declaration(vec![
                ContextEntryPart::AuthorizedIdentity {
                    name: name("tenant"),
                    store_as: None,
                },
            ])])
            .expect("materialization plan"),
        };
        let bindings = Arc::new(NodeBindings {
            arrival: Some(binding.clone()),
            sink: None,
            ..Default::default()
        });
        let identity = AuthorizedIdentity::new().with_subject("verified");
        let arrival = Arrival {
            headers: ArrivalHeaders::Pairs(&[("x-tenant", b"untrusted")]),
            identity: Some(&identity),
            peer: Some("127.0.0.1:4317".parse().expect("peer")),
        };
        let expected = binding.form(&arrival);
        macro_rules! check_handler {
            ($handler:expr) => {{
                let mut handler = $handler;
                handler.set_capture_policy(binding.header_capture.clone());
                handler.set_authorized_identity_policy(binding.authorized_identity_capture.clone());
                handler.set_context_materializer(
                    binding.materialization.clone(),
                    Metadata::materialize_context,
                );
                assert!(handler.node_bindings().is_none());
                assert_eq!(handler.form_context(&arrival), expected);
                handler.set_node_bindings(Arc::clone(&bindings));
                let cloned = handler.clone();
                assert!(Arc::ptr_eq(
                    cloned.node_bindings().expect("cloned bindings"),
                    &bindings
                ));
                assert!(std::ptr::eq(
                    handler.capture_policy().expect("capture"),
                    bindings.header_capture_policy().expect("compiled capture")
                ));
                assert!(std::ptr::eq(
                    handler.authorized_identity_policy().expect("identity"),
                    bindings
                        .authorized_identity_policy()
                        .expect("compiled identity")
                ));
                assert_eq!(handler.form_context(&arrival), expected);
                handler.set_capture_policy(None);
                handler.set_authorized_identity_policy(None);
                assert!(handler.capture_policy().is_none());
                assert!(handler.authorized_identity_policy().is_none());
                assert_eq!(cloned.form_context(&arrival), expected);
            }};
        }
        check_handler!(crate::local::receiver::EffectHandler::<Metadata>::new(
            test_node("local"),
            Default::default(),
            None,
            control.clone(),
            reporter.clone(),
            services.clone(),
        ));
        check_handler!(crate::shared::receiver::EffectHandler::<Metadata>::new(
            test_node("shared"),
            Default::default(),
            None,
            control,
            reporter,
            services,
        ));
    }

    /// Scenario: a sink propagates a constant without any primitive transport headers.
    /// Guarantees: missing header storage does not suppress materialized-only propagation.
    #[test]
    fn sink_propagates_materialized_only_context() {
        let policy = CompiledHeaderPropagationPolicy::compile_standalone(
            propagation_policy(
                serde_json::json!({"type": "named", "named": ["request:route"]}),
                "stored_name",
            ),
            &[declaration(vec![ContextEntryPart::Constant {
                name: name("route"),
                value: "fixed".to_owned(),
            }])],
        )
        .expect("propagation compiles");
        let mut metadata = Metadata::default();
        metadata.materialize_context(policy.materialization_plan());
        let empty = TransportHeaders::new();
        let expected = policy
            .propagate_materialized(&empty, metadata.materialized_context())
            .map(|header| (header.header_name.to_owned(), header.value.to_vec()))
            .collect::<Vec<_>>();
        let sink = SinkBinding::new(policy);
        let mut actual = Vec::new();
        sink.propagate(&metadata, |name, value| {
            actual.push((name.to_owned(), value.bytes.to_vec()))
        });
        assert_eq!(actual, expected);
        assert_eq!(actual, [("route".to_owned(), b"fixed".to_vec())]);
        sink.propagate(&Metadata::default(), |_, _| {
            panic!("unmaterialized metadata must emit nothing")
        });
    }
}
