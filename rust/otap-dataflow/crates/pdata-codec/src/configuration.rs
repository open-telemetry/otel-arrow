// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Startup preparation of immutable implementation resources.
//!
//! Configuration is resolved before mutable instances are created. A preparation
//! hook may own typed settings or shared compiled resources in its returned
//! factory. This boundary can accommodate a future WASM adapter without putting
//! configuration or guest state in payloads. It does not load modules, define a
//! guest ABI, or offload work. Synchronous and incremental decoders use the same
//! prepared factory. Runtime-loaded metadata, bounded guest calls/counting and
//! async offloading remain future work.

use crate::{
    CodecError, DecodePolicy, EncodePolicy, PdataBatcher, PdataDecoder, PdataEncoder, ResolvedCodec,
};
use otel_arrow_dfe_config::codecs::CodecConfiguration;
use std::sync::Arc;

/// Validates implementation-owned options and prepares immutable factory resources.
///
/// Called at startup, never per record. Reject unknown options and avoid printing
/// their values in diagnostics. Do not create mutable decoder/encoder instances.
/// Common decode settings are enforced by the service and passed to lazy factories.
pub type CodecPreparer =
    fn(&CodecConfiguration) -> Result<Arc<dyn PreparedCodecFactory>, CodecError>;

/// Immutable, pipeline-owned factory for reusable mutable codec instances.
///
/// Native implementations may override only the directions needing configured
/// state. Future hosted factories can retain shared compiled code here and create
/// isolated guest instances below. A factory must preserve its registration's
/// encoding, signals, counting contract, and capabilities. No resource is stored
/// in a payload and no factory borrow crosses an asynchronous suspension.
pub trait PreparedCodecFactory: Send + Sync {
    /// Creates a decoder honoring the selected policy, including strict mode.
    fn create_decoder(
        &self,
        codec: ResolvedCodec,
        policy: DecodePolicy,
    ) -> Result<Box<dyn PdataDecoder>, CodecError> {
        codec.create_decoder(policy)
    }

    /// Creates an encoder honoring both implementation settings and output limits.
    fn create_encoder(
        &self,
        codec: ResolvedCodec,
        policy: EncodePolicy,
    ) -> Result<Box<dyn PdataEncoder>, CodecError> {
        codec.create_encoder(policy)
    }

    /// Creates a native batcher preserving the registration's declared contract.
    fn create_batcher(&self, codec: ResolvedCodec) -> Result<Box<dyn PdataBatcher>, CodecError> {
        codec.create_batcher()
    }

    /// Whether compatible encoded bytes may bypass this configured encoder.
    ///
    /// Default false: options may require rewriting already-compatible input.
    /// Return true only when forwarding satisfies every configured output option.
    /// Output-plan size limits still apply. Decode validation is not an ingress
    /// barrier and does not by itself prohibit forwarding.
    fn allows_encoded_forwarding(&self) -> bool {
        false
    }
}

/// Default native preparation, accepting no implementation-specific options.
pub(crate) fn prepare_native(
    config: &CodecConfiguration,
) -> Result<Arc<dyn PreparedCodecFactory>, CodecError> {
    if let Some(key) = config.options.keys().next() {
        return Err(CodecError::configuration(format!("unknown option `{key}`")));
    }
    Ok(Arc::new(NativeFactory))
}

struct NativeFactory;
impl PreparedCodecFactory for NativeFactory {
    fn allows_encoded_forwarding(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AcceptedEncodings, CodecMetadata, CodecRegistration, CodecRegistry, CodecService,
        CodecServiceBuilder, DecodeValidation, EncodingPlan, PdataEncoding, PdataPayload,
        PdataView,
    };
    use bytes::Bytes;
    use otel_arrow_dfe_config::{SignalType, codecs::CodecsConfig};
    use otel_arrow_dfe_pdata::OtapArrowRecords;
    use otel_arrow_dfe_pdata::testing::fixtures::logs_with_full_resource_and_scope;
    use prost::Message;

