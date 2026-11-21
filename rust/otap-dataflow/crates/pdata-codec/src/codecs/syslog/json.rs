// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Streaming JSON selection followed by the shared syslog/CEF Arrow append path.

use std::{borrow::Cow, sync::Arc, time::Instant};

use crate::{
    CodecError, CodecMetadata, CodecOperation, CodecRegistration, DecodeBudget, DecodeContext,
    DecodePolicy, DecodeTask, PdataDecodeJob, PdataDecoder, PdataEncoding, PreparedCodecFactory,
    ResolvedCodec, register_pdata_codec,
    selector::{CompiledSelector, SelectorStep},
};
use bytes::Bytes;
use chrono::Utc;
use otel_arrow_dfe_config::{SignalType, codecs::CodecConfiguration};
use otel_arrow_dfe_pdata::OtapArrowRecords;
use serde::{Deserialize, de::IgnoredAny};

use super::{
    MAX_MESSAGE_SIZE,
    arrow_records_encoder::ReceivedArrowRecordsBuilder,
    codec::{MAX_BATCH_BYTES, MAX_BATCH_ITEMS, append_syslog_message},
};

/// Stable encoding identity for JSON-selected syslog and CEF messages.
pub const SYSLOG_JSON_ENCODING: PdataEncoding = PdataEncoding::new("syslog-json");
/// Hard admission limit for one independently decodable JSON document.
pub const MAX_JSON_INPUT_BYTES: usize = MAX_BATCH_BYTES;
const MAX_DEPTH: usize = 32;
const MAX_TOKEN_BYTES: usize = MAX_MESSAGE_SIZE * 6 + 2;
const MAX_SYNC_BYTES: usize = 16 * 1024;

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
struct JsonError(&'static str);

fn decode_error(reason: &'static str) -> CodecError {
    CodecError::operation(
        &SYSLOG_JSON_ENCODING,
        CodecOperation::Decode,
        JsonError(reason),
    )
}

#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Options {
    records: String,
    record_decoder: RecordDecoder,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            records: "$".to_owned(),
            record_decoder: RecordDecoder::default(),
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RecordDecoder {
    #[serde(rename = "type")]
    kind: DecoderKind,
    from: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum DecoderKind {
    Syslog,
}

impl Default for RecordDecoder {
    fn default() -> Self {
        Self {
            kind: DecoderKind::Syslog,
            from: "$.message".to_owned(),
        }
    }
}

struct Plan {
    steps: Box<[SelectorStep]>,
    record_boundary: usize,
}

fn prepare(config: &CodecConfiguration) -> Result<Arc<dyn PreparedCodecFactory>, CodecError> {
    let options: Options = serde_json::from_value(serde_json::Value::Object(
        config.options.clone().into_iter().collect(),
    ))
    .map_err(|_| CodecError::configuration("invalid json codec options"))?;
    let records = CompiledSelector::compile(&options.records)
        .map_err(|error| CodecError::configuration(format!("records: {error}")))?;
    let from = CompiledSelector::compile(&options.record_decoder.from)
        .map_err(|error| CodecError::configuration(format!("record_decoder.from: {error}")))?;
    let DecoderKind::Syslog = options.record_decoder.kind;
    if from.steps().contains(&SelectorStep::Each) {
        return Err(CodecError::configuration(
            "record_decoder.from must select one string, without array iteration",
        ));
    }
    if records.steps().len() + from.steps().len() > MAX_DEPTH {
        return Err(CodecError::configuration(
            "combined json selector exceeds 32 steps",
        ));
    }
    let plan = Arc::new(Plan {
        steps: records
            .steps()
            .iter()
            .chain(from.steps())
            .cloned()
            .collect(),
        record_boundary: records.steps().len(),
    });
    Ok(Arc::new(Factory(plan)))
}

struct Factory(Arc<Plan>);

impl PreparedCodecFactory for Factory {
    fn create_decoder(
        &self,
        _: ResolvedCodec,
        _: DecodePolicy,
    ) -> Result<Box<dyn PdataDecoder>, CodecError> {
        Ok(Box::new(JsonDecoder(self.0.clone())))
    }
}

struct JsonDecoder(Arc<Plan>);

impl JsonDecoder {
    fn job(
        &self,
        signal: SignalType,
        bytes: &Bytes,
        context: DecodeContext,
    ) -> Result<JsonDecodeJob, CodecError> {
        if signal != SignalType::Logs {
            return Err(CodecError::Unsupported {
                encoding: SYSLOG_JSON_ENCODING,
                operation: CodecOperation::Decode,
                signal,
            });
        }
        if bytes.len() > MAX_JSON_INPUT_BYTES {
            return Err(decode_error("JSON input exceeds 2 MiB"));
        }
        let observed_time = Utc::now()
            .timestamp_nanos_opt()
            .ok_or_else(|| decode_error("observed timestamp is out of range"))?;
        Ok(JsonDecodeJob {
            bytes: bytes.clone(),
            position: 0,
            traversal: Traversal {
                plan: self.0.clone(),
                frames: Vec::new(),
                root_started: false,
                records: 0,
            },
            builder: Some(ReceivedArrowRecordsBuilder::default()),
            observed_time,
            received_local_year: context.received_local_year(),
        })
    }
}

impl PdataDecoder for JsonDecoder {
    fn decode(
        &mut self,
        signal: SignalType,
        bytes: &Bytes,
    ) -> Result<OtapArrowRecords, CodecError> {
        self.decode_with_context(signal, bytes, DecodeContext::default())
    }

    fn decode_with_context(
        &mut self,
        signal: SignalType,
        bytes: &Bytes,
        context: DecodeContext,
    ) -> Result<OtapArrowRecords, CodecError> {
        let mut job = self.job(signal, bytes, context)?;
        loop {
            if let Some(records) = job.step(DecodeBudget::default())? {
                return Ok(records);
            }
        }
    }

    fn start_decode(
        &mut self,
        signal: SignalType,
        bytes: &Bytes,
    ) -> Result<DecodeTask, CodecError> {
        self.start_decode_with_context(signal, bytes, DecodeContext::default())
    }

    fn start_decode_with_context(
        &mut self,
        signal: SignalType,
        bytes: &Bytes,
        context: DecodeContext,
    ) -> Result<DecodeTask, CodecError> {
        if bytes.len() <= MAX_SYNC_BYTES {
            return self
                .decode_with_context(signal, bytes, context)
                .map(DecodeTask::Ready);
        }
        self.job(signal, bytes, context)
            .map(|job| DecodeTask::Incremental(Box::new(job)))
    }
}

struct JsonDecodeJob {
    bytes: Bytes,
    position: usize,
    traversal: Traversal,
    builder: Option<ReceivedArrowRecordsBuilder>,
    observed_time: i64,
    received_local_year: Option<i32>,
}

impl PdataDecodeJob for JsonDecodeJob {
    fn step(&mut self, budget: DecodeBudget) -> Result<Option<OtapArrowRecords>, CodecError> {
        if self.builder.is_none() {
            return Err(decode_error("JSON decode job already finished"));
        }
        let started = Instant::now();
        let start_position = self.position;
        let mut records = 0;
        let mut tokens = 0_usize;
        loop {
            let token = next_token(&self.bytes, &mut self.position)?;
            if matches!(token, Token::End) {
                self.traversal.finish()?;
                let builder = self
                    .builder
                    .take()
                    .ok_or_else(|| decode_error("missing JSON builder"))?;
                return builder.build().map(Some).map_err(|error| {
                    CodecError::operation(&SYSLOG_JSON_ENCODING, CodecOperation::Decode, error)
                });
            }
            if let Some(message) = self.traversal.advance(token)? {
                let builder = self
                    .builder
                    .as_mut()
                    .ok_or_else(|| decode_error("missing JSON builder"))?;
                let _ = append_syslog_message(
                    builder,
                    message.as_bytes(),
                    self.observed_time,
                    self.received_local_year,
                )
                .map_err(|error| {
                    CodecError::operation(&SYSLOG_JSON_ENCODING, CodecOperation::Decode, error)
                })?;
                records += 1;
            }
            tokens += 1;
            if records >= budget.max_records()
                || self.position - start_position >= budget.max_bytes()
                || (tokens.is_multiple_of(16) && started.elapsed() >= budget.max_duration())
            {
                return Ok(None);
            }
        }
    }
}

#[derive(Clone, Copy)]
enum State {
    ObjectKey(bool),
    ObjectColon,
    ObjectValue,
    ObjectAfter,
    ArrayValue(bool),
    ArrayAfter,
}

struct Frame {
    state: State,
    progress: Option<usize>,
    child_progress: Option<usize>,
    found_field: bool,
}

struct Traversal {
    plan: Arc<Plan>,
    frames: Vec<Frame>,
    root_started: bool,
    records: usize,
}

impl Traversal {
    fn finish(&self) -> Result<(), CodecError> {
        if !self.root_started || !self.frames.is_empty() {
            return Err(decode_error("incomplete JSON document"));
        }
        Ok(())
    }

    fn close(&mut self) -> Result<(), CodecError> {
        let frame = self
            .frames
            .pop()
            .ok_or_else(|| decode_error("unexpected JSON container end"))?;
        if frame
            .progress
            .is_some_and(|index| matches!(self.plan.steps.get(index), Some(SelectorStep::Field(_))))
            && !frame.found_field
        {
            return Err(decode_error("selected JSON field is missing"));
        }
        Ok(())
    }

    fn advance<'a>(&mut self, token: Token<'a>) -> Result<Option<Cow<'a, str>>, CodecError> {
        if matches!(token, Token::Whitespace) {
            return Ok(None);
        }
        let progress = if let Some(frame) = self.frames.last_mut() {
            match frame.state {
                State::ObjectKey(allow_end) => {
                    if allow_end && matches!(token, Token::ObjectEnd) {
                        self.close()?;
                        return Ok(None);
                    }
                    let Token::String(key) = token else {
                        return Err(decode_error("expected JSON object key"));
                    };
                    frame.child_progress = match frame.progress {
                        Some(index)
                            if matches!(
                                self.plan.steps.get(index),
                                Some(SelectorStep::Field(field)) if field == key.as_ref()
                            ) =>
                        {
                            if frame.found_field {
                                return Err(decode_error("selected JSON field is duplicated"));
                            }
                            frame.found_field = true;
                            Some(index + 1)
                        }
                        _ => None,
                    };
                    frame.state = State::ObjectColon;
                    return Ok(None);
                }
                State::ObjectColon => {
                    if !matches!(token, Token::Colon) {
                        return Err(decode_error("expected colon after JSON object key"));
                    }
                    frame.state = State::ObjectValue;
                    return Ok(None);
                }
                State::ObjectValue => {
                    frame.state = State::ObjectAfter;
                    frame.child_progress
                }
                State::ObjectAfter | State::ArrayAfter => {
                    let object = matches!(frame.state, State::ObjectAfter);
                    match token {
                        Token::Comma => {
                            frame.state = if object {
                                State::ObjectKey(false)
                            } else {
                                State::ArrayValue(false)
                            };
                        }
                        Token::ObjectEnd if object => self.close()?,
                        Token::ArrayEnd if !object => self.close()?,
                        _ => return Err(decode_error("expected JSON comma or container end")),
                    }
                    return Ok(None);
                }
                State::ArrayValue(allow_end) => {
                    if allow_end && matches!(token, Token::ArrayEnd) {
                        self.close()?;
                        return Ok(None);
                    }
                    frame.state = State::ArrayAfter;
                    frame.progress.map(|index| index + 1)
                }
            }
        } else {
            if self.root_started {
                return Err(decode_error("trailing content after JSON document"));
            }
            self.root_started = true;
            Some(0)
        };
        if let Some(index) = progress {
            if index == self.plan.record_boundary {
                self.records += 1;
                if self.records > MAX_BATCH_ITEMS {
                    return Err(decode_error("JSON selection exceeds 8192 records"));
                }
            }
            match self.plan.steps.get(index) {
                None => {
                    let Token::String(message) = token else {
                        return Err(decode_error("selected syslog value must be a JSON string"));
                    };
                    return Ok(Some(message));
                }
                Some(SelectorStep::Field(_)) if !matches!(token, Token::ObjectStart) => {
                    return Err(decode_error("JSON field selector requires an object"));
                }
                Some(SelectorStep::Each) if !matches!(token, Token::ArrayStart) => {
                    return Err(decode_error("JSON array selector requires an array"));
                }
                _ => {}
            }
        }
        let state = match token {
            Token::ObjectStart => State::ObjectKey(true),
            Token::ArrayStart => State::ArrayValue(true),
            Token::String(_) | Token::Scalar => return Ok(None),
            _ => return Err(decode_error("expected JSON value")),
        };
        if self.frames.len() == MAX_DEPTH {
            return Err(decode_error("JSON nesting exceeds 32 levels"));
        }
        self.frames.push(Frame {
            state,
            progress,
            child_progress: None,
            found_field: false,
        });
        Ok(None)
    }
}

