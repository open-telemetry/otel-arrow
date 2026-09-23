# Filelog incremental source decoder

The decoder converts source bytes into individual decoded units while preserving
original bytes and source offsets. File reading, receiver configuration, framing,
delivery, and checkpointing belong to its callers.

The [Filelog receiver design][design-pr] describes the broader receiver contract.
This document describes the decoder's behavior and the obligations of callers.

## Interface and source ownership

```text
StreamDecoder::new(encoding, on_decode_error, DecodeStart::NewStream)
StreamDecoder::new(encoding, on_decode_error, DecodeStart::ResumeAt(offset))
decoder.next(input_offset, borrowed_input) -> Result<DecodeStep, DecodeFailure>
decoder.finish_incomplete_unit() -> Result<Option<DecodeEvent>, DecodeFailure>
```

`Encoding` and `OnDecodeError` are plain enums, with UTF-8 and preserve-raw
defaults. Callers map validated configuration into these types; the decoder
defines no YAML representation or checkpoint wire discriminant.

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
For a genuinely new stream, construct a decoder with `DecodeStart::NewStream`.
`DecodeStart::ResumeAt(offset)` starts exactly at the supplied offset, including
zero, without BOM probing. It does not rewind, align, or search for a character
boundary; the caller establishes a safe boundary or an intentional exclusion.

## Caller-authorized clean-yield reconstruction

Ordinary pauses and descriptor eviction retain the same decoder. A clean yield
is a separate, caller-authorized transition. A
sufficient decoder-side condition at boundary `B` is that, when the event ending
at `B` is returned, `highest_delivered_source_boundary() == B`,
`pending_source_start() == None`, and `terminal_error() == None`. There must be
no buffered replay or incomplete unit to discard. Capture these facts at `B`;
checking the live decoder after multiline lookahead has advanced past `B` does
not establish that earlier boundary's state.

After the framer establishes a completed-record yield point and the receiver
validates source identity and continuity, decoding can restart with
`StreamDecoder::new(encoding, policy, DecodeStart::ResumeAt(B))`. Preserve the
configured encoding and policy. The constructor does not align the offset or
restart BOM probing; a UTF-16 boundary must come from the actual decoded units,
not an assumption that every valid boundary has an even absolute offset.

These decoder facts do not establish record completion, source continuity, or
checkpoint authority. Pending failure handling belongs to the framer and cannot
be erased by reconstruction. The receiver must preserve completed output and
may replay only the speculative later input allowed by the clean-yield contract.

## Malformed-unit and BOM contract

Malformed input is grouped into source units as follows.

| Input class | Unit / outcome |
| --- | --- |
| UTF-8 | Rust `str::from_utf8` / `Utf8Error::error_len` maximal-subpart grouping |
| ASCII outside initial probing | Each byte above `0x7f` is one malformed unit |
| UTF-16 | A lone surrogate is one two-byte malformed unit; a valid following unit is not swallowed |
| Eligible incomplete UTF-16 high + odd byte | One three-byte incomplete-pair outcome |
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

The decoder uses fixed inline state and no heap allocation or retained input
borrows. Stream and record lengths do not increase its memory use. Live
undelivered source is bounded to three distinct bytes after a successful call
and four after a fatal failure. These are logical source-byte bounds, not
measurements of the decoder object's layout.

Input scratch buffers, unconsumed input, and raw/text frame retention remain
caller-owned. Callers must bound those buffers independently and account for
separate copies. See [memory resource management][memory] for the distinction
between logical retention and physical capacity.

## Integration responsibilities

The framer owns record boundaries, size policy, bounded raw/text
retention and discard scans. It must stop before later decoding where required,
and under `fail` validate a discarded truncate tail before emitting that same
record's prefix. Earlier independent complete output must remain available.

The receiver owns eligibility, identity, descriptor continuity, scheduling,
pressure admission, aggregate capacity and quarantine coordination. Delivery
and checkpoint code own authoritative progress and persistence. Processors own
JSON/CSV/timestamp/severity interpretation and enrichment.

See [benchmark documentation][benchmarks] for measurements and reproduction
instructions.

[design-pr]: https://github.com/open-telemetry/otel-arrow/pull/3939
[memory]: ../../../../../../docs/memory-resource-management.md
[benchmarks]: ../../../../benches/filelog_decode/README.md
