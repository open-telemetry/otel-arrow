// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Synthetic syslog and CEF profiles sharing the receiver's encoded batch construction.

use chrono::{DateTime, Datelike, SecondsFormat, Utc};
use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_pdata_codec::{
    CodecGeneratorRegistration, GenerationConfig, GenerationError, GenerationProfile, OtapPayload,
    PdataGenerator, ResolvedCodec,
};
use rand::{RngExt, SeedableRng, rngs::SmallRng};
use std::net::Ipv4Addr;

use super::codec::{
    BATCH_HEADER_LEN, FRAME_HEADER_LEN, MAX_BATCH_BYTES, MAX_BATCH_ITEMS, RUN_HEADER_LEN,
    SyslogBatchBuilder,
};
use super::parser::parsed_message::{
    FORMAT_CEF, FORMAT_CEF_RFC3164, FORMAT_CEF_RFC5424, FORMAT_RFC3164, FORMAT_RFC5424,
};

pub(super) const REGISTRATION: CodecGeneratorRegistration =
    CodecGeneratorRegistration::new(PROFILES, create_generator);

const PROFILES: &[GenerationProfile] = &[
    GenerationProfile {
        name: FORMAT_RFC3164,
        signals: &[SignalType::Logs],
    },
    GenerationProfile {
        name: FORMAT_RFC5424,
        signals: &[SignalType::Logs],
    },
    GenerationProfile {
        name: FORMAT_CEF,
        signals: &[SignalType::Logs],
    },
    GenerationProfile {
        name: FORMAT_CEF_RFC3164,
        signals: &[SignalType::Logs],
    },
    GenerationProfile {
        name: FORMAT_CEF_RFC5424,
        signals: &[SignalType::Logs],
    },
];

const SUFFIX: &str = " request completed";

struct EventFields {
    priority: u8,
    host: u8,
    pid: u16,
    src: Ipv4Addr,
    dst: Ipv4Addr,
    spt: u16,
    dpt: u16,
    user: u16,
    session: u64,
    severity: u8,
}

impl EventFields {
    const MAX: Self = Self {
        priority: 150,
        host: 15,
        pid: 9999,
        src: Ipv4Addr::new(192, 0, 2, 254),
        dst: Ipv4Addr::new(198, 51, 100, 254),
        spt: 65535,
        dpt: 65535,
        user: 9999,
        session: u64::MAX,
        severity: 10,
    };

    fn random(rng: &mut SmallRng) -> Self {
        const PRIORITIES: [u8; 8] = [134, 131, 132, 133, 142, 139, 150, 147];
        Self {
            priority: PRIORITIES[rng.random_range(0..PRIORITIES.len())],
            host: rng.random_range(0..=15),
            pid: rng.random_range(1000..=9999),
            src: Ipv4Addr::new(192, 0, 2, rng.random_range(1..=254)),
            dst: Ipv4Addr::new(198, 51, 100, rng.random_range(1..=254)),
            spt: rng.random_range(1024..=65535),
            dpt: rng.random_range(1..=65535),
            user: rng.random_range(1000..=9999),
            session: rng.random(),
            severity: rng.random_range(1..=10),
        }
    }
}

struct SyslogGenerator {
    codec: ResolvedCodec,
    profile: String,
    sequence: u64,
    rng: SmallRng,
    max_batch_size: usize,
}

fn format_timestamp(profile: &str, timestamp: DateTime<Utc>) -> Result<String, GenerationError> {
    match profile {
        FORMAT_RFC3164 | FORMAT_CEF_RFC3164 => Ok(timestamp.format("%b %e %T").to_string()),
        FORMAT_RFC5424 | FORMAT_CEF_RFC5424 => {
            Ok(timestamp.to_rfc3339_opts(SecondsFormat::Micros, true))
        }
        FORMAT_CEF => Ok(String::new()),
        profile => Err(GenerationError::Configuration(format!(
            "unsupported syslog profile `{profile}`"
        ))),
    }
}