enum Token<'a> {
    ObjectStart,
    ObjectEnd,
    ArrayStart,
    ArrayEnd,
    Colon,
    Comma,
    String(Cow<'a, str>),
    Scalar,
    Whitespace,
    End,
}

#[derive(Deserialize)]
struct JsonString<'a>(#[serde(borrow)] Cow<'a, str>);

// Only container boundaries are handled here. Serde validates every scalar,
// including unselected fields, and unescapes strings without a document tree.
fn next_token<'a>(bytes: &'a [u8], position: &mut usize) -> Result<Token<'a>, CodecError> {
    let Some(&first) = bytes.get(*position) else {
        return Ok(Token::End);
    };
    let start = *position;
    *position += 1;
    Ok(match first {
        b'{' => Token::ObjectStart,
        b'}' => Token::ObjectEnd,
        b'[' => Token::ArrayStart,
        b']' => Token::ArrayEnd,
        b':' => Token::Colon,
        b',' => Token::Comma,
        b' ' | b'\r' | b'\n' | b'\t' => {
            while *position - start < MAX_SYNC_BYTES
                && bytes
                    .get(*position)
                    .is_some_and(|byte| matches!(byte, b' ' | b'\r' | b'\n' | b'\t'))
            {
                *position += 1;
            }
            Token::Whitespace
        }
        b'"' => {
            let mut escaped = false;
            loop {
                if *position - start >= MAX_TOKEN_BYTES {
                    return Err(decode_error("encoded JSON string exceeds the byte limit"));
                }
                let byte = *bytes
                    .get(*position)
                    .ok_or_else(|| decode_error("unterminated JSON string"))?;
                *position += 1;
                if !escaped && byte == b'"' {
                    break;
                }
                escaped = !escaped && byte == b'\\';
            }
            let JsonString(string) = serde_json::from_slice(&bytes[start..*position])
                .map_err(|_| decode_error("invalid JSON string"))?;
            if string.len() > MAX_MESSAGE_SIZE {
                return Err(decode_error("JSON string exceeds 16 KiB"));
            }
            Token::String(string)
        }
        _ => {
            while bytes.get(*position).is_some_and(|byte| {
                !matches!(
                    byte,
                    b' ' | b'\r' | b'\n' | b'\t' | b',' | b']' | b'}' | b'[' | b'{' | b':' | b'"'
                )
            }) {
                if *position - start >= MAX_MESSAGE_SIZE {
                    return Err(decode_error("JSON scalar exceeds 16 KiB"));
                }
                *position += 1;
            }
            let _: IgnoredAny = serde_json::from_slice(&bytes[start..*position])
                .map_err(|_| decode_error("invalid JSON scalar"))?;
            Token::Scalar
        }
    })
}