    static METADATA: CodecMetadata = CodecMetadata::new(PdataEncoding::OTLP, &[SignalType::Logs]);
    static REGISTRATIONS: [CodecRegistration; 2] = [
        CodecRegistration::new(&METADATA)
            .with_implementation("test.default")
            .with_default_for_encoding()
            .with_decoder(|_| Box::new(crate::builtins::OtlpBestEffortDecoder))
            .with_encoder(|_| panic!("matching forwarding must not create an encoder"))
            .with_item_counter(|_, _| Some(4)),
        CodecRegistration::new(&METADATA)
            .with_implementation("test.configured")
            .with_preparer(prepare_test)
            .with_decoder(|_| Box::new(crate::builtins::OtlpBestEffortDecoder))
            .with_encoder(|_| panic!("configured encoder must use the prepared factory"))
            .with_item_counter(|_, _| Some(4)),
    ];

    struct TestFactory {
        output: Bytes,
        forward: bool,
    }
    impl PreparedCodecFactory for TestFactory {
        fn create_encoder(
            &self,
            _: ResolvedCodec,
            _: EncodePolicy,
        ) -> Result<Box<dyn PdataEncoder>, CodecError> {
            Ok(Box::new(TestEncoder(self.output.clone())))
        }
        fn allows_encoded_forwarding(&self) -> bool {
            self.forward
        }
    }
    struct TestEncoder(Bytes);
    impl PdataEncoder for TestEncoder {
        fn encode(&mut self, _: OtapArrowRecords) -> Result<Bytes, CodecError> {
            Ok(self.0.clone())
        }
    }
    fn output_bytes(marker: &str) -> Bytes {
        let mut output = logs_with_full_resource_and_scope().encode_to_vec();
        // Unknown field 100 preserves valid OTLP while making rewritten output observable.
        output.extend_from_slice(&[0xa2, 0x06, u8::try_from(marker.len()).unwrap()]);
        output.extend_from_slice(marker.as_bytes());
        output.into()
    }

    fn prepare_test(
        config: &CodecConfiguration,
    ) -> Result<Arc<dyn PreparedCodecFactory>, CodecError> {
        for key in config.options.keys() {
            if !["output", "forward"].contains(&key.as_str()) {
                return Err(CodecError::configuration(format!("unknown option `{key}`")));
            }
        }
        let output = match config.options.get("output") {
            None => Bytes::new(),
            Some(value) => output_bytes(
                value
                    .as_str()
                    .ok_or_else(|| CodecError::configuration("output must be a string"))?,
            ),
        };
        let forward = match config.options.get("forward") {
            None => false,
            Some(value) => value
                .as_bool()
                .ok_or_else(|| CodecError::configuration("forward must be a boolean"))?,
        };
        Ok(Arc::new(TestFactory { output, forward }))
    }
    fn service(yaml: &str) -> Result<CodecService, CodecError> {
        let declaration: CodecsConfig = serde_yaml::from_str(yaml).unwrap();
        CodecServiceBuilder::from_registry(Arc::new(
            CodecRegistry::validate(&REGISTRATIONS).unwrap(),
        ))
        .with_configuration(CodecsConfig::resolve([&declaration]))
        .build()
    }

    fn finish_immediately<F: Future>(future: F) -> F::Output {
        let mut future = std::pin::pin!(future);
        let mut context = std::task::Context::from_waker(std::task::Waker::noop());
        match future.as_mut().poll(&mut context) {
            std::task::Poll::Ready(value) => value,
            std::task::Poll::Pending => panic!("test yields must complete immediately"),
        }
    }

