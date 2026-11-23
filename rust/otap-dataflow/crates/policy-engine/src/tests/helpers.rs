// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared test builders and OTLP/CBOR helpers for `policy-engine` unit tests.

use std::sync::Arc;

use arrow::array::{
    Array, BinaryArray, BooleanArray, FixedSizeBinaryArray, Float64Array, Int32Array, Int64Array,
    RecordBatch, StringArray, StructArray, UInt8Array, UInt16Array,
};
use arrow::buffer::NullBuffer;
use arrow::datatypes::{DataType, Field, Schema};
use otel_arrow_dfe_pdata::otap::{Logs, OtapArrowRecords};
use otel_arrow_dfe_pdata::otlp::attributes::AttributeValueType;
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_dfe_pdata::schema::{FieldExt, consts};
use regex_syntax::ParserBuilder;
use regex_syntax::hir::Hir;

use crate::PolicyEngine;
use crate::model::{Policy, ScalarValue};

pub(super) fn compile_engine(policies: &[Policy]) -> PolicyEngine {
    let out = PolicyEngine::compile(policies).expect("PolicyEngine compile");
    assert!(
        out.errors.is_empty(),
        "unexpected policy compile errors: {:?}",
        out.errors
    );
    out.engine
}

pub(super) fn encode_cbor(val: &ciborium::Value) -> Vec<u8> {
    let mut buf = Vec::new();
    ciborium::into_writer(val, &mut buf).unwrap();
    buf
}

pub(super) fn build_attr_batch(
    attr_rows: Vec<(u16, &str, ScalarValue)>,
    cbor_attr_rows: Vec<(u16, &str, ciborium::Value)>,
) -> RecordBatch {
    let total = attr_rows.len() + cbor_attr_rows.len();
    let mut pids = Vec::with_capacity(total);
    let mut keys = Vec::with_capacity(total);
    let mut types = Vec::with_capacity(total);
    let mut strs: Vec<Option<String>> = Vec::with_capacity(total);
    let mut ints: Vec<Option<i64>> = Vec::with_capacity(total);
    let mut doubles: Vec<Option<f64>> = Vec::with_capacity(total);
    let mut bools: Vec<Option<bool>> = Vec::with_capacity(total);
    let mut bytes_vec: Vec<Option<Vec<u8>>> = Vec::with_capacity(total);
    let mut ser_vec: Vec<Option<Vec<u8>>> = Vec::with_capacity(total);

    for (pid, k, v) in attr_rows {
        pids.push(pid);
        keys.push(k);
        ser_vec.push(None);
        match v {
            ScalarValue::String(s) => {
                types.push(AttributeValueType::Str as u8);
                strs.push(Some(s));
                ints.push(None);
                doubles.push(None);
                bools.push(None);
                bytes_vec.push(None);
            }
            ScalarValue::Int(i) => {
                types.push(AttributeValueType::Int as u8);
                strs.push(None);
                ints.push(Some(i));
                doubles.push(None);
                bools.push(None);
                bytes_vec.push(None);
            }
            ScalarValue::Double(d) => {
                types.push(AttributeValueType::Double as u8);
                strs.push(None);
                ints.push(None);
                doubles.push(Some(d));
                bools.push(None);
                bytes_vec.push(None);
            }
            ScalarValue::Bool(b) => {
                types.push(AttributeValueType::Bool as u8);
                strs.push(None);
                ints.push(None);
                doubles.push(None);
                bools.push(Some(b));
                bytes_vec.push(None);
            }
            ScalarValue::Bytes(b) => {
                types.push(AttributeValueType::Bytes as u8);
                strs.push(None);
                ints.push(None);
                doubles.push(None);
                bools.push(None);
                bytes_vec.push(Some(b));
            }
        }
    }

    for (pid, k, cval) in cbor_attr_rows {
        pids.push(pid);
        keys.push(k);
        let vtype = match &cval {
            ciborium::Value::Array(_) => AttributeValueType::Slice as u8,
            _ => AttributeValueType::Map as u8,
        };
        types.push(vtype);
        strs.push(None);
        ints.push(None);
        doubles.push(None);
        bools.push(None);
        bytes_vec.push(None);
        ser_vec.push(Some(encode_cbor(&cval)));
    }

    let schema = Arc::new(Schema::new(vec![
        Field::new(consts::PARENT_ID, DataType::UInt16, false).with_plain_encoding(),
        Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
        Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
        Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
        Field::new(consts::ATTRIBUTE_INT, DataType::Int64, true),
        Field::new(consts::ATTRIBUTE_DOUBLE, DataType::Float64, true),
        Field::new(consts::ATTRIBUTE_BOOL, DataType::Boolean, true),
        Field::new(consts::ATTRIBUTE_BYTES, DataType::Binary, true),
        Field::new(consts::ATTRIBUTE_SER, DataType::Binary, true),
    ]));
    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(UInt16Array::from(pids)),
            Arc::new(StringArray::from(keys)),
            Arc::new(UInt8Array::from(types)),
            Arc::new(StringArray::from(strs)),
            Arc::new(Int64Array::from(ints)),
            Arc::new(Float64Array::from(doubles)),
            Arc::new(BooleanArray::from(bools)),
            Arc::new(BinaryArray::from_opt_vec(
                bytes_vec.iter().map(|b| b.as_deref()).collect(),
            )),
            Arc::new(BinaryArray::from_opt_vec(
                ser_vec.iter().map(|b| b.as_deref()).collect(),
            )),
        ],
    )
    .unwrap()
}

