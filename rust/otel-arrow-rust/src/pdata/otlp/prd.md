# OTLP Visitor Pattern Implementation - Product Requirements Document

## Visitor Pattern - COMPLETED ✅

The visitor pattern implementation has been successfully completed as the foundation for efficient OTLP message processing. The pattern defines two traits per OTLP data type, generated automatically through the procedural macro system.

For each type there is a **Visitor** (an actor) and a **Visitable**. These traits enable type-safe, in-order traversal of OTLP data structures with the following characteristics:

- **Visitable** impls are immutable, passed by `&self`, presenting OTLP data types through generated adapter structs
- **Visitor** impls are mutable, passed by `&mut self`, carrying processing logic as traversal descends
- **MessageAdapter** structs automatically generated for all OTLP types providing seamless integration
- **NoopVisitor** implementations generated for composable visitor patterns

The visitor calls the visitable, and the visitable calls child visitors. At the top-level, specialized visitors (e.g., `LogsVisitor`) consume the visitor and return associated results.

### Production Implementation Status

The complete visitor pattern has been implemented with the following achievements:

**ItemCounter Example - Production Ready**

Log record counting using the visitor pattern is now production-ready:

```rust
pub fn LogRecordCount(ld: &LogsData) -> usize {
    ItemCounter::new().visit_logs(&LogsDataAdapter::new(&ld))
}
```

**Performance Benchmarking Results**

Initial benchmarks for counting 10 resources × 10 scopes × 10 records each show:

```
OTLP Logs counting/Visitor
                        time:   [1.4456 ns 1.4878 ns 1.5354 ns]
OTLP Logs counting/Manual
                        time:   [930.08 ps 962.65 ps 996.07 ps]
```

The ~50% overhead represents the abstraction cost, which is acceptable for the flexibility gained. More complex operations should show better relative performance as traversal complexity increases.

## Generated Implementation Examples

All visitor traits, adapter structs, and implementations are now automatically generated through the procedural macro system. The following examples demonstrate the production-ready implementation:

### Generated Visitor Traits

For the `LogsData` type, the system generates:

```rust
pub trait LogsDataVisitor {
    fn accept_logs_data(&mut self, v: impl LogsDataVisitable);
}

pub trait LogsDataVisitable {
    fn accept_logs_data(&self, v: impl ResourceLogsVisitor);
}
```

### Generated MessageAdapter

The OTLP adapter is automatically generated:

```rust
pub struct LogsDataAdapter<'a> {
    data: &'a LogsData,
}

impl<'a> LogsDataAdapter<'a> {
    pub fn new(data: &'a LogsData) -> Self {
        Self { data }
    }
}

impl<'a> LogsDataVisitable for &LogsDataAdapter<'a> {
    fn accept_logs_data(&self, mut v: impl ResourceLogsVisitor) {
        for rl in &self.data.resource_logs {
            v.accept_resource_logs(&ResourceLogsAdapter::new(rl));
        }
    }
}
```

### ItemCounter Implementation

The complete production-ready ItemCounter using the generated visitor traits:

```rust
pub struct ItemCounter {
    count: usize,
}

impl ItemCounter {
    pub fn new() -> Self {
        Self { count: 0 }
    }
}

impl LogsVisitor for ItemCounter {
    type Return = usize;

    fn visit_logs(mut self, v: impl LogsDataVisitable) -> Self::Return {
        self.accept_logs_data(v);
        self.count
    }
}

impl LogsDataVisitor for ItemCounter {
    fn accept_logs_data(&mut self, v: impl LogsDataVisitable) {
        v.accept_logs_data(self);
    }
}

impl ResourceLogsVisitor for ItemCounter {
    fn accept_resource_logs(&mut self, v: impl ResourceLogsVisitable) {
        v.accept_resource_logs(NoopVisitor::new(), self);
    }
}

impl ScopeLogsVisitor for ItemCounter {
    fn accept_scope_logs(&mut self, v: impl ScopeLogsVisitable) {
        v.accept_scope_logs(NoopVisitor::new(), self);
    }
}

impl LogRecordVisitor for ItemCounter {
    fn accept_log_record(&mut self, _: impl LogRecordVisitable) {
        self.count += 1;
    }
}
```

## Next Phase: OTLP Protobuf Encoding/Decoding via Visitor Pattern

With the visitor pattern and MessageAdapter implementation complete, the next objective is to implement protobuf encoding and decoding through the visitor abstraction. This will enable direct performance comparison with Prost's generated code.

## Objective

Implement visitor-based OTLP protobuf encoding/decoding to benchmark against the standard Prost-generated implementation. We aim to demonstrate that visitor-based encoding can outperform intermediate protobuf object creation for large datasets.

## Implementation Strategy

### Phase 1: Understanding Prost Code Generation

Use `cargo expand` to examine Prost's generated code, specifically:
- `encoded_len()` methods for calculating message sizes
- `encode_raw()` methods for binary serialization
- Field annotations: `#[prost(string, tag="3")]` providing:
  - **Base type**: string, int32, bytes, etc.
  - **Tag number**: protobuf field identifier

