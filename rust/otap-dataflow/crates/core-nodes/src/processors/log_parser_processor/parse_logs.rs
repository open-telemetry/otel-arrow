//! Record-local extraction and normalization for already-framed logs.

use chrono::{DateTime, Timelike};
use serde::Deserialize;
use std::{collections::BTreeMap, num::NonZeroUsize};

mod json;
mod update;
pub(super) use update::Counts;

#[derive(Debug, Clone, Copy, PartialEq, Eq, otel_arrow_dfe_telemetry_macros::AttributeEnum)]
pub(super) enum DataError {
    Limit,
    Extraction,
    Body,
    Timestamp,
    Severity,
    UnsupportedBody,
    ObservedFallback,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct Candidate {
    pub body: Option<String>,
    pub timestamp: Option<u64>,
    pub severity: Option<(String, u8)>,
    pub fallback: bool,
}

pub(super) struct Parser {
    config: ParseConfig,
    regex: Option<regex_automata::nfa::thompson::pikevm::PikeVM>,
    sources: Vec<Vec<String>>,
    #[cfg(test)]
    pub(super) fail_after: Option<usize>,
}

impl Parser {
    pub(super) fn new(config: ParseConfig) -> Result<Self, String> {
        config.validate()?;
        let regex = if let Some(pattern) = &config.pattern {
            Some(
                regex_automata::nfa::thompson::pikevm::PikeVM::builder()
                    .thompson(
                        regex_automata::nfa::thompson::Config::new()
                            .nfa_size_limit(Some(config.limits.max_compiled_regex_bytes.get())),
                    )
                    .build(&format!("\\A(?:{pattern})\\z"))
                    .map_err(|_| "invalid or oversized regex")?,
            )
        } else {
            None
        };
        if let Some(regex) = &regex {
            for source in config.sources() {
                if regex
                    .get_nfa()
                    .group_info()
                    .to_index(regex_automata::PatternID::ZERO, source)
                    .is_none()
                {
                    return Err("unknown regex capture".into());
                }
            }
        }
        let sources = config
            .sources()
            .map(|source| {
                if config.format == Format::Json {
                    source
                        .split('/')
                        .skip(1)
                        .map(|token| token.replace("~1", "/").replace("~0", "~"))
                        .collect()
                } else {
                    vec![source.to_owned()]
                }
            })
            .collect();
        Ok(Self {
            config,
            regex,
            sources,
            #[cfg(test)]
            fail_after: None,
        })
    }

    pub(super) fn format(&self) -> Format {
        self.config.format
    }

    pub(super) fn scratch_bound(&self, input: &str) -> Result<usize, DataError> {
        let extra = input.len().checked_mul(2).ok_or(DataError::Limit)?;
        let scratch = match self.config.format {
            Format::Json => {
                json::Document::scratch_bound(input.len(), self.config.limits.max_entries.get())
            }
            Format::Csv => input.len().checked_add(
                self.config
                    .columns
                    .as_ref()
                    .map_or(0, Vec::len)
                    .checked_add(1)
                    .and_then(|count| count.checked_mul(size_of::<usize>()))
                    .ok_or(DataError::Limit)?,
            ),
            Format::Regex => self.regex.as_ref().and_then(|regex| {
                let states = regex.get_nfa().states().len();
                let slots = regex.get_nfa().group_info().slot_len();
                states
                    .checked_add(1)?
                    .checked_mul(slots)?
                    .checked_mul(2 * size_of::<usize>())?
                    .checked_add(states.checked_mul(128)?)?
                    .checked_add(slots.checked_mul(size_of::<usize>())?)?
                    .checked_add(512)
            }),
        }
        .and_then(|amount| amount.checked_add(extra))
        .ok_or(DataError::Limit)?;
        Ok(scratch)
    }

