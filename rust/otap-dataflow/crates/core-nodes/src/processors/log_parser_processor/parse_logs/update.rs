// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Staged native OTAP field updates.

use super::{Candidate, DataError, Parser};
use arrow::{
    array::{
        Array, ArrayRef, DictionaryArray, Int32Array, RecordBatch, StringArray, StringBuilder,
        StructArray, TimestampNanosecondArray, UInt8Array, UInt16Array,
    },
    compute::cast,
    datatypes::{ArrowDictionaryKeyType, DataType, Field, Schema, TimeUnit, UInt8Type, UInt16Type},
};
use otel_arrow_dfe_pdata::arrays::{MaybeDictArrayAccessor, NullableArrayAccessor};
use otel_arrow_dfe_pdata::{OtapArrowRecords, error::Result, schema::consts};
use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
    sync::Arc,
};

#[derive(Default)]
pub(in super::super) struct Counts([u64; 7]);

#[derive(Default)]
struct StagedCandidate {
    body: Option<Rc<String>>,
    timestamp: Option<u64>,
    severity: Option<(Rc<String>, u8)>,
}

impl Counts {
    fn record(&mut self, reason: DataError) {
        self.0[reason as usize] += 1;
    }

    pub(in super::super) fn values(self) -> impl Iterator<Item = (DataError, u64)> {
        [
            DataError::Limit,
            DataError::Extraction,
            DataError::Body,
            DataError::Timestamp,
            DataError::Severity,
            DataError::UnsupportedBody,
            DataError::ObservedFallback,
        ]
        .into_iter()
        .zip(self.0)
    }
}

impl Parser {
    pub(in super::super) fn apply(
        &self,
        batch: OtapArrowRecords,
    ) -> Result<(OtapArrowRecords, Counts)> {
        #[cfg(test)]
        let fail_after = self.fail_after;
        #[cfg(not(test))]
        let fail_after = None;
        self.apply_inner(batch, fail_after)
    }

    fn apply_inner(
        &self,
        mut batch: OtapArrowRecords,
        fail_after: Option<usize>,
    ) -> Result<(OtapArrowRecords, Counts)> {
        let mut counts = Counts::default();
        if !matches!(batch, OtapArrowRecords::Logs(_)) {
            return Ok((batch, counts));
        }
        let Some(root) = batch.root_record_batch() else {
            return Ok((batch, counts));
        };
        let Some(body) = root
            .column_by_name(consts::BODY)
            .and_then(|array| array.as_any().downcast_ref::<StructArray>())
        else {
            counts.0[DataError::UnsupportedBody as usize] = root.num_rows() as u64;
            return Ok((batch, counts));
        };
        let time_type = DataType::Timestamp(TimeUnit::Nanosecond, None);
        let event = column(root, consts::TIME_UNIX_NANO, &time_type)?;
        let observed = column(root, consts::OBSERVED_TIME_UNIX_NANO, &time_type)?;
        let event = event
            .as_any()
            .downcast_ref::<TimestampNanosecondArray>()
            .expect("timestamp cast");
        let observed = observed
            .as_any()
            .downcast_ref::<TimestampNanosecondArray>()
            .expect("timestamp cast");
        let (candidates, counts) = self.stage_candidates(body, event, observed, fail_after)?;
        let mut output = root.clone();
        if candidates.iter().any(|candidate| candidate.body.is_some()) {
            let replacement = update_strings(
                body.column_by_name(consts::ATTRIBUTE_STR),
                candidates
                    .iter()
                    .map(|candidate| candidate.body.as_deref().map(String::as_str)),
            )?;
            let mut fields = body.fields().to_vec();
            let mut columns = body.columns().to_vec();
            upsert(
                &mut fields,
                &mut columns,
                consts::ATTRIBUTE_STR,
                replacement,
            );
            let replacement = StructArray::try_new(fields.into(), columns, body.nulls().cloned())
                .map_err(arrow_error)?;
            output = replace(&output, consts::BODY, Arc::new(replacement))?;
        }
        if candidates
            .iter()
            .any(|candidate| candidate.timestamp.is_some())
        {
            let replacement = TimestampNanosecondArray::from_iter(
                candidates.iter().enumerate().map(|(row, candidate)| {
                    candidate
                        .timestamp
                        .map(|value| value as i64)
                        .or_else(|| event.is_valid(row).then(|| event.value(row)))
                }),
            );
            output = replace(&output, consts::TIME_UNIX_NANO, Arc::new(replacement))?;
        }
        if candidates
            .iter()
            .any(|candidate| candidate.severity.is_some())
        {
            let numbers = column(root, consts::SEVERITY_NUMBER, &DataType::Int32)?;
            let numbers = numbers
                .as_any()
                .downcast_ref::<Int32Array>()
                .expect("severity cast");
            let texts = update_strings(
                root.column_by_name(consts::SEVERITY_TEXT),
                candidates
                    .iter()
                    .map(|candidate| candidate.severity.as_ref().map(|(text, _)| text.as_str())),
            )?;
            let replacement =
                Int32Array::from_iter(candidates.iter().enumerate().map(|(row, candidate)| {
                    candidate
                        .severity
                        .as_ref()
                        .map(|(_, number)| i32::from(*number))
                        .or_else(|| numbers.is_valid(row).then(|| numbers.value(row)))
                }));
            output = replace(&output, consts::SEVERITY_NUMBER, Arc::new(replacement))?;
            output = replace(&output, consts::SEVERITY_TEXT, texts)?;
        }
        batch.set(batch.root_payload_type(), output)?;
        Ok((batch, counts))
    }

