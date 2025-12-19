# Internal Telemetry Receiver Design

## Overview

This document describes the design for component-level telemetry logging using the Internal Telemetry Receiver (ITR). The system provides **orthogonal configuration options** for telemetry routing, allowing independent control of delivery location, source handling, and fallback behavior.

## Orthogonal Configuration Dimensions

The telemetry system has several independent configuration axes that can be composed:

### 1. Delivery Location (Per Telemetry Source)

**On-core ITR** (local):
- ITR receiver runs on same engine core as emitter
- Direct channel delivery, minimal latency
- Configurable per-core (some cores may have ITR, others don't)

**Global collection thread** (centralized):
- Dedicated thread receives telemetry from multiple sources
- Can be: internal telemetry overflow thread, global tokio subscriber thread, or both
- Higher latency but simpler architecture

**Direct console** (synchronous):
- Immediate formatting and output to stderr
- No buffering, no channels, always available
- Used as last-resort fallback

### 2. Telemetry Source

**Effect handler** (component logs):
- `otel_*!(effect, ...)` macros in pipeline components
- Bypasses global subscriber
- Streaming to reusable buffer (batching)

**Global tokio subscriber** (3rd party logs):
- `tracing::*!(...)` macros from any thread
- Goes through global tracing subscriber
- One OTLP bytes object per event (no batching)

### 3. Local Delivery Cascade (All Optional)

When telemetry is emitted on an engine core thread:

```
1. CORE_LOCAL_ITR_SENDER (optional) - Try on-core ITR first
   ↓ If Some and not full: Deliver to on-core ITR
   ↓ If None or full: Try next
   
2. INTERNAL_TELEMETRY_SENDER or GLOBAL_TOKIO_SENDER (optional)
   ↓ Effect handlers → INTERNAL_TELEMETRY_SENDER (small channel, per-core)
   ↓ Tokio subscriber → GLOBAL_TOKIO_SENDER (larger channel, shared)
   ↓ If not full: Deliver to global collection thread
   ↓ If None or full: Try next
   
3. Raw logger (always available) - Direct console output
```

### 4. Global Delivery Options

When telemetry reaches a global collection thread:

**OpenTelemetry SDK** (standard):
- Decode OTLP bytes → SDK LogRecord → SDK processors/exporters
- Uses SDK configuration (console, OTLP, file, etc.)
- Anti-recursion: `IN_TELEMETRY_COLLECTION = true` during processing

**Internal OTAP pipeline** (advanced):
- Wrap OTLP bytes as OtapPdata → inject into dedicated pipeline
- Full dataflow processing (batch, retry, transform, export)
- Anti-recursion: `IN_TELEMETRY_COLLECTION = true` in pipeline thread

**Direct console** (minimal):
- Decode OTLP bytes → format → stderr
- No SDK, no pipeline, just logging
- Anti-recursion: Uses raw logger (no decoding)

### 5. ITR-Downstream Component Protection

Pipeline components processing internal telemetry must avoid loops:

**Routing options** (never includes CORE_LOCAL_ITR_SENDER):
```
1. INTERNAL_TELEMETRY_SENDER or GLOBAL_TOKIO_SENDER (if available)
   ↓ Routes to global collection thread
   
2. Raw logger (always)
   ↓ Direct console output
```

**Anti-recursion enforcement**:
- ITR and downstream components: Never set CORE_LOCAL_ITR_SENDER
- Global collection threads: Check `IN_TELEMETRY_COLLECTION` flag before routing
- Result: No feedback loops possible

## Configuration Matrix

### Effect Handler Configurations (Component Logs)

| ITR Enabled | Global Thread | Buffer Overflow Routes To | Notes |
|------------|---------------|---------------------------|-------|
| Yes (on-core) | Internal telemetry | CORE_LOCAL_ITR_SENDER → INTERNAL_TELEMETRY_SENDER → Raw | Preferred: core isolation + overflow safety |
| No | Internal telemetry | INTERNAL_TELEMETRY_SENDER → Raw | Simple: per-core channels to global |
| No | None (direct) | Raw logger (synchronous) | Minimal: console output only, no threading |

### Global Tokio Subscriber Configurations (3rd Party Logs)

| ITR on Core | Global Thread | Routes To | Notes |
|------------|---------------|-----------|-------|
| Yes | Tokio collector | CORE_LOCAL_ITR_SENDER → GLOBAL_TOKIO_SENDER → Raw | Engine core threads prefer on-core |
| No | Tokio collector | GLOBAL_TOKIO_SENDER → Raw | Non-engine threads go to global |
| N/A | None (direct) | Raw logger (synchronous) | Console output only |

### Global Collection Thread Destinations

| Configuration | Destination | Anti-Recursion | Notes |
|--------------|-------------|----------------|-------|
| SDK enabled | OpenTelemetry SDK | IN_TELEMETRY_COLLECTION = true | Standard exporters |
| ITR pipeline | Dedicated OTAP pipeline | IN_TELEMETRY_COLLECTION = true | Full dataflow processing |
| Console only | Raw logger | No decoding needed | Minimal footprint |

## Thread-Local Routing Architecture

### Core-Local Delivery

**Key Innovation**: Thread-local lookup routes telemetry to ITR on the same core, preventing one core's excessive logging from affecting others.

**Routing Paths**:
1. **On-core ITR** (preferred): Deliver to ITR receiver running on same core
2. **Internal telemetry overflow** (fallback): When on-core channel full, route to dedicated internal telemetry thread
3. **Raw logger** (last resort): When both channels full, synchronous console output

**Channel Isolation**: Each engine core has a dedicated overflow channel to the internal telemetry collection thread. This enables per-core throttling - a noisy logger on one core doesn't affect other cores' telemetry or metrics reporting.

**Thread-Local Variables**:
- `CORE_LOCAL_ITR_SENDER`: Per-core channel to local ITR (if configured)
- `INTERNAL_TELEMETRY_SENDER`: Dedicated overflow channel for internal telemetry (logs only)
- `GLOBAL_TOKIO_SENDER`: Channel for global tokio subscriber events (separate, larger capacity)
- `IN_TELEMETRY_COLLECTION`: Flag to prevent recursion in telemetry processing threads

**Separate Channels Architecture**: Internal telemetry overflow and global tokio subscriber use **separate dedicated channels**, providing isolation and independent throttling. Metrics reports continue using existing metrics collection channel.

## Bounded Memory Architecture

**Critical design principle:** All telemetry operations must have **bounded memory usage** and **never block component operations**.

### Memory Bounds

1. **Per-thread buffer**: Pre-allocated StatefulOtlpEncoder (default 64 KiB)
   - Fixed capacity, allocated at effect handler creation
   - Shared by all log events from that component/thread
   - Individual log records limited to 16 KiB (enables 2-byte length placeholders)
   - Returns BufferFull error when buffer full

2. **Bounded channel**: OtlpBytesChannel or ITR channel (default 1000 records)
   - Configurable capacity prevents unbounded growth
   - Non-blocking try_send used for backpressure handling
   - Worst case: 1000 × 64 KiB = 64 MB

3. **Total per component**: ~64 KiB buffer + share of channel capacity
   - Predictable, bounded memory footprint
   - No risk of OOM from telemetry
   - Graceful degradation under load

### Graceful Degradation

When limits are reached, the system degrades gracefully:

1. **Buffer full** → Flush buffer → Retry encoding in empty buffer
2. **Channel full** → Fall back to raw console logger (synchronous, never fails)
3. **Flush fails** → Fall back to raw console logger
4. **Never blocks** → Component operation continues regardless of telemetry state

### Raw Logger Fallback

- Initialized early via `LoggerProvider::init_default_console_tracing()`
- Synchronous console output (writes to stderr)
- Never fails, never allocates
- Used when telemetry path is unavailable
- No recursion risk (separate code path)

## Architecture Paths

### Thread-Local Routing for All Telemetry

**Core Concept**: All telemetry (3rd party and component logs) uses thread-local lookup to route to on-core ITR first, then global collection point as overflow.

```
Thread-local routing:
  1. Check CORE_LOCAL_ITR_SENDER (on-core ITR)
     ↓ if available and not full
  Send to on-core ITR receiver
     ↓
  ITR processes on same core (isolated)
  
  2. If on-core full or not available:
     ↓
  Check INTERNAL_TELEMETRY_SENDER (dedicated overflow thread)
     ↓ if not full
  Send to internal telemetry collection thread
     ↓
  Routes to SDK or dedicated OTAP pipeline
  
  3. If both channels full:
     ↓
  Raw logger fallback (console, never fails)

**Channel Sizing**:
- CORE_LOCAL_ITR_SENDER: 1000 batches (default, configurable)
- INTERNAL_TELEMETRY_SENDER: 1-2 buffers (~64-128 KiB, small since overflow only)
- GLOBAL_TOKIO_SENDER: Larger capacity (not overflow case, handles 3rd party libs)
```

### Path 1: 3rd Party Logs (Global Tracing Subscriber) with Core-Local Delivery

**Use case:** Logs from 3rd party libraries via `tokio::tracing!()` macros

```
tokio::info!("message")
  ↓ Global tracing subscriber (OtlpTracingLayer)
  ↓ TracingLogRecord (LogRecordView)
  ↓ StatefulOtlpEncoder.encode_log_record() 
  ↓ encoder.flush() → OTLP bytes (one per event)
  ↓ Thread-local routing:
      ├─ CORE_LOCAL_ITR_SENDER (try first - if on engine core)
      ├─ GLOBAL_TOKIO_SENDER (global tokio subscriber channel)
      └─ Raw logger (last resort)
  ↓
  On-core ITR (preferred, engine cores only) OR Global tokio collector
```

**Key characteristics:**
- One OTLP bytes object per event (synchronous flush from global subscriber)
- Thread-local routing provides core isolation
- Overflow path prevents blocking
- Global collector only used when on-core unavailable or full

### Path 2: Component Logs with Core-Local Delivery

**Use case:** Component logging with `otel_info!(effect, "event", key=val)` in both Send+Sync and !Send components

```
otel_info!(effect, "event", key=val)
  ↓ Create tracing::Event + Metadata (bypass global subscriber)
  ↓ TracingLogRecord (LogRecordView)
  ↓ effect.log_event(&log_record_view)
  ↓ Shared: Arc<Mutex<StatefulOtlpEncoder>>
  ↓ Local: Rc<RefCell<StatefulOtlpEncoder>>
  ↓ encoder.encode_log_record()
  ↓ Check size threshold
      ↙ Overflow                    ↘ Under threshold
  encoder.flush()                   Continue accumulating
  Thread-local routing:                   ↓
    ├─ CORE_LOCAL_ITR_SENDER       Timer tick or manual flush
    ├─ INTERNAL_TELEMETRY_SENDER          ↓
    └─ Raw logger                  encoder.flush()
                                   Thread-local routing (same as overflow)
```

**Key characteristics:**
- Batching in effect handler buffer (StatefulOtlpEncoder)
- Size-based threshold flush (synchronous in caller context)
- Thread-local routing provides core isolation
- Overflow path: on-core ITR → global collector → raw logger
- Timer-based flush for accumulated bytes (periodic in component runtime)

### Path 4: Component Logs to SDK (No ITR Configured)

**Use case:** Component logging when `internal_collection.enabled: false` (default)

This is the SDK fallback path - component logs share the same export path as 3rd party logs.

```
otel_info!(effect, "event", key=val)
  ↓ Create tracing::Event + Metadata (bypass global subscriber)
  ↓ TracingLogRecord (LogRecordView)
  ↓ effect.log_event(&log_record_view)
  ↓ Shared/Local StatefulOtlpEncoder
  ↓ encoder.encode_log_record()
  ↓ Check size threshold
      ↙ Overflow                    ↘ Under threshold
  encoder.flush() → Bytes           Continue accumulating
  OtlpBytesChannel.send()                 ↓
      ↓                              Timer in admin runtime
  Admin runtime task                      ↓
      ↓                              encoder.flush() → Bytes
  Decode OTLP bytes                 OtlpBytesChannel.send()
      ↓                                    ↓
  Create SDK LogRecord              Admin runtime task
      ↓                                    ↓
  Call SDK exporter                 (same as overflow)
```

**Key imp3: Global Telemetry Collection Point

**Use case:** Centralized collection for metrics and logs when on-core ITR unavailable or as overflow

**Extended Metrics Channel**: The existing per-core → global metrics collection channel now carries:
```rust
enum TelemetryMessage {
    Metrics(MetricsReport),
    Logs(Vec<u8>),  // OTLP bytes (batched)
}
```

**Global Collection Flow**:
```
TelemetryMessage arrives at global collection point
  ↓
Check IN_TELEMETRY_COLLECTION (thread-local)
  ↓ if true: DROP (prevent recursion)
  ↓ if false: set flag = true
  ↓
Match message type:
  ├─ Metrics → Export via OTel Metrics SDK (existing)
  └─ Logs → Route based on mode:
       ├─ Mode 1 (SDK only): Decode → SDK LogRecord → SDK exporter
       ├─ Mode 2 (Hybrid): Find ITR → wrap as OtapPdata → inject
       └─ Mode 3 (ITR only): Find ITR → wrap as OtapPdata → inject
  ↓
Clear IN_TELEMETRY_COLLECTION flag
```

**Anti-Recursion**: The `IN_TELEMETRY_COLLECTION` thread-local prevents tokio-tracing events in the global collector thread from creating feedback loops.
## Implementation Components

### 1. Effect Handler Integration

Effect handler owns telemetry buffer and handles encoding/routing.

### 2. Macro Modifications

Update `otel_info!` and family to accept effect handler.

### 3. Internal Telemetry Receiver

The ITR is a standard OTAP receiver that:
- Has no inputs (generates data from effect handlers or GLOBAL_TELEMETRY_SENDER)
- Has one output port (forwards OTLP bytes as OtapPdata)
- Supports both on-core and global modes
- On-core: Receives from CORE_LOCAL_ITR_SENDER, timer-based flush
- Global: Receives from GLOBAL_TELEMETRY_SENDER (Mode 3 pipeline)
- **CRITICAL**: ITR and all downstream components configured with raw logger only

## Configuration Examples

### Example 1: Minimal (Console Only)

**Use case**: Development, debugging, minimal dependencies

```yaml
service:
  telemetry:
    logs:
      level: debug
      # No SDK processors, no ITR, no global threads
      # Everything goes directly to console (raw logger)
```

**Thread-locals**:
- CORE_LOCAL_ITR_SENDER: None
- INTERNAL_TELEMETRY_SENDER: None (direct console instead)
- GLOBAL_TOKIO_SENDER: None (direct console instead)

**Behavior**:
- Effect handlers: Encode → raw logger (synchronous console)
- Tokio subscriber: Encode → raw logger (synchronous console)
- Zero threading overhead, immediate output

---

### Example 2: SDK Only (Standard OpenTelemetry)

**Use case**: Simple deployments, standard observability backends

```yaml
service:
  telemetry:
    logs:
      level: info
      global_collection:
        destination: "sdk"
        sdk:
          processors:
            - batch:
                exporter:
                  otlp:
                    endpoint: "http://localhost:4317"
      internal_collection:
        enabled: false  # No on-core ITR
```

**Thread-locals**:
- CORE_LOCAL_ITR_SENDER: None (ITR disabled)
- INTERNAL_TELEMETRY_SENDER: Some (per-core → internal telemetry thread)
- GLOBAL_TOKIO_SENDER: Some (any thread → tokio subscriber thread)

**Behavior**:
- Effect handlers: Encode → INTERNAL_TELEMETRY_SENDER → internal telemetry thread → SDK
- Tokio subscriber: Encode → GLOBAL_TOKIO_SENDER → tokio thread → SDK
- Both threads set IN_TELEMETRY_COLLECTION = true
- Fallback: raw logger if channels full

---

### Example 3: On-Core ITR + SDK Overflow (Hybrid)

**Use case**: Production, core isolation, SDK fallback

```yaml
service:
  telemetry:
    logs:
      level: info
      global_collection:
        destination: "sdk"
        sdk:
          processors:
            - batch:
                exporter:
                  otlp:
                    endpoint: "http://collector:4317"
      internal_collection:
        enabled: true  # On-core ITR
        buffer_size_bytes: 65536
        overflow_destination: "global"
```

**OTAP pipeline**:
```yaml
pipelines:
  - name: internal_telemetry
    receivers:
      - internal_telemetry  # On-core ITR instances
    processors:
      - batch:
    exporters:
      - otlp:
          endpoint: "http://observability:4317"
```

**Thread-locals**:
- CORE_LOCAL_ITR_SENDER: Some (per-core ITR)
- INTERNAL_TELEMETRY_SENDER: Some (overflow → internal telemetry thread → SDK)
- GLOBAL_TOKIO_SENDER: Some (tokio subscriber thread → SDK)

**Behavior**:
- Effect handlers: CORE_LOCAL_ITR_SENDER → on-core ITR (best case)
- Effect handler overflow: INTERNAL_TELEMETRY_SENDER → SDK
- Tokio subscriber (on engine core): CORE_LOCAL_ITR_SENDER → on-core ITR (preferred)
- Tokio subscriber overflow: GLOBAL_TOKIO_SENDER → SDK
- Final fallback: raw logger

---

### Example 4: Full OTAP (No SDK)

**Use case**: Maximum control, all telemetry through dataflow engine

```yaml
service:
  telemetry:
    logs:
      level: info
      global_collection:
        destination: "otap"
        otap:
          pipeline_name: "internal_telemetry_overflow"
      internal_collection:
        enabled: true
        buffer_size_bytes: 65536
        overflow_destination: "global"
```

**OTAP pipelines**:
```yaml
pipelines:
  - name: internal_telemetry
    receivers:
      - internal_telemetry  # On-core ITRs
    processors:
      - batch:
    exporters:
      - otlp:
          endpoint: "http://observability:4317"
  
  - name: internal_telemetry_overflow
    receivers:
      - telemetry_channel  # Global collection thread
    processors:
      - batch:
    exporters:
      - otlp:
          endpoint: "http://observability:4317"
```

**Thread-locals**:
- CORE_LOCAL_ITR_SENDER: Some (per-core ITR)
- INTERNAL_TELEMETRY_SENDER: Some (overflow → dedicated OTAP pipeline)
- GLOBAL_TOKIO_SENDER: Some (tokio → dedicated OTAP pipeline)

**Behavior**:
- Effect handlers: CORE_LOCAL_ITR_SENDER → on-core ITR → internal_telemetry pipeline
- Effect handler overflow: INTERNAL_TELEMETRY_SENDER → internal_telemetry_overflow pipeline
- Tokio subscriber: Similar routing to dedicated pipelines
- No SDK dependency
- Final fallback: raw logger

---

### Example 5: Per-Core Channels Without ITR

**Use case**: Core isolation without ITR overhead

```yaml
service:
  telemetry:
    logs:
      level: info
      global_collection:
        destination: "sdk"
        channel_per_core: true  # Dedicated channel per core
        sdk:
          processors:
            - batch:
                exporter:
                  otlp:
                    endpoint: "http://localhost:4317"
      internal_collection:
        enabled: false  # No ITR
```

**Thread-locals**:
- CORE_LOCAL_ITR_SENDER: None (no ITR)
- INTERNAL_TELEMETRY_SENDER: Some (per-core dedicated channel, 1-2 buffers)
- GLOBAL_TOKIO_SENDER: Some (shared, larger capacity)

**Behavior**:
- Effect handlers: INTERNAL_TELEMETRY_SENDER (per-core isolated channel) → SDK
- Each core has dedicated overflow channel (noisy core doesn't affect others)
- Smaller channel capacity provides fast backpressure
- No ITR pipeline overhead
- Fallback: raw logger

---

## Configuration Reference

### `service.telemetry.logs.global_collection`

Controls global collection thread behavior:

```yaml
global_collection:
  destination: "sdk" | "otap" | "console" | "none"
  channel_per_core: true | false  # Default: false
  
  # When destination: "sdk"
  sdk:
    processors: [...]  # Standard OTel SDK config
  
  # When destination: "otap"
  otap:
    pipeline_name: "internal_telemetry_overflow"
```

**Options**:
- `destination: "sdk"`: Route to OpenTelemetry SDK (standard exporters)
- `destination: "otap"`: Route to dedicated OTAP pipeline (advanced)
- `destination: "console"`: Direct raw logger (minimal)
- `destination: "none"`: No global collection threads (direct console only)
- `channel_per_core`: Create dedicated overflow channel per core (isolation without ITR)

### `service.telemetry.logs.internal_collection`

Controls on-core ITR behavior:

```yaml
internal_collection:
  enabled: true | false  # Default: false
  buffer_size_bytes: 65536  # Per-thread buffer (default: 64 KiB)
  max_record_bytes: 16384   # Max single record (default: 16 KiB)
  max_record_count: 1000    # On-core channel capacity
  overflow_destination: "global" | "console"
```

**Options**:
- `enabled: true`: Create on-core ITR instances (CORE_LOCAL_ITR_SENDER)
- `overflow_destination: "global"`: Use INTERNAL_TELEMETRY_SENDER for overflow
- `overflow_destination: "console"`: Use raw logger for overflow (no global thread)

### Thread-Local Initialization Logic

```rust
fn initialize_engine_core(config: &TelemetryConfig) {
    // 1. On-core ITR (optional)
    if config.internal_collection.enabled {
        let itr = create_itr_receiver(config);
        CORE_LOCAL_ITR_SENDER.set(Some(itr.sender()));
    } else {
        CORE_LOCAL_ITR_SENDER.set(None);
    }
    
    // 2. Internal telemetry overflow (optional)
    match config.global_collection.destination {
        Destination::None => {
            INTERNAL_TELEMETRY_SENDER.set(None);  // Direct console
        }
        _ => {
            let capacity = if config.global_collection.channel_per_core {
                2  // Small: 1-2 buffers per core
            } else {
                1000  // Shared channel, larger
            };
            let sender = create_internal_telemetry_channel(capacity);
            INTERNAL_TELEMETRY_SENDER.set(Some(sender));
        }
    }
    
    // 3. Global tokio subscriber (optional)
    if config.global_collection.destination != Destination::None {
        let sender = create_tokio_subscriber_channel();
        GLOBAL_TOKIO_SENDER.set(Some(sender));
    } else {
        GLOBAL_TOKIO_SENDER.set(None);  // Direct console
    }
    
    // 4. Anti-recursion flag
    IN_TELEMETRY_COLLECTION.set(false);
}
```

---

## Routing Decision Trees

### Effect Handler Routing (Component Logs)

```rust
fn route_effect_handler_logs(otlp_bytes: Vec<u8>) {
    // Check anti-recursion flag
    if IN_TELEMETRY_COLLECTION.get() {
        raw_logger(otlp_bytes);
        return;
    }
    
    // Try on-core ITR
    if let Some(sender) = CORE_LOCAL_ITR_SENDER.get() {
        if sender.try_send(otlp_bytes.clone()).is_ok() {
            return;  // Delivered to on-core ITR
        }
    }
    
    // Try internal telemetry overflow thread
    if let Some(sender) = INTERNAL_TELEMETRY_SENDER.get() {
        if sender.try_send(otlp_bytes.clone()).is_ok() {
            return;  // Delivered to global thread
        }
    }
    
    // Last resort: raw logger
    raw_logger(otlp_bytes);
}
```

### Global Tokio Subscriber Routing (3rd Party Logs)

```rust
fn route_tokio_subscriber_logs(otlp_bytes: Vec<u8>) {
    // Check anti-recursion flag
    if IN_TELEMETRY_COLLECTION.get() {
        raw_logger(otlp_bytes);
        return;
    }
    
    // Try on-core ITR (if on engine core)
    if let Some(sender) = CORE_LOCAL_ITR_SENDER.get() {
        if sender.try_send(otlp_bytes.clone()).is_ok() {
            return;  // Delivered to on-core ITR
        }
    }
    
    // Try global tokio subscriber thread
    if let Some(sender) = GLOBAL_TOKIO_SENDER.get() {
        if sender.try_send(otlp_bytes.clone()).is_ok() {
            return;  // Delivered to global thread
        }
    }
    
    // Last resort: raw logger
    raw_logger(otlp_bytes);
}
```

### ITR-Downstream Component Routing (Anti-Loop)

```rust
fn route_itr_downstream_logs(otlp_bytes: Vec<u8>) {
    // Never check CORE_LOCAL_ITR_SENDER (prevents loops)
    
    // Check anti-recursion flag
    if IN_TELEMETRY_COLLECTION.get() {
        raw_logger(otlp_bytes);
        return;
    }
    
    // Try appropriate global thread
    let sender = if is_effect_handler() {
        INTERNAL_TELEMETRY_SENDER.get()
    } else {
        GLOBAL_TOKIO_SENDER.get()
    };
    
    if let Some(sender) = sender {
        if sender.try_send(otlp_bytes.clone()).is_ok() {
            return;  // Delivered to global thread
        }
    }
    
    // Last resort: raw logger
    raw_logger(otlp_bytes);
}
```

---

## Design Summary

### Orthogonal Configuration Axes

1. **ITR enabled/disabled**: Independent of global collection
2. **Global collection destination**: SDK, OTAP, console, or none
3. **Channel-per-core**: Optional for isolation without ITR overhead
4. **Overflow routing**: Global thread or direct console
5. **Anti-recursion**: Automatic via thread-locals and component marking

### Key Design Principles

- **All options are optional**: Can disable any component independently
- **Graceful fallbacks**: Always terminates at raw logger (never fails)
- **Bounded memory**: Every channel and buffer has fixed capacity
- **Core isolation**: Per-core channels prevent cross-core amplification
- **Zero recursion**: Multi-layer protection prevents feedback loops
- **Composition**: Mix and match options for specific deployment needs

### Common Patterns

- **Development**: destination="none" (direct console, no threads)
- **Simple production**: destination="sdk", ITR disabled (standard OTel)
- **High-performance**: ITR enabled, destination="sdk" (core isolation + overflow)
- **Full control**: ITR enabled, destination="otap" (all logs through dataflow)
- **Isolation without ITR**: channel_per_core=true, ITR disabled (lightweight throttling)



### Overview

The telemetry routing system provides three-tier delivery with configurable destinations:

1. **Core-local ITR** (optional, preferred): On-core processing, fastest path
2. **Global collection** (required, overflow): Centralized processing or OTAP pipeline  
3. **Raw logger** (always available, last resort): Synchronous console, never fails

### Thread-Local Variables

**Per-core thread-locals** (set during engine core initialization):

1. **CORE_LOCAL_ITR_SENDER**: `Option<mpsc::Sender<Vec<u8>>>`
   - Present when ITR configured on this core
   - Routes telemetry to on-core ITR receiver
   - Bounded channel (default 1000 batches)

2. **INTERNAL_TELEMETRY_SENDER**: `mpsc::Sender<Vec<u8>>`
   - Dedicated overflow channel for internal telemetry (effect handler logs)
   - Routes to internal telemetry collection thread
   - Small capacity: 1-2 buffers (~64-128 KiB, provides fast backpressure)
   - Per-core isolated channel (noisy logger doesn't affect other cores)

3. **GLOBAL_TOKIO_SENDER**: `Option<mpsc::Sender<Vec<u8>>>`
   - Routes to global tokio subscriber collection thread
   - Used by global tracing subscriber for 3rd party library logs
   - Larger capacity (not overflow case)
   - Separate from internal telemetry and metrics channels

**Telemetry collection thread-locals**:

4. **IN_TELEMETRY_COLLECTION**: `bool`
   - Set to `true` when processing telemetry in collection threads
   - Used by: internal telemetry thread, global tokio thread, dedicated pipeline (Mode 3)
   - Checked by OtlpTracingLayer before routing
   - Prevents recursion in telemetry processing contexts
   - When true: tokio-tracing-rs events use raw logger fallback

### Routing Algorithm

**Effect handler routing** (internal telemetry from components):

```rust
fn route_internal_telemetry(bytes: Vec<u8>) {
    // Check if we're in a telemetry collection thread (prevent recursion)
    if IN_TELEMETRY_COLLECTION.with(|flag| *flag.borrow()) {
        raw_logger_fallback(&bytes);
        return;
    }
    
    // Try on-core ITR first
    if let Some(sender) = CORE_LOCAL_ITR_SENDER.with(|s| s.clone()) {
        if sender.try_send(bytes.clone()).is_ok() {
            return;  // Delivered to on-core ITR
        }
    }
    
    // Overflow to internal telemetry collection thread
    if let Some(sender) = INTERNAL_TELEMETRY_SENDER.with(|s| s.clone()) {
        if sender.try_send(bytes.clone()).is_ok() {
            return;  // Delivered to overflow thread
        }
    }
    
    // Last resort: raw logger
    raw_logger_fallback(&bytes);
}
```

**Global tokio subscriber routing** (3rd party library logs):

```rust
fn route_tokio_event(bytes: Vec<u8>) {
    // Check if we're in a telemetry collection thread (prevent recursion)
    if IN_TELEMETRY_COLLECTION.with(|flag| *flag.borrow()) {
        raw_logger_fallback(&bytes);
        return;
    }
    
    // Try on-core ITR first (if on engine core)
    if let Some(sender) = CORE_LOCAL_ITR_SENDER.with(|s| s.clone()) {
        if sender.try_send(bytes.clone()).is_ok() {
            return;  // Delivered to on-core ITR
        }
    }
    
    // Route to global tokio subscriber thread
    if let Some(sender) = GLOBAL_TOKIO_SENDER.with(|s| s.clone()) {
        if sender.try_send(bytes.clone()).is_ok() {
            return;  // Delivered to tokio collector
        }
    }
    
    // Last resort: raw logger
    raw_logger_fallback(&bytes);
   Open Questions

### 1. ITR Operating Modes

**Problem:** How do we route telemetry when ITR is configured vs. when using SDK only?

**Solution:** Configuration-driven routing with three modes

1. **SDK only (default)**: `processors.len() > 0` and `internal_collection.enabled: false`
   - CORE_LOCAL_ITR_SENDER: None (not configured)
   - All telemetry routes to GLOBAL_TELEMETRY_SENDER
   - Global collector decodes → SDK LogRecord → SDK exporter
   - Uses OpenTelemetry SDK configuration (console, OTLP, etc.)

2. **Hybrid mode**: `processors.len() > 0` and `internal_collection.enabled: true`
   - CORE_LOCAL_ITR_SENDER: Some (per-core ITR configured)
   - Component logs prefer on-core ITR, overflow to global
   - 3rd party logs on engine cores also prefer on-core ITR
   - 3rd party logs on other threads (admin, etc.) go to global only
   - Global collector routes logs to appropriate destination

3. **ITR only (no SDK)**: `processors.is_empty()` and `internal_collection.enabled: true`
   - CORE_LOCAL_ITR_SENDER: Some (per-core ITR configured)
   - All telemetry prefers on-core ITR
   - Global collector forwards overflow to ITR (no SDK)
   - Maximum consistency - all logs through OTAP pipeline
- Excessive logging on one core only affects that core's ITR
- Other cores continue processing normally
- Failure domain is limited to single core's runtime

**Overflow provides safety**:
- When on-core ITR overloaded, global collector takes over
- Global collector can apply additional backpressure
- Raw logger ensures logging never fails completely

### Routing Destination Options

Based on configuration, `GLOBAL_TELEMETRY_SENDER` routes to one of three destinations:

#### Option 1: OpenTelemetry SDK (Mode 1 & 2)

**When**: `internal_collection.global_destination: "sdk"` (default)

**Global collector behavior**:
- Runs on admin/telemetry thread
- Sets `IN_TELEMETRY_COLLECTION = true` during processing
- TelemetryMessage::Metrics → OTel Metrics SDK (existing)
- TelemetryMessage::Logs → Decode OTLP bytes → SDK LogRecord → SDK exporter
- Uses OpenTelemetry SDK configuration (console, OTLP, etc.)

**Characteristics**:
- Standard SDK exporters and processors
- Declarative configuration via SDK config
- Good for simple deployments

#### Option 2: Dedicated OTAP Pipeline (Mode 3)

**When**: `internal_collection.global_destination: "pipeline"` and `processors: []`

**Global collector behavior**:
- Runs dedicated OTAP dataflow engine in separate thread
- Thread sets `IN_TELEMETRY_COLLECTION = true` (prevents recursion)
- TelemetryMessage::Logs → Wrap as OtapPdata → inject into pipeline
- Uses full OTAP components: batch, retry, transform, export
- Pipeline components configured with raw logger only

**Characteristics**:
- Maximum control and consistency
- All telemetry through OTAP pipeline
- "Be our own SDK" completely
- ITR-like behavior at global level

#### Option 3: Raw Logger Only (Mode 3 alternative)

**When**: `internal_collection.global_destination: "raw"`

**Global collector behavior**:
- Minimal processing, direct to console
- Decode OTLP bytes → format → stderr
- No SDK, no pipeline, just logging
- Useful for debugging or minimal footprint

**Characteristics**:
- Simplest possible path
- No dependencies
- Synchronous only

### Anti-Recursion Guarantees

**Multi-layer protection**:

1. **Core-local ITR components**: Use raw logger only (no effect handler telemetry)
2. **Global collector thread/pipeline**: `IN_TELEMETRY_COLLECTION` flag prevents recursion
3. **Downstream ITR components**: Can overflow to global OR raw logger
   - Check `IN_TELEMETRY_COLLECTION` before routing
   - If true: skip global, use raw logger directly
4. **Raw logger**: Separate code path, synchronous, never fails
5. **Bounded channels**: All channels have capacity limits
6. **Non-blocking send**: `try_send()` prevents blocking on full channels

**Result**: Zero amplification at any level, complete isolation, cascading fallbacks
      
      # OpenTelemetry SDK configuration for all logs
      processors:
        - batch:
            exporter:
              console:
```

**Mode 2: Hybrid** - Component logs to ITR, 3rd party logs to SDK:
```yaml
service:
  telemetry:
    logs:
      level: info
      # Enable internal collection - component logs routed to ITR receiver
      internal_collection:
        enabled: true
        buffer_size_bytes: 65536       # 64 KiB per-thread buffer (pre-allocated)
        max_record_bytes: 16384        # 16 KiB max single record (enables encoder optimization)
        max_record_count: 1000         # Bounded channel (records)
        flush_interval: "1s"           # ITR periodic flush
      
      # OpenTelemetry SDK configuration for 3rd party logs
      # Component logs bypass this when internal_collection.enabled: true
      processors:
        - batch:
            exporter:
              otlp:
                endpoint: "http://localhost:4317"
```

**Mode 3: ITR only (no SDK)** - Both 3rd party and component logs to ITR:
```yamlimplementation:**

**Engine core initialization** (per-core runtime startup):
1. Check if ITR receiver configured on this core
2. If yes: Set CORE_LOCAL_ITR_SENDER to ITR's input channel
3. Create dedicated overflow channel (1-2 buffer capacity)
4. Set INTERNAL_TELEMETRY_SENDER to overflow channel sender
5. Set GLOBAL_TOKIO_SENDER to global tokio subscriber channel (if available)
6. Initialize effect handlers with telemetry buffer

**Internal telemetry collection thread initialization**:
1. Set IN_TELEMETRY_COLLECTION = false initially
2. Receive overflow from all engine cores (separate channels per core)
3. Set IN_TELEMETRY_COLLECTION = true during processing
4. Route based on mode (SDK, hybrid, or OTAP pipeline)
5. Clear IN_TELEMETRY_COLLECTION after processing

**Global tokio subscriber thread initialization** (separate thread):
1. Set IN_TELEMETRY_COLLECTION = false initially
2. Receive events from GLOBAL_TOKIO_SENDER
3. Set IN_TELEMETRY_COLLECTION = true during processing
4. Route to SDK or configured destination
5. Clear IN_TELEMETRY_COLLECTION after processing

**OtlpTracingLayer** (global subscriber):
1. Encode log record to OTLP bytes
2. Call thread-local routing function
3. Routing function checks thread-locals and delivers accordingly

**Effect handler log_event()**:
1. Encode to StatefulOtlpEncoder buffer
2. On threshold/flush: Get OTLP bytes
3. Call thread-local routing function (same as global subscriber)
4. Core isolation: each core routes independently

**ITR pipeline components** (anti-recursion):
1. CORE_LOCAL_ITR_SENDER = None (explicitly disabled)
2. GLOBAL_TELEMETRY_SENDER routed to raw logger only
3. IN_TELEMETRY_COLLECTION checked, drops if true
4. Complete isolation prevents recursion
    processors:
      - batch:
          timeout: 1s
          send_batch_size: 1000
      - retry:
          max_attempts: 3
    exporters:
      - otlp:
          endpoint: "http://collector:4317"
  
  # ITR export pipeline (when internal_collection.enabled: true)
  # ITR and all downstream nodes automatically configured with SDK fallback
  - name: internal_telemetry
    receivers:
      - internal_telemetry  # References internal collection from service.telemetry
    processors:
      - batch:
          timeout: 5s
          send_batch_size: 100
    exporters:
      - otlp:
          endpoint: "http://observability-backend:4317"
```

**Routing logic in admin component:**

[SNIP]

**Note:** When ITR is configured, the pipeline engine automatically:
- Routes component logs to ITR receiver (bypasses SDK processors in modes 2 & 3)
- Routes 3rd party logs to ITR receiver in mode 3 (no SDK - admin locates and registers ITR as global subscriber)
- Marks ITR pipeline components as "anti-recursion zone" (they always use SDK fallback or console)
- Prevents infinite recursion by excluding ITR pipeline from internal_collection

**Implementation:** Effect handler creation receives destination channel based on configuration
- **Mode 1** (SDK only): All effect handlers get OtlpBytesChannel sender → SDK
- **Mode 2** (Hybrid): Component effect handlers get ITR sender, 3rd party uses SDK
- **Mode 3** (ITR only): All effect handlers get ITR sender, no SDK
- ITR pipeline components always get OtlpBytesChannel sender or raw console logger (anti-recursion)
- Both channels receive OTLP bytes with same encoding
- Pi2. Resource and Scope Name

**Problem:** What Resource and scope name should each component use?

**Options:**
- A) Shared Resource per pipeline, scope = node_id
- B) Per-component Resource, scope = component type
- C) Configurable per component

