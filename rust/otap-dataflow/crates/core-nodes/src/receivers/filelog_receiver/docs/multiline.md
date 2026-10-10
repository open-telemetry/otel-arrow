# Filelog multiline patterns

`BoundaryPattern` validates the [executable profile][spec] and searches bounded
physical-line bodies. It does not join lines or register a receiver. The caller
supplies complete bodies without terminal LF and bypasses oversized lines.

## Semantics

Text input is already validated UTF-8. Raw input is uninterpreted source bytes.
Mode mismatches are errors. ASCII Perl classes and word boundaries are
normalized in the syntax tree. RE2's `\s` excludes vertical tab; text case
folding precedes class negation. Captures do not change the boolean result.

In raw mode, `\xFF` and `\x{FF}` both denote byte `FF`. This also applies to
classes and range endpoints. Braced values above `FF` are rejected. Ordinary
non-ASCII pattern characters outside classes retain their UTF-8 source bytes:
the literal character U+00E9 matches bytes `C3 A9`. Text-mode hex escapes retain
Unicode semantics.

The profile limits source to 4,096 bytes, counted repetitions and nested count
products to 1,000, and parsed syntax-tree depth to 64. Unsupported syntax is
rejected structurally, so escaped look-alikes remain literal. Initial parse
errors refer to original source; translation errors report only their kind.
Persist original source in framing-profile digests, never normalized syntax.

## Construction and execution policy

`compile()` delegates to `compile_with_limits()` using
`PatternLimits::default()`:
10 MiB for the forward-NFA compiler and aggregate engine-reported program
payload, and 2 MiB for each lazy cache. Callers may lower either limit; zero
cache capacity explicitly disables lazy acceleration. There is no automatic
worker-budget downgrade based on a memory estimate.

Construction parses, validates, normalizes, and translates once. It builds a
canonical forward NFA for validation and scratch sizing, then drops that sizing
copy before constructing meta. If the forward accelerator's documented minimum
cache exceeds the request, construction skips the unused reverse program and
reports that reason. This extra forward compilation is cold-path
work; it avoids both guessed scratch multipliers and repeated forward-size
failures. Meta construction is attempted at most twice from the same HIR: an
accelerated build and, if necessary, a build without acceleration. Reverse-NFA
size failure remains recoverable; `size_limit()` does not identify direction.

Meta uses implicit captures because `search_half_with` returns no match when
captures are disabled. Full and one-pass DFAs are disabled. Normal construction
also disables the bounded backtracker; qualification can enable it separately.
Searches use `earliest(true)` and caller-owned caches, bypassing the convenience
API's pool. Literal shortcuts and equivalent PikeVM fallback remain enabled.

`execution_policy()` reports whether a lazy accelerator was actually built and,
when it was not, whether this was requested, caused by aggregate program size,
a build failure, an insufficient requested cache, or meta's strategy selection.
This is a compile-time policy, not a claim that every search uses the same
engine.
Meta strategy selection can choose a literal-search shortcut, so absence of a
lazy DFA does not imply slower execution. Startup telemetry belongs to the
caller; there is no per-line logging here.

The matcher owns an `Arc` to immutable program state. It can be stored directly
in a worker and can outlive the original pattern handle. Sharing and cloning
occur at construction; matching does not clone the handle or synchronize caches.

## Memory estimates and funded admission

This primitive does not own a receiver budget ledger. `matcher()` therefore
accepts no plain integer claiming that memory was reserved. The caller must
fund construction, shared program storage, and each worker's scratch before
activation, and retain that funding until the corresponding storage is dropped.
An estimate is not a reservation token or an allocator-level cap.

`MemoryEstimate` exposes the engine-reported program payload, initial cache
report, reachable growing cache count, fallback bounds, and worker peak heap
bound. Immutable payload reports exclude allocator and wrapper bookkeeping;
receiver-level shared-program charging remains a separate integration concern.
Compilation workspace is also separate: forward sizing, meta construction,
reverse construction, and a possible retry have temporary allocations. The
per-NFA limit and post-build check are not a process-wide compilation cap.

