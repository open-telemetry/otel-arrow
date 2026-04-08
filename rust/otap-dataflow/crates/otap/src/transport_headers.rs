// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Protocol-neutral transport header abstraction for end-to-end header
//! propagation through the pipeline.
//!
//! Core types and engines are defined in [`otap_df_config::transport_headers`]
//! and re-exported here for backward compatibility and convenience.

// Re-export all public items from the config crate's transport_headers module.
pub use otap_df_config::transport_headers::{TransportHeader, TransportHeaders, ValueKind};
// Re-export policy types that include capture and propagation logic.
pub use otap_df_config::transport_headers_policy::{
    CaptureStats, HeaderCapturePolicy, HeaderPropagationPolicy, PropagatedHeader,
};

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_config::transport_headers_policy::{
        CaptureDefaults, CaptureRule, PropagationAction, PropagationDefault, PropagationMatch,
        PropagationOverride, PropagationSelector,
    };

    // -- Helper functions for tests ------------------------------------------

    fn make_capture_policy(rules: Vec<CaptureRule>) -> HeaderCapturePolicy {
        HeaderCapturePolicy::new(CaptureDefaults::default(), rules)
    }

    fn rule(names: &[&str], store_as: Option<&str>) -> CaptureRule {
        CaptureRule {
            match_names: names.iter().map(|s| s.to_string()).collect(),
            store_as: store_as.map(|s| s.to_string()),
            sensitive: false,
            value_kind: None,
        }
    }

    // -- End-to-end integration tests (depend on OtapPdata) ------------------

    /// End-to-end test demonstrating the full transport header lifecycle:
    ///
    /// 1. **Receiver extraction** — Simulate a receiver capturing headers from
    ///    inbound gRPC metadata using `HeaderCapturePolicy`.
    /// 2. **Pdata context attachment** — Attach captured headers to `OtapPdata`.
    /// 3. **Processor transparency** — Verify headers survive `clone_without_context()`
    ///    (what happens at pipeline boundaries / processor pass-through).
    /// 4. **Exporter propagation** — Apply `HeaderPropagationPolicy` to filter headers
    ///    for egress, including dropping sensitive headers like `authorization`.
    ///    This test exercises the scenario from the design spec:
    /// - `otlp_ingest` captures `x-tenant-id`, `x-request-id`, `authorization`
    /// - `batch` processor preserves headers unchanged
    /// - `otap_export` propagates all except `authorization` (dropped by override)
    #[test]
    fn end_to_end_capture_preserve_propagate() {
        // ========== Step 1: Simulate receiver header capture ==========

        let capture_policy = make_capture_policy(vec![
            rule(&["x-tenant-id"], Some("tenant_id")),
            rule(&["x-request-id"], None),
            rule(&["authorization"], None),
        ]);

        let inbound_metadata: Vec<(&str, &[u8])> = vec![
            ("X-Tenant-Id", b"tenant-abc-123"),
            ("X-Request-Id", b"req-xyz-789"),
            ("Authorization", b"Bearer super-secret-token"),
            ("X-Unrelated-Header", b"should-be-ignored"),
        ];

        let mut captured = TransportHeaders::new();
        let stats = capture_policy.capture_from_pairs(inbound_metadata.into_iter(), &mut captured);
        assert!(stats.is_none());

        assert_eq!(
            captured.len(),
            3,
            "should capture exactly 3 matching headers"
        );
        assert_eq!(captured.as_slice()[0].name, "tenant_id");
        assert_eq!(captured.as_slice()[0].wire_name, "X-Tenant-Id");
        assert_eq!(captured.as_slice()[0].value, b"tenant-abc-123");
        assert_eq!(captured.as_slice()[1].name, "x-request-id");
        assert_eq!(captured.as_slice()[2].name, "authorization");

        // ========== Step 2: Attach to OtapPdata context ==========

        let pdata = crate::testing::create_test_pdata().with_transport_headers(captured);

        assert!(pdata.transport_headers().is_some());
        assert_eq!(pdata.transport_headers().unwrap().len(), 3);

        // ========== Step 3: Simulate processor pass-through ==========

        let pdata_after_processor = pdata.clone_without_context();

        assert!(
            pdata_after_processor.transport_headers().is_some(),
            "transport headers must survive clone_without_context()"
        );
        let headers_after = pdata_after_processor.transport_headers().unwrap();
        assert_eq!(headers_after.len(), 3);
        assert_eq!(headers_after.as_slice()[0].name, "tenant_id");
        assert_eq!(headers_after.as_slice()[1].name, "x-request-id");
        assert_eq!(headers_after.as_slice()[2].name, "authorization");

        // ========== Step 4: Simulate exporter propagation ==========

        let propagation_policy = HeaderPropagationPolicy::new(
            PropagationDefault {
                selector: PropagationSelector::AllCaptured,
                ..PropagationDefault::default()
            },
            vec![PropagationOverride {
                match_rule: PropagationMatch {
                    stored_names: vec!["authorization".to_string()],
                },
                action: PropagationAction::Drop,
                name: None,
                on_error: None,
            }],
        );

        let propagated: Vec<_> = propagation_policy.propagate(headers_after).collect();

        assert_eq!(
            propagated.len(),
            2,
            "authorization should be dropped, leaving 2 headers"
        );
        assert_eq!(propagated[0].header_name, "X-Tenant-Id");
        assert_eq!(propagated[0].value, b"tenant-abc-123");
        assert_eq!(propagated[1].header_name, "X-Request-Id");
        assert_eq!(propagated[1].value, b"req-xyz-789");

        assert!(
            propagated.iter().all(|h| h.header_name != "Authorization"),
            "authorization header must not be propagated"
        );
    }

    /// Test that demonstrates duplicate header names are preserved throughout
    /// the entire pipeline flow (a key semantic requirement).
    #[test]
    fn end_to_end_duplicate_headers_preserved() {
        let capture_policy = make_capture_policy(vec![rule(&["x-forwarded-for"], None)]);

        let inbound: Vec<(&str, &[u8])> = vec![
            ("X-Forwarded-For", b"10.0.0.1"),
            ("X-Forwarded-For", b"192.168.1.1"),
            ("X-Forwarded-For", b"172.16.0.1"),
        ];

        let mut captured = TransportHeaders::new();
        let stats = capture_policy.capture_from_pairs(inbound.into_iter(), &mut captured);
        assert!(stats.is_none());
        assert_eq!(
            captured.len(),
            3,
            "all duplicate headers should be captured"
        );

        let pdata = crate::testing::create_test_pdata().with_transport_headers(captured);
        let pdata_after = pdata.clone_without_context();

        let headers = pdata_after.transport_headers().unwrap();
        assert_eq!(
            headers.len(),
            3,
            "duplicates must survive clone_without_context"
        );

        let propagation_policy = HeaderPropagationPolicy::new(
            PropagationDefault {
                selector: PropagationSelector::AllCaptured,
                ..PropagationDefault::default()
            },
            vec![],
        );
        let propagated: Vec<_> = propagation_policy.propagate(headers).collect();
        assert_eq!(propagated.len(), 3, "duplicates must survive propagation");

        let values: Vec<&[u8]> = propagated.iter().map(|h| h.value).collect();
        let expected: Vec<&[u8]> = vec![b"10.0.0.1", b"192.168.1.1", b"172.16.0.1"];
        assert_eq!(values, expected);
    }

    /// Test binary header preservation through the entire flow.
    #[test]
    fn end_to_end_binary_headers_preserved() {
        let capture_policy = make_capture_policy(vec![rule(&["trace-context-bin"], None)]);

        let binary_value: Vec<u8> = vec![0x00, 0x01, 0xFF, 0xFE, 0x80, 0x7F];
        let inbound: Vec<(&str, &[u8])> = vec![("trace-context-bin", &binary_value)];

        let mut captured = TransportHeaders::new();
        let stats = capture_policy.capture_from_pairs(inbound.into_iter(), &mut captured);
        assert!(stats.is_none());
        assert_eq!(captured.len(), 1);
        assert_eq!(captured.as_slice()[0].value_kind, ValueKind::Binary);
        assert_eq!(captured.as_slice()[0].value, binary_value);

        let pdata = crate::testing::create_test_pdata().with_transport_headers(captured);
        let pdata_after = pdata.clone_without_context();

        let headers = pdata_after.transport_headers().unwrap();
        let propagation_policy = HeaderPropagationPolicy::new(
            PropagationDefault {
                selector: PropagationSelector::AllCaptured,
                ..PropagationDefault::default()
            },
            vec![],
        );
        let propagated: Vec<_> = propagation_policy.propagate(headers).collect();

        assert_eq!(*propagated[0].value_kind, ValueKind::Binary);
        assert_eq!(propagated[0].value, binary_value.as_slice());
    }
}