    pub(super) fn parse(
        &self,
        input: &str,
        event: u64,
        observed: u64,
    ) -> Result<Candidate, DataError> {
        if input.len() > self.config.limits.max_input_bytes.get() {
            return Err(DataError::Limit);
        }
        let ErrorPolicy::Preserve = self.config.on_error;
        let scratch = self.scratch_bound(input)?;
        if scratch > self.config.limits.max_scratch_bytes.get() {
            return Err(DataError::Limit);
        }
        match self.config.format {
            Format::Json => {
                let document = json::Document::parse(
                    input,
                    self.config.limits.max_entries.get(),
                    self.config.limits.max_json_depth.get(),
                )
                .map_err(|limit| {
                    if limit {
                        DataError::Limit
                    } else {
                        DataError::Extraction
                    }
                })?;
                self.map(
                    |index| document.lookup(&self.sources[index]),
                    event,
                    observed,
                )
            }
            Format::Regex => {
                let regex = self.regex.as_ref().ok_or(DataError::Extraction)?;
                let mut cache = regex.create_cache();
                let mut captures = regex.create_captures();
                regex.captures(&mut cache, input, &mut captures);
                if !captures.is_match() {
                    return Err(DataError::Extraction);
                }
                self.map(
                    |index| {
                        Ok(captures
                            .get_group_by_name(&self.sources[index][0])
                            .map(|span| &input[span.start..span.end]))
                    },
                    event,
                    observed,
                )
            }
            Format::Csv => {
                let columns = self.config.columns.as_ref().ok_or(DataError::Extraction)?;
                let delimiter = self
                    .config
                    .delimiter
                    .as_ref()
                    .ok_or(DataError::Extraction)?
                    .as_bytes()[0];
                if input.is_empty() && columns.len() == 1 {
                    return self.map(|_| Ok(Some("")), event, observed);
                }
                let fields = validate_csv(input, delimiter)?;
                if fields > self.config.limits.max_entries.get() {
                    return Err(DataError::Limit);
                }
                let mut reader = csv_core::ReaderBuilder::new().delimiter(delimiter).build();
                let mut output = vec![0; input.len()];
                let mut ends = vec![0; columns.len() + 1];
                let (mut result, consumed, written, mut fields) =
                    reader.read_record(input.as_bytes(), &mut output, &mut ends);
                if result == csv_core::ReadRecordResult::InputEmpty && consumed == input.len() {
                    let (next, _, _, added) =
                        reader.read_record(&[], &mut output[written..], &mut ends[fields..]);
                    result = next;
                    fields += added;
                }
                if result != csv_core::ReadRecordResult::Record || fields != columns.len() {
                    return Err(DataError::Extraction);
                }
                let text = std::str::from_utf8(&output).map_err(|_| DataError::Extraction)?;
                self.map(
                    |index| {
                        let column = columns
                            .iter()
                            .position(|column| column == &self.sources[index][0])
                            .ok_or(())?;
                        let start = if column == 0 { 0 } else { ends[column - 1] };
                        Ok(Some(&text[start..ends[column]]))
                    },
                    event,
                    observed,
                )
            }
        }
    }

    fn map<'input>(
        &self,
        mut lookup: impl FnMut(usize) -> Result<Option<&'input str>, ()>,
        event: u64,
        observed: u64,
    ) -> Result<Candidate, DataError> {
        let mut result = Candidate::default();
        let mut index = 0;
        if self.config.body.is_some() {
            result.body = Some(
                lookup(index)
                    .map_err(|_| DataError::Body)?
                    .ok_or(DataError::Body)?
                    .to_owned(),
            );
            index += 1;
        }
        if let Some(mapping) = &self.config.timestamp {
            let TimestampFormat::Rfc3339 = mapping.format;
            match lookup(index).map_err(|_| DataError::Timestamp)? {
                Some(value) => {
                    result.timestamp = Some(parse_timestamp(value).ok_or(DataError::Timestamp)?)
                }
                None if event != 0 => (),
                None => match mapping.on_missing {
                    MissingTimestamp::Preserve => (),
                    MissingTimestamp::Observed => {
                        if observed == 0 {
                            return Err(DataError::Timestamp);
                        }
                        result.timestamp = Some(observed);
                        result.fallback = true;
                    }
                },
            }
            index += 1;
        }
        if let Some(mapping) = &self.config.severity {
            let text = lookup(index)
                .map_err(|_| DataError::Severity)?
                .ok_or(DataError::Severity)?;
            let number = mapping.mapping.get(text).ok_or(DataError::Severity)?;
            result.severity = Some((text.to_owned(), *number));
        }
        Ok(result)
    }
}