**Proposed:** 
- Resource: Pipeline-level (service.name, service.version, core.id)
- Scope: Component's node_id.to_string() (e.g., "receiver.otlp.0")
- Core ID in resource enables correlation of telemetry to specific coree?
3
**Options:**
- A) Shared Resource per pipeline, scope = node_id
- B) Per-component Resource, scope = component type
- C) Configurable per component

**Proposed:** 
- Resource: Pipeline-level (service.name, service.version)
- Scope: Component's node_id.to_string() (e.g., "receiver.otlp.0")

### 4. Buffer Size and Record Limits

**Problem:** What buffer size and individual record size limits should we use?

**Proposed:** 
- Buffer size: 64 KiB per thread (configurable)
- Max log record size: 16 KiB (configurable)
- Rationale: 
  - 16 KiB limit enables encoder optimization (2-byte length placeholders for 14-bit sizes)
  - 64 KiB buffer provides ~4× headroom for multiple records
  - Large enough to batch efficiently, small enough to avoid memory pressure
- Wh4n buffer is full: flush synchronously and retry encoding
- When record exceeds limit: drop with counter increment
- When channel is full: fall back to raw console logger

**Bounded Memory Guarantees:**
- StatefulOtlpEncoder: Pre-allocated 64 KiB per thread (fixed size)
- Channel capacity: 1000 records (configurable) × ~64 KiB = ~64 MB worst case
- Total per component: ~64 KiB buffer + share of channel capacity
- Graceful degradation when limits reached

