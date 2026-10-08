# DFE release build profiles

DFE preserves its existing `release` build settings and provides three opt-in
profiles for different binary-size and processing performance priorities.
Choose the profile independently of the component features required by a
pipeline.

| Profile | Priority | Optimization level | Debug information | Symbol stripping |
| --- | --- | --- | --- | --- |
| `release` | Existing release build | `3` (Cargo default) | Line tables for profiling | None |
| `release-perf` | Runtime performance | `3` | Line tables for profiling | None |
| `release-balanced` | Compact binary with speed-oriented optimization | `2` | None | Symbols |
| `release-size` | Minimum binary size, accepting potential throughput loss | `z` | None | Symbols |

All three custom profiles use fat LTO, one codegen unit, disabled incremental
compilation, and `panic="unwind"`. Unwinding preserves the controller's
supervised panic handling.
`release-perf` inherits from `release`; `release-balanced` and `release-size`
inherit from `release-perf`. Existing `release`, `profiling`, `bench`, and
`release-debug` build settings are unchanged. The custom profiles take longer
to compile than the default release build without cross-crate LTO.

The default `release` profile is left unchanged for now. A future change could
merge `release` and `release-perf` after evaluating throughput and build time
on representative workloads.

The balanced profile retains speed-oriented optimization and loop vectorization
while avoiding some level-3 code expansion. The size profile prioritizes code
size and disables loop vectorization. These are intended tradeoffs, not a
promise that any level always produces the fastest or smallest executable.
Measure throughput for the intended workload before choosing a profile. See the
[Cargo profile reference](https://doc.rust-lang.org/cargo/reference/profiles.html).

## Build

From `rust/otap-dataflow`, build only the engine binary and select the needed
features. For an OTLP receiver, Batch, Fan-out, and OTLP/gRPC plus OTAP exporters:

```sh
cargo build --locked -p otel-arrow-dfe --bin df_engine \
  --profile release-balanced \
  --no-default-features --features otlp,otap,crypto-ring
```

Replace `release-balanced` with `release-perf` or `release-size` as appropriate.
Use `--release` for the existing default release build settings.
Add `transform` to the feature list when the pipeline uses Transform. Batch and
Fan-out are always available; the crypto feature retains TLS capability.
The output is under `target/<profile>/`, or
`target/<target-triple>/<profile>/` for cross builds.

## Binary-size comparison

Normal Cargo output, using each profile's configured debug and stripping settings:

| Profile | x86_64 without Transform (MiB) | x86_64 with Transform (MiB) | arm64 without Transform (MiB) | arm64 with Transform (MiB) |
| --- | ---: | ---: | ---: | ---: |
| `release` (unchanged settings, pre-PR baseline) | 407.07 | Not measured | Not measured | Not measured |
| `release-perf` | 170.35 | 385.07 | 176.28 | 401.63 |
| `release-balanced` | 35.09 | 69.64 | 30.34 | 59.14 |
| `release-size` | 22.75 | 41.36 | 19.15 | 32.32 |

`release` and `release-perf` retain line tables and symbols; the compact profiles
disable debug information and strip symbols. The `release` row is the historical
native x86_64 build before this PR, using the same minimal feature selection and
the release settings preserved by this PR. The custom-profile rows use
cargo-zigbuild with a glibc 2.34 baseline. Therefore, the 407.07-to-170.35 MiB
difference includes toolchain differences and cannot be attributed solely to LTO
and one codegen unit. The default `release` has not been rebuilt in the custom
profiles' toolchain matrix.

Deployment copies after additional stripping, with binary-only ZIP sizes:

| Profile | Transform | x86_64 stripped (MiB) | x86_64 ZIP (MiB) | arm64 stripped (MiB) | arm64 ZIP (MiB) |
| --- | --- | ---: | ---: | ---: | ---: |
| `release-perf` | No | 36.14 | 13.61 | 31.36 | 12.62 |
| `release-perf` | Yes | 72.10 | 26.35 | 62.09 | 24.42 |
| `release-balanced` | No | 35.09 | 13.19 | 30.34 | 12.19 |
| `release-balanced` | Yes | 69.64 | 25.39 | 59.14 | 23.40 |
| `release-size` | No | 22.75 | 8.56 | 19.15 | 8.46 |
| `release-size` | Yes | 41.36 | 15.26 | 32.32 | 14.90 |

Without Transform, `release-balanced` reduces stripped size by about 3% versus
`release-perf`; `release-size` reduces it by 37-39%. Much of the reduction in
normal Cargo output comes from removing debug and symbol information. These size
results do not establish a processing-throughput ranking.

The pipeline is OTLP receiver -> optional Transform -> Batch -> Fan-out ->
OTLP/gRPC and OTAP exporters. The Transform configuration uses OTTL to set log
severity text. Selecting this single statement still compiles the full
Transform feature. The default feature bundle is disabled in every row.

Custom-profile measurements use Rust 1.98.1, cargo-zigbuild 0.23.4, Zig 0.15.2,
and a glibc 2.34 baseline. Each artifact was built with a named workspace profile,
without experimental profile overrides. The performance artifacts were measured
with the optimized settings under the name `release`, before those settings moved
to `release-perf`.
The compact profiles retain the same effective optimization settings. These
tables reuse those measurements; the artifacts were not rebuilt when the
performance profile was renamed.
MiB means 1,048,576 bytes. ZIPs contain one executable and use DEFLATE level 9
with matching timestamps and permissions.

The stripped columns make code-size comparisons consistent across profiles.
`release-perf` intentionally retains line tables and symbols in its normal build;
its deployment copy was stripped separately for this table. The two compact
profiles already strip their normal output. Debug and symbol information
changes package size without implying proportional startup or throughput gains.

The custom-profile measurements include the startup fix below;
the historical `release` baseline predates it. They are engine binaries, not
complete Lambda extension layers. Both custom-profile targets declare only libc
and libm dependencies; compatibility and behavior still need testing on the
actual deployment image. Arm64 is cross-built and ELF-inspected, not executed
locally.

To reproduce the cross builds, install the pinned Rust arm64 target,
LLVM tools, Zig, and cargo-zigbuild. For example:

```sh
cargo zigbuild --locked -p otel-arrow-dfe --bin df_engine \
  --profile release-size --target aarch64-unknown-linux-gnu.2.34 \
  --no-default-features --features otlp,otap,crypto-ring
```

Use `x86_64-unknown-linux-gnu.2.34` for x86_64 and add `transform` for the optional
processor. Retain the normal output first, then use `llvm-strip --strip-all`
for the comparison copy. Build times depend on cache state and are not measured
as a profile-performance comparison.

This Zig toolchain warns that it ignores the deprecated linker optimization
setting `1` emitted by rustc. The named Rust optimization and LTO settings
remain in effect; the full workspace check reports no source warnings.

## Startup optimization

The CLI help and startup banner each construct the system-information string.
Previously both used `sysinfo::System::new_all()`, collecting CPU and process
information the banner does not use. Available core counts come from
`std::thread::available_parallelism()`. The banner now creates an empty
`System` and refreshes only memory before displaying the same core and memory
information.

In the isolated native-release comparison using the original profile settings,
median OTLP receiver readiness
fell from 748.3 ms to 36.5 ms over ten launches per variant. The stripped
executable changed by only 232 bytes. These measurements demonstrate reduced
initialization work independently of binary size.

| Profile | Transform | Receiver ready median (ms) | Both exports delivered median (ms) | RSS median (MiB) |
| --- | --- | ---: | ---: | ---: |
| `release-perf` | No | 33.5 | 143.7 | 24.2 |
| `release-perf` | Yes | 34.6 | 148.6 | 36.6 |
| `release-balanced` | No | 34.3 | 144.1 | 24.1 |
| `release-balanced` | Yes | 36.4 | 150.2 | 35.0 |
| `release-size` | No | 34.2 | 142.9 | 19.9 |
| `release-size` | Yes | 37.3 | 151.1 | 30.0 |

Local readiness is measured from process launch until the OTLP TCP listener
accepts a connection. First delivery is measured until separate OTLP and OTAP
sinks have both received the test log; it includes the 100 ms batch window.
Transform runs verify the changed severity text at both sinks. The test sends
logs over OTLP/HTTP protobuf and checks successful SIGTERM shutdown.

These are warm-filesystem-cache measurements on an x86_64 host with many
processes, not AWS cold-start estimates or sustained-throughput benchmarks.
Native arm64 startup, TLS handshakes, all-signal workloads, retry behavior, and
Lambda invocation/freeze/shutdown coordination need separate validation.