fn message(
    profile: &str,
    timestamp: &str,
    sequence: u64,
    event: &EventFields,
) -> Result<String, GenerationError> {
    let mut message = match profile {
        FORMAT_RFC3164 | FORMAT_CEF_RFC3164 => format!(
            "<{}>{timestamp} trafficgen-{:02} app[{}]: ",
            event.priority, event.host, event.pid
        ),
        FORMAT_RFC5424 | FORMAT_CEF_RFC5424 => format!(
            "<{}>1 {timestamp} trafficgen-{:02} app {} ID47 - ",
            event.priority, event.host, event.pid
        ),
        FORMAT_CEF => String::new(),
        profile => {
            return Err(GenerationError::Configuration(format!(
                "unsupported syslog profile `{profile}`"
            )));
        }
    };
    let fields = format!(
        "src={} dst={} spt={} dpt={} proto=TCP suser=user{} deviceExternalId={:016x}",
        event.src, event.dst, event.spt, event.dpt, event.user, event.session
    );
    if matches!(
        profile,
        FORMAT_CEF | FORMAT_CEF_RFC3164 | FORMAT_CEF_RFC5424
    ) {
        message.push_str(&format!(
            "CEF:0|trafficgen|synthetic|1.0|100|request completed|{}|{fields} msg=event\\={sequence}{SUFFIX}",
            event.severity
        ));
    } else {
        message.push_str(&format!("event={sequence} {fields}{SUFFIX}"));
    }
    Ok(message)
}

fn create_generator(
    codec: ResolvedCodec,
    config: &GenerationConfig,
) -> Result<Box<dyn PdataGenerator>, GenerationError> {
    let timestamp = format_timestamp(&config.profile, DateTime::from_timestamp_nanos(0))?;
    if config.max_batch_size > MAX_BATCH_ITEMS {
        return Err(GenerationError::Configuration(format!(
            "syslog max_batch_size must not exceed {MAX_BATCH_ITEMS}"
        )));
    }
    // Reserve the widest fields and any u64 sequence, including after counter wrap.
    // Nonnegative i64 nanoseconds always have four-digit years and fixed-width headers.
    let max_message_bytes =
        message(&config.profile, &timestamp, u64::MAX, &EventFields::MAX)?.len();
    let max_bytes = BATCH_HEADER_LEN
        + RUN_HEADER_LEN
        + config.max_batch_size * (FRAME_HEADER_LEN + max_message_bytes);
    let limit = config.max_batch_bytes.min(MAX_BATCH_BYTES);
    if max_bytes > limit {
        return Err(GenerationError::Configuration(format!(
            "syslog batch may require {max_bytes} bytes, exceeding limit {limit}; reduce max_batch_size"
        )));
    }

    Ok(Box::new(SyslogGenerator {
        codec,
        profile: config.profile.clone(),
        sequence: 0,
        rng: SmallRng::seed_from_u64(config.seed),
        max_batch_size: config.max_batch_size,
    }))
}

impl PdataGenerator for SyslogGenerator {
    fn generate(
        &mut self,
        count: usize,
        timestamp_unix_nano: i64,
    ) -> Result<OtapPayload, GenerationError> {
        if count == 0 || count > self.max_batch_size {
            return Err(GenerationError::Configuration(
                "syslog item count exceeds the configured batch bounds".into(),
            ));
        }
        let timestamp = DateTime::from_timestamp_nanos(timestamp_unix_nano);
        let header_time = format_timestamp(&self.profile, timestamp)?;
        let mut builder = SyslogBatchBuilder::with_max_items(count);
        for _ in 0..count {
            let message = message(
                &self.profile,
                &header_time,
                self.sequence,
                &EventFields::random(&mut self.rng),
            )?;
            builder
                .append_with_year(message.as_bytes(), timestamp.year())
                .map_err(|error| GenerationError::Output(error.to_string()))?;
            self.sequence = self.sequence.wrapping_add(1);
        }
        let batch = builder
            .finish(timestamp_unix_nano)
            .map_err(|error| GenerationError::Output(error.to_string()))?;
        Ok(self.codec.admit_built(batch)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::AnyValue;
    use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::{LogRecord, LogsData};
    use otel_arrow_dfe_pdata::{OtlpProtoBytes, TryIntoWithOptions};
    use otel_arrow_dfe_pdata_codec::CodecService;
    use prost::Message;

    const TIMESTAMP: i64 = 1_704_067_200_123_456_789;

    fn config(profile: &str) -> GenerationConfig {
        GenerationConfig {
            profile: profile.into(),
            signal: SignalType::Logs,
            seed: 42,
            max_batch_size: 10,
            max_batch_bytes: MAX_BATCH_BYTES,
        }
    }

    fn decode_logs(payload: OtapPayload, service: &CodecService) -> LogsData {
        let otlp: OtlpProtoBytes = payload
            .try_into_otap(service)
            .unwrap()
            .try_into_with_default()
            .unwrap();
        let OtlpProtoBytes::ExportLogsRequest(bytes) = otlp else {
            panic!("expected logs");
        };
        LogsData::decode(bytes).unwrap()
    }

    fn attribute<'a>(record: &'a LogRecord, key: &str) -> &'a AnyValue {
        record
            .attributes
            .iter()
            .find(|attribute| attribute.key == key)
            .expect("generated log attribute")
            .value
            .as_ref()
            .unwrap()
    }

