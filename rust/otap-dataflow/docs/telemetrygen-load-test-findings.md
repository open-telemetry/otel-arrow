# df_engine Memory Under Load: Experiment Report

## Objective

Investigate the memory behavior of `df_engine` (a Rust-based OpenTelemetry
dataflow engine) under high-volume log ingestion via `telemetrygen`. Identify
where memory goes, which configuration knobs control it, and the trade-offs
between throughput and memory usage.

## Test Setup

**Pipeline**: OTLP gRPC receiver → batch processor → noop exporter

**Load generator**: `telemetrygen logs` from `opentelemetry-collector-contrib`,
generating ~2 KB log records (two ~1 KB attribute strings per record), batched
in groups of 100 (the default `--batch-size`).

**Engine**: `df_engine` release build with jemalloc allocator, `--num-cores 1`.

**Metrics**: Prometheus pull exporter at `http://127.0.0.1:9090/metrics` for
engine telemetry; `/proc/PID/status` sampled every 2 seconds for RSS/Anon.

**Common telemetrygen command**:
```bash
telemetrygen logs \
  --otlp-endpoint localhost:4317 --otlp-insecure \
  --rate 20000 --duration 30s --workers N \
  --telemetry-attributes 'key1="<~1KB>"' \
  --telemetry-attributes 'key2="<~1KB>"'
```

## Part 1: telemetrygen Throughput Characteristics

### Finding: A single worker cannot reach 20k logs/sec

telemetrygen's logs implementation **bypasses the OTel SDK entirely** — there is
no `LoggerProvider`, no `BatchLogRecordProcessor`, no async queue. Each worker
builds records with `logtest.RecordFactory` and calls `exporter.Export()`
**synchronously**, blocking until the gRPC response arrives before generating
the next batch.

The `--rate` flag is a token bucket limiter on *generation*, not on export. When
export is the bottleneck, `--rate` has no effect.

### Finding: `--rate` is per-worker

With `--workers 10 --rate 20000`, each worker independently requests 20,000/sec,
for a total of 200,000/sec requested. The actual achieved rate depends on export
round-trip latency.

### Throughput scaling results (30s, noop exporter, pdata=128)

| Client Configuration | Total Logs | Achieved Rate | Requests |
|---------------------|-----------|---------------|----------|
| 1 worker (default) | 58,140 | ~1,938/sec | 572 |
| 10 separate processes | 565,700 | ~18,856/sec | ~5,700 |
| 1 process, 10 workers | 1,276,936 | ~42,564/sec | ~12,700 |
| 1 worker, wait_for_result | 1,000 | ~33/sec | 10 |

A single worker achieved only ~1,900/sec because each Export call takes ~0.5 ms
round-trip, and with 100 records per batch that gives
100 records × 1/(0.5ms) = ~200k records/sec theoretical maximum — but actual
overhead brings this down to ~1,900/sec sustained. Using 10 workers multiplies
this nearly linearly.

## Part 2: Memory Behavior Under Load

All tests below use 10 workers, 30-second duration, same ~2 KB records.

### Baseline: Standard config (pdata=128, no wait)

**Config** (`otlp-batch-noop.yaml`):
- `channel_capacity.pdata: 128`
- `max_concurrent_requests`: auto (defaults to pdata capacity = 128)
- Batch: `otlp.min_size: 65536` (64 KB), `flush_timeout: 3s`

**Memory timeline** (sampled via `/proc/PID/status` every 2s):

```
Baseline:  RSS=25 MB, Anon=9 MB
t=2s:      RSS=63 MB, Anon=44 MB
t=4s:      RSS=109 MB, Anon=90 MB
t=8s:      RSS=153 MB, Anon=134 MB
t=10s:     RSS=88 MB, Anon=69 MB   (← batch flush released data)
t=18s:     RSS=167 MB, Anon=150 MB
t=28s:     RSS=235 MB, Anon=216 MB (← PEAK)
t=34s:     RSS=57 MB, Anon=38 MB   (post-load, settling)
```