### 5. Error Handling

**Problem:** What happens if `log_event()` fails?

**Solution:** Drop with metrics and occasional raw logging - **Critical for recursion prevention**

- `log_event()` returns void (never fails)
- **Buffer overflow**: Try to flush, then retry encoding; if flush fails, use raw logger
- **Channel full**: Try non-blocking send; if fails, use raw logger
- **Encoding errors**: Increment `telemetry.dropped_events` counter, use raw logger
- **Raw logger**: `LoggerProvider::init_default_console_tracing()` - synchronous console output
- **No per-event logging** - would cause recursion
- **No error returns** - caller must not handle telemetry failures

**Bounded Memory Guarantees:**

1. **Pre-allocated per-thread buffer**: StatefulOtlpEncoder with fixed capacity (e.g., 64 KiB)
2. **Max log record size**: 16 KiB limit (enables 2-byte length placeholders)
3. **Bounded channel**: OtlpBytesChannel/ITR channel with configurable depth (e.g., 1000 records)
4. **Graceful overflow**:
   - Buffer full → flush → retry encoding
   - Record too large (>16 KiB) → drop with counter
   -5Channel full → raw logger (console)
   - Never blocks the component operation
   - Always bounded memory usage

**Implementation:**

[SNIP]

**Rationale:** Telemetry must never fail the component operation. Bounded resources ensure we never cause OOM. When buffers/channels are full, degrade gracefully to synchronous console logging.

