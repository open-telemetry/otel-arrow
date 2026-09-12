//! Staged native OTAP field updates.

use super::{Candidate, DataError, Parser};
use arrow::{
    array::{
        Array, ArrayRef, Int32Array, RecordBatch, StringArray, StructArray,
        TimestampNanosecondArray, UInt8Array,
    },
    compute::cast,
    datatypes::{DataType, Field, Schema, TimeUnit},
};
use otel_arrow_dfe_pdata::arrays::{MaybeDictArrayAccessor, NullableArrayAccessor};
use otel_arrow_dfe_pdata::{OtapArrowRecords, error::Result, schema::consts};
use std::sync::Arc;

#[derive(Default)]
pub(in super::super) struct Counts([u64; 7]);

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
        let types = body
            .column_by_name(consts::ATTRIBUTE_TYPE)
            .map(MaybeDictArrayAccessor::<UInt8Array>::try_new)
            .transpose()?;
        let strings = body
            .column_by_name(consts::ATTRIBUTE_STR)
            .map(MaybeDictArrayAccessor::<StringArray>::try_new)
            .transpose()?;
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
        let mut candidates = Vec::with_capacity(root.num_rows());
        for row in 0..root.num_rows() {
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
                Ok(candidate) => {
                    if candidate.fallback {
                        counts.record(DataError::ObservedFallback);
                    }
                    candidates.push(candidate);
                }
                Err(reason) => {
                    counts.record(reason);
                    candidates.push(Candidate::default());
                }
            }
        }
        let mut output = root.clone();
        if candidates.iter().any(|candidate| candidate.body.is_some()) {
            let replacement =
                StringArray::from_iter(candidates.iter().enumerate().map(|(row, candidate)| {
                    candidate
                        .body
                        .as_deref()
                        .or_else(|| strings.as_ref().and_then(|strings| strings.str_at(row)))
                }));
            let mut fields = body.fields().to_vec();
            let mut columns = body.columns().to_vec();
            upsert(
                &mut fields,
                &mut columns,
                consts::ATTRIBUTE_STR,
                Arc::new(replacement),
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
            let texts = column(root, consts::SEVERITY_TEXT, &DataType::Utf8)?;
            let texts = texts
                .as_any()
                .downcast_ref::<StringArray>()
                .expect("text cast");
            let replacement =
                Int32Array::from_iter(candidates.iter().enumerate().map(|(row, candidate)| {
                    candidate
                        .severity
                        .as_ref()
                        .map(|(_, number)| i32::from(*number))
                        .or_else(|| numbers.is_valid(row).then(|| numbers.value(row)))
                }));
            output = replace(&output, consts::SEVERITY_NUMBER, Arc::new(replacement))?;
            let replacement =
                StringArray::from_iter(candidates.iter().enumerate().map(|(row, candidate)| {
                    candidate
                        .severity
                        .as_ref()
                        .map(|(text, _)| text.as_str())
                        .or_else(|| texts.is_valid(row).then(|| texts.value(row)))
                }));
            output = replace(&output, consts::SEVERITY_TEXT, Arc::new(replacement))?;
        }
        batch.set(batch.root_payload_type(), output)?;
        Ok((batch, counts))
    }
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
        proto::opentelemetry::{common::v1::AnyValue, logs::v1::LogRecord},
        testing::round_trip::to_otap_logs,
    };

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