    fn stage_candidates(
        &self,
        body: &StructArray,
        event: &TimestampNanosecondArray,
        observed: &TimestampNanosecondArray,
        fail_after: Option<usize>,
    ) -> Result<(Vec<StagedCandidate>, Counts)> {
        let types = body
            .column_by_name(consts::ATTRIBUTE_TYPE)
            .map(MaybeDictArrayAccessor::<UInt8Array>::try_new)
            .transpose()?;
        let strings = body
            .column_by_name(consts::ATTRIBUTE_STR)
            .map(MaybeDictArrayAccessor::<StringArray>::try_new)
            .transpose()?;
        let mut counts = Counts::default();
        let mut staged_strings = HashSet::<Rc<String>>::new();
        let mut intern = |value: String| {
            if let Some(shared) = staged_strings.get(&value) {
                return Rc::clone(shared);
            }
            let shared = Rc::new(value);
            let _ = staged_strings.insert(Rc::clone(&shared));
            shared
        };
        let mut candidates = Vec::with_capacity(body.len());
        for row in 0..body.len() {
            if fail_after == Some(row) {
                return Err(arrow_error(arrow::error::ArrowError::ComputeError(
                    "injected staging failure".into(),
                )));
            }
            let string = types
                .as_ref()
                .zip(strings.as_ref())
                .filter(|(types, _)| body.is_valid(row) && types.value_at(row) == Some(1))
                .and_then(|(_, strings)| strings.str_at(row));
            let candidate = match string {
                None => Err(DataError::UnsupportedBody),
                Some(input) => self.parse(
                    input,
                    if event.is_valid(row) {
                        event.value(row) as u64
                    } else {
                        0
                    },
                    if observed.is_valid(row) {
                        observed.value(row) as u64
                    } else {
                        0
                    },
                ),
            };
            match candidate {
                Ok(Candidate {
                    body,
                    timestamp,
                    severity,
                    fallback,
                }) => {
                    if fallback {
                        counts.record(DataError::ObservedFallback);
                    }
                    candidates.push(StagedCandidate {
                        body: body.map(&mut intern),
                        timestamp,
                        severity: severity.map(|(text, number)| (intern(text), number)),
                    });
                }
                Err(reason) => {
                    counts.record(reason);
                    candidates.push(StagedCandidate::default());
                }
            }
        }
        Ok((candidates, counts))
    }
}