### 6. Macro Bypass Global Subscriber

**Problem:** How do we create tracing::Event and bypass the global subscriber?

**Current approach:** Macros use `tracing::info!()` which goes through global dispatcher.

**Options:**
- A) Don't bypass - use global subscriber for everything
  - Con: Performance overhead, recursive logging risk
- B) Direct `Event::dispatch_to()` with custom collector
  - Pro: Full control, no global state
  - 6on: Need to implement minimal collector
- C) Use `tracing::Span::new()` + `Event::new()` directly without dispatch
  - Pro: No subscriber overhead
  - Con: Must manually construct metadata

**Proposed:** Option C for now, reconsider if needed
- Construct Event and Metadata directly in macro
- Pass to effect.log_event() as TracingLogRecord
- No global subscriber involvement

### 7. Testing Strategy

**ProAdd TelemetryBuffer to EffectHandlerCore
3. ✅ Implement log_event() for local::EffectHandler
4. ✅ Implement log_event() for shared::EffectHandler
5. ✅ Add InternalCollectionConfig (internal_collection.enabled flag + bounded memory params)
6. ✅ Raw logger fallback (LoggerProvider::init_default_console_tracing - already exists)
7. ❌ **Extend metrics channel to carry logs**
   - Define TelemetryMessage enum (Metrics, Logs)
   - Update existing metrics collection channel
   - Extend global collector to handle both types
