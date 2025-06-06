# OTLP Derive Package - Design Document

## URGENT STATUS: BROKEN - REQUIRES RESTORATION 🚨

**Current State**: The derive package is in a broken state after refactoring. The procedural macros are generating incorrect trait references that don't match actual trait definitions.

**Critical Issues**:
1. Trait naming mismatch: generating `StringVisitable` vs actual `StringVisitor<Argument>`
2. Wrong module paths: generating `prost::alloc::string::` vs `crate::pdata::`
3. Missing generic parameters in generated trait references

**Working Reference**: `src/pdata/otlp/derive/src/original.rs` contains the working implementation that needs to be restored.

## Overview

The OTLP Derive package is a procedural macro system that automatically generates visitor pattern implementations for OpenTelemetry Protocol (OTLP) message types. It enables type-safe, efficient traversal of OTLP data structures without manual implementation overhead.

## Purpose

### Core Objectives
1. **Automatic Code Generation**: Generate visitor traits, visitable traits, and adapter structs for all OTLP message types
2. **Type Safety**: Ensure compile-time correctness for visitor pattern implementations
3. **Performance**: Enable efficient protobuf encoding/decoding through visitor-based approaches
4. **Maintainability**: Reduce manual boilerplate and ensure consistency across message types

### Problem Statement
OTLP messages are complex, hierarchical protobuf structures. Manual implementation of visitor patterns for each message type would be:
- Error-prone due to repetitive boilerplate
- Inconsistent across different message types
- Difficult to maintain as protobuf schemas evolve
- Time-consuming for developers

## Architecture

### High-Level Design

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│  OTLP Message   │───▶│  Derive Macro    │───▶│  Generated Code │
│  (Protobuf)     │    │  (#[derive])     │    │  (Traits +      │
│                 │    │                  │    │   Adapters)     │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

### Generated Components

For each OTLP message type `T`, the system generates:

1. **`TVisitor<Argument>` trait**: Defines how to process the message with generic argument threading
2. **`TVisitable<Argument>` trait**: Defines how the message exposes itself for processing with state passing
3. **`TMessageAdapter<'a>` struct**: Wraps the original message for visitor pattern integration
4. **NoopVisitor implementations**: Default no-op implementations that thread arguments unchanged for flexible composition

### Phase 2 Generic Argument Pattern ✅ COMPLETE

The visitor pattern has been upgraded to support generic `<Argument>` parameters, enabling mutable state passing through the visitor traversal. This upgrade is essential for implementing the two-pass encoding algorithm in Phase 3.

**Key Features:**
- **Generic State Threading**: All visitor methods accept `Argument` and return `Argument` for state passing
- **Type-Safe**: Generic parameters ensure compile-time correctness of state threading
- **NoopVisitor Support**: Default implementations correctly thread arguments unchanged
- **Backward Compatible**: Existing code continues to work with the enhanced pattern

**Generated Pattern Example:**
```rust
pub trait TracesDataVisitor<Argument> {
    fn visit_traces_data(&mut self, arg: Argument, v: impl TracesDataVisitable<Argument>) -> Argument;
}

pub trait TracesDataVisitable<Argument> {
    fn accept_traces_data(&self, arg: Argument, resource_spans: impl ResourceSpansVisitor<Argument>) -> Argument;
}

impl<Argument> TracesDataVisitor<Argument> for crate::pdata::NoopVisitor {
    fn visit_traces_data(&mut self, arg: Argument, _v: impl TracesDataVisitable<Argument>) -> Argument {
        arg  // Thread argument unchanged
    }
}
```

## Implementation Structure

### File Organization

```
src/pdata/otlp/derive/src/
├── lib.rs                    # Main entry point and macro definitions
├── Cargo.toml               # Package dependencies
└── (utility modules)        # Organized within lib.rs
```

### Core Modules (within lib.rs)

```rust
mod ident_utils {     // Identifier generation utilities
mod type_utils {      // Type analysis and primitive detection  
mod path_utils {      // Module path resolution
mod field_utils {     // Field information extraction
```

