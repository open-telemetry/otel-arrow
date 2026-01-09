# Internal Logs Collection Design

This document describes the internal logging architecture for OTAP-Dataflow,
enabling first-party and third-party log events to be captured, buffered,
and routed without creating feedback loops.

## Goals

1. **Unified capture**: Both first-party (`otel_info!`) and third-party
   (`tracing::info!`) log events are captured in the same buffer
2. **Per-core buffering**: Each EffectHandler thread accumulates logs in
   its own heap-allocated buffer, avoiding cross-thread contention
3. **No feedback loops**: The global telemetry collection thread cannot
   create log events that cycle back through the system
4. **Non-blocking**: Log emission never blocks the EffectHandler thread
5. **Configurable routing**: Buffered logs can be sent to the global
   collector, routed through an Internal Telemetry Receiver (ITR)
   pipeline, or both

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ EffectHandler Thread (one per core/pipeline node)                           │
│                                                                             │
│  ┌────────────────────────┐        ┌──────────────────────────────────┐    │
│  │ EffectHandlerCore      │        │ Thread-Local State               │    │
│  │                        │        │                                  │    │
│  │  log_buffer: LogBuffer ├───────►│ CURRENT_BUFFER: *mut LogBuffer   │    │
│  │  (heap: 128KB-1MB)     │        │                                  │    │
│  └────────────────────────┘        └──────────────┬───────────────────┘    │
│           │                                       │                         │
│           │                                       │                         │
│   ┌───────┴───────┐                               │                         │
│   │               │                               │                         │
│   ▼               ▼                               ▼                         │
│ otel_info!    tracing::info!              BufferWriterLayer                 │
│ (first-party)  (third-party)              (global Subscriber)               │
│   │               │                               │                         │
│   │               └───────────────────────────────┘                         │
│   │                         │                                               │
│   │                         ▼                                               │
│   │              ┌──────────────────────┐                                   │
│   └─────────────►│ log_buffer.push()    │                                   │
│                  └──────────────────────┘                                   │
│                                                                             │
│  On timer tick: flush buffer ──────────────────────────────────────────────┼──┐
└─────────────────────────────────────────────────────────────────────────────┘  │
                                                                                  │
                                          ┌───────────────────────────────────────┘
                                          │
                                          ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ Global Telemetry Thread                                                      │
│                                                                             │
│  Subscriber: stderr-only or NoSubscriber (NO BufferWriterLayer)             │
│                                                                             │
│  ┌─────────────────────┐     ┌─────────────────────┐                       │
│  │ LogsRegistry        │     │ ITR Pipeline        │                       │
│  │ (ring buffer for    │     │ (OTLP export,       │                       │
│  │  /logs endpoint)    │     │  processing, etc.)  │                       │
│  └─────────────────────┘     └─────────────────────┘                       │
│                                                                             │
│  tracing::info!("...") → stderr (safe, no feedback)                        │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Key Components

### 1. LogBuffer

A heap-allocated ring buffer owned by each EffectHandler. Log records are
encoded to OTLP bytes before storage.

```rust
pub struct LogBuffer {
    /// Heap-allocated storage (e.g., 128KB to 1MB)
    data: Box<[u8]>,
    
    /// Ring buffer state
    write_pos: usize,
    read_pos: usize,
    
    /// Statistics
    record_count: usize,
    dropped_count: usize,
}
```

**Behavior:**
- Fixed capacity, configured at startup
- When full, oldest records are evicted (ring buffer semantics)
- Tracks dropped record count for observability
- Non-blocking push operation

### 2. Thread-Local Buffer Pointer

A thread-local variable provides the bridge between the tracing subscriber
and the EffectHandler's buffer.

```rust
thread_local! {
    static CURRENT_BUFFER: Cell<Option<NonNull<LogBuffer>>> = const { Cell::new(None) };
}
```

**Lifecycle:**
1. EffectHandler calls `install_buffer()` when its thread starts
2. Thread-local points to the handler's `log_buffer`
3. EffectHandler calls (or guard drops) `uninstall_buffer()` on shutdown
4. Thread-local is cleared, subsequent events are dropped

### 3. BufferWriterLayer

