# `otel-arrow-dfe-policy-engine` (Proof of Concept)

Pure-Rust, zero-`unsafe` columnar telemetry policy evaluation engine prototype
for OTAP `OtapArrowRecords` log batches ([issue #4726][i4726], companion to
[#4718][i4718] and [#4738][p4738]).

## Scope

- Implements the data model from
  [open-telemetry/opentelemetry-proto#865][proto865]
  (`opentelemetry.proto.policies.v1development`) and evaluates
  `LogFilterPolicy` and `LogTransformPolicy` directly over Apache Arrow
  `OtapArrowRecords` batches.
- Supports all 9 spec predicates (`exists`, `equals`, `glob`, `gt`, `gte`,
  `lt`, `lte`, `contains`, and RE2-compatible `regex`) and all 4 log transform
  stages (`rename`, `add`, `remove`, `redact` including RE2 capture-template
  redaction).
- Uses zero C/C++ build dependencies (`cmake`, Boost, `vectorscan-sys`) and
  zero `unsafe` code, relying only on crates already present in
  `rust/otap-dataflow/Cargo.lock` (`ahash`, `aho-corasick`,
  `regex-automata`, `regex-syntax`, `memchr`, `hex`, `thiserror`).
- Compiles string `contains`, `glob`, and `regex` matchers into an inverted
  index of mandatory contiguous byte literals (`len >= 2`) backed by a single
  `AhoCorasick` automaton per field, verifying full regex/glob matches only
  after all required literals match.
- Resolves filter masks via lazily allocated pooled dense bitsets with
  `ACTION_KEEP` overriding `ACTION_DROP`, and applies `LogTransformPolicy`
  stages globally in spec order (`1. Rename -> 2. Add -> 3. Remove -> 4. Redact`,
  sorted by ascending `Policy.id`) via row-masked columnar mutations without
  splitting and re-concatenating batches per policy.

## Unsupported Standard Policy Types & Limitations

Per `policy.md` ("Conformance"), this log-only engine documents every standard
policy type and feature it does not support (all unsupported policy types are
rejected atomically as compilation errors and kept inert without affecting
other policies in the batch):

- **Unsupported policy types**: `MetricFilterPolicy`, `SpanSamplingPolicy`, and
  `Policy.custom_policy` (`CustomPolicy`) are not supported by this log-only
  engine.
- **Signals**: Only `OtapArrowRecords::Logs` batches are evaluated; metrics and
  traces batches return `PolicyError::Evaluation`.
- **Wire decoding**: Protobuf binary/JSON wire deserialization (`PolicySet`,
  unknown wire field/enum rejection, and `google.protobuf.Any` unpacking) is
  handled by the caller/config layer; this crate consumes typed Rust `Policy`
  structs directly.
- **Resource bounds**: Regex patterns are bounded to 4 KiB source length,
  nesting depth 64, 256 KiB NFA size, and 64 KiB hybrid DFA cache; CBOR
  container nesting for nested `log_attribute` and `body_field` traversal/writes
  is bounded to 64 levels.
- **Async execution**: Evaluation is intentionally synchronous and CPU-bound;
  async runtime integration is deferred until the policy processor node is
  wired into an OTAP dataflow pipeline.

## Full Prototype, Multi-Candidate Conformance Suite & Benchmarks

This PR branch contains only the selected n-gram columnar engine and its unit
tests so workspace CI builds and checks remain pure Rust without Boost/CMake.

For the full prototype containing all four pure-Rust candidates (hybrid prefix
`AhoCorasick`, sharded `regex-automata`, n-gram inverted index, and compiled
OPL `query-engine`), the optional `policy-rs` / Vectorscan baseline, the
cross-engine differential equivalence suite, negative-control tests, and the
full 280-row benchmark matrix, see the companion branch:

- <https://github.com/dashpole/otel-arrow/tree/policy-engine-full-prototype>

[i4726]: https://github.com/open-telemetry/otel-arrow/issues/4726
[i4718]: https://github.com/open-telemetry/otel-arrow/issues/4718
[p4738]: https://github.com/open-telemetry/otel-arrow/pull/4738
[proto865]: https://github.com/open-telemetry/opentelemetry-proto/pull/865