## Data Structures

### FieldInfo Structure

The core data structure representing a protobuf field:

```rust
#[derive(Clone, Debug)]
struct FieldInfo {
    ident: syn::Ident,           // Field name
    is_param: bool,              // Is this a visitor parameter?
    is_optional: bool,           // Option<T> wrapper?
    is_repeated: bool,           // Vec<T> wrapper?
    is_oneof: bool,              // Part of oneof group?
    field_type: syn::Type,       // Full field type
    as_type: Option<syn::Type>,  // Converted type (for enums)
    tag: u32,                    // Protobuf field tag number
    prost_type: String,          // Protobuf type ("string", "message", etc.)
}
```

### Key Insights

1. **Tag and Type Parsing**: Essential for Phase 3 protobuf encoding
2. **Optional/Repeated Detection**: Required for proper visitor method generation
3. **Oneof Handling**: Complex fields requiring special visitor parameter generation

## Code Generation Process

### Entry Points

The derive macro system has two main entry points:

1. **`#[derive(Message)]`** → `qualified()` function (line ~630) 
2. **Direct derive calls** → Individual derive functions

### Major Code Generation Functions

#### 1. `derive_otlp_visitors()` (line 1024)

**Purpose**: Generates visitor and visitable traits for OTLP message types

**Quote Block**: Lines 1074-1089
```rust
let expanded = quote! {
    pub trait #visitor_name<Argument> {
        type Return;
        fn #visitor_method_name(&mut self, arg: Argument, v: impl #visitable_name<Argument>) -> Self::Return;
    }

    pub trait #visitable_name<Argument> {
        fn #visitable_method_name(&self, arg: Argument, #(#visitable_args),*) -> Argument;
    }

    impl<Argument> #visitor_name<Argument> for crate::pdata::NoopVisitor {
        type Return = Argument;
        fn #visitor_method_name(&mut self, arg: Argument, _v: impl #visitable_name<Argument>) -> Self::Return {
            arg
        }
    }
};
```

**Generates**:
- Visitor trait with generic `Argument` support
- Visitable trait with argument threading
- NoopVisitor implementation for composition

#### 2. `derive_otlp_adapters()` (line 1238)

**Purpose**: Generates MessageAdapter structs and their Visitable implementations

**Quote Block**: Lines 1305-1328 (main adapter structure)
```rust
let expanded = quote! {
    pub struct #adapter_name<'a> {
        data: &'a #struct_name,
    }

    impl<'a> #adapter_name<'a> {
        pub fn new(data: &'a #struct_name) -> Self {
            Self { data }
        }
    }

    impl<'a> #visitable_name<Argument> for &#adapter_name<'a> {
        fn #accept_method_name(&self, mut arg: Argument, #(#visitor_params),*) -> Argument {
            #(#field_calls)*
            arg
        }
    }
};
```

**Generates**:
- Adapter struct wrapping original OTLP message
- Constructor for adapter
- Visitable trait implementation with field traversal logic

#### 3. Field Traversal Generation (lines 380-460)

**Purpose**: Generates visitor calls for different field types within adapter implementations

**Multiple Quote Blocks for Different Field Types**:

**Required Fields** (line 380):
```rust
Some(quote! {
    arg = #visitor_param.#visit_method(arg, &#adapter_name::new(&#field_access));
})
```

**Optional Fields** (line 393):
```rust
Some(quote! {
    arg = if let Some(ref field_value) = #field_access {
        #visitor_param.#visit_method(arg, &#adapter_name::new(field_value))
    } else {
        arg
    };
})
```

**Repeated Fields** (line 410):
```rust
Some(quote! {
    for item in &#field_access {
        arg = #visitor_param.#visit_method(arg, &#adapter_name::new(item));
    }
})
```

**Oneof Variants** (lines 421-459): Complex pattern matching for oneof fields

#### 4. `derive_otlp_builders()` (line 853)

**Purpose**: Generates builder pattern implementations (legacy/future use)