A `tracing_subscriber::Layer` installed as part of the global subscriber.
It writes events to whichever buffer is installed in the current thread.

```rust
impl<S> Layer<S> for BufferWriterLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        CURRENT_BUFFER.with(|c| {
            if let Some(mut ptr) = c.get() {
                let buffer = unsafe { ptr.as_mut() };
                let encoded = encode_event_to_otlp(event, &ctx);
                buffer.push(&encoded);
            }
            // No buffer installed: event is dropped
        });
    }
    
    fn enabled(&self, _metadata: &Metadata<'_>, _ctx: Context<'_, S>) -> bool {
        // Only process events if a buffer is installed
        CURRENT_BUFFER.with(|c| c.get().is_some())
    }
}
```

### 4. Global Telemetry Thread

A dedicated thread for collecting logs from all EffectHandler threads and
routing them to their destinations. This thread uses a **different**
subscriber that does not include `BufferWriterLayer`.

```rust
pub fn spawn_global_telemetry_thread() -> JoinHandle<()> {
    std::thread::spawn(|| {
        // Safe subscriber: stderr only, or completely silent
        let safe_subscriber = tracing_subscriber::fmt()
            .with_writer(std::io::stderr)
            .with_max_level(tracing::Level::WARN)
            .finish();
        
        // Override the default subscriber for this thread only
        tracing::subscriber::with_default(safe_subscriber, || {
            // Any tracing::info! in here goes to stderr
            // NOT back through BufferWriterLayer
            run_collection_loop();
        });
    })
}
```

## Event Flow

### First-Party Events (otel_info!, etc.)

Code with access to the EffectHandler can log directly:

```rust
impl<PData> EffectHandlerCore<PData> {
    pub fn log_info(&mut self, name: &str, attrs: &[(&str, &dyn Debug)]) {
        let encoded = encode_log_record(Level::INFO, name, attrs);
        self.log_buffer.push(&encoded);
    }
}

// Usage in a receiver/processor/exporter:
effect_handler.log_info("batch.processed", &[
    ("count", &batch.len()),
    ("duration_ms", &elapsed.as_millis()),
]);
```

### Third-Party Events (tracing::info!, etc.)

Library code or deeply nested code without EffectHandler access:

```rust
// Somewhere in a library
tracing::info!(records = count, "Parsed input");

// Flow:
// 1. tracing::info! → global subscriber → BufferWriterLayer::on_event()
// 2. BufferWriterLayer reads CURRENT_BUFFER thread-local
// 3. If set, encodes event and pushes to that buffer
// 4. If not set (wrong thread), event is dropped
```

### Buffer Flush

EffectHandlers periodically flush their buffers:

```rust
impl<PData> EffectHandlerCore<PData> {
    pub async fn flush_logs(&mut self) -> Result<(), Error> {
        let logs = self.log_buffer.drain();
        if logs.is_empty() {
            return Ok(());
        }
        
        // Send to global collector via channel
        self.log_sender.send(logs).await?;
        
        Ok(())
    }
}
```

The flush can be triggered by:
- Timer tick (e.g., every 1 second)
- Buffer reaching high-water mark
- Explicit flush request from pipeline

## Feedback Loop Prevention

The architecture prevents feedback loops through subscriber isolation:

| Thread Type | Subscriber | BufferWriterLayer? | Effect of `tracing::info!` |
|-------------|------------|-------------------|---------------------------|
| EffectHandler | Global (with BufferWriterLayer) | Yes, buffer installed | Written to handler's buffer |
| Global Telemetry | Thread-local override (stderr/noop) | No | Stderr or dropped |
| Other | Global (with BufferWriterLayer) | No buffer installed | Dropped |

**Why this prevents cycles:**

1. EffectHandler thread emits `otel_info!("something")`
2. Event is buffered locally (no channel send yet)
3. On timer, buffer is flushed to global telemetry thread via channel
4. Global thread receives the event
5. If global thread calls `tracing::info!()` while processing:
   - Its subscriber is the stderr/noop override
   - BufferWriterLayer is NOT in its subscriber stack
   - Event goes to stderr (or nowhere), NOT back to a buffer
   - No channel send, no cycle

## Encoding Format

