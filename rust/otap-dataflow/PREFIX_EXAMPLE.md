# Pipeline Factory Macro with Prefix Parameter

This document demonstrates how the `pipeline_factory` macro now requires a mandatory prefix parameter to avoid name conflicts when used multiple times.

## Overview

The `pipeline_factory` macro has been updated to require a prefix parameter that is used to generate unique static variable names for each usage. This prevents naming conflicts when the macro is used in multiple crates or modules.

## Usage

### Before (Old Syntax)
```rust
#[pipeline_factory(DataType)]
static FACTORY: PipelineFactory<DataType> = build_factory();
```

### After (New Syntax)
```rust
#[pipeline_factory(PREFIX, DataType)]
static FACTORY: PipelineFactory<DataType> = build_factory();
```

## Example

Here's how the macro is used in the codebase:

### OTAP Crate
```rust
#[pipeline_factory(OTAP, OTAPData)]
static OTAP_PIPELINE_FACTORY: PipelineFactory<OTAPData> = build_factory();
```

This generates:
- `OTAP_RECEIVER_FACTORIES` - distributed slice for receiver factories
- `OTAP_PROCESSOR_FACTORIES` - distributed slice for processor factories  
- `OTAP_EXPORTER_FACTORIES` - distributed slice for exporter factories
- `get_otap_receiver_factory_map()` - helper function
- `get_otap_processor_factory_map()` - helper function
- `get_otap_exporter_factory_map()` - helper function

### OTLP Crate
```rust
#[pipeline_factory(OTLP, OTLPData)]
static OTLP_PIPELINE_FACTORY: PipelineFactory<OTLPData> = build_factory();
```

This generates:
- `OTLP_RECEIVER_FACTORIES` - distributed slice for receiver factories
- `OTLP_PROCESSOR_FACTORIES` - distributed slice for processor factories
- `OTLP_EXPORTER_FACTORIES` - distributed slice for exporter factories
- `get_otlp_receiver_factory_map()` - helper function
- `get_otlp_processor_factory_map()` - helper function
- `get_otlp_exporter_factory_map()` - helper function

## Benefits

1. **No Naming Conflicts**: Each usage of the macro generates uniquely named static variables
2. **Multiple Usages**: The macro can be safely used multiple times in the same codebase
3. **Clear Separation**: Different prefixes make it clear which factories belong to which system
4. **Backward Compatibility**: Existing code can be easily updated by adding a prefix parameter

## Migration

To migrate existing code:

1. Add a prefix parameter as the first argument to the macro
2. Update any direct references to the generated static variables to use the prefixed names
3. Update any references to the helper functions to use the prefixed function names

For example, if you were using:
```rust
use crate::RECEIVER_FACTORIES;
#[distributed_slice(RECEIVER_FACTORIES)]
```

Change it to:
```rust
use crate::MY_PREFIX_RECEIVER_FACTORIES;
#[distributed_slice(MY_PREFIX_RECEIVER_FACTORIES)]
```
