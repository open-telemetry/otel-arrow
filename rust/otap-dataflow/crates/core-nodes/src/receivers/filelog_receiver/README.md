# Filelog incremental source decoder

`decoder` is a pure source-byte primitive, not a registered receiver. It has
no filesystem, configuration serialization, framing, checkpoint, engine,
processor, timer, or asynchronous-runtime dependency of its own. It lives in
the existing core-nodes crate because its consumer is the future Filelog framer.
There is no new runtime dependency.

## References and scope

The implementation uses upstream main
`6f96123c02aae9b2839d470e8b33aa7b543e4b5b` and these refreshed references:

| Reference | Commit used |
| --- | --- |
| [Design: open-telemetry/otel-arrow#3939][design-pr] | `0d527a506873c35137b291e74e3c0da426e0427c` |
| Prototype: `lalitb/otel-arrow`, `feat/filelog-receiver-phase1` | `86b4fb2e08cec44c3798241d440323d9ab949d22` |
| [Checkpoint context: open-telemetry/otel-arrow#3980][checkpoint-pr] | `829061b8e1f7b63e0edea6d777fbc4d4434ca896` |

The [architecture][architecture], [runtime specification][spec],
[conformance document][conformance], and [checkpoint format][checkpoint] form
one design contract with distinct ownership. None silently overrides another.
They are linked at the inspected commit rather than copied into this change.
This implements only the source-decoding part of
[open-telemetry/otel-arrow#2844][epic], independently of the checkpoint codec
and processor work.

## Interface and source ownership

```text
StreamDecoder::new(encoding, on_decode_error, source_offset, new_stream_start)
decoder.next(input_offset, borrowed_input) -> Result<DecodeStep, DecodeFailure>
decoder.finish_incomplete_unit() -> Result<Option<DecodeEvent>, DecodeFailure>
```

`Encoding` and `OnDecodeError` are plain enums, with UTF-8 and preserve-raw
defaults. A future configuration adapter maps its validated settings into these
types; no YAML representation or checkpoint wire discriminant is defined here.

Every range is half-open and counts **original source bytes**, never output
UTF-8 bytes or characters. Each event owns up to four exact source bytes and
either one scalar, one raw byte, or a stripped initial BOM. NUL and CR are
ordinary data; neither CRLF nor Unicode is normalized. Text LF is U+000A;
raw LF is byte `0x0a`. The decoder does not assemble or remove delimiters.

Both successful steps and failures report `consumed`: advance the input slice
by exactly that prefix. The remainder belongs to the caller. No input borrow
survives a call, so consumed scratch can be reused immediately.

Three positions must not be confused:

| Position | Meaning |
| --- | --- |
| `next_expected_input_offset()` | Next fresh byte to supply, including bytes already buffered by the decoder |
| `highest_delivered_source_boundary()` | End of the last successfully returned event |
| `pending_source_start()` | Start of the gap between those frontiers, or `None` |

For UTF-16 high-surrogate + `A`, the first replacement event can own `[0,2)`
while `consumed == 4`. `A` owns `[2,4)` and is returned by a subsequent
empty-input call. Divergent BOM probes similarly replay consumed bytes.

These are decoding facts, **not permission to frame, acknowledge or commit**.
Even a fully decoded prefix of a failing unfinished record may be
uncommittable. The framer and delivery/checkpoint layers own those decisions.

## State machine and explicit boundaries

1. **Initial probe:** only for a new textual stream at source offset zero.
   Accumulate a bounded BOM prefix. Strip a match, apply policy to a conflict,
   or replay a divergent probe as ordinary configured-encoding input.
2. **Normal decoding:** return one complete scalar/raw byte or malformed unit.
   A partial UTF-8 prefix, odd UTF-16 byte, high surrogate, or BOM waits for
   more input. UTF-16 can queue a complete lookahead code unit.
3. **Drain:** `next(expected_offset, &[])` delivers complete buffered events,
   or moves queued bytes into incomplete state. Repeat until no event remains.
   This is not terminal processing.
4. **Eligible incomplete boundary:** only after the caller establishes the
   design's idle-flush or permanent-EOF eligibility, drains earlier input, and
   drains the decoder, call `finish_incomplete_unit`.
5. **Terminal failure:** malformed input under `fail`, or source-offset
   overflow, latches the error. Only observation or reconstruction is allowed.

`finish_incomplete_unit` rejects undrained BOM replay or queued UTF-16 input
with `DrainRequired`, without modifying state. This includes a queued high
surrogate that becomes an incomplete tail only after draining.

After draining, preserve/replace return one malformed event for the entire
incomplete range; fail returns exact fatal evidence without advancing delivery.
No pending unit returns `None`. Repeated successful completion emits nothing.
Later source bytes start a new unit, rather than completing a resolved tail.
Completion does not re-enable BOM probing.

An ordinary empty read, scheduler pause, backpressure or descriptor eviction
must not invoke completion. Retain the same decoder across those events.
For a genuinely new stream, explicitly construct a new decoder at zero with
`new_stream_start == true`. Resume otherwise starts exactly at the supplied
offset, without rewinding, aligning, or searching for a character boundary;
the caller establishes a safe boundary or an intentional exclusion.

## Malformed-unit and BOM contract

The design does not enumerate every malformed grouping or BOM signature.
The following choices deliberately preserve the prototype's observable rules.
They are explicit decoder contracts, not claims that every case was already
specified by the design documents.

| Input class | Unit / outcome |
| --- | --- |
| UTF-8 | Rust `str::from_utf8` / `Utf8Error::error_len` maximal-subpart grouping |
| ASCII outside initial probing | Each byte above `0x7f` is one malformed unit |
| UTF-16 | A lone surrogate is one two-byte malformed unit; a valid following unit is not swallowed |
| Eligible incomplete UTF-16 high + odd byte | One three-byte incomplete-pair outcome, matching the prototype's explicit timeout vector |
| Initial BOM signatures | Only `EF BB BF`, `FF FE`, and `FE FF`; no UTF-32 detection |
| Complete conflicting BOM | One malformed unit covering the signature; configured decoding resumes immediately afterward |
| Eligible incomplete BOM probe | One malformed unit covering the entire probe in every textual encoding |

Thus `E2 82 41` yields malformed `E2 82`, then `A`; `ED A0 80` yields
three malformed units. A high surrogate followed by a nonsurrogate preserves
the latter for the next event. Consecutive high surrogates can leave the second
available to pair with a later low surrogate.

A UTF-8 BOM conflicting with UTF-16 occupies `[0,3)`, and the next configured
UTF-16 unit begins at offset 3. `FF FE 00 00` in UTF-16LE means its matching
BOM plus NUL, not UTF-32. BOM-shaped bytes away from the initial probe use
ordinary content/error semantics; U+FEFF and U+FFFE are valid scalars.
Raw mode never probes or strips a BOM, under any policy.

The UTF-8 primitive was checked against Rust 1.98.0's
[validation source][utf8-source]. The existing UTF-16 lookahead agrees with the
standard library's [surrogate decoder][utf16-source], but retains bytes and
does not interpret iterator exhaustion as final EOF. The
[Unicode encoding FAQ][unicode-faq] supplies validity, noncharacter and
valid-successor preservation context. No allocating lossy conversion is used
in the runtime decoder.

## Errors and preserve-raw obligations

`DecodeFailure` includes the current call's consumption even if failure follows
lookahead. `FatalMalformed` includes the exact range and source bytes.
Offset overflow is reported immediately, including any representable partial
consumption; it is not converted into success or an eligible-tail replacement.
Source-position advancement uses checked arithmetic.

After a terminal error, every `next` and completion call returns the original
error with zero consumption, even if the caller supplies a different offset.
Frontier getters remain unchanged and usable. Recovering requires a new decoder
at a caller-authorized position, not retrying through the bad unit.
`OffsetDiscontinuity` and `DrainRequired` are nonterminal caller errors; they
preserve all state and can be corrected.

Both preserve-raw and replace return U+FFFD as a malformed unit's shadow
scalar, plus its exact evidence and `malformed` flag. The caller determines
body representation. For preserve-raw it must retain original bytes from the
**beginning of the affected frame**, including clean units, rather than
starting retention only when an error appears.

In particular, an open-ended preserve-raw split sequence needs byte bodies
from its first fragment, including a clean UTF-16 fragment. A clean unsplit
record may remain text. The decoder does not store a second complete raw
record. Its stripped-BOM event retains exact evidence separately from body
content, so the caller can distinguish body bytes from frame ownership.

## Bounded work and memory

Each `next` call consumes at most **four fresh bytes**, replays at most three
already-owned BOM bytes, and returns at most one event. Neither input length
nor line/record length can increase that work bound. A nonempty call consumes
input, returns an event, or fails. The single output slot is part of the return
value: there is no caller output buffer or minimum-capacity retry loop.

The direct fast path handles one complete raw byte or ASCII scalar in UTF-8/
ASCII only when BOM/replay state is clear and no UTF-8 prefix is pending.
All other cases use the prototype's bounded encoding machinery. This permits
the caller to stop after any event, without decoding a later unit.

| Owner | Bound / accounting |
| --- | --- |
| Decoder object | Fixed inline state; measured 176 bytes on x86-64 Rust 1.98.0, versus prototype 152 |
| Live undelivered source | At most 3 distinct bytes after a successful call; at most 4 after fatal failure |
| Inline source storage | UTF-8 array 4; UTF-16 odd/high/queued payloads up to 5; probe 3; replay 3; sticky error evidence 4. Encoding alternatives and active states are not all simultaneously live |
| Decoder heap | Zero; no drop-managed allocation or retained input borrow |
| Caller output | One event (32 bytes measured); step / result 40; failure 32. Layout measurements are not ABI promises |
| Caller frame retention | Independently bounded raw and text capacities; charge independent copies separately |
| Source-turn scratch/remainder | Caller-owned; retained unconsumed input is not charged as decoder heap |
| Temporary work | Fixed stack candidates/copies; no temporary heap peak or buffer proportional to stream length |

Pending length describes logical source work, not physical state size. The
regression guard on decoder object size is 192 bytes; the measured object
includes offsets, enum tags, cached code units, inactive storage and padding.
The receiver's proposed aggregate partial-state limit is not a local
allocation allowance.

This follows the distinctions in [memory resource management][memory],
[the current process limiter][limiter], [proposed retained-work accounting][retained],
and [pressure-aware throttling][throttling]. Physical capacity, logical
retention, and process memory are different measurements. No proposed
`MemoryTicket`, global limiter, allocator query, RSS sampling, synchronization
or runtime task enters this decoder.

## Requirement-to-test summary

All tests use the repository's Scenario/Guarantees convention. The original
18 decoder tests remain, adapted only for the isolated options, BOM evidence,
and explicit failure contract.

| Requirement | Tests / evidence |
| --- | --- |
| Five encodings, BOM matches/conflicts and partitions | `matching_boms_are_stripped_across_all_partitions`, `conflicting_boms_follow_each_decode_error_policy`, `raw_mode_emits_every_byte_without_bom_handling` |
| UTF validity and exact source widths | `every_unicode_scalar_decodes_with_its_original_width` covers all 1,112,064 scalars in UTF-8 and both UTF-16 orders |
| Independent malformed grouping | `utf8_malformed_prefixes_are_exact_at_every_partition`, `additional_unicode_maximal_subpart_vectors_are_independent`, `all_ascii_bytes_have_independent_expected_outcomes`, `utf16_units_match_the_standard_library_oracle` |
| Empty/temporary EOF versus eligible tails | `empty_stream_does_not_manufacture_output_or_reset_bom_state`, `eligible_encoding_tails_cover_their_exact_source_range`, `eligible_partial_bom_is_atomic_in_every_text_encoding` |
| Complete buffered input before tail policy | `queued_utf16_units_must_be_drained_before_completion`, `divergent_bom_replay_must_be_drained_before_completion` |
| Fatal evidence, consumption and permitted operations | `fatal_decode_paths_latch_error_and_consumption`, `fatal_incomplete_prefix_does_not_consume_valid_successor`, `offset_overflow_is_immediate_and_sticky_without_wrapping` |
| Source ownership across all partitions and offsets | `exhaustive_partitions_preserve_terminal_and_pending_state`, `seeded_arbitrary_streams_preserve_source_ownership` |
| Earlier output and safe stopping | `complete_record_precedes_later_error`, `consumer_can_stop_at_an_exact_safe_source_boundary` |
| Discarded malformed tails / preserve-raw prefix | `discard_scan_does_not_hide_later_malformed_units`, `clean_first_fragment_has_exact_raw_evidence` |
| Pause, scratch reuse, reset and controls | `pause_and_input_buffer_reuse_preserve_pending_ownership`, `new_stream_reset_is_distinct_from_pause_and_resume`, `text_controls_and_embedded_lf_byte_are_not_normalized` |
| Constant state and allocations | `long_discard_scan_retains_constant_state`, isolated `filelog_decoder_heap_allocation_is_zero` |

Seeded testing uses the existing `rand` dependency: 128 byte vectors, all five
encodings and three policies, five schedules each (9,600 decodes), with normal
and near-overflow offsets. It compares actual pending bytes as well as events
and errors. This is deterministic property coverage, not a coverage-guided fuzz
campaign or proof over arbitrary streams. No new fuzzing framework is introduced.

## Prototype reuse and integration boundary

| Prototype item | Disposition / reason |
| --- | --- |
| `SourceRange`, `SourceBytes`, `DecodedValue` | Reused fixed-size evidence primitives and license headers |
| UTF-8 validation, ASCII/raw general paths, UTF-16 endian/surrogate machinery | Reused, not replaced with a new Unicode implementation |
| `ByteCursor`, BOM recognizer/probe, replay storage | Reused bounded state and existing grouping |
| `Encoding`, `OnDecodeError` | Adapted to plain decoder options; no receiver configuration or wire types |
| `DecodeEvent`, `DecodeStep` | Adapted with BOM source evidence, public range/source accessors and explicit consumption documentation |
| `next` | Adapted with consumed-on-error and sticky failure; measured direct one-byte success path |
| `finish_incomplete_unit` | Adapted with enforced drain precondition and sticky failure |
| `earliest_uncommittable_offset` alias | Removed; encoding frontiers cannot grant checkpoint permission |
| Decoder partition/Unicode tests | Preserved; supplemented with independent expectations, fatal-state, boundary, property and allocation coverage |
| Framer exact-bound, timeout, preserve split and truncate-tail cases | Adapted into small decoder consumers, not copied as a framer |
| Framer, worker, configuration, coordinator, checkpoint modules | Deferred; not imported or registered |

The future framer owns record boundaries, size policy, bounded raw/text
retention and discard scans. It must stop before later decoding where required,
and under `fail` validate a discarded truncate tail before emitting that same
record's prefix. Earlier independent complete output must remain available.

The receiver owns eligibility, identity, descriptor continuity, scheduling,
pressure admission, aggregate capacity and quarantine coordination. Delivery
and checkpoint code own authoritative progress and persistence. Processors own
JSON/CSV/timestamp/severity interpretation and enrichment.

Caller-level design wording still needs integration review: "complete framed
source slice" versus BOM/final-LF body exclusions, alignment policy at an
intentional `start_at: end` inside an encoded unit, and broad conformance-matrix
phrasing about rereads and permanent EOF. This module exposes all evidence and
does not silently decide those framing/lifecycle policies.

See [benchmark qualification][benchmarks] for reproducible measurements,
the bounded-output experiment and limitations.

[architecture]: https://github.com/lalitb/otel-arrow/blob/0d527a506873c35137b291e74e3c0da426e0427c/rust/otap-dataflow/docs/filelog-receiver.md
[spec]: https://github.com/lalitb/otel-arrow/blob/0d527a506873c35137b291e74e3c0da426e0427c/rust/otap-dataflow/docs/filelog-receiver-phase1-spec.md
[conformance]: https://github.com/lalitb/otel-arrow/blob/0d527a506873c35137b291e74e3c0da426e0427c/rust/otap-dataflow/docs/filelog-receiver-phase1-conformance.md
[checkpoint]: https://github.com/lalitb/otel-arrow/blob/0d527a506873c35137b291e74e3c0da426e0427c/rust/otap-dataflow/docs/filelog-checkpoint-format.md
[design-pr]: https://github.com/open-telemetry/otel-arrow/pull/3939
[checkpoint-pr]: https://github.com/open-telemetry/otel-arrow/pull/3980
[epic]: https://github.com/open-telemetry/otel-arrow/issues/2844
[utf8-source]: https://doc.rust-lang.org/1.98.0/src/core/str/validations.rs.html
[utf16-source]: https://doc.rust-lang.org/1.98.0/src/core/char/decode.rs.html
[unicode-faq]: https://www.unicode.org/faq/utf_bom.html
[memory]: ../../../../../docs/memory-resource-management.md
[limiter]: ../../../../../docs/memory-limiter-phase1.md
[retained]: ../../../../../rfcs/0000-observe-only-retained-work-accounting.md
[throttling]: ../../../../../rfcs/0002-pressure-aware-rate-throttling.md
[benchmarks]: ../../../benches/filelog_decode/README.md
