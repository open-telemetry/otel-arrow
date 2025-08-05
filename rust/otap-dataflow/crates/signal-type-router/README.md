# SignalTypeRouter Processor

A zero-copy signal routing processor for the OTAP (OpenTelemetry Arrow Protocol) dataflow engine. The SignalTypeRouter efficiently routes OpenTelemetry signals (traces, metrics, logs, profiles) to different output ports based on their signal type without deserializing the underlying telemetry data.

## Features

- **Zero-copy routing**: Routes signal references without cloning or deserializing telemetry data
- **Efficient signal detection**: Uses native `signal_type()` method for fast signal type identification
- **Multi-port routing**: Routes different signal types to different output ports
- **Flexible dispatch strategies**: Supports broadcast, round-robin, random, and least-loaded routing
- **Configuration validation**: Comprehensive configuration parsing and validation

## Architecture

The SignalTypeRouter operates by:

1. **Signal Type Detection**: Efficiently identifies the type of incoming OpenTelemetry signals using the native `signal_type()` method
2. **Zero-Copy Routing**: Passes signal references (not clones) to downstream processors based on configured routing rules
3. **Port-Based Distribution**: Routes signals to different output ports based on signal type
4. **Dispatch Strategy Application**: Applies configured dispatch strategies (broadcast, round-robin, etc.) when multiple destinations exist

## Configuration

The SignalTypeRouter requires **explicit configuration** for all instances. Each router must specify:

- **Output ports**: Named ports for routing different signal types
- **Dispatch strategies**: How signals are distributed to destinations
- **Destinations**: Target nodes for each output port
- **Router-specific settings**: Configuration for signal handling behavior

### Complete Configuration Example

```yaml
type: otap
description: "Advanced signal type routing pipeline with explicit configuration"
settings:
  default_control_channel_size: 100
  default_pdata_channel_size: 100
nodes:
  # SignalTypeRouter - requires explicit port and destination configuration
  signal_router:
    kind: processor
    plugin_urn: "urn:otap:processor:signal_type_router"
    description: "Routes signals by type with zero-copy forwarding"
    out_ports:
      traces_out:
        dispatch_strategy: round_robin
        destinations: [traces_processor_1, traces_processor_2]
      metrics_out:
        dispatch_strategy: broadcast
        destinations: [metrics_storage, metrics_analytics]
      logs_out:
        dispatch_strategy: random
        destinations: [logs_storage_1, logs_storage_2]
      profiles_out:
        dispatch_strategy: least_loaded
        destinations: [profiles_processor_1, profiles_processor_2]
    config:
      drop_unknown_signals: false

  # All downstream processors must also be explicitly configured
  traces_processor_1:
    kind: processor
    plugin_urn: "urn:otap:processor:batch"
    config:
      timeout: "1s"
      send_batch_size: 512

  traces_processor_2:
    kind: processor
    plugin_urn: "urn:otap:processor:resource"
    config:
      attributes:
        service.name: "distributed-service"

  # ... additional explicit configurations for all referenced nodes
```

### Configuration Requirements

- **No Default Routing**: All signal types must have explicitly defined output ports
- **Named Destinations**: All `destinations` must reference explicitly configured nodes
- **Complete Specification**: Partial or simplified configurations are not supported
- **Validation**: Configuration validation ensures all references are resolvable

## Dispatch Strategies

- **Broadcast**: Sends each signal to ALL destinations (zero-copy fan-out)
- **Round-Robin**: Cycles through destinations for load balancing
- **Random**: Randomly selects a destination
- **Least-Loaded**: Routes to the least busy destination (requires load monitoring)

## Usage

### Basic Usage

```rust
use otap_df_signal_type_router::{SignalTypeRouter, SignalTypeRouterConfig};

// Create with default configuration
let config = SignalTypeRouterConfig::default();
let router = SignalTypeRouter::new(config);

// Create with custom configuration
let config = SignalTypeRouterConfig {
    drop_unknown_signals: true,
};
let router = SignalTypeRouter::new(config);
```

### Factory Usage

```rust
use serde_json::json;
use otap_df_signal_type_router::create_signal_type_router;

let config = json!({
    "drop_unknown_signals": false
});

let processor_config = otap_df_engine::config::ProcessorConfig::new("my_router");
let wrapper = create_signal_type_router(&config, &processor_config)?;
```

## Current Implementation Status

This is a **bootstrap MVP** implementation that includes:

✅ **Configuration parsing and validation**
✅ **Signal type detection for traces, metrics, logs, and profiles**  
✅ **Basic processor wrapper integration**
✅ **Zero-copy signal forwarding to default output**
✅ **Comprehensive test coverage**
✅ **Documentation and examples**

### Future Implementation (Pending OTAPPData enhancements)

🔄 **Multi-port routing based on signal type**
🔄 **Dispatch strategy implementation (broadcast, round-robin, etc.)**
🔄 **Port-specific configuration**
🔄 **Load-based routing for least-loaded strategy**

## Performance Characteristics

- **Zero Memory Allocation**: No cloning of telemetry data during routing
- **Constant Time Signal Detection**: O(1) pattern matching on signal types
- **Minimal CPU Overhead**: Lightweight routing logic
- **High Throughput**: Designed for production-scale telemetry processing

## Testing

Run the test suite:

```bash
cargo test -p otap-df-signal-type-router
```


## Integration

The SignalTypeRouter integrates with the OTAP dataflow engine through:

- **ProcessorFactory registration**: Registered as `"signal_type_router"`
- **Standard processor interface**: Implements both `LocalProcessor` and `SharedProcessor` traits
- **Configuration system**: Uses the standard node configuration framework
- **Effect handlers**: Works with both local and shared effect handlers

## Documentation

- Check the [API documentation](https://docs.rs/otap-df-signal-type-router) for full API reference

## License

This project is licensed under the Apache-2.0 License.
