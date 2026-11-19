# Profiling

This section covers CPU and memory profiling for `df_engine`.

The `profiling` build profile uses the level `3`, fat LTO, and single codegen
unit settings from `release-perf`, while retaining full debug information and
symbols. This enables profiling with the production optimization settings.

The custom release profiles remove debug information and retain function
symbols. On Linux, this preserves application function names in backtraces and
live CPU profiles. Source file/line information requires a build with debug
information. Package the configured release output directly; an additional
`--strip-all` pass removes the function names again.

The pinned CPU profiler discards unresolved frames and exports neither their
instruction addresses nor mappings. Separate debug artifacts cannot recover
those frames after export. Retaining function symbols in the deployed binary
keeps the current production profiling path useful. See the
[release profile measurements](docs/release-profiles.md#stack-capture-cost)
for diagnostic validation and observed toolchain limitations.

## CPU profiling (samply)

**Requirements**:

- [samply](https://github.com/mstange/samply)

**Installation**:

```cmd/pwsh/bash
cargo install --locked samply
```

**Build**:

```cmd/pwsh/bash
cargo build --profile profiling --workspace
```

**Run**:

```pwsh/bash
samply record ./target/profiling/df_engine --config ./configs/otap-noop.yaml
```

**Result**:

CPU profiling output will be automatically rendered on browser.

## Memory profiling (dhat)

**Build**:

```cmd/pwsh/bash
cargo build --profile profiling --no-default-features --features dhat-heap --workspace
```

> [!NOTE]
> In this command, all default features are disabled.
> Use specific flags to enable individual features.

**Run**:

```pwsh/bash
./target/profiling/df_engine --config ./configs/otap-noop.yaml
```

**Result**:

On graceful shutdown of `df_engine`, it will generate `dhat-heap.json` file
for memory profiling that needs to be rendered by uploading it to:
<https://nnethercote.github.io/dh_view/>.

> [!NOTE]
> `dhat` needs a clean shutdown to generate `dhat-heap.json` file.

## Heap profiling (mimalloc pprof)

Select mimalloc v3 with the existing `mimalloc` feature. Cargo builds Microsoft's
Rust bindings and the native library from a pinned Git fork of v3.7.0. The fork
adds bounded in-memory snapshots for the admin endpoint and enables stack
capture in the Linux Rust build. Heap sampling is
off by default. Set `MIMALLOC_PROFILE` to a writable file prefix before starting
the engine to collect sampled allocations made through mimalloc.

```bash
cargo build --bin df_engine --no-default-features \
  --features core-nodes,dev-tools,admin-live-logs-ws,mimalloc,crypto-ring
profile_dir=$(mktemp -d)
MIMALLOC_PROFILE="$profile_dir/heap" \
  ./target/debug/df_engine --config ./configs/otap-noop.yaml \
  --http-admin-bind 127.0.0.1:8080
```

Mimalloc writes numbered snapshots (`heap.0001.pb`, `heap.0002.pb`, etc.) as
the engine runs and takes a final snapshot on normal exit. Use a new prefix for
each run. By default a snapshot is triggered after about 1 GiB of allocations.
Adjust `MIMALLOC_PROFILE_ALLOC_INTERVAL` to change that threshold, or set it to
`0` to disable allocation-triggered snapshots. See [mimalloc's profiling
options](https://github.com/microsoft/mimalloc/blob/v3.7.0/readme.md#profiling)
for other triggers and sampling settings.

Fetch a live snapshot from the admin server while profiling is active.

```bash
curl --fail http://127.0.0.1:8080/api/v1/debug/pprof/heap -o heap.pb
```

The endpoint takes a fresh snapshot of sampled allocations and returns it as
protobuf without writing a file for the request. Concurrent snapshots return
HTTP 429. Responses are limited to 16 MiB. Set all three snapshot intervals to
`0` to skip periodic files and keep the final snapshot at shutdown.

Use [Google's pprof tool](https://github.com/google/pprof) with the matching
engine binary to resolve function names and inspect live bytes. Pass either the
HTTP download or one of the numbered `.pb` files.

```bash
pprof -sample_index=inuse_bytes -http=localhost:18080 \
  ./target/debug/df_engine heap.pb
```

## Live heap profiling (jemalloc pprof)

Jemalloc is the default allocator on Linux and macOS. The same endpoint uses
jemalloc's profiler. Per-pipeline heap allocation metrics and the
`jemalloc_resident` and `purge_on_hard` memory limiter options also require
a jemalloc build.

```bash
cargo build --bin df_engine
```

**Run** with profiling enabled:

```bash
_RJEM_MALLOC_CONF="prof:true,prof_active:true,lg_prof_sample:19" \
  ./target/debug/df_engine --config ./configs/otap-noop.yaml
```

On Linux, use `MALLOC_CONF` instead of `_RJEM_MALLOC_CONF`.

**Fetch** a heap profile:

```bash
curl http://localhost:8080/api/v1/debug/pprof/heap -o heap.pprof
```

**View** with `go tool pprof`:

```bash
go tool pprof -http=:18080 ./target/debug/df_engine ./heap.pprof
```

## Live heap profiling (snmalloc pprof)

Select snmalloc alongside the other default engine features.

```bash
cargo build --bin df_engine --no-default-features \
  --features core-nodes,dev-tools,admin-live-logs-ws,snmalloc,crypto-ring
```

Set `SNMALLOC_PROFILE=1` before starting the engine to sample Rust allocations
made through its global allocator. Start the admin server and fetch a live
protobuf profile using the same endpoint as the other allocators.

```bash
SNMALLOC_PROFILE=1 ./target/debug/df_engine --config ./configs/otap-noop.yaml \
  --http-admin-bind 127.0.0.1:8080
```

```bash
curl --fail http://127.0.0.1:8080/api/v1/debug/pprof/heap -o heap.pb
pprof -sample_index=inuse_space -http=localhost:18080 \
  ./target/debug/df_engine heap.pb
```

Sampling starts during engine startup. It covers Rust global allocations from
that point forward. The endpoint returns live and interval allocation
samples and limits the response to 16 MiB. Direct C allocations are outside
the Rust global allocator's coverage.

All three builds expose process RSS and their available native memory
statistics through [engine metrics](docs/self-observability.md#engine-pipeline-and-runtime)
independently of heap sampling.

## Live CPU profiling (pprof)

The admin server exposes a CPU profiling endpoint at
`/api/v1/debug/pprof/profile` that collects a CPU profile and returns it
in pprof format. Not available on Windows.

Optional query parameters:

- `seconds` -- sampling duration in seconds (default 30).
- `frequency` -- sampling frequency in Hz (default 100).

**Fetch** a 10-second CPU profile:

```bash
curl "http://localhost:8080/api/v1/debug/pprof/profile?seconds=10" -o cpu.pprof
```

**View** with `go tool pprof`:

```bash
go tool pprof -http=:18080 ./target/debug/df_engine ./cpu.pprof
```