8. ❌ **Implement thread-local routing infrastructure**
   - Add CORE_LOCAL_ITR_SENDER thread-local
   - Add GLOBAL_TELEMETRY_SENDER thread-local
   - Add IN_TELEMETRY_COLLECTION thread-local
   - Implement route_otlp_bytes() function
9. ❌ **Update StatefulOtlpEncoder with fixed capacity**
   - Add max_capacity parameter to constructor
   - Return EncodingError::BufferFull when capacity exceeded
   - Pre-allocate buffer to max_capacity
10. ❌ **Update log_event() to use thread-local routing**
    - Try encode → BufferFull → flush → route_otlp_bytes()
    - Non-blocking delivery via thread-locals
    - Raw logger fallback when both channels full
11. ❌ **Update OtlpTracingLayer to use thread-local routing**
    - After encoding, call route_otlp_bytes()
    - Check IN_TELEMETRY_COLLECTION flag
    - Enables core-local delivery for 3rd party logs
12. ❌ **Implement SDK path in global collector** (Mode 1)
    - TelemetryMessage::Logs → Decode OTLP bytes
    - Create SDK LogRecord
    - Call SDK exporter
13. ❌ **Modify otel_* macros to accept effect handler**
14. ❌ **Implement ITR receiver with on-core delivery**
    Separate Channels Architecture (Step 7)

