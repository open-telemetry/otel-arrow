// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

fn reserve<T>(buffer: &mut Vec<T>, count: usize, limit: usize) -> Result<(), MultilineError> {
    let needed = buffer
        .len()
        .checked_add(count)
        .ok_or(MultilineError::Allocation)?;
    if needed > limit {
        return Err(MultilineError::Allocation);
    }
    if needed > buffer.capacity() {
        let target = needed.max(buffer.capacity().saturating_mul(2).max(64).min(limit));
        buffer
            .try_reserve_exact(target - buffer.len())
            .map_err(|_| MultilineError::Allocation)?;
    }
    Ok(())
}

fn reserve_text(text: &mut String, count: usize, limit: usize) -> Result<(), MultilineError> {
    let needed = text
        .len()
        .checked_add(count)
        .ok_or(MultilineError::Allocation)?;
    if needed > limit {
        return Err(MultilineError::Allocation);
    }
    if needed > text.capacity() {
        let target = needed.max(text.capacity().saturating_mul(2).max(64).min(limit));
        text.try_reserve_exact(target - text.len())
            .map_err(|_| MultilineError::Allocation)?;
    }
    Ok(())
}

fn push_text(text: &mut String, value: DecodedValue, limit: usize) -> Result<(), MultilineError> {
    if let DecodedValue::Scalar(c) = value {
        reserve_text(text, c.len_utf8(), limit)?;
        text.push(c);
    }
    Ok(())
}

fn text_bytes(event: DecodeEvent) -> usize {
    match event {
        DecodeEvent::Unit {
            value: DecodedValue::Scalar(c),
            ..
        } => c.len_utf8(),
        _ => 0,
    }
}

#[derive(Debug)]
pub(super) struct Line {
    pub source: Vec<u8>,
    pub text: String,
    pub start: u64,
    pub end: u64,
    pub body_bytes: usize,
    pub body_start: u64,
    pub malformed_units: u64,
    pub lf: Option<DecodeEvent>,
}
impl Line {
    pub fn new(offset: u64) -> Self {
        Self {
            source: Vec::new(),
            text: String::new(),
            start: offset,
            end: offset,
            body_bytes: 0,
            body_start: offset,
            malformed_units: 0,
            lf: None,
        }
    }
    pub fn fits(&self, event: DecodeEvent, config: LineConfig) -> bool {
        if is_lf(event) || matches!(event, DecodeEvent::StrippedBom { .. }) {
            return true;
        }
        text_bytes(event) <= config.max_line_bytes - self.text.len()
            && (!keeps_raw(config)
                || event.source().as_slice().len() <= config.max_line_bytes - self.body_bytes)
    }
    pub fn push(&mut self, event: DecodeEvent, config: LineConfig) -> Result<(), MultilineError> {
        let bytes = event.source();
        reserve(
            &mut self.source,
            bytes.as_slice().len(),
            if keeps_raw(config) {
                config.max_line_bytes + 8
            } else {
                2 * config.max_line_bytes + 8
            },
        )?;
        self.source.extend_from_slice(bytes.as_slice());
        self.end = event.range().end;
        match event {
            DecodeEvent::StrippedBom { range, .. } => self.body_start = range.end,
            DecodeEvent::Unit {
                value, malformed, ..
            } => {
                if is_lf(event) {
                    self.lf = Some(event);
                } else {
                    push_text(&mut self.text, value, config.max_line_bytes)?;
                    self.body_bytes += bytes.as_slice().len();
                    self.malformed_units += u64::from(malformed);
                }
            }
        }
        Ok(())
    }
    pub fn reset(&mut self, offset: u64) {
        self.source.clear();
        self.text.clear();
        self.start = offset;
        self.end = offset;
        self.body_bytes = 0;
        self.body_start = offset;
        self.malformed_units = 0;
        self.lf = None;
    }
    pub fn capacity(&self) -> usize {
        self.source.capacity() + self.text.capacity()
    }
}

