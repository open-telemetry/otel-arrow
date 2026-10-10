# Multiline grouping

`MultilineFramer` turns source bytes into bounded logical records using a start
or end pattern. It reuses `StreamDecoder` and the [pattern matcher](multiline.md).
It does not read files, run timers, construct OTAP batches, or commit checkpoints.

## Calling it

Create one framer per reader and share the compiled `BoundaryPattern`. Pass a
worker-owned `BoundaryMatcher` to each operation. A cache for another program
is rejected; caches do not need to be stored with every pending file.

- `next(offset, input, matcher)` accepts a bounded decoding or replay step and
  returns at most one record. Advance input by `consumed`, including on failure.
- Keep calling with remaining input, then empty input, until `advanced` is false.
  Stop at any step to apply backpressure or a worker turn budget.
- `complete_partial(Idle | PermanentEof, matcher)` requires caller-established
  completion authority after all read bytes have been supplied. Repeat the same
  reason until `complete`, keeping every
  returned frame. Ordinary EOF, a pause, or shutdown does not authorize this.
- Use `next_expected_input_offset()` for fresh input. Decoder lookahead can put
  it beyond the last output's frame end. Do not discard that retained state.

Pattern searches run synchronously on one bounded decoded line. The caller's
turn budget cannot interrupt a search already in progress.

## Grouping and bounds

Start mode emits unmatched lines independently until a matching line starts a
record. A later start completes the previous record. End mode includes the
matching line in the completed record. Internal LFs remain in the body; only
the final LF is omitted. CR and NUL remain ordinary content.

Physical-line and record limits are independent. A completed line that would
overflow the record establishes its deterministic end. Its pattern and
line-count decisions are suppressed. An exactly full body completes cleanly
before another line is read. Line-count, byte-bound, and idle completion reset
start mode to seeking.

An oversized physical line first completes any earlier grouped content, then
runs independently through LF or authorized completion. It is not matched.
Split preserves all body bytes in safe-unit fragments; truncate keeps the
largest safe prefix and scans the rest without retaining it. Physical-line
fragments use the smaller of the line and record bounds.

`RecordEnding` supplies completion evidence, including unmatched fallback,
oversized-line isolation, and terminal-unterminated EOF. The receiver owns
telemetry and policy actions for those reasons.

## Decoding and failures

A bounded line retains decoded text for matching and exact source bytes for
replay. Replaying it into the record buffer preserves source-unit boundaries
without a per-character offset table. Each buffered line is decoded twice;
this trades some CPU work for bounded storage and exact offsets, including
replacements.

Under `preserve_raw`, a malformed grouped body becomes exact bytes. All split
fragments use exact bytes from the start, even if malformed input appears later.
A clean truncated prefix may remain text; malformed discarded units still count.

Decode-fail suppresses the failing record's truncated prefix. Earlier returned
records remain caller-owned and must resolve before quarantine. Terminal errors
latch. Wrong-cache, offset, and completion-sequencing errors consume nothing and
leave the original operation usable.

## Restart

Nonfinal fragments provide record start, known record end (or zero for a scan
to LF), and next fragment index. The receiver combines the origin with file
identity and epoch for the specified fragment ID. These coordinates propose
progress; they do not authorize Ack or checkpoint advancement.

Before constructing a continuation, validate source identity, framing profile,
committed-frontier evidence, and source size. A file shorter than a known end
requires the receiver's truncation policy before any output. Resume from the
committed offset, never the original record start.

Recovery suppresses fresh pattern and line-count decisions until the stored end
or scan-to-LF boundary. Idle/EOF cannot shorten a known end. Empty recovery emits
nothing. The final fragment can use `u32::MAX`; emitting a nonfinal fragment at
that index fails before output.

## Memory

The framer retains one bounded physical line, one record buffer, and inline
lookahead. It has no output queue. Line replay needs exact source storage even
under `replace` and `fail`; UTF-16 source bytes can be twice the decoded size.

`MultilineConfig::payload_peak_bytes()` estimates payload storage including
buffer-growth overlap. `retained_capacity()` reports currently owned buffer
capacities. Add inline state, allocator overhead, and the worker regex cache
separately. Transferred output belongs to its destination's memory charge.
Neither method reserves memory from a receiver budget.

See the [conformance memory model][memory] for the formula. The tests cover
chunk partitions, encoding and malformed-input cases, overflow and recovery,
and allocation-free truncate-tail scanning.

[memory]: ../../../../../../docs/filelog-receiver-phase1-conformance.md#framer-payload