fn validate_csv(input: &str, delimiter: u8) -> Result<usize, DataError> {
    let mut fields = 1;
    let mut quoted = false;
    let mut after_quote = false;
    let mut field_start = true;
    let mut bytes = input.bytes().peekable();
    while let Some(byte) = bytes.next() {
        if matches!(byte, b'\r' | b'\n') {
            return Err(DataError::Extraction);
        }
        if quoted {
            if byte == b'"' {
                if bytes.peek() == Some(&b'"') {
                    let _ = bytes.next();
                } else {
                    quoted = false;
                    after_quote = true;
                }
            }
        } else if byte == delimiter {
            fields += 1;
            after_quote = false;
            field_start = true;
        } else if after_quote {
            return Err(DataError::Extraction);
        } else if byte == b'"' {
            if !field_start {
                return Err(DataError::Extraction);
            }
            quoted = true;
            field_start = false;
        } else {
            field_start = false;
        }
    }
    if quoted {
        Err(DataError::Extraction)
    } else {
        Ok(fields)
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ParseConfig {
    pub format: Format,
    pub pattern: Option<String>,
    pub columns: Option<Vec<String>>,
    pub delimiter: Option<String>,
    pub header: Option<Header>,
    pub body: Option<Source>,
    pub timestamp: Option<Timestamp>,
    pub severity: Option<Severity>,
    pub on_error: ErrorPolicy,
    pub limits: Limits,
}

#[derive(
    Debug, Clone, Copy, Deserialize, PartialEq, Eq, otel_arrow_dfe_telemetry_macros::AttributeEnum,
)]
#[serde(rename_all = "snake_case")]
pub(super) enum Format {
    Regex,
    Json,
    Csv,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Header {
    None,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum ErrorPolicy {
    Preserve,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Source {
    pub source: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Timestamp {
    pub source: String,
    pub format: TimestampFormat,
    pub on_missing: MissingTimestamp,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum TimestampFormat {
    Rfc3339,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum MissingTimestamp {
    Observed,
    Preserve,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Severity {
    pub source: String,
    pub mapping: BTreeMap<String, u8>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Limits {
    pub max_input_bytes: NonZeroUsize,
    pub max_scratch_bytes: NonZeroUsize,
    pub max_pattern_bytes: NonZeroUsize,
    pub max_compiled_regex_bytes: NonZeroUsize,
    pub max_json_depth: NonZeroUsize,
    pub max_entries: NonZeroUsize,
}

impl ParseConfig {
    pub(super) fn validate(&self) -> Result<(), String> {
        match self.format {
            Format::Regex => {
                let pattern = self.pattern.as_ref().ok_or("regex requires pattern")?;
                if pattern.len() > self.limits.max_pattern_bytes.get() {
                    return Err("regex pattern exceeds limit".into());
                }
                if self.columns.is_some() || self.delimiter.is_some() || self.header.is_some() {
                    return Err("CSV options require CSV format".into());
                }
            }
            Format::Csv => {
                if self.pattern.is_some() {
                    return Err("pattern requires regex format".into());
                }
                let columns = self.columns.as_ref().ok_or("CSV requires columns")?;
                let delimiter = self.delimiter.as_ref().ok_or("CSV requires delimiter")?;
                if self.header.is_none()
                    || columns.is_empty()
                    || columns.len() > self.limits.max_entries.get()
                    || delimiter.len() != 1
                    || !delimiter.is_ascii()
                    || matches!(delimiter.as_bytes()[0], b'"' | b'\r' | b'\n')
                {
                    return Err("invalid CSV columns, delimiter or header".into());
                }
                for (index, column) in columns.iter().enumerate() {
                    if column.is_empty() || columns[..index].contains(column) {
                        return Err("empty or duplicate CSV column".into());
                    }
                }
                for source in self.sources() {
                    if !columns.iter().any(|column| column == source) {
                        return Err("unknown CSV column".into());
                    }
                }
            }
            Format::Json => {
                if self.limits.max_json_depth.get() > 64 {
                    return Err("max_json_depth cannot exceed 64".into());
                }
                if self.pattern.is_some()
                    || self.columns.is_some()
                    || self.delimiter.is_some()
                    || self.header.is_some()
                {
                    return Err("format-specific options do not apply to JSON".into());
                }
                for source in self.sources() {
                    if !valid_pointer(source) {
                        return Err("invalid JSON pointer".into());
                    }
                }
            }
        }
        if let Some(severity) = &self.severity
            && (severity.mapping.is_empty()
                || severity
                    .mapping
                    .values()
                    .any(|number| !(1..=24).contains(number)))
        {
            return Err("severity mapping requires numbers in 1..=24".into());
        }
        Ok(())
    }

    fn sources(&self) -> impl Iterator<Item = &str> {
        self.body
            .iter()
            .map(|mapping| mapping.source.as_str())
            .chain(self.timestamp.iter().map(|mapping| mapping.source.as_str()))
            .chain(self.severity.iter().map(|mapping| mapping.source.as_str()))
    }
}

fn valid_pointer(source: &str) -> bool {
    if !source.is_empty() && !source.starts_with('/') {
        return false;
    }
    let mut bytes = source.bytes();
    while let Some(byte) = bytes.next() {
        if byte == b'~' && !matches!(bytes.next(), Some(b'0' | b'1')) {
            return false;
        }
    }
    true
}

fn parse_timestamp(value: &str) -> Option<u64> {
    let time_start = value.find('T').or_else(|| value.find('t'))?;
    let time = value.get(time_start + 1..)?;
    if let Some(fraction_start) = time.find('.') {
        let precision = time[fraction_start + 1..]
            .bytes()
            .take_while(u8::is_ascii_digit)
            .count();
        if precision == 0 || precision > 9 {
            return None;
        }
    }
    let parsed = DateTime::parse_from_rfc3339(value).ok()?;
    if parsed.nanosecond() >= 1_000_000_000 {
        return None;
    }
    let seconds = u64::try_from(parsed.timestamp()).ok()?;
    seconds
        .checked_mul(1_000_000_000)?
        .checked_add(u64::from(parsed.nanosecond()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(format: &str) -> serde_json::Value {
        serde_json::json!({
            "format": format,
            "on_error": "preserve",
            "limits": {
                "max_input_bytes": 1048576, "max_scratch_bytes": 8388608,
                "max_pattern_bytes": 4096, "max_compiled_regex_bytes": 1048576,
                "max_json_depth": 32, "max_entries": 4096
            }
        })
    }

    fn parser_config(format: &str) -> serde_json::Value {
        let mut value = config(format);
        let prefix = if format == "json" { "/" } else { "" };
        value["body"] = serde_json::json!({"source": format!("{prefix}message")});
        value["timestamp"] = serde_json::json!({"source": format!("{prefix}ts"), "format": "rfc3339", "on_missing": "observed"});
        value["severity"] =
            serde_json::json!({"source": format!("{prefix}sev"), "mapping": {"ERROR":17,"INFO":9}});
        match format {
            "regex" => {
                value["pattern"] = r"(?s)START: (?P<ts>\S+) (?P<sev>\S+) (?P<message>.*)".into()
            }
            "csv" => {
                value["columns"] = serde_json::json!(["ts", "sev", "message"]);
                value["delimiter"] = ",".into();
                value["header"] = "none".into();
            }
            _ => (),
        }
        value
    }

    /// Scenario: Complete framed records use regex, JSON or quoted headerless CSV.
    /// Guarantees: All formats produce equivalent exact timestamp and severity candidates.
    #[test]
    fn extracts_all_formats() {
        for (format, input, body, severity) in [
            (
                "regex",
                "START: 1970-01-01T00:00:02Z ERROR failure\n detail",
                "failure\n detail",
                ("ERROR", 17),
            ),
            (
                "json",
                r#"{"message":"failure","ts":"1970-01-01T00:00:02Z","sev":"ERROR"}"#,
                "failure",
                ("ERROR", 17),
            ),
            (
                "csv",
                "1970-01-01T00:00:02Z,INFO,\"hello, \"\"world\"\"\"",
                "hello, \"world\"",
                ("INFO", 9),
            ),
        ] {
            let parser =
                Parser::new(serde_json::from_value(parser_config(format)).unwrap()).unwrap();
            assert_eq!(
                parser.parse(input, 0, 3_000_000_000),
                Ok(Candidate {
                    body: Some(body.into()),
                    timestamp: Some(2_000_000_000),
                    severity: Some((severity.0.into(), severity.1)),
                    fallback: false,
                }),
                "{format}"
            );
        }
    }

    /// Scenario: Timestamp fallback is followed by successful or invalid severity mapping.
    /// Guarantees: Fallback preserves existing event time and a later error discards all candidates.
    #[test]
    fn fallback_and_atomic_candidates() {
        let parser = Parser::new(serde_json::from_value(parser_config("json")).unwrap()).unwrap();
        let input = r#"{"message":"failure","ts":null,"sev":"ERROR"}"#;
        let candidate = parser.parse(input, 0, 3_000_000_000).unwrap();
        assert_eq!(candidate.timestamp, Some(3_000_000_000));
        assert!(candidate.fallback);
        let candidate = parser.parse(input, 2_000_000_000, 3_000_000_000).unwrap();
        assert_eq!(candidate.timestamp, None);
        assert!(!candidate.fallback);
        assert_eq!(parser.parse(input, 0, 0), Err(DataError::Timestamp));
        assert_eq!(
            parser.parse(r#"{"message":"failure","sev":"unknown"}"#, 0, 3_000_000_000),
            Err(DataError::Severity)
        );
        assert_eq!(
            parser.parse(
                r#"{"message":42,"ts":"bad","sev":"unknown"}"#,
                0,
                3_000_000_000
            ),
            Err(DataError::Body)
        );
    }

    /// Scenario: CSV input has headers, malformed quotes, wrong counts or embedded newlines.
    /// Guarantees: Rows are independently rejected without consuming header state.
    #[test]
    fn rejects_malformed_csv_and_partial_regex() {
        let parser = Parser::new(serde_json::from_value(parser_config("csv")).unwrap()).unwrap();
        for input in [
            "ts,sev,message",
            "one,two",
            "one,two,three,four",
            "\"unterminated",
            "one,two,\"bad\"tail",
            "one,two,bad\"quote",
            "one,two,\"embedded\nnewline\"",
        ] {
            assert!(parser.parse(input, 0, 3_000_000_000).is_err(), "{input}");
        }
        let parser = Parser::new(serde_json::from_value(parser_config("regex")).unwrap()).unwrap();
        assert_eq!(
            parser.parse("prefix START: 1970-01-01T00:00:02Z ERROR failure", 0, 0),
            Err(DataError::Extraction)
        );
    }

    /// Scenario: Each parser is configured exactly at and one byte below its input and scratch reservation.
    /// Guarantees: Exact limits succeed, smaller limits preserve input, and many-capture regex caches are included.
    #[test]
    fn exact_input_and_scratch_limits() {
        for (format, input) in [
            ("regex", "START: 1970-01-01T00:00:02Z ERROR failure"),
            (
                "json",
                r#"{"message":"failure","ts":"1970-01-01T00:00:02Z","sev":"ERROR"}"#,
            ),
            ("csv", "1970-01-01T00:00:02Z,ERROR,failure"),
        ] {
            let mut parser =
                Parser::new(serde_json::from_value(parser_config(format)).unwrap()).unwrap();
            parser.config.limits.max_input_bytes = NonZeroUsize::new(input.len()).unwrap();
            let bound = parser.scratch_bound(input).unwrap();
            parser.config.limits.max_scratch_bytes = NonZeroUsize::new(bound).unwrap();
            assert!(parser.parse(input, 0, 0).is_ok(), "{format}");
            parser.config.limits.max_scratch_bytes = NonZeroUsize::new(bound - 1).unwrap();
            assert_eq!(parser.parse(input, 0, 0), Err(DataError::Limit));
            parser.config.limits.max_scratch_bytes = NonZeroUsize::new(bound).unwrap();
            parser.config.limits.max_input_bytes = NonZeroUsize::new(input.len() - 1).unwrap();
            assert_eq!(parser.parse(input, 0, 0), Err(DataError::Limit));
        }
        let mut config = config("regex");
        config["pattern"] = "(a?)".repeat(40).into();
        let parser = Parser::new(serde_json::from_value(config).unwrap()).unwrap();
        let regex = parser.regex.as_ref().unwrap();
        let cache = regex.create_cache();
        assert!(cache.memory_usage() < parser.scratch_bound("").unwrap());
    }

    /// Scenario: A headerless single-column CSV record is an empty string.
    /// Guarantees: The field is present and can map to an empty body without being treated as missing.
    #[test]
    fn empty_csv_field_is_present() {
        let mut config = config("csv");
        config["columns"] = serde_json::json!(["body"]);
        config["delimiter"] = ",".into();
        config["header"] = "none".into();
        config["body"] = serde_json::json!({"source":"body"});
        let parser = Parser::new(serde_json::from_value(config).unwrap()).unwrap();
        assert_eq!(parser.parse("", 0, 0).unwrap().body.as_deref(), Some(""));
    }

    /// Scenario: JSON timestamp values are missing, null, empty or malformed under preserve fallback.
    /// Guarantees: Only missing values preserve event time while malformed values discard the whole candidate.
    #[test]
    fn missing_timestamp_preserve_policy() {
        let mut config = parser_config("json");
        config["timestamp"]["on_missing"] = "preserve".into();
        let parser = Parser::new(serde_json::from_value(config).unwrap()).unwrap();
        for input in [
            r#"{"message":"ok","sev":"ERROR"}"#,
            r#"{"message":"ok","sev":"ERROR","ts":null}"#,
        ] {
            let result = parser.parse(input, 0, 0).unwrap();
            assert_eq!(result.timestamp, None);
            assert!(!result.fallback);
            assert_eq!(result.body.as_deref(), Some("ok"));
        }
        for timestamp in ["", "bad", "1970-01-01T00:00:02"] {
            let input =
                serde_json::json!({"message":"ok","sev":"ERROR","ts":timestamp}).to_string();
            assert_eq!(
                parser.parse(&input, 2_000_000_000, 3_000_000_000),
                Err(DataError::Timestamp)
            );
        }
    }

    /// Scenario: Parsing configurations contain unknown keys, zero limits or invalid references.
    /// Guarantees: Invalid configuration is rejected before processing input.
    #[test]
    fn invalid_configuration_is_rejected() {
        let mut value = config("json");
        value["unknown"] = true.into();
        assert!(serde_json::from_value::<ParseConfig>(value).is_err());
        let mut value = config("json");
        value["limits"]["max_input_bytes"] = 0.into();
        assert!(serde_json::from_value::<ParseConfig>(value).is_err());
        for source in ["dotted.path", "/bad~", "/bad~2"] {
            let mut value = config("json");
            value["body"] = serde_json::json!({"source": source});
            assert!(
                serde_json::from_value::<ParseConfig>(value)
                    .unwrap()
                    .validate()
                    .is_err()
            );
        }
        let mut value = config("regex");
        value["pattern"] = "(?P<message>.*)".into();
        value["body"] = serde_json::json!({"source": "absent"});
        assert!(Parser::new(serde_json::from_value(value).unwrap()).is_err());
        let mut value = config("csv");
        value["columns"] = serde_json::json!(["body", "body"]);
        value["delimiter"] = ",".into();
        value["header"] = "none".into();
        assert!(
            serde_json::from_value::<ParseConfig>(value)
                .unwrap()
                .validate()
                .is_err()
        );
    }

    /// Scenario: RFC 6901 pointers select escaped keys, array indices or the document root.
    /// Guarantees: Pointer syntax accepts only the two defined tilde escapes.
    #[test]
    fn json_pointer_syntax() {
        for pointer in ["", "/a~1b", "/a~0b", "/nested/0/value", "/"] {
            assert!(valid_pointer(pointer));
        }
    }

    #[test]
    /// Scenario: RFC 3339 inputs express equivalent offsets and nanosecond precision.
    /// Guarantees: Event time is an exact unsigned Unix nanosecond count.
    fn timestamp_offsets_and_precision() {
        assert_eq!(parse_timestamp("1970-01-01T00:00:02Z"), Some(2_000_000_000));
        assert_eq!(
            parse_timestamp("1970-01-01T01:00:02+01:00"),
            Some(2_000_000_000)
        );
        assert_eq!(
            parse_timestamp("1970-01-01T00:00:02.123456789Z"),
            Some(2_123_456_789)
        );
        assert_eq!(parse_timestamp("1970-01-01T00:00:00Z"), Some(0));
    }

    #[test]
    /// Scenario: Source timestamps lack required precision, range, or timezone validity.
    /// Guarantees: Invalid timestamps are rejected without rounding or timezone inference.
    fn timestamp_rejects_prohibited_values() {
        for value in [
            "",
            "1970-01-01T00:00:02",
            "1969-12-31T23:59:59.999999999Z",
            "2016-12-31T23:59:60Z",
            "1970-01-01T00:00:02.1234567890Z",
            "1970-01-01T00:00:02.Z",
            "9999-12-31T23:59:59Z",
        ] {
            assert_eq!(parse_timestamp(value), None, "{value}");
        }
    }
}