#[derive(Debug)]
pub(super) struct Body {
    pub text: String,
    pub raw: Vec<u8>,
    pub frame_start: u64,
    pub frame_end: u64,
    range: SourceRange,
    pub truncated: bool,
    malformed: u64,
    body_malformed: bool,
    discarded: u64,
}
impl Body {
    pub fn new(offset: u64) -> Self {
        Self {
            text: String::new(),
            raw: Vec::new(),
            frame_start: offset,
            frame_end: offset,
            range: SourceRange {
                start: offset,
                end: offset,
            },
            truncated: false,
            malformed: 0,
            body_malformed: false,
            discarded: 0,
        }
    }
    pub fn has_progress(&self) -> bool {
        self.frame_end > self.frame_start
    }
    pub fn capacity(&self) -> usize {
        self.text.capacity() + self.raw.capacity()
    }
    pub fn size(&self, config: LineConfig) -> usize {
        if keeps_raw(config) {
            self.text.len().max(self.raw.len())
        } else {
            self.text.len()
        }
    }
    pub fn fits(&self, event: DecodeEvent, config: LineConfig) -> bool {
        if matches!(event, DecodeEvent::StrippedBom { .. }) {
            return true;
        }
        text_bytes(event) <= config.max_record_bytes - self.text.len()
            && (!keeps_raw(config)
                || event.source().as_slice().len() <= config.max_record_bytes - self.raw.len())
    }
    pub fn push(&mut self, event: DecodeEvent, config: LineConfig) -> Result<(), MultilineError> {
        self.frame_end = event.range().end;
        let DecodeEvent::Unit {
            range,
            source,
            value,
            malformed,
        } = event
        else {
            self.range = SourceRange {
                start: event.range().end,
                end: event.range().end,
            };
            return Ok(());
        };
        self.malformed += u64::from(malformed);
        if self.truncated {
            self.discarded += source.as_slice().len() as u64;
            return Ok(());
        }
        push_text(&mut self.text, value, config.max_record_bytes)?;
        if keeps_raw(config) {
            reserve(
                &mut self.raw,
                source.as_slice().len(),
                config.max_record_bytes,
            )?;
            self.raw.extend_from_slice(source.as_slice());
        }
        self.range.end = range.end;
        self.body_malformed |= malformed;
        Ok(())
    }
    // A complete validated line fits the record, so no internal unit boundary
    // is needed. Copy its decoded text and exact bytes with one reservation each.
    pub fn append_line(
        &mut self,
        line: &Line,
        separator: Option<DecodeEvent>,
        config: LineConfig,
    ) -> Result<(), MultilineError> {
        let text = line.text.len() + separator.map_or(0, text_bytes);
        let raw = line.body_bytes + separator.map_or(0, |event| event.source().as_slice().len());
        reserve_text(&mut self.text, text, config.max_record_bytes)?;
        if keeps_raw(config) {
            reserve(&mut self.raw, raw, config.max_record_bytes)?;
        }
        if let Some(separator) = separator {
            self.push(separator, config)?;
        }
        if !self.has_progress() {
            self.range.start = line.body_start;
        }
        self.text.push_str(&line.text);
        if keeps_raw(config) {
            let start = usize::try_from(line.body_start - line.start)
                .map_err(|_| MultilineError::ContinuationBoundary)?;
            self.raw
                .extend_from_slice(&line.source[start..start + line.body_bytes]);
        }
        self.range.end = line.body_start + line.body_bytes as u64;
        self.frame_end = line.end;
        self.malformed += line.malformed_units;
        self.body_malformed |= line.malformed_units != 0;
        Ok(())
    }

    pub fn take(
        &mut self,
        config: LineConfig,
        ending: RecordEnding,
        fragment: Option<RecordFragment>,
        continuation: Option<RecordContinuation>,
    ) -> RecordFrame {
        let bytes = config.encoding == Encoding::Raw
            || (config.on_decode_error == OnDecodeError::PreserveRaw
                && (self.body_malformed || fragment.is_some()));
        let body = if bytes {
            self.text.clear();
            LineBody::Bytes(std::mem::take(&mut self.raw))
        } else {
            self.raw.clear();
            LineBody::Text(std::mem::take(&mut self.text))
        };
        let frame = RecordFrame {
            body,
            body_range: self.range,
            frame_range: SourceRange {
                start: self.frame_start,
                end: self.frame_end,
            },
            ending,
            fragment,
            continuation,
            truncated: self.truncated,
            discarded_source_bytes: self.discarded,
            malformed_units: self.malformed,
        };
        self.frame_start = self.frame_end;
        self.range = SourceRange {
            start: self.frame_end,
            end: self.frame_end,
        };
        self.truncated = false;
        self.malformed = 0;
        self.body_malformed = false;
        self.discarded = 0;
        frame
    }
}