**Quote Blocks**: Lines 926-960+ (builder implementation)

#### 5. Utility Functions with Quote Blocks

**`type_utils::get_primitive_visitor_trait()`** (lines 168-175):
```rust
match type_name {
    "String" => quote! { crate::pdata::StringVisitor },
    "bool" => quote! { crate::pdata::BooleanVisitor },
    "i32" => quote! { crate::pdata::I32Visitor },
    // ... etc
}
```

**`generate_visitor_type_for_oneof_variant()`** (line 1096): Handles complex oneof type resolution

### Field Analysis Pipeline

#### 1. `parse_prost_tag_and_type()` (line 574)

**Purpose**: Extracts protobuf annotations from `#[prost(...)]` attributes

**Key Logic**:
- Parses tag numbers from `tag="N"` 
- Extracts type information ("string", "message", etc.)
- Returns `(u32, String)` tuple for FieldInfo

#### 2. Field Property Detection

**`FieldInfo` Construction**: Happens in main derive functions
- `is_optional`: Detects `Option<T>` wrappers
- `is_repeated`: Detects `Vec<T>` wrappers  
- `is_oneof`: Identifies oneof group membership

#### 3. Adapter Requirement Analysis

**`needs_adapter_for_field()`** (line 1159):
- Uses `prost_type` for fast primitive type filtering
- Avoids expensive path resolution for 70%+ of fields
- Falls back to complex type analysis for message types

**`get_adapter_name_for_field()`** (line 1187):
- Generates qualified adapter names for message types
- Handles module path resolution through `path_utils`

### Type Resolution System

#### 1. Primitive Type Handling

**Location**: `type_utils` module functions
- Direct mapping to predefined visitor traits
- No generic parameters needed
- Used for strings, integers, booleans, etc.

#### 2. Message Type Handling  

**Location**: `generate_visitor_trait_for_type()` (line 1379)
- Full path resolution for cross-module references
- Generic parameter support for visitor traits
- Complex module qualification logic

#### 3. Path Resolution

**`resolve_visitor_trait_path_for_type()`** (line 1406):
- Delegates to `path_utils::resolve_type_path()`
- Ensures proper module qualification
- Handles crate-level and external type references

## Field Type Handling

### Primitive Types

Handled through specialized visitor traits:
- `StringVisitor` for string fields
- `IntVisitor` for integer fields  
- `BytesVisitor` for byte arrays
- etc.

### Message Types

Generate full visitor/visitable trait pairs:
- Resolved through module path analysis
- Proper qualification for cross-module references
- Recursive visitor pattern application

### Complex Types

#### Optional Fields (Option<T>)
```rust
if let Some(value) = &self.data.optional_field {
    visitor.visit_field(arg, &FieldAdapter::new(value))
} else {
    arg  // Pass through unchanged
}
```

#### Repeated Fields (Vec<T>)
```rust
for item in &self.data.repeated_field {
    arg = visitor.visit_field(arg, &FieldAdapter::new(item));
}
```

#### Oneof Fields
Generate separate visitor parameters for each variant:
```rust
// For oneof with variants A, B, C:
fn accept_message(&self, arg: Argument, 
                  variant_a: impl AVisitor<Argument>,
                  variant_b: impl BVisitor<Argument>, 
                  variant_c: impl CVisitor<Argument>) -> Argument
```

## Utility Modules

### ident_utils
- **Purpose**: Consistent identifier generation
- **Key Functions**:
  - `create_ident()`: Generate identifiers with proper spans
  - `visitor_name()`, `visitable_name()`: Naming conventions
  - `adapter_name()`: Adapter struct naming

### type_utils  
- **Purpose**: Type analysis and classification
- **Key Functions**:
  - `is_primitive_type()`: Detect built-in types
  - `get_primitive_visitor_trait()`: Map types to visitor traits
  - `extract_inner_type()`: Unwrap generic containers
  - `is_bytes_type()`: Special handling for byte arrays