Log records are encoded to OTLP bytes (`opentelemetry.proto.logs.v1.LogRecord`)
before storage in the buffer. This enables:

- Zero-copy access via `LogsDataView` for formatting
- Direct forwarding to OTLP exporters
- Consistent format for `/logs` HTTP endpoint
- Efficient storage (no per-field overhead)

## Flush Strategy: Timer-Based with Drop on Full

Unlike metrics (which are pre-aggregated), individual log events can be
lost if the buffer fills. The current approach is simple:

- **Timer-based flush**: The pipeline runtime flushes on its telemetry timer
- **Drop new events when full**: If buffer fills before flush, new events are dropped
- **Track dropped count**: `LogBuffer::dropped_count()` for observability

This keeps the implementation simple. Future enhancements could include:
- Sampling at high volume
- Priority levels (always keep ERROR events)
- Dynamic buffer sizing

## Configuration

*(To be defined)*

```yaml
service:
  telemetry:
    logs:
      level: info
      internal_collection:
        enabled: true
        buffer_size_bytes: 131072    # 128KB per handler
        flush_interval: "1s"
        # Routing options:
        # - global_collector: send to global thread
        # - local_pipeline: route through ITR on same core
        # - both: send to both destinations
        routing: global_collector
```

## Integration with Existing Metrics System

This design parallels the existing metrics infrastructure. Understanding
the metrics flow is essential for implementing consistent logging.

### Metrics System Architecture

The metrics system follows a clear data flow pattern:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Controller (lib.rs)                                                         │
│                                                                             │
│  MetricsSystem::new(config)                                                 │
│    ├── MetricsRegistryHandle::new()     ← Shared registry for aggregation │
│    ├── MetricsCollector::new()          ← Runs on metrics-aggregator thread│
│    └── MetricsReporter::new(sender)     ← Cloned to each pipeline thread   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                    │
                    │ metrics_reporter.clone()
                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ Pipeline Thread (one per core)                                              │
│                                                                             │
│  PipelineContext::new(controller_context, pipeline_id, core_id, thread_id) │
│    └── with_node_context(node_id, node_urn, node_kind)                     │
│          └── register_metrics<T>()                                          │
│                └── registry.register::<T>(self.node_attribute_set())       │
│                                                                             │
│  Each component (receiver/processor/exporter):                              │
│    1. Receives PipelineContext via build() method                          │
│    2. Calls pipeline_ctx.register_metrics::<ComponentMetrics>()            │
│    3. Gets MetricSet<ComponentMetrics> with pre-registered attributes      │
│    4. On timer tick: metrics_reporter.report(&mut metric_set)              │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                    │
                    │ flume channel (MetricSetSnapshot)
                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ Metrics Aggregator Thread ("metrics-aggregator")                            │
│                                                                             │
│  MetricsCollector::run_collection_loop()                                    │
│    loop {                                                                   │
│        snapshot = receiver.recv_async().await                               │
│        registry.accumulate_snapshot(snapshot.key, &snapshot.metrics)       │
│    }                                                                        │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                    │
                    │ MetricsRegistryHandle (Arc<Mutex<MetricsRegistry>>)
                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ Admin HTTP Server ("http-admin" thread)                                     │
│                                                                             │
│  GET /metrics or /telemetry/metrics                                        │
│    registry.visit_metrics_and_reset(|desc, attrs, iter| {                  │
│        // Format as JSON, Prometheus, Line Protocol, etc.                  │
│        // desc: MetricsDescriptor (name, field definitions)                │
│        // attrs: NodeAttributeSet (resource + node attributes)             │
│        // iter: MetricsIterator (field, value) pairs                       │
│    })                                                                       │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Key Components

1. **MetricsRegistryHandle**: Thread-safe handle wrapping `Arc<Mutex<MetricsRegistry>>`.
   Passed to admin for HTTP endpoints, passed to controller for aggregation.

2. **MetricsReporter**: Cloneable sender side of a flume channel. Each pipeline
   thread gets a clone to send `MetricSetSnapshot` messages.

3. **MetricsCollector**: Runs on a dedicated thread, receives snapshots via
   channel, and calls `registry.accumulate_snapshot()` to merge them.

