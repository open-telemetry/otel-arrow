# Send With Subscription API Design

## Overview

This document describes the new `send_message_subscribed` API that unifies the previously separate `subscribe_to` + `send_message` pattern into a single operation. This design supports automatic instrumentation while maintaining efficient frame reuse for the Ack/Nack mechanism.

## Background: OpenTelemetry Collector RFC

The [Pipeline Component Telemetry RFC](https://github.com/open-telemetry/opentelemetry-collector/blob/main/docs/rfcs/component-universal-telemetry.md) specifies:

- **Metrics**: `consumed.items` and `produced.items` with `outcome` attribute (`success`, `failure`, `refused`)
- **Two instrumentation layers per edge**: One for producer, one for consumer
- **Auto-instrumentation via graph edges**: Capture telemetry at handoff points between components

The otap-dataflow engine uses asynchronous message passing with an Ack/Nack mechanism. This design enables RFC-aligned instrumentation on both the forward path (channel sends) and return path (Ack/Nack delivery).

## New API

### Traits

Two new traits replace the deprecated `ProducerEffectHandlerExtension`:

```rust
/// For local (!Send) effect handlers
#[async_trait(?Send)]
pub trait SendWithSubscriptionLocalExtension<PData> {
    async fn send_message_subscribed(
        &self,
        data: PData,
        interests: Interests,
        calldata: CallData,
    ) -> Result<(), TypedError<PData>>;

    fn try_send_message_subscribed(...) -> Result<(), TypedError<PData>>;
    
    async fn send_message_subscribed_to<P>(...) -> Result<(), TypedError<PData>>;
    
    fn try_send_message_subscribed_to<P>(...) -> Result<(), TypedError<PData>>;
}

/// For shared (Send) effect handlers
#[async_trait]
pub trait SendWithSubscriptionSharedExtension<PData: Send + 'static> {
    // Same methods
}
```

### Frame Reuse Behavior

The `subscribe_or_update` method in `Context` implements intelligent frame reuse:

| Scenario | Result |
|----------|--------|
| No existing frame for node | Push new frame |
| Top frame is same node | Update: merge interests (OR), replace calldata |
| Frame exists but not at top | Push new frame |

This enables:
- **Single frame per node**: Component subscriptions and auto-instrumentation share one frame
- **Retry-friendly**: When RetryProcessor re-sends after NACK, its frame gets updated
- **Interests merge**: `ACKS` + `NACKS` → `ACKS | NACKS` in one frame
- **Calldata replacement**: Latest calldata takes precedence

## Migration Guide

### Before (deprecated)

```rust
// Two separate calls
effect_handler.subscribe_to(
    Interests::ACKS | Interests::NACKS | Interests::RETURN_DATA,
    RetryState::new(num_items, deadline).into(),
    &mut data,
);
effect_handler.send_message(data).await?;
```

### After (recommended)

```rust
// Unified call
effect_handler.send_message_subscribed(
    data,
    Interests::ACKS | Interests::NACKS | Interests::RETURN_DATA,
    RetryState::new(num_items, deadline).into(),
).await?;
```

### Imports

```rust
use otap_df_engine::{
    Interests,
    SendWithSubscriptionLocalExtension,    // for local effect handlers
    SendWithSubscriptionSharedExtension,   // for shared effect handlers
    control::CallData,
};
```

## Implementation Details

### Files Modified

| File | Changes |
|------|---------|
| `crates/engine/src/lib.rs` | Added `SendWithSubscriptionLocalExtension` and `SendWithSubscriptionSharedExtension` traits; deprecated `ProducerEffectHandlerExtension` |
| `crates/otap/src/pdata.rs` | Implemented new traits for all 4 effect handler variants; added `subscribe_or_update` method to `Context` |

### Context Stack Methods

```rust
impl Context {
    /// Original method - always pushes a new frame
    pub(crate) fn subscribe_to(
        &mut self,
        interests: Interests,
        calldata: CallData,
        node_id: usize,
    );

    /// New method - updates existing frame if same node is at top
    pub(crate) fn subscribe_or_update(
        &mut self,
        interests: Interests,
        calldata: CallData,
        node_id: usize,
    );
}
```

## Future Work: Auto-Instrumentation

With this foundation, auto-instrumentation can be added by:

1. **Forward Path**: Channels can push an instrumentation frame with `item_count` and `byte_size` before sending
2. **Return Path**: Pipeline controller records metrics when delivering Ack/Nack, extracting size info from calldata
3. **Outcome Tracking**: Add `downstream` flag to `NackMsg` to distinguish `failure` vs `refused` per RFC

The `subscribe_or_update` behavior ensures component subscriptions and auto-instrumentation frames are consolidated, avoiding stack bloat.

## Tests

New unit tests verify the frame reuse behavior:

- `test_context_subscribe_or_update_pushes_new_frame`
- `test_context_subscribe_or_update_updates_existing_frame`
- `test_context_subscribe_or_update_only_updates_top_frame`
- `test_context_subscribe_or_update_inherits_return_data`

All 354 existing tests continue to pass.

## Deprecation

The `ProducerEffectHandlerExtension` trait and its `subscribe_to` method are deprecated but remain functional for backward compatibility. Components should migrate to the new `send_message_subscribed` API.

```rust
#[deprecated(
    since = "0.2.0",
    note = "Use SendWithSubscriptionLocalExtension or SendWithSubscriptionSharedExtension instead"
)]
pub trait ProducerEffectHandlerExtension<PData> {
    fn subscribe_to(&self, int: Interests, ctx: CallData, data: &mut PData);
}
```