**Key metrics** (from Prometheus):
- Total requests completed: 10,882
- Total request bytes: 2.32 GB
- Average bytes/request: ~213 KB (100 records × ~2 KB each, plus protobuf overhead)
- Batch processor: consumed 10,882 input batches → produced 18 output batches (all size-triggered)
- jemalloc per-core heap: 1.3 MB (post-load)

**Analysis**: Peak RSS of 235 MB is explained by:

| Component | Estimated Memory |
|-----------|-----------------|
| pdata channel (128 × ~213 KB) | ~27 MB |
| gRPC server buffers (128 concurrent requests × recv+send buffers) | ~30-50 MB |
| Batch processor accumulator (aggregating toward 64 KB flush) | ~1-5 MB |
| jemalloc arena retention (allocated but freed, not returned to OS) | ~100-150 MB |
| Binary + libraries mapped | ~21 MB |

The dominant factor is **jemalloc arena retention**: jemalloc allocated ~11 GB
total over 30 seconds (`memory_allocated` metric) and freed ~11 GB
(`memory_freed`), but retains virtual memory mappings for reuse. The 24 MB
anonymous region at address `0x7c5a54400000` is a single jemalloc arena — 61 MB
mapped, 24 MB resident. The sawtooth pattern (RSS rising then dropping) aligns
with the 3-second batch flush timeout releasing accumulated data.

### Low-memory config (pdata=8, reduced batch)

**Config** (`otlp-batch-noop-lowmem.yaml`):
- `channel_capacity.pdata: 8`
- `max_concurrent_requests: 8`
- Batch: `otlp.min_size: 32768` (32 KB), `flush_timeout: 1s`

**Memory timeline**:

```
Baseline:  RSS=25 MB, Anon=9 MB
t=2s:      RSS=65 MB, Anon=47 MB
t=6s:      RSS=77 MB, Anon=58 MB
t=14s:     RSS=74 MB, Anon=55 MB
t=22s:     RSS=99 MB, Anon=80 MB   (← PEAK)
t=36s:     RSS=83 MB, Anon=65 MB   (post-load)
```

**Analysis**: Peak dropped from 235 MB to 99 MB — a **58% reduction**. However,
throughput was essentially unchanged (~35k/sec vs ~36k/sec), because the noop
exporter drains so fast that the smaller channel rarely fills. The improvement
comes from bounding the maximum in-flight data: 8 messages × 213 KB = ~1.7 MB
in channels (vs 27 MB with pdata=128), which reduces jemalloc's high-water mark.

Post-load RSS was higher (83 MB vs 57 MB) — this is jemalloc retention behavior
and not meaningful; the allocator retains memory pools sized to peak usage.

### wait_for_result (backpressure to client)

**Config** (`otlp-batch-noop-waitresult.yaml`):
- `channel_capacity.pdata: 128` (same as baseline)
- `wait_for_result: true` on gRPC receiver
- Same batch config as baseline

**Memory timeline**:

```
Baseline:  RSS=26 MB, Anon=9 MB
t=2s:      RSS=30 MB, Anon=11 MB
t=8s:      RSS=36 MB, Anon=18 MB
t=22s:     RSS=37 MB, Anon=19 MB   (← PEAK)
t=36s:     RSS=35 MB, Anon=17 MB   (post-load)
```

**Key metrics**:
- Requests completed: **100** (vs 10,882 baseline)
- Total logs: **10,000** (vs 1,087,973 baseline)
- Request bytes: 21 MB (vs 2.32 GB baseline)
- jemalloc per-core heap: 0.84 MB

**Analysis**: `wait_for_result` is the most effective memory control, holding
peak RSS to just **37 MB** — a 6.3× reduction from the 235 MB baseline. The
mechanism is simple: the gRPC handler doesn't respond until the data has
traversed the entire pipeline and the exporter has acknowledged it. With 10
telemetrygen workers, at most 10 requests can be in-flight at any time (each
worker blocks on its synchronous Export call). This means:

- Max in-flight data: 10 × 213 KB = ~2.1 MB
- No channel backlog: data enters and exits before new data arrives
- No jemalloc retention spike: peak allocation is tiny

The cost is throughput: only **333 logs/sec** (vs ~36,000/sec), a **99%
reduction**. Each worker completes ~10 full round-trips in 30 seconds (~3
seconds per round-trip), because the batch processor's `flush_timeout: 3s`
becomes the bottleneck — data sits in the batch accumulator until the timer
fires.

## Part 3: Summary of Configuration Knobs

| Knob | Location | Effect on Memory | Effect on Throughput |
|------|----------|-----------------|---------------------|
| `channel_capacity.pdata` | Pipeline policies | Bounds max messages buffered between nodes | Minimal with fast exporter |
| `max_concurrent_requests` | Receiver config | Limits gRPC requests accepted concurrently | Backpressure when saturated |
| `wait_for_result` | Receiver gRPC/HTTP config | **Dramatic** — bounds in-flight to client count | **Dramatic** — throughput drops to flush_timeout⁻¹ |
| `otlp.min_size` | Batch processor | Controls flush granularity | Minimal |
| `flush_timeout` | Batch processor | Controls max data residence time | With wait_for_result, directly limits throughput |
| `--workers` | telemetrygen | N/A (client-side) | Linear scaling up to server capacity |

## Part 4: Profiling Attempts

### heaptrack

We attempted to use `heaptrack` for detailed allocation profiling. heaptrack
uses `LD_PRELOAD` to intercept `malloc`/`free` calls.

- **With jemalloc build**: heaptrack's interceptors never see allocations because
  Rust's `#[global_allocator]` routes directly to jemalloc, bypassing glibc
  malloc entirely. The output file was 0 bytes.
- **With system allocator build** (`--no-default-features`): heaptrack captured
  startup allocations (256 KB output), but the engine crashed under any
  significant load. The combination of heaptrack's per-allocation bookkeeping
  overhead and the high allocation rate of the async runtime proved unstable.

### /proc/PID/smaps analysis

The most effective profiling approach was direct `/proc/PID/smaps` inspection:

**Post-load anonymous memory regions** (baseline config):
```
24,396 kB  0x7c5a54400000  (jemalloc arena, 61 MB mapped, 24 MB resident)
 3,752 kB  0x7c5a0de00000  (jemalloc arena)
 3,320 kB  binary .bss/.data
 1,752 kB  0x7c5aa9000000  (jemalloc arena)
 1,568 kB  0x7c5a64000000  (jemalloc arena)
 1,388 kB  0x7c5a64e00000  (jemalloc arena)
```

**Memory by mapping type**:
```
36,624 kB  [anonymous]  (jemalloc arenas + tokio runtime + gRPC buffers)
20,728 kB  df_engine binary (code + read-only data)
 1,432 kB  libc.so
   464 kB  libm.so
```

### jemalloc internal metrics

The engine exposes per-core jemalloc thread-local stats via `tikv-jemalloc-ctl`:
- `memory_usage`: current thread-local heap (post-load: ~1.3 MB)
- `memory_allocated` / `memory_freed`: cumulative counters (~11 GB each over 30s)
- The tiny delta (1.3 MB) vs large RSS (57 MB) confirms jemalloc retention as
  the dominant memory consumer

jemalloc profiling via `MALLOC_CONF=prof:true` was attempted but produced no
output — the crate (`tikv-jemallocator 0.6.1`) is built without
`--enable-prof`. The codebase has a TODO noting `jemalloc_pprof` as a future
integration path for heap profiling.

## Conclusions

1. **Memory under load scales with concurrency, not throughput**. The key factor
   is how many requests are in-flight simultaneously, not the data rate. Each
   in-flight request holds ~213 KB, and jemalloc retains arena pages well beyond
   the peak.

2. **`wait_for_result` is the strongest memory bound** but is impractical for
   high-throughput testing because it serializes the pipeline on
   `flush_timeout`. It's appropriate for production deployments where
   backpressure to the client is acceptable.