**Design Decision**: Use **separate dedicated channels** for different telemetry types:

1. **Metrics Channel** (existing):
   - Per-core → admin thread
   - Metrics reports only
   - Continues unchanged

2. **Internal Telemetry Overflow Channel** (new):
   - Per-core → dedicated internal telemetry thread
   - Component logs from effect handlers (overflow only)
   - Small capacity: 1-2 buffers (~64-128 KiB)
   - Provides fast backpressure and throttling

3. **Global Tokio Subscriber Channel** (new):
   - Any thread → global tokio collector thread
   - 3rd party library logs from tokio-tracing
   - Larger capacity (not overflow case)

**Benefits**:
- **Isolation**: Noisy logger on one core doesn't affect metrics or other cores
- **Independent throttling**: Each channel has appropriate capacity for its use case
- **Clear separation**: Internal telemetry, tokio events, and metrics are distinct flows
- **Predictable backpressure**: Small overflow channel quickly signals overload

### SDK Path in Internal Telemetry Thread (Mode 1 & 2)

When `internal_collection.enabled: false` (Mode 1) or as overflow in Mode 2, the internal telemetry collection thread handles logs:

1. **Receive OTLP bytes** from core's overflow channel (INTERNAL_TELEMETRY_SENDER)
2. **Set IN_TELEMETRY_COLLECTION = true** (prevent recursion)
3. **Decode OTLP bytes** to LogRecordView
4. **Create SDK LogRecord** (emulate tracing bridge)
5. **Call SDK LogProcessor/Exporter**
6. **Clear IN_TELEMETRY_COLLECTION = false**