4. **NodeAttributeSet**: Consistent attributes attached to every metric set
   registered by a component. Includes:
   - Resource: `process_instance_id`, `host_id`, `container_id`
   - Engine: `core_id`, `numa_node_id`
   - Pipeline: `pipeline_id`
   - Node: `node_id`, `node_urn`, `node_type`

### Unified Registration: Shared MetricsKey for Logs and Metrics

The key insight is that `MetricsKey` already identifies a component's
`NodeAttributeSet` in the registry. Logs should reuse this same key
rather than duplicating attribute storage.

**Existing MetricsEntry (in registry.rs):**

```rust
pub struct MetricsEntry {
    pub metrics_descriptor: &'static MetricsDescriptor,
    pub attributes_descriptor: &'static AttributesDescriptor,
    pub metric_values: Vec<MetricValue>,
    pub attribute_values: Box<dyn AttributeSetHandler + Send + Sync>,  // ← NodeAttributeSet
}
```

When `pipeline_ctx.register_metrics::<T>()` is called:
1. Returns `MetricSet<T>` containing a `MetricsKey` (slotmap index)
2. The `NodeAttributeSet` is stored in the registry under that key
3. **Both metrics and logs use the same `MetricsKey`**

### Parallel Logs Architecture

| Metrics | Logs |
|---------|------|
| `MetricSet<T>` | `LogBuffer` |
| `MetricsReporter` (channel sender) | `LogsReporter` (channel sender) |
| `MetricsRegistry` (aggregates metrics) | `LogsRing` (ring buffer for recent logs) |
| `MetricsCollector` (receives snapshots) | `LogsCollector` (receives batches) |
| `MetricSetSnapshot { key, metrics }` | `LogBatch { producer_key, records }` |
| `/metrics` endpoint | `/logs` endpoint |

**Shared:**
- `MetricsKey` identifies the producer (same key for metrics and logs)
- `NodeAttributeSet` stored once in `MetricsRegistry`, looked up by key

### Channel Data Types

```rust
/// A batch of logs from one producer - compact, just carries the key
pub struct LogBatch {
    /// Same key returned from register_metrics() - identifies NodeAttributeSet
    pub producer_key: MetricsKey,
    
    /// The log records
    pub records: Vec<LogRecord>,
}

/// A single log record
pub struct LogRecord {
    pub callsite_id: Identifier,      // Pointer to static Metadata
    pub timestamp_ns: u64,
    pub body_attrs_bytes: Bytes,      // Pre-encoded body + event attributes
}

/// Reporter for sending log batches (parallel to MetricsReporter)
#[derive(Clone)]
pub struct LogsReporter {
    sender: flume::Sender<LogBatch>,
}

impl LogsReporter {
    pub fn try_report(&self, batch: LogBatch) -> Result<(), Error> {
        match self.sender.try_send(batch) {
            Ok(_) => Ok(()),
            Err(flume::TrySendError::Full(_)) => Ok(()), // Drop if full
            Err(flume::TrySendError::Disconnected(_)) => Err(Error::LogsChannelClosed),
        }
    }
}
```

### EffectHandler with Shared Key

```rust
pub struct EffectHandlerCore<PData> {
    pub node_id: NodeId,
    pub producer_key: MetricsKey,         // Shared identifier for metrics & logs
    pub metrics_reporter: MetricsReporter,
    pub logs_reporter: LogsReporter,      // NEW
    pub log_buffer: LogBuffer,            // NEW
    // ...
}

impl<PData> EffectHandlerCore<PData> {
    pub async fn flush_logs(&mut self) -> Result<(), Error> {
        if self.log_buffer.is_empty() {
            return Ok(());
        }
        
        let batch = LogBatch {
            producer_key: self.producer_key,  // Just the 8-byte key
            records: self.log_buffer.drain(),
        };
        self.logs_reporter.try_report(batch)
    }
}
```

### Consumer Side: LogsRing with Key Lookup

