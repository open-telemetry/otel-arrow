# DFE release build profiles

DFE provides two opt-in profiles for different binary-size and processing
performance priorities. Choose the profile independently of the component
features required by a pipeline.

| Profile | Priority | Optimization level | Debug information | Function symbols |
| --- | --- | --- | --- | --- |
| `release` | Existing release build | `3` (Cargo default) | Line tables for profiling | Retained |
| `release-perf` | Runtime performance | `3` | None | Retained |
| `release-size` | Minimum binary size, accepting potential throughput loss | `z` | None | Retained |

Both custom profiles use fat LTO, one codegen unit, disabled incremental
compilation, `debug = 0`, `strip = "debuginfo"`, and `panic = "unwind"`.
Unwinding preserves the controller's supervised panic handling. `release-perf`
inherits from `release`; `release-size` inherits from `release-perf`.
Cross-crate LTO generally increases build time.

The `profiling` profile inherits the performance optimizations and retains full
debug information and symbols (`debug = 2`, `strip = "none"`). Existing `release`,
`bench`, and `release-debug` settings are unchanged. A future change could merge
`release` and `release-perf` after evaluating throughput and build time on
representative workloads.

The size profile prioritizes code size and disables loop vectorization. These
are intended tradeoffs, not a promise that an optimization level always produces
the fastest or smallest executable. Measure throughput for the intended workload
before choosing a profile. See the [Cargo profile reference](https://doc.rust-lang.org/cargo/reference/profiles.html).

The experimental `release-balanced` profile (level `2`) was removed for now:
its ZIP savings over `release-perf` were only 2.9-3.8% (0.4-1.0 MiB) across
both architectures with and without Transform. No representative throughput
benefit has been demonstrated to justify another supported profile. It could
return if workload benchmarks show a useful tradeoff.

## Production diagnostics

Package the configured custom-profile output directly. In the measured Linux
x86_64 builds, retained symbols provide function names in backtraces and live
CPU profiles without DWARF file/line information. Additional `--strip-all` stripping
removes those names. A separate debug artifact cannot recover application frames
already discarded by the current CPU profiler: the pinned `pprof` backend drops
unresolved frames, and its exported profile omits instruction addresses and
mappings. The controller's short backtrace rendering also omits addresses for
unresolved frames. See [profiling](../PROFILING.md) for diagnostic builds.

Heap profiling requires the `jemalloc` feature and runtime profiling
configuration. The minimal feature selection measured below disables jemalloc;
its heap endpoint was not tested.

## Build

From `rust/otap-dataflow`, build only the engine binary and select the needed
features. For an OTLP receiver, Batch, Fan-out, and OTLP/gRPC plus OTAP exporters:

```sh
cargo build --locked -p otel-arrow-dfe --bin df_engine \
  --profile release-perf \
  --no-default-features --features otlp,otap,crypto-ring
```

Use `release-size` to prioritize binary size, or `--release` for the existing
default settings. Add `transform` when the pipeline uses Transform. Batch and
Fan-out are always available; the crypto feature retains TLS capability.
Output is under `target/<profile>/`, or
`target/<target-triple>/<profile>/` for cross builds.

## Binary-size comparison

Reevaluated on 2026-10-12 with function symbols retained. Each cell shows
**build / ZIP**, in MiB. ZIPs contain the exact custom-profile output.

| Profile | x86_64 | x86_64 + Transform | arm64 | arm64 + Transform |
| --- | ---: | ---: | ---: | ---: |
| `release` (pre-PR baseline) | 407.07 / 18.29 | Not measured | Not measured | Not measured |
| `release-perf` | 42.69 / 14.58 | 86.29 / 28.34 | 39.40 / 13.80 | 79.22 / 26.81 |
| `release-size` | 30.84 / 9.79 | 56.75 / 17.51 | 32.33 / 10.48 | 57.03 / 18.65 |

The historical `release` row is the native x86_64 build before this PR. Its ZIP
was regenerated with debug-only stripping, retaining function symbols. The
407.07 MiB build was not rebuilt or modified. Custom profiles use cargo-zigbuild
with a glibc 2.34 baseline, so differences from that historical build include
toolchain differences and cannot be attributed solely to profile settings.
The baseline has not been rebuilt in this matrix.

Without Transform, `release-size` saves 24.0-32.8% in ZIP size versus
`release-perf`. Compared with fully stripped copies of these same rebuilt
binaries, retaining function symbols adds 0.97-3.74 MiB to the ZIP across the
eight combinations. Symbol tables and names compress well, so their uncompressed
size is a poor proxy for package overhead.

Architecture also affects retained-symbol overhead. Without Transform,
`release-size` has a 5.18 MiB symbol table on arm64 versus 1.17 MiB on x86_64,
even though arm64 has a smaller code section. This contributes to the arm64
size-profile build being larger in this matrix.

The pipeline is OTLP receiver -> optional Transform -> Batch -> Fan-out ->
OTLP/gRPC and OTAP exporters. Transform uses OTTL to set log severity text;
selecting this statement still compiles the full Transform feature. Default
features are disabled in every row. Size results do not establish a
processing-throughput ranking or imply proportional startup improvements.

The eight custom-profile combinations shown above were built with Rust 1.98.1,
cargo-zigbuild 0.23.4, Zig 0.15.2, and a glibc 2.34 baseline, using named workspace
profiles without optimization overrides. ELF inspection verifies function symbol
tables, absence of bulk DWARF sections, architecture, and glibc requirements.
Both targets declare only libc and libm dependencies. MiB means 1,048,576 bytes.
Binary-only ZIPs use DEFLATE level 9 with matching timestamps and permissions.
These measurements come from the 2026-10-12 experiment; removing the balanced
profile leaves the remaining build settings unchanged, so their results are
reused without rebuilding.

### Cross-linker reproducibility

The measured Zig 0.15.2 toolchain treats rustc's `--strip-debug` linker argument
as full stripping. See the [Zig driver source][zig-strip-debug]. The experiment
used a local compatibility launcher that removed that argument before linking
and ran `llvm-strip --strip-debug` on the result. Stale executable cache entries
were bypassed when relinking. Every accepted artifact was checked for a retained
`.symtab`. This workaround does not change the profile's optimization settings
and is not part of the engine changes.

[zig-strip-debug]: https://github.com/ziglang/zig/blob/0.15.2/src/main.zig#L2608-L2613

With an unmodified copy of that Zig version, reproduce the same stripping policy
by disabling link-time stripping and applying debug-only stripping afterward:

```sh
cargo zigbuild --locked -p otel-arrow-dfe --bin df_engine \
  --profile release-size --config 'profile.release-size.strip="none"' \
  --target aarch64-unknown-linux-gnu.2.34 \
  --no-default-features --features otlp,otap,crypto-ring
llvm-strip --strip-debug target/aarch64-unknown-linux-gnu/release-size/df_engine
```

Install the pinned Rust target, LLVM tools, Zig, and cargo-zigbuild first, and
put `llvm-strip` on PATH. Use `x86_64-unknown-linux-gnu.2.34` for x86_64 and add
`transform` for the optional processor. Standard GNU-linked Cargo builds do not
need this measured Zig workaround. Verify symbol retention before packaging.
Build times depend on cache state and were not compared.

This Zig toolchain also warns that it ignores the deprecated linker optimization
setting `1` emitted by rustc; named Rust optimization and LTO settings remain in
effect.

## Frame-pointer comparison

Frame pointers are not forced by these profiles. A second set of all eight builds
adds `-C force-frame-pointers=yes`; the first set uses the compiler's target
defaults. The comparison below shows **build / ZIP**, in MiB, for the forced set.

| Profile | x86_64 | x86_64 + Transform | arm64 | arm64 + Transform |
| --- | ---: | ---: | ---: | ---: |
| `release-perf` | 42.07 / 14.59 | 84.75 / 28.34 | 39.62 / 13.84 | 79.57 / 26.86 |
| `release-size` | 30.15 / 9.65 | 55.15 / 17.23 | 32.55 / 10.50 | 57.43 / 18.69 |

Forced frame pointers change ZIP size by -1.61% to +0.31%
across the eight combinations, at most 0.28 MiB in absolute terms. Uncompressed
size changes by -2.82% to +0.70%. This measures binary size, not
processing-throughput overhead.

The host x86_64 configuration requests `-fuse-ld=mold`; Zig ignores that
selection flag. A linker control produced identical outputs with and without it.
The forced variants supply only the frame-pointer flag to Zig-linked builds.

These flags affect compiled Rust crates. Prebuilt Rust standard libraries and
native dependencies were not rebuilt with forced frame pointers. Enabling this
compiler flag does not switch the pinned CPU profiler to its optional
`frame-pointer` backend; the engine currently uses its default `backtrace`
backend. See [rustc's frame-pointer option](https://doc.rust-lang.org/rustc/codegen-options/index.html#force-frame-pointers).

### Stack-capture cost

A standalone fixture uses the engine's pinned `backtrace` 0.3.76 and `pprof`
0.15.0, the same two profiles, and identical native linker settings across the
frame-pointer variants. The fixture uses the native GNU unwinder (`libgcc_s`);
the cross-built engines use Zig-linked LLVM `libunwind`. These API costs do not
measure the engine's Zig unwinder directly. Non-inlined recursion supplies depths
16 and 64. Each operation uses warmed caches and nine adaptive batches targeting
20 ms each, with the process pinned to one x86_64 CPU. Cells show median
microseconds per operation at **depth 16 / depth 64**.

| Profile | Frame pointers | pprof unwind (us) | Capture (us) | Capture + render (us) |
| --- | --- | ---: | ---: | ---: |
| `release-perf` | target-default | 3.11 / 8.58 | 5.18 / 13.98 | 21.59 / 61.19 |
| `release-perf` | forced | 3.17 / 9.00 | 5.25 / 14.41 | 21.78 / 61.86 |
| `release-size` | target-default | 3.64 / 9.76 | 5.90 / 15.64 | 23.38 / 64.38 |
| `release-size` | forced | 3.45 / 9.69 | 5.68 / 15.51 | 23.77 / 66.19 |

`pprof unwind` measures `backtrace::trace_unsynchronized`, the current CPU
profiler's stack-walking API. `Capture` measures
`std::backtrace::Backtrace::force_capture`; `Capture + render` also calls
`to_string`, as the controller does when logging a captured panic. The fixture
includes recursion overhead and has fewer symbols than the engine. It excludes
signal delivery, profile aggregation, logger I/O, and first-capture cold-cache
costs. These are API measurements, not production logging or throughput results.

Frame-pointer effects vary by operation and optimization level in this fixture.
The current profiler still uses its existing unwinder; these results do not
establish a benefit from forcing frame pointers in the release profiles.

All four fixture variants retain logging function names and application frames
in sampled CPU profiles. A fully stripped control loses both. The control's
exported CPU profile has no addresses or mappings for later symbolization.
The real engine CPU endpoint passed checks for all eight retained x86_64 variants,
with application function names under OTLP traffic. In the earlier experiment,
live profiling of the removed level-2 profile with Transform intermittently
segfaulted on the local host; GDB reproduced a fault in the Zig-linked
`libunwind` register-restoration path during `SIGPROF`,
interrupted inside glibc allocation code. The host runs glibc 2.43; the binaries
target glibc 2.34. The cause has not been isolated to profile settings, and
retaining symbols or forcing frame pointers does not establish profiler
reliability. This requires separate unwinder/toolchain investigation; ordinary
pipeline and shutdown checks passed. The profiler setup and unwinding backend
are unchanged by this PR. Removing the balanced profile does not resolve this
shared-backend reliability issue.

## Startup optimization

The CLI help and startup banner each construct the system-information string.
Previously both used `sysinfo::System::new_all()`, collecting CPU and process
information the banner does not use. Available core counts come from
`std::thread::available_parallelism()`. The banner now creates an empty
`System` and refreshes only memory before displaying the same core and memory
information.

In the isolated native-release comparison using the original profile settings,
median OTLP receiver readiness fell from 748.3 ms to 36.5 ms over ten launches
per variant. The stripped executable changed by only 232 bytes. This demonstrates
reduced initialization work independently of binary size.

Measurements of the function-symbol-retaining outputs, with target-default
frame pointers and ten launches per variant:

| Profile | Transform | Receiver ready median (ms) | Both exports delivered median (ms) | RSS median (MiB) |
| --- | --- | ---: | ---: | ---: |
| `release-perf` | No | 33.7 | 158.1 | 25.3 |
| `release-perf` | Yes | 35.7 | 164.9 | 39.9 |
| `release-size` | No | 34.3 | 143.7 | 21.4 |
| `release-size` | Yes | 37.9 | 151.2 | 32.0 |

Local readiness is measured from process launch until the OTLP TCP listener
accepts a connection. First delivery is measured until independent OTLP and OTAP
sinks have both received the test log; it includes the 100 ms batch window.
Transform runs verify the changed severity text at both sinks. The test sends
logs over OTLP/HTTP protobuf and checks successful SIGTERM shutdown. The forced
frame-pointer variants passed the same checks, for 80 pipeline launches across
the two retained profiles.

These are warm-filesystem-cache measurements on an x86_64 host with many
processes. Arm64 was cross-built and ELF-inspected, not executed locally. These
are engine binaries, not complete Lambda extension layers. AWS cold starts,
native arm64 startup, TLS handshakes, all-signal workloads, sustained throughput,
retry behavior, and Lambda invocation/freeze/shutdown coordination need separate
validation on the deployment image.
