# `otel-arrow-dfe-policy-engine`

Pure-Rust, zero-`unsafe` columnar telemetry policy evaluation engine for OTAP
`OtapArrowRecords` log batches (tracked in [issue #4726][i4726] and
[issue #4718][i4718]).

> **Work in progress:** This crate currently provides the policy data model and
> core matching helpers. Policy compilation, columnar log filtering, and log
> transforms are not yet implemented.

## Implemented

- Data model from [open-telemetry/opentelemetry-proto#865][proto865]
  (`opentelemetry.proto.policies.v1development`) for `LogFilterPolicy` and
  `LogTransformPolicy`, including all 9 spec predicates (`exists`, `equals`,
  `glob`, `gt`, `gte`, `lt`, `lte`, `contains`, and RE2-compatible `regex`)
  and all 4 log transform stages (`rename`, `add`, `remove`, `redact`).
- RE2-compatible regular expression validation (bounded to 4 KiB source length,
  nesting depth 64, 256 KiB NFA size, and 64 KiB hybrid DFA cache),
  non-exponential (`O(|P| * |T|)`, linear in text length for fixed pattern
  length) `glob` matching, and exact `i64`/`f64` numeric comparison.
- Zero C/C++ build dependencies and zero `unsafe` code, relying only on crates
  already present in `rust/otap-dataflow/Cargo.lock`.

## Not Yet Implemented & Limitations

- **Evaluation engine**: Policy compilation, columnar log filtering, and
  4-stage log transforms are not yet implemented.
- **Unsupported policy types**: Per `policy.md` ("Conformance"),
  `MetricFilterPolicy`, `SpanSamplingPolicy`, and `Policy.custom_policy`
  (`CustomPolicy`) are modeled for compile-time rejection rather than
  evaluation in this log-only engine.
- **Wire decoding**: Protobuf binary/JSON wire deserialization (`PolicySet`,
  unknown wire field/enum rejection, and `google.protobuf.Any` unpacking) is
  handled by the caller/config layer; this crate consumes typed Rust `Policy`
  structs directly.

[i4726]: https://github.com/open-telemetry/otel-arrow/issues/4726
[i4718]: https://github.com/open-telemetry/otel-arrow/issues/4718
[proto865]: https://github.com/open-telemetry/opentelemetry-proto/pull/865