**Current Status:** Configuration structure defined, separate channels architecture clarified
### SDK Path Implementation Details (Mode 1)

When `internal_collection.enabled: false` (default), component logs route through internal telemetry overflow:

1. **Effect handler overflow routes to INTERNAL_TELEMETRY_SENDER**
   - Small capacity channel (1-2 buffers per core)
   - Dedicated internal telemetry collection thread

2. **Internal telemetry thread processes overflow**:
   - Receives OTLP bytes from core overflow channels
   - Decodes → SDK LogRecord → SDK exporter
   - Sets IN_TELEMETRY_COLLECTION flag during processing

3. **Global tokio subscriber** (separate thread and channel):
   - Receives 3rd party logs via GLOBAL_TOKIO_SENDER
   - Larger capacity (not overflow case)
   - Routes to SDK or configured destination

**Current Status:** Configuration structure defined, separate channels architecture clarified

## Summary: Routing Decision Tree

**For effect handler telemetry** (internal component logs):

```
1. Check IN_TELEMETRY_COLLECTION
   ↓ If true: raw_logger_fallback() [prevents recursion]
   ↓ If false: continue

2. Check CORE_LOCAL_ITR_SENDER
   ↓ If Some: try_send()
       ↓ Success: delivered to on-core ITR [best case]
       ↓ Full: continue to step 3
   ↓ If None: continue to step 3

3. Check INTERNAL_TELEMETRY_SENDER
   ↓ try_send(bytes)  [small channel: 1-2 buffers]
       ↓ Success: delivered to internal telemetry overflow thread
           ↓ Routes based on mode:
               ├─ Mode 1 & 2: Decode → SDK exporter
               └─ Mode 3: Wrap → inject into OTAP pipeline
       ↓ Full: continue to step 4 [backpressure - throttle this core]

4. raw_logger_fallback() [last resort, never fails]
```