fn update_strings<'a>(
    original: Option<&'a ArrayRef>,
    updates: impl ExactSizeIterator<Item = Option<&'a str>> + Clone,
) -> Result<ArrayRef> {
    let strings = original
        .map(MaybeDictArrayAccessor::<StringArray>::try_new)
        .transpose()?;
    if let Some(original) = original {
        if let Some(dictionary) = original
            .as_any()
            .downcast_ref::<DictionaryArray<UInt8Type>>()
        {
            return update_dictionary_strings(dictionary, updates);
        }
        if let Some(dictionary) = original
            .as_any()
            .downcast_ref::<DictionaryArray<UInt16Type>>()
        {
            return update_dictionary_strings(dictionary, updates);
        }
    }
    let values = updates.enumerate().map(|(row, update)| {
        update.or_else(|| strings.as_ref().and_then(|strings| strings.str_at(row)))
    });
    let bytes = values
        .clone()
        .flatten()
        .try_fold(0, |bytes, value| checked_string_bytes(bytes, value.len()))?;
    let mut builder = StringBuilder::with_capacity(values.len(), bytes);
    builder.extend(values);
    Ok(Arc::new(builder.finish()))
}

fn update_dictionary_strings<'a, Key: ArrowDictionaryKeyType>(
    original: &'a DictionaryArray<Key>,
    updates: impl ExactSizeIterator<Item = Option<&'a str>>,
) -> Result<ArrayRef> {
    let original_values = original
        .values()
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("validated string dictionary");
    let mut remapped = vec![None; original_values.len()];
    let mut values = Vec::new();
    let mut indices = HashMap::new();
    let mut bytes = 0;
    let mut intern = |value: &'a str| -> Result<u16> {
        if let Some(&key) = indices.get(value) {
            return Ok(key);
        }
        let key = u16::try_from(values.len())
            .map_err(|_| arrow_error(arrow::error::ArrowError::DictionaryKeyOverflowError))?;
        bytes = checked_string_bytes(bytes, value.len())?;
        values.push(value);
        let _ = indices.insert(value, key);
        Ok(key)
    };
    let keys = updates
        .enumerate()
        .map(|(row, update)| {
            if let Some(value) = update {
                return intern(value).map(Some);
            }
            let Some(original_key) = original.key(row) else {
                return Ok(None);
            };
            if original_values.is_null(original_key) {
                return Ok(None);
            }
            if let Some(key) = remapped[original_key] {
                return Ok(Some(key));
            }
            let key = intern(original_values.value(original_key))?;
            remapped[original_key] = Some(key);
            Ok(Some(key))
        })
        .collect::<Result<Vec<_>>>()?;
    let mut builder = StringBuilder::with_capacity(values.len(), bytes);
    for value in values {
        builder.append_value(value);
    }
    let values = Arc::new(builder.finish());
    if Key::DATA_TYPE == DataType::UInt8 && values.len() <= usize::from(u8::MAX) + 1 {
        let keys = UInt8Array::from_iter(keys.into_iter().map(|key| key.map(|key| key as u8)));
        Ok(Arc::new(
            DictionaryArray::<UInt8Type>::try_new(keys, values).map_err(arrow_error)?,
        ))
    } else {
        Ok(Arc::new(
            DictionaryArray::<UInt16Type>::try_new(UInt16Array::from(keys), values)
                .map_err(arrow_error)?,
        ))
    }
}

fn checked_string_bytes(bytes: usize, additional: usize) -> Result<usize> {
    bytes
        .checked_add(additional)
        .filter(|&total| total <= i32::MAX as usize)
        .ok_or_else(|| {
            arrow_error(arrow::error::ArrowError::ComputeError(
                "log parser string output exceeds Arrow's i32 offset limit".into(),
            ))
        })
}

fn column(batch: &RecordBatch, name: &str, data_type: &DataType) -> Result<ArrayRef> {
    Ok(match batch.column_by_name(name) {
        Some(array) => cast(array, data_type).map_err(arrow_error)?,
        None => arrow::array::new_null_array(data_type, batch.num_rows()),
    })
}

