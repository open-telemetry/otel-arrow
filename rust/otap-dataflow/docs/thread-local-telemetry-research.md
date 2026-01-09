# Thread-Local Variables and Tokio Tracing: Research for Internal Telemetry

This document explains how thread-local variables work in Rust, how Tokio's
`tracing` library uses them for scoping, and how these mechanisms can be
applied to the OTAP-Dataflow internal telemetry architecture.

## Table of Contents

1. [Rust Thread-Local Variables Fundamentals](#rust-thread-local-variables-fundamentals)
2. [How Tracing Uses Thread-Locals](#how-tracing-uses-thread-locals)
3. [Reentrancy Protection in Tracing](#reentrancy-protection-in-tracing)
4. [Application to OTAP-Dataflow](#application-to-otap-dataflow)
5. [Design Patterns for EffectHandler Buffer](#design-patterns-for-effecthandler-buffer)
6. [Parallel with Existing Metrics Infrastructure](#parallel-with-existing-metrics-infrastructure)

---

## Rust Thread-Local Variables Fundamentals

### Basic Thread-Local Storage

Rust's `std::thread_local!` macro creates thread-local storage:

```rust
use std::cell::{Cell, RefCell};

thread_local! {
    // Simple value types use Cell
    static COUNTER: Cell<u64> = const { Cell::new(0) };
    
    // Complex types use RefCell for interior mutability
    static BUFFER: RefCell<Vec<LogRecord>> = RefCell::new(Vec::new());
}
```

**Key characteristics:**

1. **Initialization**: Thread-locals are lazily initialized per-thread on first access
2. **Lifetime**: Data lives as long as the thread (destroyed when thread exits)
3. **Access Pattern**: Must be accessed via closure using `.with(|value| ...)`
4. **Interior Mutability**: Use `Cell` for `Copy` types, `RefCell` for others
5. **No Cross-Thread Access**: By design, other threads cannot see this data

### Access Patterns

```rust
// Reading
COUNTER.with(|c| {
    let value = c.get();
    println!("Counter: {}", value);
});

// Writing
COUNTER.with(|c| {
    c.set(c.get() + 1);
});

// Mutable access to complex types
BUFFER.with(|b| {
    b.borrow_mut().push(record);
});
```

### The `thread_local` Crate (Used by tracing-subscriber)

The `thread_local` crate provides `ThreadLocal<T>`, which is different from
`std::thread_local!`:

```rust
use thread_local::ThreadLocal;
use std::cell::RefCell;

struct Registry {
    // Each thread gets its own RefCell<SpanStack>
    current_spans: ThreadLocal<RefCell<SpanStack>>,
}

impl Registry {
    fn enter(&self, id: &span::Id) {
        // get_or_default() returns a reference to this thread's value
        self.current_spans
            .get_or_default()  // Returns &RefCell<SpanStack>
            .borrow_mut()
            .push(id.clone());
    }
}
```

**Key difference**: `ThreadLocal<T>` is a struct field that can be shared
across threads (via `Arc` or references), but each thread accessing it
sees its own independent value.

---

## How Tracing Uses Thread-Locals

### Dispatcher Thread-Local State

The `tracing-core` dispatcher uses thread-local storage for two critical purposes:

```rust
// From tracing-core/src/dispatcher.rs
#[cfg(feature = "std")]
std::thread_local! {
    static CURRENT_STATE: State = const {
        State {
            default: RefCell<Option<Dispatch>>,
            can_enter: Cell<bool>,
        }
    };
}
```

#### 1. Per-Thread Default Subscriber (`default`)

Each thread can have its own "scoped" subscriber that overrides the global default:

```rust
// The dispatcher lookup chain:
pub fn get_default<T, F>(mut f: F) -> T
where
    F: FnMut(&Dispatch) -> T,
{
    // Fast path: if no scoped dispatchers exist, use global
    if SCOPED_COUNT.load(Ordering::Acquire) == 0 {
        return f(get_global());
    }

    // Slow path: check thread-local state
    CURRENT_STATE.try_with(|state| {
        if let Some(entered) = state.enter() {
            return f(&entered.current());
        }
        f(&NONE)
    })
    .unwrap_or_else(|_| f(&NONE))
}
```

The scoping mechanism:

```rust
pub fn with_default<T>(dispatcher: &Dispatch, f: impl FnOnce() -> T) -> T {
    // set_default stores the previous dispatcher and sets the new one
    let _guard = set_default(dispatcher);
    f()
    // When guard drops, previous dispatcher is restored
}
```

**How it works:**
- `set_default()` stores the current dispatcher in the thread-local and
  replaces it with the new one
- Returns a `DefaultGuard` that, when dropped, restores the previous dispatcher
- This creates a stack of dispatchers per thread

#### 2. Reentrancy Protection (`can_enter`)

Prevents infinite recursion when a subscriber's callback triggers more tracing:

```rust
struct State {
    default: RefCell<Option<Dispatch>>,
    can_enter: Cell<bool>,  // ← Reentrancy guard
}

impl State {
    fn enter(&self) -> Option<Entered<'_>> {
        // Atomically check and set to false
        if self.can_enter.replace(false) {
            Some(Entered(self))
        } else {
            None  // Already in a dispatch, prevent recursion
        }
    }
}

impl Drop for Entered<'_> {
    fn drop(&mut self) {
        self.0.can_enter.set(true);  // Re-enable on exit
    }
}
```

**Usage pattern:**
- Before dispatching an event, `state.enter()` is called
- If we're already dispatching (nested call), `enter()` returns `None`
- The caller then uses `Dispatch::none()` instead, preventing recursion
- When the dispatch completes, the guard's `Drop` re-enables entry

### Registry Per-Thread Span Stack

The `tracing-subscriber` Registry tracks which spans are "entered" on each thread:

```rust
// From tracing-subscriber/src/registry/sharded.rs
pub struct Registry {
    spans: Pool<DataInner>,
    // Each thread has its own stack of currently-entered spans
    current_spans: ThreadLocal<RefCell<SpanStack>>,
    next_filter_id: u8,
}

impl Subscriber for Registry {
    fn enter(&self, id: &span::Id) {
        // Push to THIS thread's span stack
        self.current_spans
            .get_or_default()
            .borrow_mut()
            .push(id.clone());
    }

    fn exit(&self, id: &span::Id) {
        // Pop from THIS thread's span stack
        if let Some(spans) = self.current_spans.get() {
            spans.borrow_mut().pop(id);
        }
    }

    fn current_span(&self) -> Current {
        // Return the top of THIS thread's span stack
        self.current_spans
            .get()
            .and_then(|spans| {
                let spans = spans.borrow();
                let id = spans.current()?;
                let span = self.get(id)?;
                Some(Current::new(id.clone(), span.metadata))
            })
            .unwrap_or_else(Current::none)
    }
}
```

---

## Reentrancy Protection in Tracing

### The Problem

When a subscriber processes an event, it might trigger more events:

```rust
impl Subscriber for MySubscriber {
    fn event(&self, event: &Event<'_>) {
        // This would cause infinite recursion!
        tracing::info!("Received event: {:?}", event);
    }
}
```

### The Solution

Tracing uses the `can_enter` flag as a guard:

```rust
// Simplified from dispatcher.rs
pub fn get_default<T, F>(f: F) -> T {
    CURRENT_STATE.try_with(|state| {
        // Try to enter dispatch mode
        if let Some(entered) = state.enter() {
            // Success: use the real dispatcher
            return f(&entered.current());
        }
        // Already dispatching: use no-op dispatcher
        f(&NONE)
    })
}
```

The test in `dispatcher.rs` demonstrates this:

```rust
#[test]
fn events_dont_infinite_loop() {
    struct TestSubscriber;
    impl Subscriber for TestSubscriber {
        fn event(&self, _: &Event<'_>) {
            static EVENTS: AtomicUsize = AtomicUsize::new(0);
            assert_eq!(
                EVENTS.fetch_add(1, Ordering::Relaxed),
                0,
                "event method called twice!"
            );
            // This nested event dispatch is blocked by can_enter
            Event::dispatch(&TEST_META, &TEST_META.fields().value_set(&[]));
        }
    }
    // ... test passes because the nested dispatch sees Dispatch::none()
}
```

---

## Application to OTAP-Dataflow

### Internal Telemetry Feedback Prevention

Your architecture document describes preventing feedback loops in internal
telemetry. Here's how to implement this using thread-local state:

```rust
use std::cell::Cell;

thread_local! {
    /// Thread-local flag indicating this thread is an internal telemetry thread.
    /// When true, all otel_* macros become no-ops to prevent feedback.
    static INTERNAL_TELEMETRY_THREAD: Cell<bool> = const { Cell::new(false) };
    
    /// Reentrancy guard for telemetry processing
    static IN_TELEMETRY_DISPATCH: Cell<bool> = const { Cell::new(false) };
}

/// Mark the current thread as an internal telemetry thread.
/// All otel_info!, otel_warn!, etc. macros will be disabled on this thread.
pub fn mark_as_internal_telemetry_thread() {
    INTERNAL_TELEMETRY_THREAD.with(|flag| flag.set(true));
}

/// Check if telemetry is enabled on this thread
pub fn is_telemetry_enabled() -> bool {
    INTERNAL_TELEMETRY_THREAD.with(|flag| !flag.get())
}

/// Guard for telemetry dispatch that prevents reentrancy
pub struct TelemetryDispatchGuard;

impl TelemetryDispatchGuard {
    pub fn try_enter() -> Option<Self> {
        IN_TELEMETRY_DISPATCH.with(|flag| {
            if flag.replace(true) {
                None  // Already dispatching
            } else {
                Some(TelemetryDispatchGuard)
            }
        })
    }
}

impl Drop for TelemetryDispatchGuard {
    fn drop(&mut self) {
        IN_TELEMETRY_DISPATCH.with(|flag| flag.set(false));
    }
}
```

### Updated Macros with Feedback Protection

```rust
/// Macro for logging informational messages with feedback protection.
#[macro_export]
macro_rules! otel_info {
    ($name:expr $(,)?) => {
        if $crate::is_telemetry_enabled() {
            $crate::_private::info!(
                name: $name,
                target: env!("CARGO_PKG_NAME"),
                name = $name,
                ""
            );
        }
    };
    // ... other variants
}
```

### Global Internal Telemetry Thread

For your global logs collection thread:

```rust
pub fn spawn_internal_telemetry_thread<F>(
    name: &str,
    task: F,
) -> std::thread::JoinHandle<()>
where
    F: FnOnce() + Send + 'static,
{
    std::thread::Builder::new()
        .name(name.into())
        .spawn(move || {
            // Mark this thread as internal telemetry
            mark_as_internal_telemetry_thread();
            
            // Configure a safe subscriber for this thread only
            let safe_subscriber = create_raw_logging_subscriber();
            tracing::subscriber::with_default(safe_subscriber, task);
        })
        .expect("Failed to spawn internal telemetry thread")
}
```

---

## Design Patterns for EffectHandler Buffer

### Option 1: Thread-Local Buffer with EffectHandler Coordination

Since your `EffectHandler` owns its thread, you can use thread-local storage:

```rust
use std::cell::RefCell;
use std::collections::VecDeque;

/// Maximum bytes to buffer per thread
const MAX_BUFFER_BYTES: usize = 65536;

/// Individual log record (pre-encoded or structured)
pub struct LogRecord {
    pub timestamp: std::time::Instant,
    pub level: tracing::Level,
    pub name: &'static str,
    pub target: &'static str,
    // Pre-encoded OTLP bytes for attributes + body
    pub encoded_data: Vec<u8>,
}

thread_local! {
    /// Per-thread log buffer for first-party telemetry
    static LOG_BUFFER: RefCell<LogBuffer> = RefCell::new(LogBuffer::new());
}

pub struct LogBuffer {
    records: VecDeque<LogRecord>,
    total_bytes: usize,
}

impl LogBuffer {
    pub fn new() -> Self {
        Self {
            records: VecDeque::new(),
            total_bytes: 0,
        }
    }
    
    /// Add a record, potentially dropping oldest if over capacity
    pub fn push(&mut self, record: LogRecord) {
        let record_size = record.encoded_data.len();
        
        // Evict old records if needed
        while self.total_bytes + record_size > MAX_BUFFER_BYTES 
              && !self.records.is_empty() 
        {
            if let Some(old) = self.records.pop_front() {
                self.total_bytes -= old.encoded_data.len();
            }
        }
        
        self.total_bytes += record_size;
        self.records.push_back(record);
    }
    
    /// Drain all records for sending
    pub fn drain(&mut self) -> Vec<LogRecord> {
        self.total_bytes = 0;
        self.records.drain(..).collect()
    }
    
    /// Check if buffer has data
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

/// Called by otel_* macros to buffer a log record
pub fn buffer_log_record(record: LogRecord) {
    LOG_BUFFER.with(|buf| {
        buf.borrow_mut().push(record);
    });
}

/// Called by EffectHandler on timer tick to flush logs
pub fn flush_log_buffer() -> Vec<LogRecord> {
    LOG_BUFFER.with(|buf| {
        buf.borrow_mut().drain()
    })
}
```

### Option 2: EffectHandler-Owned Buffer (Explicit State)

Alternatively, store the buffer directly in the `EffectHandler`:

```rust
pub struct EffectHandlerCore<PData> {
    pub(crate) node_id: NodeId,
    pub(crate) pipeline_ctrl_msg_sender: Option<PipelineCtrlMsgSender<PData>>,
    pub(crate) metrics_reporter: MetricsReporter,
    
    // NEW: Per-handler log buffer
    pub(crate) log_buffer: LogBuffer,
}

impl<PData> EffectHandlerCore<PData> {
    /// Log an info message, buffering it for later flush
    pub fn log_info(&mut self, name: &'static str, attributes: &[(&str, AttributeValue)]) {
        let record = LogRecord {
            timestamp: std::time::Instant::now(),
            level: tracing::Level::INFO,
            name,
            target: self.node_id.name.as_str(),
            encoded_data: encode_attributes_to_otlp(attributes),
        };
        self.log_buffer.push(record);
    }
    
    /// Flush buffered logs - can be called on timer or when sending to pipeline
    pub async fn flush_logs(&mut self) -> Result<(), Error> {
        let records = self.log_buffer.drain();
        if records.is_empty() {
            return Ok(());
        }
        
        // Option A: Send to global collection thread
        self.send_to_global_collector(records).await?;
        
        // Option B: Route to local ITR pipeline
        // self.route_to_local_pipeline(records).await?;
        
        Ok(())
    }
}
```

### Option 3: Hybrid Approach with Thread-Local + Handler Reference

This pattern allows macros to work anywhere while the EffectHandler controls flushing:

```rust
use std::cell::RefCell;
use std::sync::Arc;

/// Weak reference to the EffectHandler's log sink
pub struct LogSink {
    sender: flume::Sender<LogRecord>,
}

thread_local! {
    /// Thread-local pointer to this thread's log sink
    static CURRENT_LOG_SINK: RefCell<Option<Arc<LogSink>>> = RefCell::new(None);
}

impl<PData> EffectHandlerCore<PData> {
    /// Install this handler's log sink as the thread-local default
    pub fn install_log_sink(&self) {
        let sink = Arc::new(LogSink {
            sender: self.log_channel.clone(),
        });
        CURRENT_LOG_SINK.with(|s| {
            *s.borrow_mut() = Some(sink);
        });
    }
    
    /// Remove the thread-local sink (e.g., during shutdown)
    pub fn uninstall_log_sink(&self) {
        CURRENT_LOG_SINK.with(|s| {
            *s.borrow_mut() = None;
        });
    }
}

/// Called by otel_* macros
pub fn emit_log(record: LogRecord) {
    CURRENT_LOG_SINK.with(|sink| {
        if let Some(sink) = &*sink.borrow() {
            // Non-blocking send, drop if full
            let _ = sink.sender.try_send(record);
        }
        // If no sink installed, log is dropped (or use fallback)
    });
}
```

---

## Parallel with Existing Metrics Infrastructure

Your existing metrics system follows a pattern that can be mirrored for logs:

### Current Metrics Flow

```
┌──────────────────┐    report()     ┌──────────────────┐    aggregate    ┌─────────────────┐
│ MetricSet        │ ──────────────► │ MetricsReporter  │ ─────────────► │ MetricsRegistry │
│ (per-component)  │    (channel)    │ (per-handler)    │   (channel)    │ (global)        │
└──────────────────┘                 └──────────────────┘                └─────────────────┘
                                                                                 │
                                                                      dispatch_metrics()
                                                                                 ▼
                                                                    ┌─────────────────────┐
                                                                    │ MetricsDispatcher   │
                                                                    │ → OpenTelemetry SDK │
                                                                    │ → /metrics endpoint │
                                                                    └─────────────────────┘
```

### Proposed Parallel Logs Flow

```
┌──────────────────┐    buffer()     ┌──────────────────┐    flush       ┌─────────────────┐
│ LogRecord        │ ──────────────► │ LogBuffer        │ ─────────────► │ LogsRegistry    │
│ (per-event)      │  (thread-local) │ (per-handler)    │   (channel)    │ (global)        │
└──────────────────┘                 └──────────────────┘                └─────────────────┘
                                                                                 │
                                                                     dispatch_logs()
                                                                                 ▼
                                                                    ┌─────────────────────┐
                                                                    │ LogsDispatcher      │
                                                                    │ → ITR Pipeline      │
                                                                    │ → /logs endpoint    │
                                                                    │ → Raw console       │
                                                                    └─────────────────────┘
```

### Implementation Sketch for LogsRegistry

```rust
use std::sync::{Arc, RwLock};
use std::collections::VecDeque;

/// Ring buffer of recent log records for the /logs endpoint
pub struct LogsRegistry {
    /// Configurable max records to keep
    max_records: usize,
    /// Ring buffer of recent logs (OTLP-encoded bytes)
    recent_logs: RwLock<VecDeque<Vec<u8>>>,
    /// Channel to receive logs from all handlers
    receiver: flume::Receiver<Vec<u8>>,
}

impl LogsRegistry {
    /// Get recent logs for HTTP endpoint (analogous to /metrics)
    pub fn get_recent_logs(&self) -> Vec<Vec<u8>> {
        self.recent_logs.read().unwrap().iter().cloned().collect()
    }
    
    /// Collection loop (parallel to MetricsCollector::run_collection_loop)
    pub async fn run_collection_loop(&self) -> Result<(), Error> {
        while let Ok(log_bytes) = self.receiver.recv_async().await {
            let mut buffer = self.recent_logs.write().unwrap();
            
            // Ring buffer eviction
            if buffer.len() >= self.max_records {
                buffer.pop_front();
            }
            buffer.push_back(log_bytes);
            
            // Also forward to ITR pipeline if configured
            // self.forward_to_itr(&log_bytes).await?;
        }
        Ok(())
    }
}
```

### HTTP Endpoint for Logs

Similar to `/metrics`, provide a `/logs` endpoint:

```rust
/// Handler for GET /logs - returns recent internal logs
pub async fn get_internal_logs(
    registry: Arc<LogsRegistry>,
) -> impl IntoResponse {
    let logs = registry.get_recent_logs();
    
    // Could format as:
    // - JSON array of log lines
    // - OTLP LogsData protobuf
    // - Human-readable text
    
    let formatted = format_logs_as_text(&logs);
    (StatusCode::OK, formatted)
}
```

---

## Summary

### Key Thread-Local Patterns for Your Use Case

1. **Feedback Prevention Flag**: `INTERNAL_TELEMETRY_THREAD: Cell<bool>`
   - Set `true` on dedicated internal telemetry threads
   - Macros check this before emitting events

2. **Reentrancy Guard**: `IN_TELEMETRY_DISPATCH: Cell<bool>`
   - Prevents recursive telemetry events
   - Similar to tracing's `can_enter` mechanism

3. **Per-Thread Buffer**: `LOG_BUFFER: RefCell<LogBuffer>`
   - Accumulate logs without blocking
   - EffectHandler flushes on timer

4. **Thread-Local Sink Reference**: `CURRENT_LOG_SINK: RefCell<Option<Arc<LogSink>>>`
   - Allows macros to find the right destination
   - EffectHandler installs/uninstalls on thread lifecycle

### Tracing Mechanisms You Can Leverage

1. **`with_default()`**: Set thread-specific subscriber for internal threads
2. **`Dispatch::none()`**: No-op subscriber when reentrancy detected
3. **`ThreadLocal<RefCell<T>>`**: Per-thread state in shared structures
4. **Guard-based RAII**: Automatic cleanup on scope exit

### Next Steps

1. Implement the feedback prevention thread-local flag
2. Update `otel_*` macros to check the flag
3. Create `LogBuffer` structure parallel to `MetricSet`
4. Add `LogsReporter` parallel to `MetricsReporter`
5. Implement `LogsRegistry` with `/logs` endpoint
6. Wire up EffectHandler timer-based flush