    /// Scenario: Each syslog and CEF profile generates repeatable batches with advancing contents.
    /// Guarantees: Fixed seeds reproduce varied fields, different seeds change contents, and real decoding preserves counts, timestamps, formats, and escaped CEF values.
    #[test]
    fn syslog_generation_profiles_decode_reproducibly() {
        let codec = super::super::codec::resolve_syslog().unwrap();
        let service = CodecService::new().unwrap();
        for profile in ["rfc3164", "rfc5424", "cef", "cef_rfc3164", "cef_rfc5424"] {
            let settings = config(profile);
            let mut first = codec.create_generator(&settings).unwrap();
            let mut second = codec.create_generator(&settings).unwrap();
            let mut payload = first.generate(3, TIMESTAMP).unwrap();
            assert_eq!(payload.num_items(), 3);
            assert_eq!(
                payload.encoded_bytes(),
                second.generate(3, TIMESTAMP).unwrap().encoded_bytes()
            );
            assert_ne!(
                payload.encoded_bytes(),
                first.generate(3, TIMESTAMP).unwrap().encoded_bytes()
            );
            let mut other_settings = settings.clone();
            other_settings.seed += 1;
            assert_ne!(
                payload.encoded_bytes(),
                codec
                    .create_generator(&other_settings)
                    .unwrap()
                    .generate(3, TIMESTAMP)
                    .unwrap()
                    .encoded_bytes()
            );
            let logs = decode_logs(payload, &service);
            let records: Vec<_> = logs
                .resource_logs
                .iter()
                .flat_map(|resource| &resource.scope_logs)
                .flat_map(|scope| &scope.log_records)
                .collect();
            assert_eq!(records.len(), 3);
            let mut expected_rng = SmallRng::seed_from_u64(settings.seed);
            for (index, record) in records.into_iter().enumerate() {
                let expected = EventFields::random(&mut expected_rng);
                assert_eq!(record.observed_time_unix_nano, TIMESTAMP as u64);
                if matches!(profile, "rfc5424" | "cef_rfc5424") {
                    assert_eq!(record.time_unix_nano, TIMESTAMP as u64 / 1000 * 1000);
                }
                assert_eq!(
                    attribute(record, "input.format"),
                    &AnyValue::new_string(profile)
                );
                assert!(record.body.is_none());
                if profile.starts_with("cef") {
                    for (key, value) in [
                        ("cef.device_vendor", "trafficgen"),
                        ("cef.device_product", "synthetic"),
                        ("cef.device_version", "1.0"),
                        ("cef.device_event_class_id", "100"),
                        ("cef.name", "request completed"),
                        ("proto", "TCP"),
                    ] {
                        assert_eq!(attribute(record, key), &AnyValue::new_string(value));
                    }
                    for (key, value) in [
                        ("cef.severity", expected.severity.to_string()),
                        ("src", expected.src.to_string()),
                        ("dst", expected.dst.to_string()),
                        ("spt", expected.spt.to_string()),
                        ("dpt", expected.dpt.to_string()),
                        ("suser", format!("user{}", expected.user)),
                        ("deviceExternalId", format!("{:016x}", expected.session)),
                    ] {
                        assert_eq!(attribute(record, key), &AnyValue::new_string(value));
                    }
                    assert_eq!(attribute(record, "cef.version"), &AnyValue::new_int(0));
                    assert_eq!(
                        attribute(record, "msg"),
                        &AnyValue::new_string(format!("event={index} request completed"))
                    );
                    if profile == "cef" {
                        assert_eq!(record.time_unix_nano, 0);
                    }
                }
            }
        }
    }

