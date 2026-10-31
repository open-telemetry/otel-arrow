# Async Best Practices for OTAP Dataflow

Each pipeline instance runs on one runtime thread. Blocking or monopolizing
that thread delays I/O, other nodes, Ack/Nack handling, telemetry, and lifecycle
control. These guidelines cover both blocking operations and CPU-heavy work.

## Runtime responsiveness

An `.await` is a possible suspension point, not proof that another task runs.
Buffered reads and sends to channels with available capacity can complete
immediately. Tokio cannot preempt arbitrary synchronous parsing, construction,
sorting, encoding, or compression.

- Bound uninterrupted CPU work by records, bytes, and elapsed time, where
  practical. Check progress within expensive loops, not only between batches.
- Yield deliberately after bounded progress. Excessive per-record yielding adds
  overhead; `yield_now()` does not guarantee which task or I/O event runs next.
- Treat time budgets as cooperative targets, not latency guarantees. A single
  record, library call, or finalization step can exceed the budget.
- Keep mutable codec borrows and synchronous locks scoped before yielding.
  Cancellation must release partial work and preserve delivery ownership.
- Preserve backpressure and bound retained work. Larger batches can increase
  queued bytes even when channel slot counts do not change.
- Measure finalization, sorting, encoding, and compression separately: making
  the decode loop cooperative does not make those stages cooperative.

### Measuring responsiveness

- Run a full pipeline with a concurrent independent I/O/control probe on the
  same runtime. Define what completion means, such as downstream Ack rather
  than admission, and distinguish probe latency from telemetry delivery latency.
- Compare small and large batches at fixed load and under backpressure. Record
  CPU per delivered item, RSS/queued bytes, tail latency, errors, and achieved
  throughput together. Report actual probe rate; closed-loop probes can hide
  demand during stalls and do not establish an open-loop latency SLO.
- Pin the SUT to an idle core, monitor its SMT sibling and host interference,
  and control or verify active frequency. Interleave revisions, repeat pairs,
  and report confidence intervals and exclusions.
- Separate CPU-only and probe runs to expose probe overhead. To attribute a
  gain to cooperation, compare the same optimized code with yielding enabled
  and disabled; a cumulative before/after comparison is insufficient.
- Check cancellation, shutdown, cleanup, and exact delivery independently.
  Keep throughput and scheduling-latency thresholds in benchmarks, not ordinary
  correctness tests; see the [testing guide](testing-guide.md).

### Interpreting flow timing

`flow.compute.duration` sums wall-clock intervals measured within processors;
it is not CPU time. Its send markers keep running across awaits, including
cooperative codec yields, so those intervals can include time spent running
other tasks. Only processors are directly instrumented. Do not use this metric
alone to attribute CPU cost or compare cooperative scheduling.
Suspension-aware timing is engine-wide follow-up work: excluding only codec
yields would leave other awaits with inconsistent accounting.

## Common Blocking Operations to Avoid

### 1. Synchronous I/O Operations

**Don't use:**

```rust
use std::io::Write;
use std::fs::File;

async fn bad_example() {
    let mut file = File::create("example.txt").unwrap(); // BLOCKING!
    file.write_all(b"data").unwrap(); // BLOCKING!
}
```

**Use instead:**

```rust
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

async fn good_example() {
    let mut file = File::create("example.txt").await.unwrap();
    file.write_all(b"data").await.unwrap();
}
```

### 2. File System Operations

**Don't use:**

```rust
use std::fs;

async fn bad_fs_operations() {
    let contents = std::fs::read_to_string("file.txt").unwrap(); // BLOCKING!
    std::fs::write("output.txt", "data").unwrap(); // BLOCKING!
}
```

**Use instead:**

```rust
use tokio::fs;

async fn good_fs_operations() {
    let contents = fs::read_to_string("file.txt").await.unwrap();
    fs::write("output.txt", "data").await.unwrap();
}
```

### 3. Thread Sleep

**Don't use:**

```rust
async fn bad_sleep() {
    std::thread::sleep(Duration::from_secs(1)); // BLOCKING!
}
```

**Use instead:**

```rust
use tokio::time::{sleep, Duration};

async fn good_sleep() {
    sleep(Duration::from_secs(1)).await;
}
```

### 4. Synchronous Network Operations

**Don't use:**

```rust
use std::net::TcpStream;

async fn bad_network() {
    let stream = TcpStream::connect("127.0.0.1:8080").unwrap(); // BLOCKING!
}
```

**Use instead:**

```rust
use tokio::net::TcpStream;

async fn good_network() {
    let stream = TcpStream::connect("127.0.0.1:8080").await.unwrap();
}
```

## Script To Detect Issues

Run our custom script to detect blocking operations:

```bash
./scripts/check-async-blocking.sh
```

This script checks for:

- `std::io::Write` usage in async contexts
- Blocking `std::fs` operations
- Blocking `File::open/create` calls
- `thread::sleep` usage

## When You Must Use Blocking Operations

Some libraries require blocking execution. Offloading with `spawn_blocking`
needs an explicit design: bound submitted work and concurrency, integrate
backpressure, expose resource use, and define cancellation and shutdown behavior.
Cancelling the caller does not stop an already running blocking closure.
Account for the loss of thread/core locality and avoid hidden queues.

Do not use offloading as the default fix for CPU-heavy loops that can cooperate
on the pipeline thread. See the
[architectural review requirements](ai/ai-assisted-pr-review.md#core-architectural-invariants).

## Code Review Checklist

When reviewing async code, check for:

- [ ] Runtime I/O uses async interfaces or justified, bounded offload
- [ ] No `std::thread::sleep` - use `tokio::time::sleep` instead
- [ ] Necessary blocking offload is bounded, observable, and integrated with
  backpressure, cancellation, and shutdown
- [ ] CPU-heavy loops offer scheduling opportunities after bounded progress,
  including when surrounding awaits are immediately ready
- [ ] Finalization and other indivisible work are assessed separately
- [ ] Benchmarks assess CPU, memory, and responsiveness together
- [ ] No synchronous locks or mutable codec borrows held across `.await`

## Quick Reference

| Blocking              | Async Alternative       |
|-----------------------|-------------------------|
| `std::fs::read`       | `tokio::fs::read`       |
| `std::fs::write`      | `tokio::fs::write`      |
| `std::fs::File`       | `tokio::fs::File`       |
| `std::io::Write`      | `tokio::io::AsyncWrite` |
| `std::io::Read`       | `tokio::io::AsyncRead`  |
| `std::net::TcpStream` | `tokio::net::TcpStream` |
| `std::thread::sleep`  | `tokio::time::sleep`    |
| `std::sync::mpsc`     | `tokio::sync::mpsc`     |