### path_utils
- **Purpose**: Module path resolution for cross-module references
- **Key Functions**:  
  - `resolve_type_path()`: Generate qualified paths for visitor traits
  - Module qualification ensures proper trait resolution

### field_utils
- **Purpose**: Field information extraction and analysis
- **Key Functions**:
  - `parse_prost_tag_and_type()`: Extract protobuf annotations
  - `detect_field_properties()`: Identify optional/repeated characteristics

## Phase 2 Upgrade: Generic Argument Support

### Motivation
The original visitor pattern lacked the ability to pass mutable state through traversal, which is essential for:
- **Pass 1**: Size calculation using `&mut Vec<usize>`
- **Pass 2**: Encoding using `Iterator<Item = usize>`
- **Memory Safety**: Rust's borrow checker requires careful state management

### Implementation Changes

1. **Visitor Traits**: Added `<Argument>` generic parameter
2. **Associated Return Type**: `type Return` for visitor flexibility
3. **Method Signatures**: Accept `Argument` parameter, return `Self::Return` or `Argument`
4. **State Threading**: Arguments pass down the traversal tree and return up

### Borrow Checker Considerations

```rust
// The pattern enables:
let mut size_vec = Vec::new();
let final_vec = visitor.visit_message(size_vec, message_adapter);
// final_vec can be used after traversal completes
```

## Integration Points

### With Prost Generated Code
- **Field Annotations**: `#[prost(string, tag="3")]` parsing
- **Type Compatibility**: Seamless integration with generated structs
- **Module Paths**: Proper qualification for cross-crate references

### With OpenTelemetry Protocols
- **Message Coverage**: Supports all OTLP message types (logs, metrics, traces)
- **Schema Evolution**: Robust handling of protobuf schema changes
- **Standards Compliance**: Maintains compatibility with OTLP specifications

## Usage Patterns

### Basic Item Counting
```rust
pub fn log_record_count(logs: &LogsData) -> usize {
    ItemCounter::new().visit_logs((), &LogsDataAdapter::new(logs))
}
```

### Size Calculation (Phase 3)
```rust
pub fn calculate_sizes(message: &LogsData) -> Vec<usize> {
    let mut size_vec = Vec::new();
    PrecomputeSizeVisitor::new().visit_logs(size_vec, &LogsDataAdapter::new(message))
}
```

### Direct Encoding (Phase 3)  
```rust
pub fn encode_direct(message: &LogsData, sizes: Vec<usize>) -> Vec<u8> {
    EncodeVisitor::new().visit_logs(sizes.into_iter(), &LogsDataAdapter::new(message))
}
```

## Performance Characteristics

### Code Generation Impact
- **Compile Time**: Procedural macros add build overhead
- **Binary Size**: Generated code increases final binary size
- **Runtime**: Zero-cost abstractions with inlining

### Visitor Pattern Overhead
- **Initial Benchmarks**: ~50% overhead vs manual implementation
- **Scalability**: Better relative performance for complex operations
- **Memory**: Streaming-friendly with controlled allocation

## Future Enhancements

### Phase 3: Protobuf Encoding
- **Two-Pass Algorithm**: O(n) complexity vs Prost's O(n*m)
- **Direct Encoding**: Bypass intermediate object allocation
- **Size Optimization**: Pre-computed field sizes

### Additional Features
- **Custom Visitors**: Support for domain-specific processing
- **Streaming**: Enhanced support for large dataset processing
- **Error Handling**: Structured error propagation through visitor chain

## Maintenance Guidelines

### Adding New Message Types
1. Ensure protobuf annotations are present
2. Test with `cargo expand` to verify generation
3. Add integration tests for new visitor patterns

### Modifying Field Handling
1. Update FieldInfo structure if needed
2. Extend type_utils for new type classifications
3. Test edge cases thoroughly

### Performance Optimization
1. Profile generated code with benchmarks
2. Optimize hot paths in utility functions
3. Consider macro expansion efficiency

## Debugging and Development