    /// Scenario: A codec receives different batch timestamps across calendar years without a fixed override.
    /// Guarantees: All profiles refresh observed time, syslog headers and receipt years follow the batch time, and raw CEF retains zero event time.
    #[test]
    fn syslog_generation_uses_each_batch_timestamp() {
        let codec = super::super::codec::resolve_syslog().unwrap();
        let service = CodecService::new().unwrap();
        for profile in ["rfc3164", "rfc5424", "cef", "cef_rfc3164", "cef_rfc5424"] {
            let settings = config(profile);
            let mut generator = create_generator(codec, &settings).unwrap();
            for year in [2024, 2025] {
                let timestamp = DateTime::parse_from_rfc3339(&format!("{year}-06-15T12:00:00Z"))
                    .unwrap()
                    .timestamp_nanos_opt()
                    .unwrap();
                let logs = decode_logs(generator.generate(1, timestamp).unwrap(), &service);
                let record = &logs.resource_logs[0].scope_logs[0].log_records[0];
                assert_eq!(record.observed_time_unix_nano, timestamp as u64);
                match profile {
                    "cef" => assert_eq!(record.time_unix_nano, 0),
                    "rfc5424" | "cef_rfc5424" => {
                        assert_eq!(record.time_unix_nano, timestamp as u64);
                    }
                    _ => assert_eq!(
                        DateTime::from_timestamp_nanos(record.time_unix_nano as i64).year(),
                        year
                    ),
                }
            }
        }
    }

    /// Scenario: Syslog generation settings exceed item, byte, signal, or profile limits.
    /// Guarantees: Unsupported requests fail during factory construction rather than allocating oversized batches or generating another format.
    #[test]
    fn syslog_generation_rejects_unsupported_settings() {
        let codec = super::super::codec::resolve_syslog().unwrap();
        let mut settings = config("rfc5424");
        settings.max_batch_size = MAX_BATCH_ITEMS + 1;
        assert!(codec.create_generator(&settings).is_err());
        settings = config("rfc5424");
        settings.max_batch_bytes = 1;
        assert!(codec.create_generator(&settings).is_err());
        settings = config("missing");
        assert!(codec.create_generator(&settings).is_err());
        settings = config("rfc5424");
        settings.signal = SignalType::Metrics;
        assert!(codec.create_generator(&settings).is_err());
    }

    /// Scenario: All profiles generate variable-width fields under a worst-case byte budget.
    /// Guarantees: The widest possible record fits exactly, seeded batches stay within that bound, and a one-byte-smaller budget is rejected at startup.
    #[test]
    fn syslog_generation_honors_worst_case_byte_limits() {
        let codec = super::super::codec::resolve_syslog().unwrap();
        let service = CodecService::new().unwrap();
        for profile in ["rfc3164", "rfc5424", "cef", "cef_rfc3164", "cef_rfc5424"] {
            let mut settings = config(profile);
            settings.max_batch_size = 1;
            let timestamp = DateTime::from_timestamp_nanos(i64::MAX);
            let header_time = format_timestamp(profile, timestamp).unwrap();
            let largest = message(profile, &header_time, u64::MAX, &EventFields::MAX).unwrap();
            settings.max_batch_bytes =
                BATCH_HEADER_LEN + RUN_HEADER_LEN + FRAME_HEADER_LEN + largest.len();
            let mut builder = SyslogBatchBuilder::with_max_items(1);
            builder
                .append_with_year(largest.as_bytes(), timestamp.year())
                .unwrap();
            let payload = codec
                .admit_built(builder.finish(i64::MAX).unwrap())
                .unwrap();
            assert_eq!(
                payload.encoded_bytes().unwrap().len(),
                settings.max_batch_bytes
            );
            assert_eq!(
                decode_logs(payload, &service).resource_logs[0].scope_logs[0]
                    .log_records
                    .len(),
                1
            );
            for seed in [0, 42, u64::MAX] {
                settings.seed = seed;
                let mut generator = codec.create_generator(&settings).unwrap();
                for _ in 0..10 {
                    let payload = generator.generate(1, i64::MAX).unwrap();
                    assert!(payload.encoded_bytes().unwrap().len() <= settings.max_batch_bytes);
                }
            }
            settings.max_batch_bytes -= 1;
            assert!(matches!(
                codec.create_generator(&settings),
                Err(GenerationError::Configuration(_))
            ));
        }
    }
}