**For global tokio subscriber** (3rd party library logs):

```
1. Check IN_TELEMETRY_COLLECTION
   ↓ If true: raw_logger_fallback() [prevents recursion]
   ↓ If false: continue

2. Check CORE_LOCAL_ITR_SENDER (if on engine core)
   ↓ If Some: try_send()
       ↓ Success: delivered to on-core ITR
       ↓ Full: continue to step 3
   ↓ If None: continue to step 3

3. Check GLOBAL_TOKIO_SENDER
   ↓ try_send(bytes)  [larger channel: not overflow case]
       ↓ Success: delivered to global tokio collector thread
           ↓ Routes to SDK or configured destination
       ↓ Full: continue to step 4

4. raw_logger_fallback() [last resort, never fails]
```

**Channel isolation benefits**:
- Internal telemetry overflow: Per-core isolated, small capacity (fast backpressure)
- Global tokio subscriber: Larger capacity, handles 3rd party logs
- Metrics reporting: Separate channel (unaffected by log volume)

**Anti-recursion protection**:
- Global collector: `IN_TELEMETRY_COLLECTION = true` during processing
- Dedicated pipeline thread: `IN_TELEMETRY_COLLECTION = true` always
- ITR components: Check flag, use raw logger if true
- Result: Zero amplification, complete isolation

## Metrics

The ITR should expose:
- `telemetry.events_received`: Counter of log events
- `telemetry.events_dropped`: Counter of dropped events (overflow, errors)
- `telemetry.bytes_flushed`: Counter of bytes sent
- `telemetry.flush_count`: Counter of flush operations
- `telemetry.overflow_flush_count`: Counter of synchronous overflow flushes
- `telemetry.timer_flush_count`: Counter of timer-based flushes

## Performance Targets

Based on the original design doc:

- `otel_info!(effect, ...)`: < 100ns when not flushing
- Flush operation: < 10μs for typical batch (100 events)
- Overflow flush: Acceptable latency spike (blocking caller)
- No allocations in hot path (encoder reuses buffer)

## Migration Path

1. Phase 1: Implement ITR infrastructure (this design)
2. Phase 2: Migrate existing `otel_info!()` calls to use effect handler
3. Phase 3: Add configuration to enable/disable ITR per component
4. Phase 4: Performance tuning and optimization

## Key Design Points Summary

### All Major Decisions Resolved

Through design iteration, all architectural questions have been answered:

#### ✅ Routing Architecture

**Two separate paths**:

1. **tokio tracing path** (`tracing::info!()` from standard libs):
   - Goes through global subscriber (OtlpTracingLayer)
   - Global subscriber checks thread-locals:
     - Try CORE_LOCAL_ITR_SENDER (on-core ITR if configured)
     - Try GLOBAL_TELEMETRY_SENDER (global collector)
     - Fall back to raw logger
   - Works automatically for 3rd party libraries

2. **Custom macro path** (`otel_info!(effect, ...)` for pipeline components):
   - Bypasses global subscriber entirely
   - Directly calls effect handler's `log_event()`
   - Effect handler uses same thread-local routing
   - Effect handler knows if it's anti-recursion designated

#### ✅ Three Operating Modes

1. **Mode 1 - SDK Only**: GLOBAL_TELEMETRY_SENDER → OpenTelemetry SDK
2. **Mode 2 - Hybrid**: On-core ITR (preferred) → SDK (overflow)
3. **Mode 3 - OTAP Only**: On-core ITR (preferred) → Dedicated OTAP pipeline (overflow)

#### ✅ Bounded Memory Guarantees

- Buffer: 64 KiB per thread (pre-allocated, configurable)
- Max record: 16 KiB (enables encoder optimization)
- Channel: 1000 batches (configurable)
- Total worst case: ~64 MB per core
- Graceful degradation when limits reached

#### ✅ Anti-Recursion Protection

**Multi-layer safeguards**:

1. **`IN_TELEMETRY_COLLECTION` thread-local flag**
   - Set in global collector thread
   - Set in dedicated OTAP pipeline thread (Mode 3)
   - Checked before any routing

2. **Anti-recursion designated effect handlers**
   - Global telemetry pipeline components
   - ITR receivers and downstream processors
   - Designated at effect handler creation
   - When designated: skip logging or use raw logger only

3. **Cascading fallbacks**
   - On-core ITR → Global collector → Raw logger
   - Each level checks flag and bounded channels
   - Raw logger never fails

#### ✅ Resource and Scope Information

- **Resource**: Serialized at startup (service.name, service.version, core.id)
- **Scope**: Target name from tokio tracing (e.g., "otap_df::receiver::otlp")
- Core ID enables correlation to specific cores

#### ✅ Error Handling

#### ✅ Error Handling

- `log_event()` returns void (never fails to caller)
- Buffer full → flush → retry encoding
- Channel full → try next fallback level
- Encoding errors → increment counter, use raw logger
- No per-event error logging (prevents recursion)

#### ✅ Configuration Structure

- **global_collection**: Sibling to internal_collection
  - Configures where GLOBAL_TELEMETRY_SENDER delivers (SDK, OTAP pipeline, or raw)
  - Always present for metrics; logs use when internal_collection disabled or as overflow

- **internal_collection**: Optional on-core ITR
  - When disabled: All telemetry → GLOBAL_TELEMETRY_SENDER
  - When enabled: Core-local processing with configurable overflow

### Three Operating Modes

1. **Mode 1 - SDK Only**: GLOBAL_TELEMETRY_SENDER → OpenTelemetry SDK
2. **Mode 2 - Hybrid**: On-core ITR (preferred) → SDK (overflow)
3. **Mode 3 - OTAP Only**: On-core ITR (preferred) → Dedicated OTAP pipeline (overflow)

### Anti-Recursion Protection

- `IN_TELEMETRY_COLLECTION` thread-local prevents logging in telemetry processing contexts
- Global collector sets flag during processing
- Dedicated pipeline thread (Mode 3) sets flag on startup
- Raw logger used when flag is true
- ITR pipeline components always use raw logger

### Core Isolation Benefits

- One core's excessive logging doesn't affect others
- Failure domain limited to single core's runtime
- On-core ITR provides fast path
- Global overflow provides safety net

### Cascading Fallbacks

Every telemetry event follows: On-core ITR → Global collection → Raw logger

Result: Zero amplification, complete isolation, never fails.
