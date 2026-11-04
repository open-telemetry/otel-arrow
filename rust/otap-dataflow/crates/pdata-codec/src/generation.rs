// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Codec-owned construction of synthetic encoded payloads.

use otel_arrow_dfe_config::SignalType;

use crate::{CodecError, OtapPayload, RegistryError, ResolvedCodec};

/// A discoverable synthetic input shape supported by a codec.
#[derive(Debug)]
pub struct GenerationProfile {
    /// Stable profile name used in trafficgen configuration.
    pub name: &'static str,
    /// Primary signals that this profile can generate.
    pub signals: &'static [SignalType],
}

/// Fixed inputs for one pipeline-local synthetic generator.
#[derive(Clone, Debug)]
pub struct GenerationConfig {
    /// Codec-specific profile name.
    pub profile: String,
    /// Primary signal to generate.
    pub signal: SignalType,
    /// Reproducible starting point for generated contents.
    pub seed: u64,
    /// Maximum requested primary-signal items per payload.
    pub max_batch_size: usize,
    /// Maximum encoded bytes per payload.
    pub max_batch_bytes: usize,
}

/// Invalid generation settings, codec failures, or generator contract violations.
#[derive(Debug, thiserror::Error)]
pub enum GenerationError {
    /// Settings are unsupported or cannot produce a bounded payload.
    #[error("invalid codec generation configuration: {0}")]
    Configuration(String),
    /// The generator did not honor its declared output contract.
    #[error("invalid generated payload: {0}")]
    Output(String),
    /// Codec registration or batch admission failed.
    #[error(transparent)]
    Codec(#[from] CodecError),
    /// Codec discovery failed.
    #[error(transparent)]
    Registry(#[from] RegistryError),
}

/// Creates independent synthetic generator state without decoding or network I/O.
pub type GeneratorFactory =
    fn(ResolvedCodec, &GenerationConfig) -> Result<Box<dyn PdataGenerator>, GenerationError>;

/// Optional synthetic payload generation capability.
#[derive(Debug)]
pub struct CodecGeneratorRegistration {
    profiles: &'static [GenerationProfile],
    create: GeneratorFactory,
}

impl CodecGeneratorRegistration {
    /// Declares supported profiles and the factory that validates their settings.
    #[must_use]
    pub const fn new(profiles: &'static [GenerationProfile], create: GeneratorFactory) -> Self {
        Self { profiles, create }
    }

    /// Returns the codec's available synthetic input profiles.
    #[must_use]
    pub const fn profiles(&self) -> &'static [GenerationProfile] {
        self.profiles
    }
}

/// Codec-owned synthetic state; deliberately has no `Send` requirement.
pub trait PdataGenerator {
    /// Produces exactly `count` items as a complete, independently decodable payload.
    ///
    /// Implementations must validate allocation bounds before constructing output,
    /// reuse codec framing/builders, and retain only bounded pipeline-local state.
    /// The wrapper supplies a nonnegative timestamp; implementations must not read the wall clock.
    fn generate(
        &mut self,
        count: usize,
        timestamp_unix_nano: i64,
    ) -> Result<OtapPayload, GenerationError>;
}

/// Pipeline-local generator enforcing the encoding, signal, count, and size contract.
pub struct CodecGenerator {
    codec: ResolvedCodec,
    signal: SignalType,
    max_batch_size: usize,
    max_batch_bytes: usize,
    generator: Box<dyn PdataGenerator>,
}

impl CodecGenerator {
    pub(crate) fn new(
        codec: ResolvedCodec,
        registration: &CodecGeneratorRegistration,
        config: &GenerationConfig,
    ) -> Result<Self, GenerationError> {
        if config.max_batch_size == 0 || config.max_batch_bytes == 0 {
            return Err(GenerationError::Configuration(
                "max_batch_size and max_batch_bytes must be positive".into(),
            ));
        }
        codec.require_decoder(config.signal)?;
        if !registration.profiles.iter().any(|profile| {
            profile.name == config.profile && profile.signals.contains(&config.signal)
        }) {
            return Err(GenerationError::Configuration(format!(
                "codec `{}` has no profile `{}` for {:?}",
                codec.encoding(),
                config.profile,
                config.signal
            )));
        }
        Ok(Self {
            codec,
            signal: config.signal,
            max_batch_size: config.max_batch_size,
            max_batch_bytes: config.max_batch_bytes,
            generator: (registration.create)(codec, config)?,
        })
    }