static METADATA: CodecMetadata = CodecMetadata::new(SYSLOG_JSON_ENCODING, &[SignalType::Logs]);

fn create_decoder(_: DecodePolicy) -> Box<dyn PdataDecoder> {
    Box::new(JsonDecoder(Arc::new(Plan {
        steps: vec![SelectorStep::Field("message".to_owned())].into_boxed_slice(),
        record_boundary: 0,
    })))
}

register_pdata_codec!(
    SYSLOG_CEF_JSON_CODEC,
    CodecRegistration::new(&METADATA)
        .with_implementation("otel.syslog_cef_json")
        .with_default_for_encoding()
        .with_preparer(prepare)
        .with_decoder(create_decoder)
        // A byte-only counter cannot know the pipeline's compiled selection.
        // Preserve unknown until materialization instead of counting documents.
        .with_item_counter(|_, _| None),
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AcceptedEncodings, CodecRegistry, CodecService, CodecServiceBuilder, PdataPayload,
        PdataView,
    };
    use chrono::{Local, TimeZone};
    use otel_arrow_dfe_config::codecs::CodecsConfig;
    use otel_arrow_dfe_pdata::{
        otlp::{ProtoBuffer, ProtoBytesEncoder, logs::LogsProtoBytesEncoder},
        proto::opentelemetry::{
            collector::logs::v1::ExportLogsServiceRequest, logs::v1::LogRecord,
        },
    };
    use prost::Message;
    use serde_json::{Value, json};
    use std::{
        cell::Cell,
        future::{Future, ready},
        task::{Context, Poll, Waker},
    };

    const MESSAGE: &str = "<14>1 2026-11-04T12:00:00Z host app - - - First event";

    fn service(options: Value, validation: &str) -> Result<CodecService, CodecError> {
        let configuration: CodecsConfig = serde_json::from_value(json!({
            "bindings": {
                "syslog-json": {
                    "implementation": "otel.syslog_cef_json",
                    "decode": {"validation": validation},
                    "options": options
                }
            }
        }))
        .unwrap();
        CodecServiceBuilder::from_registry(Arc::new(
            CodecRegistry::validate(std::slice::from_ref(&SYSLOG_CEF_JSON_CODEC)).unwrap(),
        ))
        .with_configuration(CodecsConfig::resolve([&configuration]))
        .build()
    }

    fn payload(service: &CodecService, bytes: Bytes, year: Option<i32>) -> PdataPayload {
        let context = year.map_or_else(DecodeContext::default, |year| {
            DecodeContext::with_received_local_year(year).unwrap()
        });
        service
            .registry()
            .resolve_decoder(&SYSLOG_JSON_ENCODING, SignalType::Logs)
            .unwrap()
            .admit(SignalType::Logs, bytes)
            .unwrap()
            .with_decode_context(context)
            .into()
    }

    fn logs(mut records: OtapArrowRecords) -> Vec<LogRecord> {
        let mut buffer = ProtoBuffer::default();
        LogsProtoBytesEncoder::new()
            .encode(&mut records, &mut buffer)
            .unwrap();
        ExportLogsServiceRequest::decode(buffer.as_ref())
            .unwrap()
            .resource_logs
            .into_iter()
            .flat_map(|resource| resource.scope_logs)
            .flat_map(|scope| scope.log_records)
            .collect()
    }

    fn decode(
        service: &CodecService,
        document: String,
    ) -> Result<OtapArrowRecords, crate::PdataPayloadDecodeError> {
        payload(service, document.into(), Some(2024)).try_into_otap(service)
    }

    /// Scenario: Single objects, root arrays, and nested arrays contain escaped syslog and CEF strings.
    /// Guarantees: Prepared selectors emit messages in order with the existing parser's complete mapping.
    #[test]
    fn selected_shapes_reuse_syslog_and_cef_mapping() {
        let messages = [
            "<14>1 2026-11-04T12:00:00Z host app - - - quoted \"text\", \\path\nsnowman \u{2603}",
            "CEF:0|Vendor|Product|1.0|100|Event|5|src=10.0.0.1",
        ];
        let objects: Vec<_> = messages
            .iter()
            .map(|message| json!({"message": message}))
            .collect();
        for (options, document, expected_messages) in [
            (json!({}), objects[0].to_string(), &messages[..1]),
            (
                json!({}),
                format!(r#"{{"mess\u0061ge":{}}}"#, json!(messages[0])),
                &messages[..1],
            ),
            (json!({"records": "$[*]"}), json!(objects).to_string(), &messages[..]),
            (json!({"records": "$.groups[*].events[*]", "record_decoder": {"type": "syslog", "from": "$.payload.message"}}),
                json!({"ignored": [null, true, false, -12.5e2, {"nested": []}], "groups": [{"events": [
                    {"payload": {"message": messages[0]}}, {"payload": {"message": messages[1]}}
                ]}]}).to_string(), &messages[..]),
            (json!({"records": "$[*]", "record_decoder": {"type": "syslog", "from": "$"}}),
                json!(messages).to_string(), &messages[..]),
        ] {
            let service = service(options, "strict").unwrap();
            let mut actual = logs(decode(&service, document).unwrap());
            let mut expected = ReceivedArrowRecordsBuilder::default();
            for message in expected_messages {
                expected.append_syslog(super::super::parser::parse(message.as_bytes()).unwrap(), 0, 2024);
            }
            assert!(actual.iter().all(|record| record.observed_time_unix_nano > 0));
            for record in &mut actual {
                record.observed_time_unix_nano = 0;
            }
            assert_eq!(actual, logs(expected.build().unwrap()));
        }
    }

    /// Scenario: Named decoder options contain unknown fields, wrong types, or unsupported selectors.
    /// Guarantees: Preparation rejects invalid options and combined traversal depth before decoding.
    #[test]
    fn invalid_options_fail_preparation() {
        for options in [
            json!({"unknown": true}),
            json!({"records": 1}),
            json!({"records": "$[0]"}),
            json!({"record_decoder": {"type": "other", "from": "$.message"}}),
            json!({"record_decoder": {"type": "syslog"}}),
            json!({"record_decoder": {"type": "syslog", "from": "$.message", "unknown": true}}),
            json!({"record_decoder": {"type": "syslog", "from": "$.messages[*]"}}),
            json!({"records": format!("${}", ".x".repeat(32))}),
        ] {
            assert!(service(options.clone(), "strict").is_err(), "{options}");
        }
    }

    /// Scenario: Either validation mode encounters invalid selected or unselected JSON after valid logs.
    /// Guarantees: The whole input fails, exact bytes and receipt context remain recoverable, and reuse succeeds.
    #[test]
    fn invalid_documents_preserve_input_and_decoder_reuse() {
        let valid = json!({"message": MESSAGE}).to_string();
        let mut documents: Vec<Vec<u8>> = [
            "".to_owned(),
            " \n\t".to_owned(),
            "{".to_owned(),
            "[] trailing".to_owned(),
            format!("[{valid},]"),
            format!("[{valid},{{}}]"),
            format!("[{valid},{{\"message\":false}}]"),
            format!(
                r#"[{valid},{{"message":{},"mess\u0061ge":{}}}]"#,
                json!(MESSAGE),
                json!(MESSAGE)
            ),
            format!("[{valid},{{\"message\":\"\"}}]"),
            format!("[{valid},{{\"message\":\"unterminated}}]"),
            format!("[{valid},{{\"message\":\"\\q\"}}]"),
            format!(
                "[{valid},{{\"message\":{},\"message\":{}}}]",
                json!(MESSAGE),
                json!(MESSAGE)
            ),
            format!(
                "[{valid},{{\"message\":{},\"ignored\":[1,]}}]",
                json!(MESSAGE)
            ),
            format!(
                "[{valid},{{\"message\":{},\"ignored\":01}}]",
                json!(MESSAGE)
            ),
            format!("[{valid},{{\"message\" {}}}]", json!(MESSAGE)),
            format!("[{valid},{{\"message\":{}}} 1]", json!(MESSAGE)),
            format!(
                "[{valid},{{\"message\":{},\"ignored\":\"\\uD800\"}}]",
                json!(MESSAGE)
            ),
        ]
        .into_iter()
        .map(String::into_bytes)
        .collect();
        documents.push(
            [
                format!("[{valid},{{\"ignored\":\"").as_bytes(),
                &[0xff],
                b"\"}]",
            ]
            .concat(),
        );
        for validation in ["best_effort", "strict"] {
            let service = service(json!({"records": "$[*]"}), validation).unwrap();
            for document in &documents {
                let bytes = Bytes::copy_from_slice(document);
                let error = payload(&service, bytes.clone(), Some(2020))
                    .try_into_otap(&service)
                    .unwrap_err();
                let recovered = error.payload();
                assert_eq!(recovered.encoded_bytes().unwrap().as_ptr(), bytes.as_ptr());
                assert_eq!(recovered.encoded_bytes().unwrap(), &bytes);
                assert_eq!(
                    recovered
                        .encoded_view()
                        .unwrap()
                        .decode_context()
                        .received_local_year(),
                    Some(2020)
                );
                assert_eq!(recovered.known_item_count(), None);
                assert_eq!(
                    logs(decode(&service, format!("[{valid}]")).unwrap()).len(),
                    1
                );
            }
        }
    }

    /// Scenario: Selectors encounter missing, duplicated, or incorrectly typed intermediate containers.
    /// Guarantees: No selected field is silently ignored and empty selected arrays remain valid zero-log batches.
    #[test]
    fn selectors_enforce_container_types_and_empty_arrays() {
        let service = service(json!({"records": "$.events[*]"}), "strict").unwrap();
        for document in [
            json!({}).to_string(),
            json!({"events": {}}).to_string(),
            json!({"events": [null]}).to_string(),
            json!({"events": [{"message": []}]}).to_string(),
            r#"{"events":[],"events":[]}"#.to_owned(),
        ] {
            assert!(decode(&service, document.clone()).is_err(), "{document}");
        }
        let mut empty = payload(&service, Bytes::from_static(br#"{"events":[]}"#), None);
        assert_eq!(empty.num_items(), 0);
        assert_eq!(empty.known_item_count(), None);
        assert!(logs(empty.try_into_otap(&service).unwrap()).is_empty());
    }

    /// Scenario: JSON input, decoded and escaped strings, scalar tokens, nesting, and record counts reach hard limits.
    /// Guarantees: Each exact boundary decodes, while one extra byte, container, or record rejects the input.
    #[test]
    fn resource_limits_accept_exact_boundaries() {
        let service = service(json!({}), "strict").unwrap();
        let base = json!({"message": MESSAGE}).to_string();
        let exact_input = format!("{base}{}", " ".repeat(MAX_JSON_INPUT_BYTES - base.len()));
        assert!(decode(&service, exact_input.clone()).is_ok());
        assert!(decode(&service, exact_input + " ").is_err());

        for length in [MAX_MESSAGE_SIZE, MAX_MESSAGE_SIZE + 1] {
            let message = format!("{MESSAGE}{}", "x".repeat(length - MESSAGE.len()));
            let document = json!({"message": message}).to_string();
            assert_eq!(
                decode(&service, document).is_ok(),
                length == MAX_MESSAGE_SIZE
            );
            let document = format!(
                "{{\"message\":{},\"ignored\":\"{}\"}}",
                json!(MESSAGE),
                "\\u0061".repeat(length)
            );
            assert_eq!(
                decode(&service, document).is_ok(),
                length == MAX_MESSAGE_SIZE
            );
            let number = format!("0.{}", "0".repeat(length - 2));
            let document = format!("{{\"message\":{},\"ignored\":{number}}}", json!(MESSAGE));
            assert_eq!(
                decode(&service, document).is_ok(),
                length == MAX_MESSAGE_SIZE
            );
        }
        for depth in [MAX_DEPTH, MAX_DEPTH + 1] {
            let document = format!(
                "{{\"message\":{},\"ignored\":{}0{}}}",
                json!(MESSAGE),
                "[".repeat(depth - 1),
                "]".repeat(depth - 1)
            );
            assert_eq!(decode(&service, document).is_ok(), depth == MAX_DEPTH);
        }
        let service = self::service(json!({"records": "$[*]"}), "strict").unwrap();
        for count in [MAX_BATCH_ITEMS, MAX_BATCH_ITEMS + 1] {
            let document = format!("[{}]", vec![base.as_str(); count].join(","));
            match decode(&service, document) {
                Ok(records) => {
                    assert_eq!(count, MAX_BATCH_ITEMS);
                    assert_eq!(logs(records).len(), count);
                }
                Err(_) => assert_eq!(count, MAX_BATCH_ITEMS + 1),
            }
        }
    }

    /// Scenario: Yearless plain and CEF-wrapped syslog is cloned, viewed, and decoded long after receipt.
    /// Guarantees: Every materialization path uses the stored year, while missing receipt context fails explicitly.
    #[test]
    fn receipt_year_survives_views_and_materialization() {
        let service = service(json!({}), "strict").unwrap();
        for message in [
            "<34>Dec 31 23:59:58 host tag: message",
            "<34>Dec 31 23:59:58 host CEF:0|Vendor|Product|1.0|100|Event|5|",
        ] {
            let document = Bytes::from(format!(
                "{}{}",
                json!({"message": message}),
                " ".repeat(MAX_SYNC_BYTES)
            ));
            for year in [2020, 2021] {
                let original = payload(&service, document.clone(), Some(year));
                let accepted = AcceptedEncodings::accept_encoded([service
                    .registry()
                    .resolve(&SYSLOG_JSON_ENCODING)
                    .unwrap()]);
                let PdataView::Encoded(encoded) = original.view(&service, &accepted).unwrap()
                else {
                    panic!("accepted JSON must remain encoded");
                };
                assert_eq!(encoded.bytes(), document.as_ref());
                assert_eq!(encoded.decode_context().received_local_year(), Some(year));
                let expected = Local
                    .with_ymd_and_hms(year, 12, 31, 23, 59, 58)
                    .single()
                    .unwrap()
                    .timestamp_nanos_opt()
                    .unwrap() as u64;
                let PdataView::Native(viewed) = original
                    .view(&service, &AcceptedEncodings::native())
                    .unwrap()
                else {
                    panic!("native fallback must decode");
                };
                let cooperative = futures::executor::block_on(
                    original
                        .clone()
                        .try_into_otap_cooperative(&service, || ready(())),
                )
                .unwrap();
                for records in [
                    viewed.into_owned(),
                    cooperative,
                    original.try_into_otap(&service).unwrap(),
                ] {
                    assert_eq!(logs(records)[0].time_unix_nano, expected);
                }
            }
            let error = payload(&service, document, None)
                .try_into_otap(&service)
                .unwrap_err();
            assert!(error.to_string().contains("receipt year"));
        }
        assert!(
            payload(
                &service,
                json!({"message": MESSAGE}).to_string().into(),
                None
            )
            .try_into_otap(&service)
            .is_ok()
        );
    }

    /// Scenario: Large inputs suspend, are cancelled, fail near the end, or finish incrementally.
    /// Guarantees: Progress is cooperative, output is atomic, cancellation preserves the input, and jobs cannot finish twice.
    #[test]
    fn cooperative_decoding_preserves_atomicity_and_cancellation() {
        let service = service(json!({"records": "$[*]"}), "strict").unwrap();
        let record = json!({"message": MESSAGE}).to_string();
        let bytes = Bytes::from(format!("[{}]", vec![record.as_str(); 513].join(",")));
        let original = payload(&service, bytes.clone(), Some(2024));
        let yields = Cell::new(0);
        let mut future = Box::pin(original.clone().try_into_otap_cooperative(&service, || {
            yields.set(yields.get() + 1);
            std::future::pending::<()>()
        }));
        assert!(matches!(
            future
                .as_mut()
                .poll(&mut Context::from_waker(Waker::noop())),
            Poll::Pending
        ));
        drop(future);
        assert_eq!(yields.get(), 1);
        assert_eq!(original.encoded_bytes().unwrap(), &bytes);
        let records =
            futures::executor::block_on(original.try_into_otap_cooperative(&service, || {
                yields.set(yields.get() + 1);
                ready(())
            }))
            .unwrap();
        assert_eq!(logs(records).len(), 513);
        assert!(yields.get() >= 5);

        let invalid = Bytes::from(format!("[{},{{}}]", vec![record.as_str(); 513].join(",")));
        let error = futures::executor::block_on(
            payload(&service, invalid.clone(), Some(2024))
                .try_into_otap_cooperative(&service, || ready(())),
        )
        .unwrap_err();
        assert_eq!(error.payload().encoded_bytes().unwrap(), &invalid);

        let factory =
            prepare(&serde_json::from_value(json!({"options": {"records": "$[*]"}})).unwrap())
                .unwrap();
        let codec = service.registry().resolve(&SYSLOG_JSON_ENCODING).unwrap();
        let mut decoder = factory
            .create_decoder(codec, DecodePolicy::default())
            .unwrap();
        assert!(matches!(
            decoder
                .start_decode(SignalType::Logs, &Bytes::from(format!("[{record}]")))
                .unwrap(),
            DecodeTask::Ready(_)
        ));
        let DecodeTask::Incremental(mut job) =
            decoder.start_decode(SignalType::Logs, &bytes).unwrap()
        else {
            panic!("large JSON must decode cooperatively");
        };
        let mut pending = 0;
        loop {
            if let Some(records) = job.step(DecodeBudget::default()).unwrap() {
                assert_eq!(logs(records).len(), 513);
                break;
            }
            pending += 1;
            assert!(pending < bytes.len(), "decode job must make progress");
        }
        assert!(pending >= 4);
        assert!(job.step(DecodeBudget::default()).is_err());
        for signal in [SignalType::Traces, SignalType::Metrics] {
            assert!(decoder.decode(signal, &bytes).is_err());
        }
    }
}
