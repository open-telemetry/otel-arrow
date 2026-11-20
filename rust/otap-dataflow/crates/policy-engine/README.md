# `otel-arrow-dfe-policy-engine` (Proof of Concept)

Pure-Rust, zero-`unsafe` columnar telemetry policy evaluation engine prototype
for OTAP `OtapArrowRecords` log batches ([issue #4726][i4726], companion to
[#4718][i4718] and [#4738][p4738]).

## Scope

- Implements the protobuf data model from
  [open-telemetry/opentelemetry-proto#865][proto865]
  (`opentelemetry.proto.policies.v1development`) and evaluates
  `LogFilterPolicy` and `LogTransformPolicy` directly over Apache Arrow
  `OtapArrowRecords` batches.
- Uses zero C/C++ build dependencies (`cmake`, Boost, `vectorscan-sys`) and
  zero `unsafe` code, relying only on crates already present in
  `rust/otap-dataflow/Cargo.lock` (`roaring`, `ahash`, `aho-corasick`,
  `regex-automata`, `regex-syntax`, `ciborium`, `memchr`, `hex`,
  `thiserror`).
- Compiles string `contains` and `regex` matchers into an inverted index of
  mandatory contiguous byte literals (`len >= 2`) backed by a single
  `AhoCorasick` automaton per field, verifying full regex matches via
  `regex_automata::meta::Regex` (`multi_line(false)`) only after all required
  literals match.
- Resolves filter masks via `RoaringBitmap` with `ACTION_KEEP` overriding
  `ACTION_DROP`, and applies `LogTransformPolicy` stages globally in spec order
  (`1. Rename -> 2. Add -> 3. Remove -> 4. Redact`, sorted by ascending
  `Policy.id`) via row-masked columnar mutations without splitting and
  re-concatenating batches per policy.

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