### Tools
- **`cargo expand`**: Examine generated code
- **`EXPANDED` file**: Maintain current expansion snapshot
- **Unit Tests**: Verify individual component behavior
- **Integration Tests**: End-to-end visitor pattern validation

### Common Issues
1. **Borrow Checker**: Careful lifetime management in generated code
2. **Path Resolution**: Module qualification for visitor traits
3. **Type Mismatches**: Ensure generated signatures match expectations

### Complete Function-to-Quote Block Mapping

This table maps every major function to its quote blocks for easy navigation:

| Function | Line | Quote Blocks | Purpose |
|----------|------|--------------|---------|
| `derive_otlp_visitors()` | 1024 | 1074-1089 | Main visitor/visitable trait generation |
| `derive_otlp_adapters()` | 1238 | 1305-1328 | MessageAdapter struct and impl generation |
| `generate_visitor_calls_for_field()` | ~360 | 380, 393, 410, 421-459 | Field traversal logic in adapters |
| `derive_otlp_builders()` | 853 | 926-960+ | Builder pattern implementations |
| `type_utils::get_primitive_visitor_trait()` | ~160 | 168-175 | Primitive type to visitor trait mapping |
| `generate_visitor_type_for_oneof_variant()` | 1096 | Various | Oneof field visitor type resolution |
| `qualified()` | ~630 | 631, 640 | Main derive macro entry point |

### Quote Block Identification Technique

To find any quote block in the codebase:
1. Search for `quote!` to find all generation sites
2. Look at the function name containing the quote block
3. Examine the surrounding context to understand the generated code's purpose
4. Use line numbers to navigate directly to specific generation logic

Example: To find visitor trait generation, search for `pub trait.*Visitor` which leads to line 1075 in `derive_otlp_visitors()`.

This design document provides the foundation for understanding and extending the OTLP derive package efficiently.

## Function to quote! Block Mapping

This section provides a comprehensive mapping of where each `quote!` code generation block is located and what it generates. Each entry shows the function name, line numbers, and the purpose of the generated code.

### Core Visitor Trait Generation

#### `generate_visitor_trait()` (Lines 1074-1090)
- **Purpose**: Generates the main visitor trait, visitable trait, and NoopVisitor implementation
- **Key quote! block**: Lines 1074-1088
- **Generates**:
  ```rust
  pub trait SomeVisitor<Argument> {
      type Return;
      fn visit_some(&mut self, arg: Argument, v: impl SomeVisitable<Argument>) -> Self::Return;
  }
  
  pub trait SomeVisitable<Argument> {
      fn accept_some(&self, arg: Argument, visitor_params...) -> Argument;
  }
  
  impl<Argument> SomeVisitor<Argument> for crate::pdata::NoopVisitor {
      type Return = Argument;
      fn visit_some(&mut self, arg: Argument, _v: impl SomeVisitable<Argument>) -> Self::Return {
          arg
      }
  }
  ```

### Message Adapter Generation

#### `generate_message_adapter()` (Lines 1305-1320+)
- **Purpose**: Generates MessageAdapter struct and implements visitable trait
- **Key quote! block**: Lines 1305-1320
- **Generates**:
  ```rust
  /// MessageAdapter for presenting OTLP message objects as visitable.
  pub struct SomeAdapter<'a> {
      data: &'a SomeMessage,
  }
  
  impl<'a> SomeAdapter<'a> {
      pub fn new(data: &'a SomeMessage) -> Self {
          Self { data }
      }
  }
  
  impl<'a> SomeVisitable for &SomeAdapter<'a> {
      fn accept_some(&self, visitor_params...) {
          // visitor calls for each field
      }
  }
  ```

### Constructor Generation

#### `generate_constructors()` (Lines 992-1020)
- **Purpose**: Generates constructor methods and optional builder pattern
- **Main quote! block**: Lines 992-1020
- **Generates**:
  ```rust
  impl SomeMessage {
      // Constructor methods
  }
  
  // Optional builder pattern
  pub struct SomeMessageBuilder {
      inner: SomeMessage,
  }
  
  impl SomeMessageBuilder {
      // Builder methods
      pub fn finish(self) -> SomeMessage { self.inner }
  }
  ```