```rust
/// Ring buffer storing recent logs for /logs endpoint
pub struct LogsRing {
    entries: VecDeque<StoredLogEntry>,
    capacity: usize,
    total_received: u64,
    total_dropped: u64,
}

/// Stored entry - just the key, not the full attributes
pub struct StoredLogEntry {
    pub producer_key: MetricsKey,     // Lookup attrs from MetricsRegistry
    pub callsite_id: Identifier,
    pub timestamp_ns: u64,
    pub body_attrs_bytes: Bytes,
}

impl LogsRing {
    pub fn append(&mut self, batch: LogBatch) {
        for record in batch.records {
            if self.entries.len() >= self.capacity {
                self.entries.pop_front();
                self.total_dropped += 1;
            }
            self.entries.push_back(StoredLogEntry {
                producer_key: batch.producer_key,
                callsite_id: record.callsite_id,
                timestamp_ns: record.timestamp_ns,
                body_attrs_bytes: record.body_attrs_bytes,
            });
            self.total_received += 1;
        }
    }
}
```

### Admin /logs Endpoint

```rust
pub async fn get_logs(State(state): State<AppState>) -> impl IntoResponse {
    let logs_ring = state.logs_ring.lock();
    let registry = state.metrics_registry.lock();
    
    let writer = ConsoleWriter::no_color();
    let mut output = String::new();
    
    for entry in logs_ring.recent(100) {
        // Dereference Identifier to get static Metadata
        let metadata = entry.callsite_id.callsite().metadata();
        let saved = SavedCallsite::new(metadata);
        
        let record = LogRecord {
            callsite_id: entry.callsite_id,
            timestamp_ns: entry.timestamp_ns,
            body_attrs_bytes: entry.body_attrs_bytes.clone(),
        };
        
        // Format the log record
        output.push_str(&writer.format_log_record(&record, &saved));
        
        // Look up NodeAttributeSet using the shared key
        if let Some(metrics_entry) = registry.metrics.get(entry.producer_key) {
            let attrs = metrics_entry.attribute_values.as_ref();
            output.push_str(&format_node_attrs(attrs));
        }
        output.push('\n');
    }
    
    (StatusCode::OK, output)
}
```

### Benefits of Shared Key

| Aspect | Sending attrs per batch | Shared MetricsKey |
|--------|------------------------|-------------------|
| Registration | Separate for metrics/logs | Single registration |
| Per-batch overhead | Full NodeAttributeSet clone | 8-byte key |
| Attribute storage | Duplicated per batch | Single source of truth |
| Consistency | Could diverge | Guaranteed identical |
| Admin lookup | Already has attrs | Lookup from registry |

### Identifier → Metadata: Direct Field Access

The `Identifier` type wraps a pointer to static memory:

```rust
pub struct Identifier(
    #[doc(hidden)]
    pub &'static dyn Callsite,
);
```

The inner field is `pub` (for macro construction purposes), so any thread
can access it directly to get `Metadata`:

```rust
// Identifier.0 is &'static dyn Callsite
let metadata: &'static Metadata<'static> = identifier.0.metadata();
```

No need to forward `(Identifier, Metadata)` pairs between threads.
The admin thread can directly access `Identifier.0.metadata()` on any
`Identifier` received in a `LogBatch` to get the full static metadata
(level, target, file, line, name, etc.).

### Thread-Local Producer Key for Third-Party Instrumentation

Third-party libraries often use `tracing::info!()` without access to any
EffectHandler or `MetricsKey`. To attribute these logs to the correct
component, we use a thread-local "current producer key" that is set
when entering a component's execution scope.

```rust
// Thread-local current MetricsKey for third-party instrumentation.
thread_local! {
    static CURRENT_PRODUCER_KEY: RefCell<Option<MetricsKey>> = const { RefCell::new(None) };
}

/// Guard that sets the current producer key for the duration of a scope.
/// When dropped, restores the previous key (supports nesting).
pub struct ProducerKeyGuard {
    previous: Option<MetricsKey>,
}

impl ProducerKeyGuard {
    /// Enter a scope with the given producer key.
    pub fn enter(key: MetricsKey) -> Self {
        let previous = CURRENT_PRODUCER_KEY.with(|cell| cell.borrow_mut().replace(key));
        Self { previous }
    }
}

impl Drop for ProducerKeyGuard {
    fn drop(&mut self) {
        CURRENT_PRODUCER_KEY.with(|cell| {
            *cell.borrow_mut() = self.previous;
        });
    }
}

/// Get the current producer key (if any component scope is active).
pub fn current_producer_key() -> Option<MetricsKey> {
    CURRENT_PRODUCER_KEY.with(|cell| *cell.borrow())
}
```