    /// Scenario: A binding selects a prepared incremental decoder and a rewriting encoder for OTLP bytes.
    /// Guarantees: Materialization, native views, output preparation, and batch finishing honor the binding,
    /// release service access before yielding, and never fall back to synchronous decoding.
    #[test]
    fn cooperative_operations_use_the_selected_prepared_factory() {
        struct Factory;
        impl PreparedCodecFactory for Factory {
            fn create_decoder(
                &self,
                _: ResolvedCodec,
                policy: DecodePolicy,
            ) -> Result<Box<dyn PdataDecoder>, CodecError> {
                assert_eq!(policy.validation(), DecodeValidation::Strict);
                Ok(Box::new(Decoder))
            }
            fn create_encoder(
                &self,
                _: ResolvedCodec,
                _: EncodePolicy,
            ) -> Result<Box<dyn PdataEncoder>, CodecError> {
                Ok(Box::new(TestEncoder(output_bytes("rewritten"))))
            }
        }
        struct Decoder;
        impl PdataDecoder for Decoder {
            fn decode(&mut self, _: SignalType, _: &Bytes) -> Result<OtapArrowRecords, CodecError> {
                panic!("cooperative operations must call start_decode");
            }
            fn start_decode(
                &mut self,
                _: SignalType,
                bytes: &Bytes,
            ) -> Result<crate::DecodeTask, CodecError> {
                Ok(crate::DecodeTask::Incremental(Box::new(Job {
                    bytes: bytes.clone(),
                    pending: true,
                })))
            }
        }
        struct Job {
            bytes: Bytes,
            pending: bool,
        }
        impl crate::PdataDecodeJob for Job {
            fn step(
                &mut self,
                _: crate::DecodeBudget,
            ) -> Result<Option<OtapArrowRecords>, CodecError> {
                if std::mem::take(&mut self.pending) {
                    return Ok(None);
                }
                crate::builtins::OtlpBestEffortDecoder
                    .decode(SignalType::Logs, &self.bytes)
                    .map(Some)
            }
        }
        static CODECS: [CodecRegistration; 2] = [
            CodecRegistration::new(&METADATA)
                .with_implementation("test.admission")
                .with_default_for_encoding()
                .with_decoder(|_| panic!("the pipeline must use its selected implementation"))
                .with_item_counter(|_, _| Some(4)),
            CodecRegistration::new(&METADATA)
                .with_implementation("test.incremental")
                .with_preparer(|_| Ok(Arc::new(Factory)))
                .with_decoder(|_| panic!("the prepared factory must create the decoder"))
                .with_encoder(|_| panic!("the prepared factory must create the encoder"))
                .with_item_counter(|_, _| Some(4)),
        ];
        let registry = Arc::new(CodecRegistry::validate(&CODECS).unwrap());
        let admission = registry.resolve(&PdataEncoding::OTLP).unwrap();
        let config: CodecsConfig = serde_yaml::from_str("bindings: {otlp-bytes: {implementation: test.incremental, decode: {validation: strict}}}").unwrap();
        let service = CodecServiceBuilder::from_registry(registry)
            .with_configuration(CodecsConfig::resolve([&config]))
            .build()
            .unwrap();
        let selected = service.registry().resolve(&PdataEncoding::OTLP).unwrap();
        let plan = EncodingPlan::new(selected, EncodePolicy::default()).unwrap();
        let batch_plan = crate::BatchPlan::new(
            crate::PdataFormat::encoded(selected),
            crate::BatchSettings::otap(),
            false,
        )
        .unwrap();
        let bytes: Bytes = logs_with_full_resource_and_scope().encode_to_vec().into();
        let yields = std::cell::Cell::new(0);
        let yield_now = || {
            assert!(service.test_instance_count().unwrap() >= 1);
            yields.set(yields.get() + 1);
            std::future::ready(())
        };
        for codec in [admission, selected] {
            let payload =
                || PdataPayload::from(codec.admit(SignalType::Logs, bytes.clone()).unwrap());
            let records =
                finish_immediately(payload().try_into_otap_cooperative(&service, yield_now))
                    .unwrap();
            assert_eq!(records.num_items(), 4);
            let mut encoded = payload();
            let output = finish_immediately(encoded.with_encoded_output_cooperative(
                &service,
                &plan,
                |out| out.into_bytes(),
                yield_now,
            ))
            .unwrap();
            assert_eq!(output, output_bytes("rewritten"));
            assert_eq!(encoded.encoded_bytes().unwrap().as_ptr(), bytes.as_ptr());
            finish_immediately(batch_plan.finish_cooperative(&mut encoded, &service, yield_now))
                .unwrap();
            assert_eq!(encoded.encoded_bytes().unwrap(), &output_bytes("rewritten"));
            assert!(matches!(
                finish_immediately(payload().view_cooperative(
                    &service,
                    &AcceptedEncodings::native(),
                    yield_now
                ))
                .unwrap(),
                PdataView::Native(_)
            ));
        }
        assert_eq!(yields.get(), 8);
        assert_eq!(service.test_instance_count().unwrap(), 2);
    }