### Scratch derivation

The model is audited against regex-automata 0.4.18, regex-syntax 0.8.11, and
Rust
1.98.1. A lockfile audit test requires an explicit review on upgrades without
constraining workspace dependency resolution. `multiline_pattern/memory.rs`
implements checked arithmetic.

Let `N` be forward-NFA states, `K` its implicit capture slots, `I` the StateID
width, and `W` the machine-word width. The two PikeVM active-state sets contain:

```text
sparse arrays = 4 * max(N, 4) * I
slot tables   = 2 * max(N * K + K, 4) * W
```

For epsilon traversal, let `E` be one initial frame plus each union's additional
arms and each capture restore. A state is explored at most once per closure.
The private frame fits three words in the audited implementation. Allowing the
old and doubled new vector to coexist gives:

```text
epsilon stack peak = 3 * max(E, 4) * (3 * W)
PikeVM bound       = sparse arrays + slot tables + epsilon stack peak
```

PikeVM scratch is derived only from the forward program it executes.

For lazy caches, `R` is the initial meta-cache report and `C` the configured
per-cache ceiling. The report is zero if no lazy engine was built; other eager
engines are disabled and fallback caches are initially lazy. The number `G` of
caches that can grow through this fixed half-search API is:

- zero when no accelerator was built;
- one for absolute start/end anchored patterns or disabled automatic prefilters;
- conservatively three for other patterns, including reverse-search strategies.
  This allows all retained lazy caches to grow across repeated searches; it is
  not a claim that one half-search grows all three.

The initial report separately covers unused reverse-cache storage. The bound
`4 * (R + G * C)` covers length-versus-capacity differences: vector old/new
allocation overlap, the audited hash-table load/growth behavior, and serialized
states' unreported Arc headers. It is a conservative configured-capacity bound,
not four times measured use and not a charge applied to NFA payload bytes.

Add the PikeVM bound and meta wrapper/capture bookkeeping. The qualification-
only
backtracker variant additionally accounts for both its visited bitmap and
traversal stack over at most 128 bytes under meta's earliest-search policy.
It is not enabled merely because its bitmap has a cap.

Small patterns can use far less than the configured cache ceiling. Callers can
choose a smaller ceiling explicitly; measurements do not prove that a smaller
universal safety multiplier would be sound. The allocation harness compares
actual peak requested heap against the bound, including large-program searches.
These measures do not equal RSS or allocator resident memory.

`cache_memory_usage()` is diagnostic and can omit spare capacity. A detected
model violation is an error after allocation, never a nonmatch. It latches
terminal state: later calls return the same error without searching. The caller
must discard the matcher and handle a resource-accounting failure.

## Qualification and grouping integration

The [harness][bench] validates text before timing and compares lazy
acceleration,
PikeVM with and without prefilters, smaller caches, and the backtracker
candidate.
Large-program and repetition-cap cases are opt-in and should run under external
time/memory limits. Raw evidence belongs with the review artifacts rather than
in this source directory.

Tests compare accelerated and forced NFA results and use a directly configured
lazy DFA's public `GaveUp` error to prove the churn fixture requires fallback.
They do not inspect a dependency's Debug representation.

Synchronous matching cannot yield midway merely because the worker has a turn
budget. A numeric receiver control-latency target and integrated Linux
qualification remain later gates. No fixture's observed maximum is a bound on
all supported programs.

The [grouping primitive](multiline-grouping.md) keeps physical-line and record
limits independent and retains a decoded shadow for preserve-raw text matching.
Workers still own EOF, time, cancellation, and memory admission.

[spec]: ../../../../../../docs/filelog-receiver-phase1-spec.md#executable-re2-v1-subset
[bench]: ../../../../benches/filelog_multiline/README.md