### Field Processing and Code Generation

#### `generate_field_setter_call()` (Lines 320-333, 320-352)
- **Purpose**: Generates field assignment code for constructors
- **Multiple quote! blocks**: Lines 320-333 (with Option/casting), 343-352 (defaults)
- **Generates**: Field assignment statements like `self.inner.field = value.into();`

#### `generate_visitor_call()` (Lines 380-460)
- **Purpose**: Generates visitor method calls for each field
- **Multiple quote! blocks**: Lines 380-460 (various patterns for different field types)
- **Generates**: Visitor calls like `visitor.visit_field(&self.data.field);`

### Visitor and Adapter Trait Resolution

#### `generate_visitor_trait_for_field()` (Lines 1363-1401)
- **Purpose**: Determines the correct visitor trait for a field type
- **Quote! blocks**: Lines 1363, 1391, 1394, 1398, 1401
- **Returns**: Visitor trait tokens like `crate::pdata::StringVisitor` or message-specific visitor traits

#### `generate_message_adapter_for_field()` (Lines 1211-1230)
- **Purpose**: Determines the correct message adapter for a field type
- **Quote! blocks**: Lines 1211, 1215, 1219, 1222, 1230
- **Returns**: Adapter tokens like `SomeMessageAdapter` or `PrimitiveAdapter`

### Primitive Type Mappings

#### `map_primitive_to_visitor()` (Lines 168-175)
- **Purpose**: Maps Rust primitive types to visitor traits
- **Quote! blocks**: Lines 168-175 (one per primitive type)
- **Maps**:
  - `String` → `crate::pdata::StringVisitor`
  - `bool` → `crate::pdata::BooleanVisitor`
  - `i32` → `crate::pdata::I32Visitor`
  - `i64` → `crate::pdata::I64Visitor`
  - `u32`/`u8` → `crate::pdata::U32Visitor`
  - `u64` → `crate::pdata::U64Visitor`
  - `f32`/`f64` → `crate::pdata::F64Visitor`

### Constructor Parameter Generation

#### `generate_new_method()` (Lines 508-512, 886-887, 926+)
- **Purpose**: Generates constructor method signatures and bounds
- **Quote! blocks**: Lines 508-512 (method signature), 886-887 (params), 926+ (method body)
- **Generates**: Constructor methods with generic type parameters and Into bounds

### Visitor Parameter Lists

#### Multiple functions generate visitor parameter lists:
- **Line 1060**: `quote! { #variant_param_name: #visitor_type<Argument> }` (for visitable args)
- **Line 1071**: `quote! { #param_name: impl #visitor_type<Argument> }` (for visitable args)
- **Line 1280**: `quote! { mut #variant_param_name: #visitor_type }` (for adapter params)
- **Line 1301**: `quote! { mut #visitor_param: impl #visitor_trait }` (for adapter params)

### Implementation Trait Bounds

#### Various functions generate trait implementation bounds:
- **Lines 1104, 1117, 1120, 1125, 1140, 1144, 1147**: Generate `impl TraitName` bounds for different visitor types
- **Purpose**: Determine correct trait bounds for visitor implementations based on field types

### Error Cases and Fallbacks

#### Default/Unknown handlers:
- **Line 175**: `_ => quote! { UnknownVisitor }` (unknown primitive types)
- **Line 1401**: `quote! { UnknownVisitor }` (unknown visitor types)
- **Line 1222**: `_ => quote! { UnknownMessageAdapter }` (unknown adapters)
- **Line 1147**: `_ => quote! { impl UnknownVisitor }` (unknown implementations)

## Code Generation Flow

The procedural macro follows this high-level flow:

1. **Parse input** → Extract struct/enum information
2. **Generate visitor traits** → Create Visitor<Argument> and Visitable<Argument> traits (Lines 1074-1090)
3. **Generate message adapters** → Create MessageAdapter structs (Lines 1305-1320+)
4. **Generate constructors** → Create constructor methods and builders (Lines 992-1020)
5. **Process each field** → Generate visitor calls and field setters using various helper functions
6. **Combine output** → Merge all generated code into final TokenStream

This mapping should help you locate exactly where specific code is generated and understand the relationships between the different code generation functions and their corresponding `quote!` blocks.

## Key Design Principles

### Visitor Pattern Field Handling

**IMPORTANT**: The visitor pattern generation logic should NOT use the `is_param` flag for field filtering. This is a critical distinction from the builder pattern:

- **Builder Pattern**: Uses `is_param` to determine which fields become constructor parameters
- **Visitor Pattern**: ALL fields participate in visitor traversal, regardless of `is_param`

### Field Processing Rules for Visitors

1. **All Fields Generate Visitor Parameters**: Every field in a message generates at least one visitor parameter in the visitable trait
2. **All Fields Generate Visitor Calls**: Every field generates at least one visitor call in the adapter implementation
3. **Oneof Fields**: Generate multiple visitor parameters (one per variant) but may have specialized call logic
4. **Parameter/Call Matching**: The number and types of visitor parameters MUST exactly match the visitor calls

### Common Pitfalls

❌ **Incorrect**: Filtering visitor fields by `is_param`
```rust
if !info.is_param { 
    continue; // DON'T DO THIS IN VISITOR GENERATION
}
```

✅ **Correct**: All fields participate in visitor pattern
```rust
// Process ALL fields for visitor generation
for info in all_fields {
    // Generate visitor parameter and call for every field
}
```

## RESTORATION INSTRUCTIONS

### Broken Components Analysis

**Current Broken Trait Generation**:
```rust
// BROKEN - Current macro generates:
#param_name: impl prost::alloc::string::StringVisitable
#param_name: impl prost::alloc::vec::VecVisitable
```

**Working Pattern (from original.rs)**:
```rust
// WORKING - Should generate:
#param_name: impl crate::pdata::StringVisitor<Argument>
#param_name: impl crate::pdata::BytesVisitor<Argument>  // for Vec<u8>
```

### Key Functions to Restore

**1. `generate_visitor_trait_for_field()` (original.rs:1414)**
- Handles primitive vs message type detection
- Maps primitive types to correct `crate::pdata::*Visitor<Argument>` traits
- Uses `get_primitive_visitor_trait()` for primitive type mapping

**2. `get_primitive_visitor_trait()` (original.rs:167)**
- Essential mapping function for primitive types:
```rust
"String" => quote! { crate::pdata::StringVisitor<Argument> },
"bool" => quote! { crate::pdata::BooleanVisitor<Argument> },
"i32" => quote! { crate::pdata::I32Visitor<Argument> },
"u32" | "u8" => quote! { crate::pdata::U32Visitor<Argument> },
"u64" => quote! { crate::pdata::U64Visitor<Argument> },
"f32" | "f64" => quote! { crate::pdata::F64Visitor<Argument> },
```

**3. Trait Name Pattern**
- Message types: `{TypeName}Visitor<Argument>` (e.g., `SpanVisitor<Argument>`)
- Primitive types: Use `crate::pdata::{Type}Visitor<Argument>` mapping
- Special case: `Vec<u8>` → `crate::pdata::BytesVisitor<Argument>`

### Files Requiring Fix

**`field_info.rs`**:
- `related_type()` method generates wrong trait names
- Should call `generate_visitor_trait_for_field()` logic instead

**`visitor.rs`**:
- Replace current trait generation with original's `generate_visitor_trait_for_field()`
- Add `get_primitive_visitor_trait()` function from original

### Validation Steps

1. **Run**: `cargo expand > EXPANDED`
2. **Check**: No `StringVisitable` references in EXPANDED
3. **Verify**: All trait references use `crate::pdata::*Visitor<Argument>` pattern
4. **Test**: All 36 tests pass after restoration