    /// Scenario: A named configuration selects an alternative implementation for compatible bytes.
    /// Guarantees: Admission, views, and matching forwarding preserve the buffer without creating codec instances.
    #[test]
    fn alternate_binding_preserves_compatible_forwarding() {
        let service = service("configurations:\n  selected:\n    implementation: test.configured\n    decode: {validation: strict}\n    options: {forward: true}\nbindings: {otlp-bytes: {configuration: selected}}").unwrap();
        let original = CodecRegistry::validate(&REGISTRATIONS)
            .unwrap()
            .resolve(&PdataEncoding::OTLP)
            .unwrap();
        let selected = service.registry().resolve(&PdataEncoding::OTLP).unwrap();
        assert_ne!(original, selected);
        assert_eq!(
            service.decode_policy_for(original).unwrap().validation(),
            DecodeValidation::Strict
        );
        let bytes: Bytes = logs_with_full_resource_and_scope().encode_to_vec().into();
        let encoded = original.admit(SignalType::Logs, bytes.clone()).unwrap();
        assert!(matches!(
            service
                .view(&encoded, &AcceptedEncodings::accept_encoded([selected]))
                .unwrap(),
            PdataView::Encoded(_)
        ));
        let mut payload = PdataPayload::from(encoded);
        let plan = EncodingPlan::new(selected, EncodePolicy::default()).unwrap();
        assert_eq!(
            payload.encode_bytes(&service, &plan).unwrap().as_ptr(),
            bytes.as_ptr()
        );
        assert_eq!(
            finish_immediately(payload.with_encoded_output_cooperative(
                &service,
                &plan,
                |output| output.into_bytes(),
                || std::future::ready(()),
            ))
            .unwrap()
            .as_ptr(),
            bytes.as_ptr()
        );
        assert_eq!(service.test_instance_count().unwrap(), 0);
    }

    /// Scenario: Prepared output options require rewriting compatible input, in two independent pipelines.
    /// Guarantees: Pass-through cannot bypass options; immutable settings are retained and lazy instances reused locally.
    #[test]
    fn prepared_factories_own_settings_and_keep_instances_isolated() {
        let bytes: Bytes = logs_with_full_resource_and_scope().encode_to_vec().into();
        let original = CodecRegistry::validate(&REGISTRATIONS)
            .unwrap()
            .resolve(&PdataEncoding::OTLP)
            .unwrap();
        let first = service(
            "bindings: {otlp-bytes: {implementation: test.configured, options: {output: first}}}",
        )
        .unwrap();
        let second = service(
            "bindings: {otlp-bytes: {implementation: test.configured, options: {output: second}}}",
        )
        .unwrap();
        assert!(!first.shares_state_with(&second));
        for (service, expected) in [
            (&first, output_bytes("first")),
            (&second, output_bytes("second")),
        ] {
            let plan = EncodingPlan::new(original, EncodePolicy::default()).unwrap();
            for _ in 0..2 {
                let mut payload =
                    PdataPayload::from(original.admit(SignalType::Logs, bytes.clone()).unwrap());
                assert_eq!(
                    payload.encode_bytes(service, &plan).unwrap().as_ref(),
                    expected.as_ref()
                );
                assert_eq!(service.test_instance_count().unwrap(), 2);
            }
        }
    }

    /// Scenario: No bindings are declared, or only OTLP is customized while another encoding is linked.
    /// Guarantees: Discovery is not an allowlist and per-encoding strictness does not alter other decoders.
    #[test]
    fn defaults_and_independent_bindings_remain_lazy() {
        static OTHER: CodecMetadata =
            CodecMetadata::new(PdataEncoding::new("other-v1"), &[SignalType::Logs]);
        static CODECS: [CodecRegistration; 2] = [
            CodecRegistration::new(&METADATA)
                .with_implementation("test.otlp")
                .with_decoder(|_| Box::new(crate::builtins::OtlpBestEffortDecoder))
                .with_item_counter(|_, _| Some(4)),
            CodecRegistration::new(&OTHER)
                .with_decoder(|_| panic!("unused codec must stay lazy"))
                .with_item_counter(|_, _| Some(0)),
        ];
        for yaml in [
            "{}",
            "bindings: {otlp-bytes: {decode: {validation: strict}}}",
        ] {
            let declaration: CodecsConfig = serde_yaml::from_str(yaml).unwrap();
            let service = CodecServiceBuilder::from_registry(Arc::new(
                CodecRegistry::validate(&CODECS).unwrap(),
            ))
            .with_configuration(CodecsConfig::resolve([&declaration]))
            .build()
            .unwrap();
            let other = service.registry().resolve(OTHER.encoding()).unwrap();
            assert_eq!(
                service.decode_policy_for(other).unwrap().validation(),
                DecodeValidation::BestEffort
            );
            assert_eq!(service.test_instance_count().unwrap(), 0);
        }
    }

