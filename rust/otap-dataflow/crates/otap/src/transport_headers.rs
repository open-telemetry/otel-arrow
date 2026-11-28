// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use otel_arrow_dfe_config::ContextEntryName;
    use otel_arrow_dfe_config::context::transport_headers::ValueKind;
    use otel_arrow_dfe_config::context::transport_policy::{
        CaptureDefaults, CaptureRule, HeaderCapturePolicy, HeaderPropagationPolicy,
        PropagationAction, PropagationDefault, PropagationMatch, PropagationOverride,
        PropagationSelector, PropagationSelectorType,
    };
    use otel_arrow_dfe_engine::pdata_context::{
        Arrival, ArrivalHeaders, Metadata, PropagatedValue,
    };
    use otel_arrow_dfe_engine::testing::context::{ContextFixture, captured_headers};

    // -- Helper functions for tests ------------------------------------------

    /// Node id of the compile-only exporter carrying the propagation policy.
    const EXPORTER: &str = "exporter";

    /// Compiles a pipeline whose arrival receiver captures with `rules` and
    /// whose exporter propagates with `propagation`.
    fn capture_propagate_fixture(
        rules: Vec<CaptureRule>,
        propagation: &HeaderPropagationPolicy,
    ) -> ContextFixture {
        ContextFixture::builder()
            .header_capture(&HeaderCapturePolicy::new(CaptureDefaults::default(), rules))
            .sink(
                EXPORTER,
                serde_json::to_value(propagation).expect("serializable propagation policy"),
            )
            .compile_standalone()
    }

    /// Forms context from inbound header pairs and asserts that capture
    /// reported no limit violations.
    fn capture(fixture: &ContextFixture, inbound: &[(&str, &[u8])]) -> Metadata {
        let (metadata, stats) = fixture
            .arrival()
            .form(Arrival {
                headers: ArrivalHeaders::Pairs(inbound),
                identity: None,
                peer: None,
            })
            .expect("arrival forms context");
        assert!(stats.is_none());
        metadata
    }

    /// Propagates `metadata` through the fixture exporter as
    /// `(wire_name, kind, bytes)` triples, in emission order.
    fn propagate(
        fixture: &ContextFixture,
        metadata: &Metadata,
    ) -> Vec<(String, ValueKind, Vec<u8>)> {
        let mut propagated = Vec::new();
        fixture
            .node_bindings(EXPORTER)
            .sink()
            .expect("exporter has a sink binding")
            .propagate(metadata, |value: PropagatedValue<'_>| {
                propagated.push((value.wire_name.to_owned(), value.kind, value.bytes.to_vec()));
            })
            .expect("sink matches context");
        propagated
    }

    fn context_name(raw: &str) -> ContextEntryName {
        ContextEntryName::try_from(raw).expect("valid test context entry name")
    }

    fn rule(names: &[&str], store_as: Option<&str>) -> CaptureRule {
        CaptureRule {
            match_names: names.iter().map(|name| context_name(name)).collect(),
            store_as: store_as.map(context_name),
            sensitive: false,
            value_kind: None,
        }
    }

    // -- End-to-end integration tests (depend on OtapPdata) ------------------

    /// Scenario: capture, pdata attachment, and processor pass-through precede propagation.
    /// Guarantees: retained headers preserve names and values, while an override drops authorization.
    ///
    /// 1. **Receiver extraction** -- Simulate a receiver capturing headers from
    ///    inbound gRPC metadata using `HeaderCapturePolicy`.
    /// 2. **Pdata context attachment** -- Attach captured headers to `OtapPdata`.
    /// 3. **Processor transparency** -- Verify headers survive `clone_without_context()`
    ///    (what happens at pipeline boundaries / processor pass-through).
    /// 4. **Exporter propagation** -- Apply `HeaderPropagationPolicy` to filter headers
    ///    for egress, including dropping sensitive headers like `authorization`.
    ///    This test exercises the scenario from the design spec:
    /// - `otlp_ingest` captures `x-tenant-id`, `x-request-id`, `authorization`
    /// - `batch` processor preserves headers unchanged
    /// - `otap_export` propagates all except `authorization` (dropped by override)
    #[test]
    fn end_to_end_capture_preserve_propagate() {
        // ========== Pipeline: receiver capture and exporter propagation ==========

        let propagation_policy = HeaderPropagationPolicy::new(
            PropagationDefault {
                selector: PropagationSelector {
                    selector_type: PropagationSelectorType::AllCaptured,
                    named: None,
                },
                ..PropagationDefault::default()
            },
            vec![PropagationOverride {
                match_rule: PropagationMatch {
                    stored_names: vec![context_name("authorization")],
                },
                action: PropagationAction::Drop,
                name: None,
                on_error: None,
            }],
        );
        let fixture = capture_propagate_fixture(
            vec![
                rule(&["x-tenant-id"], Some("tenant_id")),
                rule(&["x-request-id"], None),
                rule(&["authorization"], None),
            ],
            &propagation_policy,
        );

        // ========== Step 1: Simulate receiver header capture ==========

        let inbound_metadata: Vec<(&str, &[u8])> = vec![
            ("X-Tenant-Id", b"tenant-abc-123"),
            ("X-Request-Id", b"req-xyz-789"),
            ("Authorization", b"Bearer super-secret-token"),
            ("X-Unrelated-Header", b"should-be-ignored"),
        ];

        let metadata = capture(&fixture, &inbound_metadata);
        let captured = captured_headers(&metadata).expect("headers captured");

        assert_eq!(
            captured.len(),
            3,
            "should capture exactly 3 matching headers"
        );
        assert_eq!(
            captured.get(0).expect("tenant header").name.as_str(),
            "tenant_id"
        );
        assert_eq!(
            captured.get(0).expect("tenant header").wire_name(),
            "X-Tenant-Id"
        );
        assert_eq!(
            captured.get(0).expect("tenant header").value.bytes,
            b"tenant-abc-123"
        );
        assert_eq!(
            captured.get(1).expect("request header").name.as_str(),
            "x-request-id"
        );
        assert_eq!(
            captured.get(2).expect("authorization header").name.as_str(),
            "authorization"
        );

        // ========== Step 2: Attach to OtapPdata context ==========

        let pdata = crate::testing::create_test_pdata().with_metadata(metadata);

        assert_eq!(
            captured_headers(pdata.metadata()).map(|headers| headers.len()),
            Some(3)
        );

        // ========== Step 3: Simulate processor pass-through ==========

        let pdata_after_processor = pdata.clone_without_context();

        let headers_after = captured_headers(pdata_after_processor.metadata())
            .expect("transport headers must survive clone_without_context()");
        assert_eq!(headers_after.len(), 3);
        assert_eq!(
            headers_after.get(0).expect("tenant header").name.as_str(),
            "tenant_id"
        );
        assert_eq!(
            headers_after.get(1).expect("request header").name.as_str(),
            "x-request-id"
        );
        assert_eq!(
            headers_after
                .get(2)
                .expect("authorization header")
                .name
                .as_str(),
            "authorization"
        );

        // ========== Step 4: Simulate exporter propagation ==========

        let propagated = propagate(&fixture, pdata_after_processor.metadata());

        assert_eq!(
            propagated.len(),
            2,
            "authorization should be dropped, leaving 2 headers"
        );
        assert_eq!(propagated[0].0, "X-Tenant-Id");
        assert_eq!(propagated[0].2, b"tenant-abc-123");
        assert_eq!(propagated[1].0, "X-Request-Id");
        assert_eq!(propagated[1].2, b"req-xyz-789");

        assert!(
            propagated
                .iter()
                .all(|(wire_name, _, _)| wire_name != "Authorization"),
            "authorization header must not be propagated"
        );
    }

    /// Scenario: capture and processor pass-through preserve repeated header names.
    /// Guarantees: propagation emits every duplicate value in its original order.
    #[test]
    fn end_to_end_duplicate_headers_preserved() {
        let propagation_policy = HeaderPropagationPolicy::new(
            PropagationDefault {
                selector: PropagationSelector {
                    selector_type: PropagationSelectorType::AllCaptured,
                    named: None,
                },
                ..PropagationDefault::default()
            },
            vec![],
        );
        let fixture =
            capture_propagate_fixture(vec![rule(&["x-forwarded-for"], None)], &propagation_policy);

        let inbound: Vec<(&str, &[u8])> = vec![
            ("X-Forwarded-For", b"10.0.0.1"),
            ("X-Forwarded-For", b"192.168.1.1"),
            ("X-Forwarded-For", b"172.16.0.1"),
        ];

        let metadata = capture(&fixture, &inbound);
        assert_eq!(
            captured_headers(&metadata).map(|headers| headers.len()),
            Some(3),
            "all duplicate headers should be captured"
        );

        let pdata = crate::testing::create_test_pdata().with_metadata(metadata);
        let pdata_after = pdata.clone_without_context();

        assert_eq!(
            captured_headers(pdata_after.metadata()).map(|headers| headers.len()),
            Some(3),
            "duplicates must survive clone_without_context"
        );

        let propagated = propagate(&fixture, pdata_after.metadata());
        assert_eq!(propagated.len(), 3, "duplicates must survive propagation");

        let values: Vec<&[u8]> = propagated.iter().map(|(_, _, value)| &value[..]).collect();
        let expected: Vec<&[u8]> = vec![b"10.0.0.1", b"192.168.1.1", b"172.16.0.1"];
        assert_eq!(values, expected);
    }

    /// Scenario: a captured binary header passes through pdata before propagation.
    /// Guarantees: propagation retains its binary kind and exact bytes.
    #[test]
    fn end_to_end_binary_headers_preserved() {
        let propagation_policy = HeaderPropagationPolicy::new(
            PropagationDefault {
                selector: PropagationSelector {
                    selector_type: PropagationSelectorType::AllCaptured,
                    named: None,
                },
                ..PropagationDefault::default()
            },
            vec![],
        );
        let fixture = capture_propagate_fixture(
            vec![rule(&["trace-context-bin"], None)],
            &propagation_policy,
        );

        let binary_value: Vec<u8> = vec![0x00, 0x01, 0xFF, 0xFE, 0x80, 0x7F];
        let inbound: Vec<(&str, &[u8])> = vec![("trace-context-bin", &binary_value)];

        let metadata = capture(&fixture, &inbound);
        let captured = captured_headers(&metadata).expect("headers captured");
        assert_eq!(captured.len(), 1);
        assert_eq!(
            captured.get(0).expect("binary header").value.value_kind,
            ValueKind::Binary
        );
        assert_eq!(
            captured.get(0).expect("binary header").value.bytes,
            binary_value
        );

        let pdata = crate::testing::create_test_pdata().with_metadata(metadata);
        let pdata_after = pdata.clone_without_context();

        let propagated = propagate(&fixture, pdata_after.metadata());

        assert_eq!(propagated[0].1, ValueKind::Binary);
        assert_eq!(propagated[0].2, binary_value.as_slice());
    }
}