3. **Reducing `pdata` channel capacity** (128 → 8) cut peak RSS by ~58% with
   negligible throughput impact when the exporter is fast. With a slow exporter,
   smaller channels would also create backpressure sooner.

4. **jemalloc retention dominates post-load RSS**. The engine's actual live heap
   is ~1.3 MB after load, but RSS stays at 57-83 MB because jemalloc keeps its
   arenas mapped. This is not a leak — it's the allocator's design for
   performance. The `MALLOC_CONF` option `dirty_decay_ms` could potentially
   accelerate page return to the OS.

5. **For realistic load testing**, use `telemetrygen --workers N` to generate
   parallel load. The `--rate` flag is per-worker and export is synchronous, so
   a single worker caps at ~1,900 logs/sec against a local df_engine.

## Appendix: Complete Results Table

| Config | Workers | Logs (30s) | Rate/s | Peak RSS | Post-load RSS | Requests | Bytes |
|--------|---------|-----------|--------|----------|---------------|----------|-------|
| Baseline (pdata=128) | 1 | 58,140 | 1,938 | ~44 MB | 44 MB | 572 | ~122 MB |
| Baseline (pdata=128) | 10 procs | 565,700 | 18,856 | ~82 MB | 82 MB | ~5,700 | ~1.2 GB |
| Baseline (pdata=128) | 10 workers | 1,087,973 | 36,266 | **235 MB** | 57 MB | 10,882 | 2.32 GB |
| Lowmem (pdata=8) | 10 workers | ~1,050,000 | ~35,000 | **99 MB** | 83 MB | ~10,500 | ~2.2 GB |
| wait_for_result | 10 workers | 10,000 | 333 | **37 MB** | 35 MB | 100 | 21 MB |
| wait_for_result | 1 worker | 1,000 | 33 | ~31 MB | 31 MB | 10 | ~2 MB |

## Appendix A: Test Configs

Three config files were created in `configs/`:
- `otlp-batch-noop.yaml` — Baseline: pdata=128, default concurrency
- `otlp-batch-noop-lowmem.yaml` — Low memory: pdata=8, max_concurrent_requests=8, smaller batch
- `otlp-batch-noop-waitresult.yaml` — Backpressure: wait_for_result=true on gRPC

## Appendix B: Command Lines and Profiling Methods

### Building the binaries

```bash
# telemetrygen (from opentelemetry-collector-contrib repo)
cd /path/to/opentelemetry-collector-contrib/cmd/telemetrygen
go build -o /tmp/telemetrygen .

# df_engine with jemalloc (default, used for all load tests)
cd /path/to/otap-dataflow
cargo build --release --bin df_engine

# df_engine with system allocator (used for heaptrack attempt)
cargo build --release --bin df_engine --no-default-features
```

### Running the engine

```bash
# Standard run
./target/release/df_engine --config configs/otlp-batch-noop.yaml --num-cores 1

# With backtrace on panic
RUST_BACKTRACE=1 ./target/release/df_engine --config configs/otlp-batch-noop.yaml --num-cores 1
```

### telemetrygen command (all tests)

```bash
# Single worker (baseline throughput measurement)
/tmp/telemetrygen logs \
  --otlp-endpoint localhost:4317 --otlp-insecure \
  --rate 20000 --duration 30s \
  --telemetry-attributes 'key1="<~1KB of repeated a>"' \
  --telemetry-attributes 'key2="<~1KB of repeated b>"'

# 10 workers (primary load test configuration)
/tmp/telemetrygen logs \
  --otlp-endpoint localhost:4317 --otlp-insecure \
  --rate 20000 --duration 30s --workers 10 \
  --telemetry-attributes 'key1="<~1KB of repeated a>"' \
  --telemetry-attributes 'key2="<~1KB of repeated b>"'
```

Note: `--batch` defaults to `true` and `--batch-size` defaults to `100`.
telemetrygen groups 100 log records into each `Export()` call. However, each
record is generated individually inside the worker loop — the "batch" is just a
client-side buffer of individually-constructed records sent in a single gRPC
request. The records are not pre-aggregated or compressed before export.

