// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Streaming JSON selection followed by the shared syslog/CEF Arrow append path.

use std::{borrow::Cow, sync::Arc, time::Instant};

use bytes::Bytes;
use chrono::Utc;
use otel_arrow_dfe_config::{SignalType, codecs::CodecConfiguration};
use otel_arrow_dfe_pdata::OtapArrowRecords;
use otel_arrow_dfe_pdata_codec::{
    CodecError, CodecMetadata, CodecOperation, CodecRegistration, DecodeBudget, DecodeContext,
    DecodePolicy, DecodeTask, PdataDecodeJob, PdataDecoder, PdataEncoding, PreparedCodecFactory,
    ResolvedCodec, register_pdata_codec,
    selector::{CompiledSelector, SelectorStep},
};
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
