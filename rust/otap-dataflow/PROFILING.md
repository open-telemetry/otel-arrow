# Profiling

This section covers CPU and memory profiling for `df_engine`.

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

## Live heap profiling (jemalloc pprof)

The admin server exposes a heap profiling endpoint at
`/api/v1/debug/pprof/heap` that dumps unfreed heap allocations in pprof
format. This requires jemalloc as the allocator (the default on Linux and
macOS) with profiling enabled.

**Run** with profiling enabled:

```bash
_RJEM_MALLOC_CONF="prof:true,prof_active:true,lg_prof_sample:19" \
  cargo run -- --config ./configs/otap-noop.yaml
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