    /// Generates a bounded payload and rejects violations without logging its contents.
    pub fn generate(
        &mut self,
        count: usize,
        timestamp_unix_nano: i64,
    ) -> Result<OtapPayload, GenerationError> {
        if count == 0 || count > self.max_batch_size {
            return Err(GenerationError::Configuration(format!(
                "requested item count {count} must be in 1..={}",
                self.max_batch_size
            )));
        }
        if timestamp_unix_nano < 0 {
            return Err(GenerationError::Configuration(
                "timestamp_unix_nano must not be negative".into(),
            ));
        }
        let mut payload = self.generator.generate(count, timestamp_unix_nano)?;
        let view = payload.encoded_view().ok_or_else(|| {
            GenerationError::Output("expected encoded pdata, got native OTAP".into())
        })?;
        if view.encoding() != self.codec.encoding() || view.signal_type() != self.signal {
            return Err(GenerationError::Output(
                "encoding or signal differs from the generation configuration".into(),
            ));
        }
        if view.bytes().len() > self.max_batch_bytes {
            return Err(GenerationError::Output(
                "encoded output exceeds max_batch_bytes".into(),
            ));
        }
        let actual_count = payload.num_items();
        if actual_count != count {
            return Err(GenerationError::Output(format!(
                "expected {count} items, got {actual_count}"
            )));
        }
        Ok(payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CodecMetadata, CodecRegistration, CodecRegistry, PdataEncoding};
    use bytes::Bytes;

    static METADATA: CodecMetadata = CodecMetadata::new(
        PdataEncoding::new("generation-fixture"),
        &[SignalType::Logs, SignalType::Metrics],
    );
    static REGISTRATIONS: [CodecRegistration; 1] = [CodecRegistration::new(&METADATA)
        .with_decoder(|_| Box::new(crate::builtins::OtlpBestEffortDecoder))
        .with_item_counter(|_, bytes| Some(bytes.len()))
        .with_generator(CodecGeneratorRegistration::new(
            &[
                GenerationProfile {
                    name: "valid",
                    signals: &[SignalType::Logs],
                },
                GenerationProfile {
                    name: "wrong_count",
                    signals: &[SignalType::Logs],
                },
                GenerationProfile {
                    name: "wrong_signal",
                    signals: &[SignalType::Logs],
                },
                GenerationProfile {
                    name: "wrong_encoding",
                    signals: &[SignalType::Logs],
                },
            ],
            |codec, config| {
                Ok(Box::new(FixtureGenerator {
                    codec,
                    profile: config.profile.clone(),
                }))
            },
        ))];

    struct FixtureGenerator {
        codec: ResolvedCodec,
        profile: String,
    }

    impl PdataGenerator for FixtureGenerator {
        fn generate(
            &mut self,
            count: usize,
            timestamp_unix_nano: i64,
        ) -> Result<OtapPayload, GenerationError> {
            let signal = if self.profile == "wrong_signal" {
                SignalType::Metrics
            } else {
                SignalType::Logs
            };
            let count = count + usize::from(self.profile == "wrong_count");
            let codec = if self.profile == "wrong_encoding" {
                crate::builtins::resolve_otlp()?
            } else {
                self.codec
            };
            let bytes: Vec<_> = timestamp_unix_nano
                .to_le_bytes()
                .into_iter()
                .cycle()
                .take(count)
                .collect();
            Ok(codec.admit(signal, Bytes::from(bytes))?.into())
        }
    }

    fn config(profile: &str) -> GenerationConfig {
        GenerationConfig {
            profile: profile.into(),
            signal: SignalType::Logs,
            seed: 0,
            max_batch_size: 4,
            max_batch_bytes: 4,
        }
    }

    fn codec() -> ResolvedCodec {
        CodecRegistry::validate(&REGISTRATIONS)
            .unwrap()
            .resolve(METADATA.encoding())
            .unwrap()
    }

    /// Scenario: A codec omits generation or a caller requests an unavailable profile or zero limit.
    /// Guarantees: Optional generation does not affect ordinary codecs and invalid generation settings fail explicitly.
    #[test]
    fn codec_generation_rejects_unsupported_configuration() {
        assert!(
            crate::builtins::resolve_otlp()
                .unwrap()
                .create_generator(&config("valid"))
                .is_err()
        );
        assert!(codec().create_generator(&config("missing")).is_err());
        let mut settings = config("valid");
        settings.max_batch_size = 0;
        assert!(codec().create_generator(&settings).is_err());
        settings = config("valid");
        settings.max_batch_bytes = 0;
        assert!(codec().create_generator(&settings).is_err());
    }

    /// Scenario: A caller supplies different timestamps for successive generation requests.
    /// Guarantees: The shared wrapper forwards each batch timestamp without retaining or replacing it.
    #[test]
    fn codec_generation_forwards_batch_timestamps() {
        let mut generator = codec().create_generator(&config("valid")).unwrap();
        for timestamp in [42_i64, 43] {
            let payload = generator.generate(4, timestamp).unwrap();
            assert_eq!(
                payload.encoded_bytes().unwrap().as_ref(),
                &timestamp.to_le_bytes()[..4]
            );
        }
    }

    /// Scenario: A generator returns the wrong item count, signal, or excessive encoded bytes.
    /// Guarantees: The shared wrapper rejects invalid output instead of injecting misleading payloads into trafficgen.
    #[test]
    fn codec_generation_enforces_output_contract() {
        for profile in ["wrong_count", "wrong_signal", "wrong_encoding"] {
            let mut generator = codec().create_generator(&config(profile)).unwrap();
            assert!(matches!(
                generator.generate(2, 0),
                Err(GenerationError::Output(_))
            ));
        }
        let mut settings = config("valid");
        settings.max_batch_bytes = 1;
        let mut generator = codec().create_generator(&settings).unwrap();
        assert!(matches!(
            generator.generate(2, 0),
            Err(GenerationError::Output(_))
        ));
    }

    /// Scenario: A valid generator receives batches within and outside its item and timestamp bounds.
    /// Guarantees: Valid batches retain counts, while zero and oversized requests fail before invoking the implementation.
    #[test]
    fn codec_generation_enforces_request_bounds() {
        let mut generator = codec().create_generator(&config("valid")).unwrap();
        assert_eq!(generator.generate(4, 0).unwrap().num_items(), 4);
        assert!(matches!(
            generator.generate(0, 0),
            Err(GenerationError::Configuration(_))
        ));
        assert!(matches!(
            generator.generate(5, 0),
            Err(GenerationError::Configuration(_))
        ));
        assert!(matches!(
            generator.generate(4, -1),
            Err(GenerationError::Configuration(_))
        ));
    }
}