fn upsert(fields: &mut Vec<Arc<Field>>, columns: &mut Vec<ArrayRef>, name: &str, array: ArrayRef) {
    match fields.iter().position(|field| field.name() == name) {
        Some(index) => {
            fields[index] = Arc::new(
                fields[index]
                    .as_ref()
                    .clone()
                    .with_data_type(array.data_type().clone()),
            );
            columns[index] = array;
        }
        None => {
            fields.push(Arc::new(Field::new(name, array.data_type().clone(), true)));
            columns.push(array);
        }
    }
}

fn replace(batch: &RecordBatch, name: &str, array: ArrayRef) -> Result<RecordBatch> {
    let mut fields = batch.schema().fields().to_vec();
    let mut columns = batch.columns().to_vec();
    upsert(&mut fields, &mut columns, name, array);
    RecordBatch::try_new(
        Arc::new(Schema::new_with_metadata(
            fields,
            batch.schema().metadata().clone(),
        )),
        columns,
    )
    .map_err(arrow_error)
}

fn arrow_error(source: arrow::error::ArrowError) -> otel_arrow_dfe_pdata::error::Error {
    otel_arrow_dfe_pdata::error::Error::WriteRecordBatch { source }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_pdata::{
        otap::transform::sanitize::sanitize_otap_batch,
        proto::opentelemetry::{common::v1::AnyValue, logs::v1::LogRecord},
        testing::round_trip::to_otap_logs,
    };

    /// Scenario: Many rows reference one valid dictionary body with large mapped strings.
    /// Guarantees: Staging retains one allocation per distinct mapped string, not per row.
    #[test]
    fn candidate_staging_shares_repeated_strings() {
        let text = "x".repeat(16 * 1024);
        let input = serde_json::json!({"body": text, "sev": text}).to_string();
        let rows = 128;
        let strings = DictionaryArray::<UInt16Type>::try_new(
            UInt16Array::from(vec![0; rows]),
            Arc::new(StringArray::from(vec![input.as_str()])),
        )
        .unwrap();
        let body = StructArray::from(vec![
            (
                Arc::new(Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false)),
                Arc::new(UInt8Array::from(vec![1; rows])) as ArrayRef,
            ),
            (
                Arc::new(Field::new(
                    consts::ATTRIBUTE_STR,
                    strings.data_type().clone(),
                    true,
                )),
                Arc::new(strings) as ArrayRef,
            ),
        ]);
        let config = serde_json::json!({"format":"json", "body":{"source":"/body"},
            "severity":{"source":"/sev", "mapping":{text.clone():9}}, "on_error":"preserve",
            "limits":{"max_input_bytes":65536,"max_scratch_bytes":1048576,"max_pattern_bytes":100,
                "max_compiled_regex_bytes":1024,"max_json_depth":4,"max_entries":10}});
        let parser = Parser::new(serde_json::from_value(config).unwrap()).unwrap();
        let times = TimestampNanosecondArray::from(vec![0; rows]);
        let (candidates, counts) = parser
            .stage_candidates(&body, &times, &times, None)
            .unwrap();
        assert_eq!(counts.0, [0; 7]);
        let mut allocations = HashSet::new();
        for candidate in &candidates {
            let body = candidate.body.as_ref().unwrap().as_str();
            let (severity, number) = candidate.severity.as_ref().unwrap();
            assert_eq!(body, text);
            assert_eq!(severity.as_str(), text);
            assert_eq!(*number, 9);
            let _ = allocations.insert(body.as_ptr());
            let _ = allocations.insert(severity.as_ptr());
        }
        assert_eq!(
            allocations.len(),
            1,
            "staged strings must share their allocation"
        );
    }

    /// Scenario: Shared bodies have distinct timestamps, a missing fallback, and unsupported rows.
    /// Guarantees: Text sharing preserves per-row fallbacks, counters and all-or-nothing mappings.
    #[test]
    fn candidate_staging_preserves_row_specific_outcomes() {
        let strings = DictionaryArray::<UInt16Type>::try_new(
            UInt16Array::from(vec![Some(0), Some(0), Some(0), Some(0), Some(0), None]),
            Arc::new(StringArray::from(vec![
                r#"{"body":"changed","sev":"INFO"}"#,
            ])),
        )
        .unwrap();
        let body = StructArray::from(vec![
            (
                Arc::new(Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false)),
                Arc::new(UInt8Array::from(vec![1, 1, 1, 1, 2, 1])) as ArrayRef,
            ),
            (
                Arc::new(Field::new(
                    consts::ATTRIBUTE_STR,
                    strings.data_type().clone(),
                    true,
                )),
                Arc::new(strings) as ArrayRef,
            ),
        ]);
        let config = serde_json::json!({"format":"json", "body":{"source":"/body"},
            "timestamp":{"source":"/ts","format":"rfc3339","on_missing":"observed"},
            "severity":{"source":"/sev","mapping":{"INFO":9}}, "on_error":"preserve",
            "limits":{"max_input_bytes":1024,"max_scratch_bytes":8192,"max_pattern_bytes":100,
                "max_compiled_regex_bytes":1024,"max_json_depth":4,"max_entries":10}});
        let parser = Parser::new(serde_json::from_value(config).unwrap()).unwrap();
        let event = TimestampNanosecondArray::from(vec![11, 0, 0, 0, 0, 0]);
        let observed = TimestampNanosecondArray::from(vec![101, 202, 0, 404, 505, 606]);
        let (candidates, counts) = parser
            .stage_candidates(&body, &event, &observed, None)
            .unwrap();
        assert_eq!(counts.0, [0, 0, 0, 1, 0, 2, 2]);
        assert_eq!(candidates[0].timestamp, None);
        assert_eq!(candidates[1].timestamp, Some(202));
        assert_eq!(candidates[3].timestamp, Some(404));
        for row in [0, 1, 3] {
            let candidate = &candidates[row];
            let text = candidate.body.as_ref().unwrap();
            assert_eq!(text.as_str(), "changed");
            assert!(Rc::ptr_eq(text, candidates[0].body.as_ref().unwrap()));
            let (severity, number) = candidate.severity.as_ref().unwrap();
            assert_eq!(severity.as_str(), "INFO");
            assert_eq!(*number, 9);
            assert!(Rc::ptr_eq(
                severity,
                &candidates[0].severity.as_ref().unwrap().0
            ));
        }
        for row in [2, 4, 5] {
            let candidate = &candidates[row];
            assert!(candidate.body.is_none());
            assert!(candidate.timestamp.is_none());
            assert!(candidate.severity.is_none());
        }
    }

    /// Scenario: One valid body changes beside repeated dictionary-backed malformed bodies.
    /// Guarantees: Body values stay shared and logically intact, including after sanitization.
    #[test]
    fn body_update_preserves_shared_unchanged_values() {
        let large_body = "x".repeat(16 * 1024);
        let repeated_rows = 128;
        let mut keys = vec![0; repeated_rows];
        keys.push(1);
        let dictionary = DictionaryArray::<UInt16Type>::try_new(
            UInt16Array::from(keys),
            Arc::new(StringArray::from(vec![
                large_body.as_str(),
                r#"{"body":"changed"}"#,
            ])),
        )
        .unwrap();
        let mut batch = to_otap_logs(vec![
            LogRecord::build()
                .body(AnyValue::new_string("placeholder"))
                .finish();
            repeated_rows + 1
        ]);
        let root = batch.root_record_batch().unwrap();
        let body = root
            .column_by_name(consts::BODY)
            .unwrap()
            .as_any()
            .downcast_ref::<StructArray>()
            .unwrap();
        let mut fields = body.fields().to_vec();
        let mut columns = body.columns().to_vec();
        upsert(
            &mut fields,
            &mut columns,
            consts::ATTRIBUTE_STR,
            Arc::new(dictionary),
        );
        let body = StructArray::try_new(fields.into(), columns, body.nulls().cloned()).unwrap();
        let root = replace(root, consts::BODY, Arc::new(body)).unwrap();
        batch.set(batch.root_payload_type(), root).unwrap();
        let config = serde_json::json!({"format":"json", "body":{"source":"/body"}, "on_error":"preserve",
            "limits":{"max_input_bytes":16384,"max_scratch_bytes":262144,"max_pattern_bytes":100,
                "max_compiled_regex_bytes":1024,"max_json_depth":4,"max_entries":10}});
        let parser = Parser::new(serde_json::from_value(config).unwrap()).unwrap();
        let (mut output, counts) = parser.apply(batch).unwrap();
        assert_eq!(
            counts.0[DataError::Extraction as usize],
            repeated_rows as u64
        );
        for sanitize in [false, true] {
            if sanitize {
                sanitize_otap_batch(&mut output);
            }
            let body = output
                .root_record_batch()
                .unwrap()
                .column_by_name(consts::BODY)
                .unwrap()
                .as_any()
                .downcast_ref::<StructArray>()
                .unwrap();
            let strings = body.column_by_name(consts::ATTRIBUTE_STR).unwrap();
            let dictionary = strings
                .as_any()
                .downcast_ref::<DictionaryArray<UInt16Type>>()
                .expect("unchanged bodies must stay shared with schema-compatible keys");
            let values = dictionary
                .values()
                .as_any()
                .downcast_ref::<StringArray>()
                .unwrap();
            assert_eq!(values.len(), 2);
            assert_eq!(
                values.value_data().len(),
                large_body.len() + "changed".len()
            );
            let accessor = MaybeDictArrayAccessor::<StringArray>::try_new(strings).unwrap();
            for row in 0..repeated_rows {
                assert_eq!(accessor.str_at(row), Some(large_body.as_str()));
            }
            assert_eq!(accessor.str_at(repeated_rows), Some("changed"));
        }
    }

    /// Scenario: One severity mapping changes beside a repeated large severity and a null text.
    /// Guarantees: Text sharing, nulls and unchanged numbers survive parsing and sanitization.
    #[test]
    fn severity_update_preserves_shared_unchanged_values() {
        let large_text = "x".repeat(16 * 1024);
        let repeated_rows = 128;
        let mut logs = vec![
            LogRecord::build()
                .body(AnyValue::new_string("malformed"))
                .severity_number(17)
                .finish();
            repeated_rows
        ];
        logs.push(
            LogRecord::build()
                .body(AnyValue::new_string(r#"{"sev":"INFO"}"#))
                .severity_number(17)
                .finish(),
        );
        logs.push(
            LogRecord::build()
                .body(AnyValue::new_string("malformed"))
                .severity_number(17)
                .finish(),
        );
        let mut keys = vec![Some(0); repeated_rows];
        keys.extend([Some(1), None]);
        let dictionary = DictionaryArray::<UInt8Type>::try_new(
            UInt8Array::from(keys),
            Arc::new(StringArray::from(vec![large_text.as_str(), "old"])),
        )
        .unwrap();
        let mut batch = to_otap_logs(logs);
        let root = replace(
            batch.root_record_batch().unwrap(),
            consts::SEVERITY_TEXT,
            Arc::new(dictionary),
        )
        .unwrap();
        batch.set(batch.root_payload_type(), root).unwrap();
        let config = serde_json::json!({"format":"json", "severity":{"source":"/sev","mapping":{"INFO":9}},
            "on_error":"preserve", "limits":{"max_input_bytes":1024,"max_scratch_bytes":8192,
                "max_pattern_bytes":100,"max_compiled_regex_bytes":1024,"max_json_depth":4,"max_entries":10}});
        let parser = Parser::new(serde_json::from_value(config).unwrap()).unwrap();
        let (mut output, counts) = parser.apply(batch).unwrap();
        assert_eq!(
            counts.0[DataError::Extraction as usize],
            (repeated_rows + 1) as u64
        );
        for sanitize in [false, true] {
            if sanitize {
                sanitize_otap_batch(&mut output);
            }
            let root = output.root_record_batch().unwrap();
            let strings = root.column_by_name(consts::SEVERITY_TEXT).unwrap();
            let dictionary = strings
                .as_any()
                .downcast_ref::<DictionaryArray<UInt8Type>>()
                .expect("unchanged severity texts must stay shared");
            let values = dictionary
                .values()
                .as_any()
                .downcast_ref::<StringArray>()
                .unwrap();
            assert_eq!(values.len(), 2);
            assert_eq!(values.value_data().len(), large_text.len() + "INFO".len());
            let texts = MaybeDictArrayAccessor::<StringArray>::try_new(strings).unwrap();
            let numbers = column(root, consts::SEVERITY_NUMBER, &DataType::Int32).unwrap();
            let numbers = numbers.as_any().downcast_ref::<Int32Array>().unwrap();
            for row in 0..repeated_rows {
                assert_eq!(texts.str_at(row), Some(large_text.as_str()));
                assert_eq!(numbers.value(row), 17);
            }
            assert_eq!(texts.str_at(repeated_rows), Some("INFO"));
            assert_eq!(numbers.value(repeated_rows), 9);
            assert_eq!(texts.str_at(repeated_rows + 1), None);
            assert_eq!(numbers.value(repeated_rows + 1), 17);
        }
    }

    /// Scenario: Sliced native and dictionary strings contain null keys, null values and empty text.
    /// Guarantees: Updates preserve logical nulls and row order without narrowing dictionary keys.
    #[test]
    fn string_updates_preserve_nulls_slices_and_key_width() {
        let values = Arc::new(StringArray::from(vec![
            Some("old"),
            None,
            Some("kept"),
            Some(""),
        ]));
        let keys = vec![
            Some(0),
            Some(0),
            None,
            Some(1),
            Some(2),
            Some(0),
            Some(3),
            None,
        ];
        let arrays: Vec<ArrayRef> = vec![
            Arc::new(StringArray::from(vec![
                Some("old"),
                Some("old"),
                None,
                None,
                Some("kept"),
                Some("old"),
                Some(""),
                None,
            ])),
            Arc::new(
                DictionaryArray::<UInt8Type>::try_new(
                    UInt8Array::from(keys.clone()),
                    values.clone(),
                )
                .unwrap(),
            ),
            Arc::new(
                DictionaryArray::<UInt16Type>::try_new(
                    UInt16Array::from_iter(keys.into_iter().map(|key| key.map(u16::from))),
                    values,
                )
                .unwrap(),
            ),
        ];
        let updates = [Some("new"), Some("new"), None, None, None, None, None];
        let expected = [
            Some("new"),
            Some("new"),
            None,
            Some("kept"),
            Some("old"),
            Some(""),
            None,
        ];
        for original in arrays {
            let original = original.slice(1, updates.len());
            let output = update_strings(Some(&original), updates.into_iter()).unwrap();
            assert_eq!(output.data_type(), original.data_type());
            let strings = MaybeDictArrayAccessor::<StringArray>::try_new(&output).unwrap();
            let actual: Vec<_> = (0..output.len()).map(|row| strings.str_at(row)).collect();
            assert_eq!(actual, expected);
        }
        let output = update_strings(None, [None, Some("new"), None].into_iter()).unwrap();
        assert_eq!(
            output.as_any().downcast_ref::<StringArray>().unwrap(),
            &StringArray::from(vec![None, Some("new"), None])
        );
    }

    /// Scenario: Updated dictionary cardinality crosses the supported unsigned key boundaries.
    /// Guarantees: Eight-bit keys promote to sixteen bits; overflow fails without a native fallback.
    #[test]
    fn string_dictionary_key_boundaries_are_checked() {
        for cardinality in [256, 257, 65536, 65537] {
            let original: ArrayRef = Arc::new(
                DictionaryArray::<UInt8Type>::try_new(
                    UInt8Array::from(vec![0; cardinality]),
                    Arc::new(StringArray::from(vec!["old"])),
                )
                .unwrap(),
            );
            let updates: Vec<_> = (0..cardinality).map(|value| value.to_string()).collect();
            let result = update_strings(
                Some(&original),
                updates.iter().map(|value| Some(value.as_str())),
            );
            if cardinality == 65537 {
                assert!(matches!(
                    result,
                    Err(otel_arrow_dfe_pdata::error::Error::WriteRecordBatch {
                        source: arrow::error::ArrowError::DictionaryKeyOverflowError,
                    })
                ));
                continue;
            }
            let output = result.unwrap();
            let key_type = if cardinality <= 256 {
                DataType::UInt8
            } else {
                DataType::UInt16
            };
            assert_eq!(
                output.data_type(),
                &DataType::Dictionary(Box::new(key_type), Box::new(DataType::Utf8))
            );
            let strings = MaybeDictArrayAccessor::<StringArray>::try_new(&output).unwrap();
            for (row, expected) in updates.iter().enumerate() {
                assert_eq!(strings.str_at(row), Some(expected.as_str()));
            }
        }
    }

    /// Scenario: Output size calculations meet or exceed Arrow's string-offset and usize limits.
    /// Guarantees: Oversized output is rejected by preflight without allocating a value buffer.
    #[test]
    fn string_output_size_rejects_overflow() {
        let maximum = i32::MAX as usize;
        assert_eq!(checked_string_bytes(0, maximum).unwrap(), maximum);
        assert_eq!(checked_string_bytes(maximum - 1, 1).unwrap(), maximum);
        assert!(checked_string_bytes(maximum, 1).is_err());
        assert!(checked_string_bytes(0, maximum + 1).is_err());
        assert!(checked_string_bytes(usize::MAX, 1).is_err());
    }

    /// Scenario: An internal failure occurs after one record has a valid staged candidate.
    /// Guarantees: No updated batch is returned and the original Arrow buffers remain unchanged.
    #[test]
    fn internal_failure_does_not_return_partial_batch() {
        let config = serde_json::json!({"format":"json", "body":{"source":"/body"}, "on_error":"preserve",
            "limits":{"max_input_bytes":1024,"max_scratch_bytes":8192,"max_pattern_bytes":100,
                "max_compiled_regex_bytes":1024,"max_json_depth":4,"max_entries":10}});
        let parser = Parser::new(serde_json::from_value(config).unwrap()).unwrap();
        let batch = to_otap_logs(vec![
            LogRecord::build()
                .body(AnyValue::new_string(r#"{"body":"changed"}"#))
                .finish();
            2
        ]);
        let original = batch.clone();
        assert!(parser.apply_inner(batch.clone(), Some(1)).is_err());
        assert_eq!(batch, original);
    }

    /// Scenario: Log bodies are bytes or maps and a non-log Arrow payload enters parsing mode.
    /// Guarantees: Unsupported bodies and signals pass through exactly, with counters only for log bodies.
    #[test]
    fn unsupported_bodies_and_signals_are_unchanged() {
        use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{KeyValueList, any_value};
        let config = serde_json::json!({"format":"json", "body":{"source":"/body"}, "on_error":"preserve",
            "limits":{"max_input_bytes":1024,"max_scratch_bytes":8192,"max_pattern_bytes":100,
                "max_compiled_regex_bytes":1024,"max_json_depth":4,"max_entries":10}});
        let parser = Parser::new(serde_json::from_value(config).unwrap()).unwrap();
        let batch = to_otap_logs(vec![
            LogRecord::build()
                .body(AnyValue {
                    value: Some(any_value::Value::BytesValue(vec![0xff, 0x00])),
                })
                .finish(),
            LogRecord::build()
                .body(AnyValue {
                    value: Some(any_value::Value::KvlistValue(KeyValueList {
                        values: vec![],
                    })),
                })
                .finish(),
        ]);
        let (output, counts) = parser.apply(batch.clone()).unwrap();
        assert_eq!(output, batch);
        assert_eq!(counts.0[DataError::UnsupportedBody as usize], 2);
        let batch = OtapArrowRecords::Metrics(Default::default());
        let (output, counts) = parser.apply(batch.clone()).unwrap();
        assert_eq!(output, batch);
        assert_eq!(counts.0, [0; 7]);
    }
}