**Usage in the engine (when calling component methods):**

```rust
impl<PData> EffectHandlerCore<PData> {
    /// Enter a scope where third-party logs are attributed to this component.
    pub fn enter_producer_scope(&self) -> ProducerKeyGuard {
        ProducerKeyGuard::enter(self.producer_key)
    }
}

// In the pipeline runtime, when calling a processor:
let _guard = effect_handler.enter_producer_scope();
processor.process(batch, effect_handler).await?;
// Guard drops here, restoring previous key
```

**How it works with the BufferWriterLayer:**

```rust
impl<S> Layer<S> for BufferWriterLayer {
    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        let record = encode_event(event, &ctx);
        // Pass None - push_to_thread_buffer will use current_producer_key()
        push_to_thread_buffer(record, None);
    }
}

// In push_to_thread_buffer:
pub fn push_to_thread_buffer(record: LogRecord, producer_key: Option<MetricsKey>) -> bool {
    CURRENT_LOG_BUFFER.with(|cell| {
        if let Some(ref mut buffer) = *cell.borrow_mut() {
            // Use explicit key if provided, otherwise use thread-current key
            let key = producer_key.or_else(current_producer_key);
            buffer.push(LogEntry { record, producer_key: key });
            true
        } else {
            false
        }
    })
}
```

**Benefits:**

| Aspect | Without ProducerKeyGuard | With ProducerKeyGuard |
|--------|-------------------------|----------------------|
| First-party logs | Attributed correctly | Attributed correctly |
| Third-party libs | `producer_key: None` | Attributed to current component |
| No EffectHandler access | Lost attribution | Correct attribution |
| Nesting support | N/A | Previous key restored on drop |

**Example flow:**

```
┌─────────────────────────────────────────────────────────────────┐
│ Pipeline Thread                                                 │
│                                                                 │
│  1. Enter processor scope: ProducerKeyGuard::enter(processor_key)
│     CURRENT_PRODUCER_KEY = Some(processor_key)                  │
│                                                                 │
│  2. Processor calls library code                                │
│     └── Library calls tracing::info!("parsing data")           │
│         └── BufferWriterLayer::on_event()                       │
│             └── push_to_thread_buffer(record, None)             │
│                 └── key = current_producer_key() = processor_key│
│                 └── buffer.push(LogEntry { key: processor_key })│
│                                                                 │
│  3. Guard drops: CURRENT_PRODUCER_KEY = None                    │
│                                                                 │
│  4. On flush: LogBatch includes entry with producer_key set    │
│                                                                 │
│  5. Admin can look up NodeAttributeSet for processor_key       │
│     → Log shows: node_id=processor, node_urn=arrow/processor   │
└─────────────────────────────────────────────────────────────────┘
```