Parse the annotations and add the results to the FieldInfo used in the derive module.

Consider DRY at this point. Take an opportunity to consider the existing code in that module after we have parsed the Prost field annotations, because we may be able to simplify the base_type extraction logic there.

### Phase 2: Upgrade Visitor pattern

For the next phase, we are going to need to be able to pass mutable
state from one node of the traversal to the children. Because of
Rust's memory safety rules, we will need to return the object from the
traversal in order to continue using it under the borrow checker.

For Phase 3, first pass, we need to share a `&mut Vec<usize>` throughout
the traversal in order to allocate each node's precomputed size. In
Phase 3, second pass, we need to pass an `Iterator<Item = usize>`
through the same traversal, but currently this is not possible without
an additional argument and return.

For the Visitor trait, add generic type Argument and associated type
Return. The method signature add the first parameter of type Argument, like:

```
pub trait TracesDataVisitor<Argument, Return> {
  type Return = Return;
  
  fn visit_traces_data(&mut self, arg: Argument, v: impl TracesDataVisitable) -> Return;
}
```

For the Visitable trait, add the same generic type Argument passes
into and returns from the visitable impl, like:

```
pub trait TracesDataVisitable<Argument> {
fn accept_traces_data(
  &self,
  arg: Argument
  resource_spans: impl ResourceSpansVisitor,
  ) -> Argument;
}
```

This will support the next phase of development. Please think
carefully about Rust syntax and borrow checker rules at this step.

### Phase 3: Visitor-Based Encoder Implementation

Implement two visitor patterns:

**Encoding Visitor (Visitable → protobuf \bytes)**
- Generate protobuf bytes directly from OTLP message objects
- Use Prost field annotations to determine encoding approach
- Two-pass algorithm:
  1. **Size calculation pass**: Build `Vec<usize>` containing length-delimited field sizes in traversal order
  2. **Encoding pass**: Generate output bytes using pre-calculated sizes

**Decoding Visitor (protobuf bytes → Visitor calls)**
- Parse protobuf bytes and invoke visitor methods
- Use tag numbers from field annotations for field identification
- Support incremental parsing for streaming scenarios

We will create two Visitors, one for to compute the length and one to encode bytes w/ precomputed length.

We will call the first pass visitor "{}PrecomputeSize" and the second pass visitor "{}EncodeBytes".

Keep in mind that the Prost implementations for encoded_len and
encode_raw are available in EXPANDED to give you an idea of the
pattern.

Note, however, that Prost uses an inefficient algorithm and that we
aim to improve upon it. Prost has no way to re-use the size computed
at each level in the traversal, so it repeatedly computes the size for
every level as it descends, which has O(n * m) complexity for depth m
and node count n. We want an O(n) algorithm which we get by computing
the sizes once.

The PrecomputedSize visitor new() a pre-allocated ("inner") Vec<usize>
by value to allow re-use. It will reset the vector and then for each
node in the order of traversal, it will claim the next slot and
calculate the the size of the body using recursive calls to the
visitables using child PrecompuedSize .

### Phase 4: Performance Benchmarking

Compare three approaches:
1. **Standard Prost**: Generated structs with intermediate objects
2. **Visitor Encoding**: Direct visitor-to-bytes conversion
3. **Visitor Decoding**: Direct bytes-to-visitor parsing

**Expected Performance Characteristics:**
- Visitor encoding should eliminate allocation overhead of intermediate protobuf objects
- Large datasets should show significant performance gains
- Memory usage should be reduced through streaming approach
- CPU efficiency should improve through reduced copying

## Technical Requirements

### Field Annotation Processing

Extract and utilize Prost field annotations:

```rust
// Example field annotation parsing
#[prost(string, tag="3")]  // → (ProtobufType::String, tag: 3)
#[prost(int64, tag="7")]   // → (ProtobufType::Int64, tag: 7)
#[prost(bytes, tag="12")]  // → (ProtobufType::Bytes, tag: 12)
```

### Two-Pass Encoding Algorithm

```rust
// Pass 1: Size calculation
let sizes: Vec<usize> = calculate_sizes_visitor(message);

// Pass 2: Encoding with known sizes  
let encoded: Vec<u8> = encode_with_sizes_visitor(message, &sizes);
```

### Integration Points

- Extend existing visitor traits with encoding/decoding capabilities
- Maintain compatibility with current MessageAdapter implementations
- Preserve type safety and error handling patterns
- Support all OTLP message types (logs, metrics, traces)

## Success Metrics

1. **Functional**: Complete protobuf encoding/decoding through visitor pattern
2. **Performance**: Benchmark comparison showing visitor approach advantages
3. **Compatibility**: Seamless integration with existing visitor infrastructure
4. **Coverage**: Support for all OTLP message types and field annotations

This phase will establish the foundation for high-performance OTLP processing and demonstrate the visitor pattern's effectiveness for protocol buffer operations.