### Memory profiling via /proc (recommended approach)

This was the most reliable method. Poll `/proc/PID/status` periodically during
the test:

```bash
# Get engine PID
DF_PID=$(ps -eo pid,args | grep "target/release/df_engine" | grep -v grep | awk '{print $1}')

# Sample VmRSS and RssAnon every 2 seconds during a 30s test
for i in $(seq 1 18); do
  sleep 2
  RSS=$(awk '/VmRSS/{print $2}' /proc/$DF_PID/status)
  ANON=$(awk '/RssAnon/{print $2}' /proc/$DF_PID/status)
  echo "t=$((i*2))s RSS=${RSS}kB Anon=${ANON}kB"
done
```

For a post-load breakdown of where anonymous memory lives:

```bash
# Top anonymous regions by resident size
awk '/^[0-9a-f]/{addr=$1; name=$6} /Anonymous:/{if($2>1024) printf "%8d kB  %s  %s\n", $2, addr, name}' \
  /proc/$DF_PID/smaps | sort -rn | head -20

# Aggregate RSS by mapping source
awk '/^[0-9a-f]/{name=$6; if(name=="") name="[anon]"} /^Rss:/{rss[name]+=$2} END{for(n in rss) if(rss[n]>100) printf "%8d kB  %s\n", rss[n], n}' \
  /proc/$DF_PID/smaps | sort -rn | head -20
```

### Prometheus metrics (engine telemetry)

The engine config includes a Prometheus pull exporter. During or after a test:

```bash
# Key memory metrics
curl -s http://127.0.0.1:9090/metrics | grep -E "memory_rss|memory_usage"

# Receiver throughput
curl -s http://127.0.0.1:9090/metrics | grep -E "request_bytes|requests_completed|requests_started"

# Batch processor behavior
curl -s http://127.0.0.1:9090/metrics | grep -E "consumed_items|consumed_batches|produced_items|produced_batches"

# jemalloc cumulative allocation counters
curl -s http://127.0.0.1:9090/metrics | grep -E "memory_allocated|memory_freed"
```

Key metrics and what they tell you:

| Metric | Meaning |
|--------|---------|
| `memory_rss` | Process-wide RSS from `/proc/self/statm` (engine scope) |
| `memory_usage` | Per-core jemalloc thread-local heap (pipeline scope) |
| `memory_allocated` / `memory_freed` | Cumulative jemalloc alloc/free since start |
| `requests_completed_total` | Number of gRPC Export calls fully processed |
| `request_bytes_bytes_total` | Total OTLP payload bytes received |
| `consumed_items_logs_total` | Log records entering the batch processor |
| `produced_batches_logs_total` | Batches emitted by the batch processor |

### heaptrack (attempted, not successful)

```bash
# With system allocator build (--no-default-features):
heaptrack -o /tmp/heaptrack_df ./target/release/df_engine \
  --config configs/otlp-batch-noop.yaml --num-cores 1
```

**Outcome**: heaptrack uses `LD_PRELOAD` with `libheaptrack_preload.so` to
intercept glibc `malloc`/`free`. Two problems were encountered:

1. **jemalloc build**: Rust's `#[global_allocator]` routes allocations directly
   to jemalloc, never calling glibc malloc. heaptrack sees nothing; output is
   0 bytes.

2. **System allocator build**: heaptrack captures startup allocations correctly
   (256 KB output file), but the engine crashes under any load. The per-call
   bookkeeping overhead of heaptrack combined with the high allocation rate of
   the tokio async runtime causes instability.

### jemalloc profiling (attempted, not available)

```bash
# Attempted: run with jemalloc heap profiling
MALLOC_CONF=prof:true ./target/release/df_engine --config configs/otlp-batch-noop.yaml --num-cores 1
```

**Outcome**: No error, but no profile files produced. The `tikv-jemallocator`
crate (v0.6.1) is compiled without `--enable-prof`. To enable jemalloc heap
profiling, the crate would need to be rebuilt with the `profiling` feature, or
the `jemalloc_pprof` crate could be integrated (noted as a TODO in
`Cargo.toml`).

