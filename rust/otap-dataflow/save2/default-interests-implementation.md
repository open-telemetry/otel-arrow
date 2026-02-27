# Default Interests Implementation for Processor Duration Metrics

## Problem

Debug processor at `verbosity: detailed` was not emitting duration metrics, while `noop_exporter` was. The issue was that processors weren't getting timestamps captured on entry because the timestamp capture was deferred to `subscribe_to`, but processors at Detailed level need timestamps captured upfront before processing begins.

## Solution Design

Processors declare upfront that they "may subscribe_to" using a new `Interests::METRICS` flag. This is called "default interests" — a processor can have empty default interests, or include `METRICS` to indicate it will produce consumer duration metrics.

**Timestamp capture condition:**
```
if default_interests.contains(METRICS) && metric_level >= Detailed {
    stamp timestamp on entry
}
```

## Changes Made

### 1. Added `Interests::METRICS` Flag

**File:** `crates/engine/src/lib.rs`

```rust
bitflags! {
    pub struct Interests: u8 {
        const ACKS = 1 << 0;
        const NACKS = 1 << 1;
        const RETURN_DATA = 1 << 2;
        const METRICS = 1 << 3;  // NEW: processor declares it may subscribe_to
    }
}
```

### 2. Updated `ReceivedAtNode` Trait

**File:** `crates/engine/src/lib.rs`

Changed signature from `stamp_time: bool` to `default_interests: Interests`:

```rust
pub trait ReceivedAtNode {
    fn received_at_node(
        &mut self,
        node_id: usize,
        metric_level: MetricLevel,
        default_interests: Interests,  // NEW
    );
}
```

### 3. Updated `OtapPdata` Implementation

**File:** `crates/otap/src/pdata.rs`

- `push_entry_frame` now takes `default_interests: Interests` instead of `stamp_time: bool`
- Timestamp is captured if `default_interests.contains(Interests::METRICS) && metric_level >= MetricLevel::Detailed`
- `received_at_node` passes `default_interests` to `push_entry_frame`
- `received_at_exporter` always passes `Interests::METRICS` (exporters always need timestamps)

### 4. Added `default_interests` to `ProcessorWrapper`

**File:** `crates/engine/src/processor.rs`

Added `default_interests: Interests` field to both `Local` and `Shared` variants:

```rust
pub enum ProcessorWrapper<PData> {
    Local {
        // ... existing fields ...
        /// Default interests declared by this processor.
        /// If `METRICS` is set, entry frames capture timestamps at Detailed level.
        default_interests: Interests,
    },
    Shared {
        // ... existing fields ...
        default_interests: Interests,
    },
}
```

Added methods:
- `set_default_interests(&mut self, value: Interests)` — set default interests after creation
- `default_interests(&self) -> Interests` — get current default interests

### 5. Updated Processor Run Loop

**File:** `crates/engine/src/processor.rs`

The processor run loop now passes `default_interests` to `received_at_node`:

```rust
let default_interests = self.default_interests();
pdata.received_at_node(
    effect_handler.processor_id().index,
    current_metric_level(),
    default_interests,
);
```

### 6. Updated `TestMsg` Implementation

**File:** `crates/engine/src/testing/mod.rs`

Updated signature to include `_default_interests: Interests` parameter.

### 7. Updated Debug Processor Factory

**File:** `crates/otap/src/debug_processor.rs`

When `verbosity == Detailed`, set `Interests::METRICS`:

```rust
pub fn create_debug_processor(...) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
    let processor = DebugProcessor::from_config(pipeline_ctx, &node_config.config)?;
    // Processors with Detailed verbosity need METRICS interest to stamp time on entry.
    let default_interests = if processor.config.verbosity() == Verbosity::Detailed {
        Interests::METRICS
    } else {
        Interests::empty()
    };
    let mut wrapper = ProcessorWrapper::local(processor, node, node_config, processor_config);
    wrapper.set_default_interests(default_interests);
    Ok(wrapper)
}
```

### 8. Updated Tests

**File:** `crates/otap/src/pdata.rs`

Updated all test calls from boolean to `Interests` types:
- `false` → `Interests::empty()` (processor without metrics interest)
- `true` → `Interests::METRICS` (processor/exporter with metrics interest)

## How It Works

1. **Processor creation:** Factory decides if processor needs duration metrics (e.g., debug_processor at Detailed verbosity) and calls `set_default_interests(Interests::METRICS)`

2. **Message entry:** When a message arrives at a processor, `received_at_node` is called with the processor's `default_interests`

3. **Timestamp capture:** `push_entry_frame` checks `if default_interests.contains(METRICS) && metric_level >= Detailed` and stamps time if true

4. **ACK processing:** When ACK arrives back at the entry frame, the entry frame has a valid `time_ns` for computing duration

5. **Metrics recording:** `record_consumed_success(duration_ns)` is called with the computed duration

## Key Insight

The previous design deferred timestamp capture to `subscribe_to`, which meant processors that didn't explicitly call `subscribe_to` (or called it later) wouldn't have timestamps. The new design uses `default_interests` as a declaration that the processor *will* need timestamps, allowing the engine to capture them immediately on entry.

Exporters always pass `Interests::METRICS` via `received_at_exporter` since they always produce ACKs with duration metrics.