## Channel Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              PRODUCER SIDE                                  │
│                                                                             │
│  Pipeline Thread 0          Pipeline Thread 1          Pipeline Thread N   │
│  ┌─────────────────┐        ┌─────────────────┐        ┌─────────────────┐ │
│  │ EffectHandler   │        │ EffectHandler   │        │ EffectHandler   │ │
│  │  producer_key   │        │  producer_key   │        │  producer_key   │ │
│  │  log_buffer     │        │  log_buffer     │        │  log_buffer     │ │
│  │  logs_reporter  │        │  logs_reporter  │        │  logs_reporter  │ │
│  └────────┬────────┘        └────────┬────────┘        └────────┬────────┘ │
│           │                          │                          │          │
│           │ on timer: flush          │                          │          │
│           ▼                          ▼                          ▼          │
│  ┌────────────────────────────────────────────────────────────────────────┐│
│  │                     Metrics Channel (existing)                         ││
│  │              flume::Sender<MetricSetSnapshot>                          ││
│  └────────────────────────────────────────────────────────────────────────┘│
│  ┌────────────────────────────────────────────────────────────────────────┐│
│  │                     Logs Channel (NEW, parallel)                       ││
│  │              flume::Sender<LogBatch>                                   ││
│  └────────────────────────────────────────────────────────────────────────┘│
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    │ Two separate channels
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                              CONSUMER SIDE                                  │
│                                                                             │
│  ┌─────────────────────────────────┐  ┌─────────────────────────────────┐  │
│  │      MetricsCollector           │  │      LogsCollector (NEW)        │  │
│  │  (metrics-aggregator thread)    │  │   (logs-collector thread OR     │  │
│  │                                 │  │    same thread as admin)        │  │
│  │  loop {                         │  │                                 │  │
│  │    snapshot = rx.recv()         │  │  loop {                         │  │
│  │    registry.accumulate(...)     │  │    batch = rx.recv()            │  │
│  │  }                              │  │    logs_ring.append(batch)      │  │
│  └─────────────────────────────────┘  │  }                              │  │
│                                       └─────────────────────────────────┘  │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────────┐│
│  │                         Admin HTTP Server                               ││
│  │                                                                         ││
│  │  GET /metrics → registry.visit_metrics_and_reset(...)                  ││
│  │                                                                         ││
│  │  GET /logs    → logs_ring.recent(limit) + registry.get(key).attrs      ││
│  │                                                                         ││
│  └─────────────────────────────────────────────────────────────────────────┘│
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Why Separate Channels?

| Aspect | Metrics Channel | Logs Channel |
|--------|-----------------|--------------|
| Volume | Lower (aggregated values) | Higher (individual events) |
| Semantics | Merge into registry | Append to ring buffer |
| Backpressure | Accumulate locally | Drop oldest |
| Consumer | Aggregate by key | Keep recent N entries |

## Future Considerations

1. **Span context propagation**: Include trace/span IDs in buffered logs
   when a span is active on the thread

2. **Sampling**: Support head-based sampling to reduce volume

3. **Priority levels**: Allow high-severity logs to bypass buffer limits

4. **Direct ITR routing**: Option to route logs directly to a same-core
   ITR pipeline without going through the global thread

5. **Backpressure signaling**: Mechanism for global collector to signal
   EffectHandlers when it's overloaded

## Code References

### Metrics System (for reference implementation)

| File | Purpose |
|------|---------|
| `crates/controller/src/lib.rs` | Creates `MetricsSystem`, spawns threads, passes `MetricsReporter` to pipeline threads |
| `crates/telemetry/src/lib.rs` | `MetricsSystem` struct holding registry, collector, reporter, dispatcher |
| `crates/telemetry/src/registry.rs` | `MetricsRegistry` and `MetricsRegistryHandle` for aggregation |
| `crates/telemetry/src/reporter.rs` | `MetricsReporter` for sending snapshots through flume channel |
| `crates/telemetry/src/collector.rs` | `MetricsCollector::run_collection_loop()` receives and aggregates snapshots |
| `crates/engine/src/context.rs` | `PipelineContext` and `NodeAttributeSet` for consistent attributes |
| `crates/engine/src/effect_handler.rs` | `EffectHandlerCore` with `report_metrics()` method |
| `crates/admin/src/telemetry.rs` | `/metrics` endpoint using `registry.visit_metrics_and_reset()` |

### Existing Self-Tracing Primitives

| File | Purpose |
|------|---------|
| `crates/telemetry/src/self_tracing.rs` | `LogRecord` and `SavedCallsite` types |
| `crates/telemetry/src/self_tracing/encoder.rs` | `DirectLogRecordEncoder`, `DirectFieldVisitor` for OTLP encoding |
| `crates/telemetry/src/self_tracing/formatter.rs` | `RawLoggingLayer`, `ConsoleWriter` for console output |
| `crates/telemetry/src/internal_events.rs` | `otel_info!`, `otel_warn!`, etc. macros wrapping tracing |

### Tokio Tracing (vendored)

| File | Purpose |
|------|---------|
| `tokio-tracing-rs/tracing-core/src/dispatcher.rs` | Thread-local `CURRENT_STATE`, `with_default()` for subscriber scoping |
| `tokio-tracing-rs/tracing-subscriber/src/registry/sharded.rs` | Example of `ThreadLocal<RefCell<T>>` for per-thread span stacks |