## Appendix C: Next Steps — Batching at the Generator

The experiments above reveal a fundamental trade-off: `wait_for_result` bounds
memory tightly (37 MB peak) but limits throughput to ~333 logs/sec with 10
workers, while the default fire-and-forget mode achieves ~36,000 logs/sec but
peaks at 235 MB RSS.

There is one important dimension we have **not yet explored**: the size of
each gRPC request from the load generator. In all tests above, telemetrygen
used the default `--batch-size 100`, meaning each `Export()` call carried
exactly 100 log records (~213 KB of protobuf payload). The records within each
batch were constructed individually — there is no pre-aggregation or
compression at the generator level.

### The opportunity

With `wait_for_result` enabled, throughput is bottlenecked by the number of
pipeline round-trips per second. Each round-trip takes ~3 seconds (dominated by
`flush_timeout: 3s` in the batch processor). With 10 workers, that's ~10
round-trips completing every 3 seconds, carrying 100 records each — hence
~333 logs/sec.

But we can **increase the records per round-trip** by raising `--batch-size`.
If each request carries 1,000 records instead of 100, the same 10 round-trips
per 3 seconds would deliver ~3,330 logs/sec — a 10× improvement with the same
memory bound.

### Variables to explore

The knobs we control on the generator side:

| Knob | Current Value | Range to Test | Effect |
|------|---------------|---------------|--------|
| `--workers` | 10 | 1, 10, 50, 100 | Max concurrent in-flight requests |
| `--batch-size` | 100 (default) | 100, 500, 1000, 5000 | Records per gRPC Export call |
| `--rate` | 20000 (per worker) | 1000–100000 | Token bucket limit on generation |
| Number of processes | 1 | 1–10 | Independent gRPC connections |

And on the engine side:

| Knob | Current Value | Effect |
|------|---------------|--------|
| `wait_for_result` | true/false | Bounds in-flight to worker count |
| `flush_timeout` | 3s | With wait_for_result, directly controls round-trip time |
| `channel_capacity.pdata` | 128 or 8 | Max messages buffered between nodes |
| `otlp.min_size` | 65536 | Batch flush size threshold |

### Expected trade-off

Larger `--batch-size` increases memory per in-flight request (each request
holds more records) but delivers more records per round-trip. With
`wait_for_result`, only `--workers` requests are ever in-flight, so the peak
memory should be approximately:

```
Peak ≈ baseline + (workers × batch_size × ~2 KB per record)
```

For 10 workers with `--batch-size 1000`:
```
Peak ≈ 25 MB + (10 × 1000 × 2 KB) = 25 MB + 20 MB ≈ 45 MB
```

For 10 workers with `--batch-size 5000`:
```
Peak ≈ 25 MB + (10 × 5000 × 2 KB) = 25 MB + 100 MB ≈ 125 MB
```

This would let us trade memory for throughput in a controlled way, unlike the
fire-and-forget mode where concurrency is unbounded and memory spikes are
unpredictable.

### Suggested experiment matrix

With `wait_for_result: true` and `flush_timeout: 200ms` (reduced from 3s to
minimize round-trip latency):

| Workers | Batch Size | Expected Max In-Flight | Expected Peak RSS |
|---------|-----------|----------------------|-------------------|
| 10 | 100 | 10 × 213 KB ≈ 2 MB | ~37 MB |
| 10 | 1000 | 10 × 2.1 MB ≈ 21 MB | ~50 MB |
| 10 | 5000 | 10 × 10.7 MB ≈ 107 MB | ~135 MB |
| 50 | 1000 | 50 × 2.1 MB ≈ 105 MB | ~135 MB |

This would establish whether `wait_for_result` + large batches can reach
throughput comparable to fire-and-forget mode (>30k logs/sec) while keeping
memory bounded and predictable. The key insight is that **memory should scale
with `workers × batch_size`, not with uncontrolled channel backlog**.
