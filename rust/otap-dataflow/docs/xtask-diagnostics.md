# `cargo xtask check --diagnostics`

`cargo xtask check --diagnostics` runs the normal full check suite and prints
an end-of-run summary to help explain where time was spent.

Use it when:

- `cargo xtask check` feels unexpectedly slow
- you want to know whether the cost is mostly `fmt`, `clippy`, or `test`
- you want a quick list of the slowest compile hotspots and test binaries

## Sections

### Step Timings

This shows wall-clock time for the main check phases:

- `structure`
- `fmt`
- `clippy`
- `test`

The percentage is relative to the total `cargo xtask check --diagnostics` run.

### Top Compile Hotspots

This is a ranked list of the most expensive crates seen in the Cargo timing
reports emitted by:

- `cargo clippy --workspace --all-targets --timings`
- `cargo test --workspace --timings`

Each entry shows:

- whether the crate is a workspace crate via the `[workspace]` marker
- the total aggregated compile time seen across `clippy`, `test`, or both
- a per-step breakdown such as `(clippy 2.10s, test 4.35s)`
- one or more heuristic hints in brackets

Important: these hints are conservative heuristics, not proofs of root cause.
They are intended to help triage, not replace deeper investigation.

### Top Test Binaries

This is a ranked list of the slowest test binaries seen in the `cargo test`
output.

On stable Rust this is a per-binary view, not a per-test-case view. For
example, `unittests src/lib.rs [quiver]` means the whole `quiver` unit-test
binary, not one specific test function inside it.

### Other Signals

This section provides a few compact summary signals:

- `build-script units observed`
- `proc-macro crates in top compile hotspots`
- `compile hotspots are mostly from`
- `test step split (approx.)`

The test split is approximate:

- `compile` comes from Cargo's timing report wall time for the `test` step
- `execution` is the remaining `cargo test` wall time

### Timing Reports

These are the Cargo timing HTML reports consumed by the diagnostics parser.
They are printed as file hyperlinks when the terminal supports hyperlinks, and
as plain absolute paths otherwise.

Open them when you want the full Cargo timing UI rather than just the `xtask`
summary.

### Diagnostics Notes

This section is used for partial-data situations, for example:

- no timing report path found in command output
- timing report parse failure
- no compile units in a timing report because artifacts were already fresh

## Hint meanings

The compile-hotspot hints come from simple rules in
[`xtask/src/diagnostics.rs`](../xtask/src/diagnostics.rs).

### `[build script]`

The crate showed build-script compile units in the Cargo timing report.

This usually means time is being spent compiling the crate's `build.rs`.

### `[build script (run)]`

The crate showed build-script execution units in the Cargo timing report.

This usually means time is being spent running `build.rs`, not just compiling
it.

### `[proc-macro crate]`

The crate is a proc-macro crate according to `cargo metadata`.

This is a classification hint only. It does not automatically mean that macro
expansion is the dominant cost.

### `[codegen-heavy]`

This is emitted when:

- codegen time is at least `25%` of the crate's aggregated compile time, and
- codegen time is at least `500ms`

Interpretation:

- LLVM/code generation is a meaningful part of the cost
- optimizations are not necessarily the issue, but this is less likely to be a
  purely parser/frontend problem

### `[feature-heavy]`

This is emitted when the crate was observed with at least `8` enabled features
in one of the timing units.

Interpretation:

- feature combinations may be widening the compile graph
- this is a hint about compile surface area, not a claim that features alone are
  the root cause

### `[frontend-heavy]`

This is emitted when frontend time is at least `85%` of the crate's aggregated
compile time, and the crate did not already match `codegen-heavy` or
`feature-heavy`.

Interpretation:

- parsing, expansion, analysis, and type checking dominate the observed cost
- common causes include large modules, macro expansion, and generally complex
  Rust source, but the hint intentionally stays broad

### `[no strong hint]`

No stronger heuristic matched.

Interpretation:

- the crate is still expensive
- the current diagnostics did not find a clear one-line explanation

## Color meanings

The diagnostics output uses color only to make scanning easier.

- section banners and labels use blue/cyan styling
- `ok` is green and `failed` is red
- hotter timings use warmer colors:
  - top hotspot ranks are red, then yellow, then cyan
  - step durations shift toward yellow/red as their share of total time grows
  - larger durations in the approximate test split also use warmer colors

Treat the colors as ranking aids, not as additional semantics beyond "hotter"
versus "cooler".

## Limitations

- Compile hotspots depend on Cargo timing reports and may be empty on a warm
  cache when nothing needed recompilation.
- Test hotspots are per binary on stable Rust, not per individual test case.
- The diagnostics summary is intentionally conservative and may under-explain a
  real hotspot.
- `unknown test binary` means a `cargo test` result line could not be matched
  back to a preceding `Running ...` line.

## Future improvements

If we decide to depend on nightly-only capabilities in the future, the
diagnostics could become more precise.

Potential improvements include:

- per-test-case timings instead of only per-test-binary timings
- richer machine-readable test output, which would reduce the need to infer
  timing data from human-oriented `cargo test` text output
- deeper compiler attribution for expensive crates, for example more detailed
  breakdowns of frontend work, macro expansion, or query-heavy compilation
  phases
- optional deeper compiler profiling for a small set of selected hotspots when
  the stable Cargo timing reports are not explanatory enough

We intentionally do not rely on those features today because the current
implementation is designed to work on stable Rust without extra toolchain
requirements.