pub(super) fn build_custom_test_batch(
    bodies: Vec<Option<&str>>,
    sev_nums: Vec<i32>,
    sev_texts: Vec<&str>,
    trace_ids: Vec<[u8; 16]>,
    span_ids: Vec<[u8; 8]>,
    attr_rows: Vec<(u16, &str, ScalarValue)>,
    cbor_attr_rows: Vec<(u16, &str, ciborium::Value)>,
) -> OtapArrowRecords {
    let n = bodies.len();
    let ids: Vec<u16> = (0..n as u16).collect();
    let body_types: Vec<u8> = bodies
        .iter()
        .map(|b| {
            if b.is_some() {
                AttributeValueType::Str as u8
            } else {
                AttributeValueType::Empty as u8
            }
        })
        .collect();
    let body_validity: Vec<bool> = bodies.iter().map(Option::is_some).collect();
    let body_struct = StructArray::new(
        vec![
            Arc::new(Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false)),
            Arc::new(Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true)),
        ]
        .into(),
        vec![
            Arc::new(UInt8Array::from(body_types)),
            Arc::new(StringArray::from(bodies)),
        ],
        Some(NullBuffer::from(body_validity)),
    );

    let trace_arr =
        FixedSizeBinaryArray::try_from_iter(trace_ids.iter().map(|b| b.as_slice())).unwrap();
    let span_arr =
        FixedSizeBinaryArray::try_from_iter(span_ids.iter().map(|b| b.as_slice())).unwrap();

    let res_struct = StructArray::new(
        vec![Arc::new(
            Field::new(consts::ID, DataType::UInt16, false).with_plain_encoding(),
        )]
        .into(),
        vec![Arc::new(UInt16Array::from(vec![0u16; n]))],
        None,
    );
    let scope_struct = StructArray::new(
        vec![
            Arc::new(Field::new(consts::ID, DataType::UInt16, false).with_plain_encoding()),
            Arc::new(Field::new(consts::NAME, DataType::Utf8, true)),
            Arc::new(Field::new(consts::VERSION, DataType::Utf8, true)),
        ]
        .into(),
        vec![
            Arc::new(UInt16Array::from(vec![0u16; n])),
            Arc::new(StringArray::from(vec!["test.scope"; n])),
            Arc::new(StringArray::from(vec!["1.0"; n])),
        ],
        None,
    );

    let root_schema = Arc::new(Schema::new(vec![
        Field::new(consts::ID, DataType::UInt16, false).with_plain_encoding(),
        Field::new(consts::BODY, body_struct.data_type().clone(), true),
        Field::new(consts::SEVERITY_NUMBER, DataType::Int32, true),
        Field::new(consts::SEVERITY_TEXT, DataType::Utf8, true),
        Field::new(consts::EVENT_NAME, DataType::Utf8, true),
        Field::new(consts::TRACE_ID, DataType::FixedSizeBinary(16), true),
        Field::new(consts::SPAN_ID, DataType::FixedSizeBinary(8), true),
        Field::new(consts::SCHEMA_URL, DataType::Utf8, true),
        Field::new(consts::RESOURCE, res_struct.data_type().clone(), true),
        Field::new(consts::SCOPE, scope_struct.data_type().clone(), true),
    ]));
    let root_rb = RecordBatch::try_new(
        root_schema,
        vec![
            Arc::new(UInt16Array::from(ids)),
            Arc::new(body_struct),
            Arc::new(Int32Array::from(sev_nums)),
            Arc::new(StringArray::from(sev_texts)),
            Arc::new(StringArray::from(vec!["evt"; n])),
            Arc::new(trace_arr),
            Arc::new(span_arr),
            Arc::new(StringArray::from(vec!["https://otel.io/schemas/1.28.0"; n])),
            Arc::new(res_struct),
            Arc::new(scope_struct),
        ],
    )
    .unwrap();

    let attr_rb = build_attr_batch(attr_rows, cbor_attr_rows);

    let mut records = OtapArrowRecords::Logs(Logs::default());
    records.set(ArrowPayloadType::Logs, root_rb).unwrap();
    records.set(ArrowPayloadType::LogAttrs, attr_rb).unwrap();
    records
}

pub(super) fn parse_hir(pattern: &str) -> Hir {
    ParserBuilder::new()
        .multi_line(false)
        .build()
        .parse(pattern)
        .unwrap()
}

pub(super) fn encode_otlp_logs(
    logs: &otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::LogsData,
) -> OtapArrowRecords {
    otel_arrow_dfe_pdata::encode::encode_logs_otap_batch(logs).expect("encode_logs_otap_batch")
}

pub(super) fn decode_otlp_logs(
    records: OtapArrowRecords,
) -> otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::LogsData {
    use otel_arrow_dfe_pdata::TryIntoWithOptions;
    use otel_arrow_dfe_pdata::otlp::OtlpProtoBytes;
    use otel_arrow_dfe_pdata::proto::OtlpProtoMessage;

    let bytes: OtlpProtoBytes = records.try_into_with_default().expect("OTAP to OTLP bytes");
    match OtlpProtoMessage::try_from(bytes).expect("OTLP bytes to message") {
        OtlpProtoMessage::Logs(logs) => logs,
        other => panic!("expected LogsData, got {other:?}"),
    }
}