    /// Scenario: Bindings reference missing or incompatible implementations, invalid options, or unused invalid definitions.
    /// Guarantees: Startup fails before mutable state exists and diagnostics do not expose option values.
    #[test]
    fn invalid_settings_fail_at_startup() {
        for yaml in [
            "bindings: {absent: {}}",
            "bindings: {otlp-bytes: {implementation: absent}}",
            "bindings: {wrong-v1: {implementation: test.configured}}",
            "bindings: {otlp-bytes: {configuration: absent}}",
            "bindings: {otlp-bytes: {options: {unknown: SECRET_VALUE}}}",
            "bindings: {otlp-bytes: {implementation: test.configured, options: {forward: SECRET_VALUE}}}",
            "configurations: {unused: {implementation: absent}}",
            "configurations: {unused: {decode: {validation: strict}}}",
        ] {
            let error = service(yaml).err().expect("invalid configuration");
            assert!(!error.to_string().contains("SECRET_VALUE"));
        }
    }

    /// Scenario: An encode-only registration receives an explicit decoding setting.
    /// Guarantees: Startup rejects the unsupported operation instead of ignoring its configuration.
    #[test]
    fn rejects_decode_settings_for_encode_only_implementation() {
        static OUTPUT: [CodecRegistration; 1] = [CodecRegistration::new(&METADATA)
            .with_implementation("output.only")
            .with_encoder(|_| panic!("startup must remain lazy"))];
        let declaration: CodecsConfig =
            serde_yaml::from_str("bindings: {otlp-bytes: {decode: {validation: strict}}}").unwrap();
        let result =
            CodecServiceBuilder::from_registry(Arc::new(CodecRegistry::validate(&OUTPUT).unwrap()))
                .with_configuration(CodecsConfig::resolve([&declaration]))
                .build();
        assert!(
            result
                .err()
                .unwrap()
                .to_string()
                .contains("require a decoder")
        );
    }

    /// Scenario: Configured strict OTLP decoding sees malformed nested content, then valid input.
    /// Guarantees: Named settings affect actual decoding, preserve recoverable bytes and counts, and recover after failure.
    #[test]
    fn configured_strict_decoding_recovers_original_input() {
        let declaration: CodecsConfig = serde_yaml::from_str("configurations: {strict: {implementation: otel.otlp, decode: {validation: strict}}}\nbindings: {otlp-bytes: {configuration: strict}}").unwrap();
        let service = CodecServiceBuilder::from_global_registry()
            .unwrap()
            .with_configuration(CodecsConfig::resolve([&declaration]))
            .build()
            .unwrap();
        let codec = service.registry().resolve(&PdataEncoding::OTLP).unwrap();
        let valid: Bytes = logs_with_full_resource_and_scope().encode_to_vec().into();
        let mut malformed = valid.to_vec();
        malformed.extend_from_slice(&[0x0a, 0x03, 0x1a, 0x05, 0x00]);
        let malformed = Bytes::from(malformed);
        for _ in 0..2 {
            let mut payload =
                PdataPayload::from(codec.admit(SignalType::Logs, malformed.clone()).unwrap());
            let count = payload.num_items();
            let error = finish_immediately(
                payload.try_into_otap_cooperative(&service, || std::future::ready(())),
            )
            .unwrap_err();
            let (_, recovered) = error.into_parts();
            assert_eq!(
                recovered.encoded_bytes().unwrap().as_ptr(),
                malformed.as_ptr()
            );
            assert_eq!(recovered.known_item_count(), Some(count));
            assert_eq!(
                service
                    .decode(&codec.admit(SignalType::Logs, valid.clone()).unwrap())
                    .unwrap()
                    .num_items(),
                4
            );
        }
        assert_eq!(service.test_instance_count().unwrap(), 1);
    }
}
