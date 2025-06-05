# OTLP Visitor Pattern Implementation - Product Requirements Document

## Phase 2: Visitor Pattern Upgrade - COMPLETED ✅

The visitor pattern upgrade has been successfully completed, implementing generic argument and return types for mutable state passing through visitor traversal. This phase was essential for enabling the two-pass encoding algorithm in Phase 3.

The enhanced visitor pattern now supports:

- **Generic Type Parameters**: All visitor traits now support `<Argument>` generic parameters for state threading
- **Argument Threading**: Visitor methods accept and return `Argument` for proper state passing through traversal
- **NoopVisitor Support**: `NoopVisitor` implementations use `type Return = ()` pattern for consistent behavior
- **Borrow Checker Compliance**: State sharing works correctly with Rust's memory safety rules

For each type there is a **Visitor** (an actor) and a **Visitable**. These traits enable type-safe, in-order traversal of OTLP data structures with the following characteristics:

- **Visitable** impls are immutable, passed by `&self`, presenting OTLP data types through generated adapter structs
- **Visitor** impls are mutable, passed by `&mut self`, carrying processing logic as traversal descends
- **MessageAdapter** structs automatically generated for all OTLP types providing seamless integration
- **State Threading**: Arguments can be passed through visitor calls for mutable state management

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

This is our current generated code for Visitor and Vistitable traits
and the NoopVisitor and MessageAdapter patterns, for example using
ResourceSpans.

```rust
                pub trait ResourceSpansVisitor<Argument> {
                    fn visit_resource_spans(
                        &mut self,
                        arg: Argument,
                        v: impl ResourceSpansVisitable<Argument>,
                    ) -> Argument;
                }
                pub trait ResourceSpansVisitable<Argument> {
                    fn accept_resource_spans(
                        &self,
                        arg: Argument,
                        resource: impl crate::proto::opentelemetry::resource::v1::ResourceVisitor<
                            Argument,
                        >,
                        scope_spans: impl ScopeSpansVisitor<Argument>,
                        schema_url: impl crate::pdata::StringVisitor<Argument>,
                    ) -> Argument;
                }
                impl<Argument> ResourceSpansVisitor<Argument>
                for crate::pdata::NoopVisitor {
                    fn visit_resource_spans(
                        &mut self,
                        arg: Argument,
                        _v: impl ResourceSpansVisitable<Argument>,
                    ) -> Argument {
                        arg
                    }
                }
                /// MessageAdapter for presenting OTLP message objects as visitable.
                pub struct ResourceSpansMessageAdapter<'a> {
                    data: &'a ResourceSpans,
                }
                impl<'a> ResourceSpansMessageAdapter<'a> {
                    /// Create a new adapter
                    pub fn new(data: &'a ResourceSpans) -> Self {
                        Self { data }
                    }
                }
                impl<'a, Argument> ResourceSpansVisitable<Argument>
                for &ResourceSpansMessageAdapter<'a> {
                    fn accept_resource_spans(
                        &self,
                        mut arg: Argument,
                        mut resource_visitor: impl crate::proto::opentelemetry::resource::v1::ResourceVisitor<
                            Argument,
                        >,
                        mut scope_spans_visitor: impl ScopeSpansVisitor<Argument>,
                        mut schema_url_visitor: impl crate::pdata::StringVisitor<
                            Argument,
                        >,
                    ) -> Argument {
                        if let Some(f) = &self.data.resource {
                            arg = resource_visitor
                                .visit_resource(
                                    arg,
                                    &(crate::proto::opentelemetry::resource::v1::ResourceMessageAdapter::new(
                                        f,
                                    )),
                                );
                        }
                        for item in &self.data.scope_spans {
                            arg = scope_spans_visitor
                                .visit_scope_spans(
                                    arg,
                                    &(ScopeSpansMessageAdapter::new(item)),
                                );
                        }
                        arg = schema_url_visitor
                            .visit_string(arg, &self.data.schema_url);
                        arg
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

### Phase 2 Implementation Details - COMPLETED ✅

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

```rust
pub trait TracesDataVisitor<Argument> {
  fn visit_traces_data(&mut self, arg: Argument, v: &TracesDataAdapter) -> Argument;
}
```

For the Visitable trait, add the same generic type Argument passes
into and returns from the visitable impl, like:

```rust
pub trait TracesDataVisitable<Argument> {
fn accept_traces_data(
  &self,
  arg: Argument,
  resource_spans: &mut impl ResourceSpansVisitor<Argument>,
  ) -> Argument;
}
```

This will support the next phase of development. Please think
carefully about Rust syntax and borrow checker rules at this step.

**IMPLEMENTATION COMPLETED**: The visitor pattern has been successfully upgraded with generic argument support for state threading.

### Phase 3: Visitor-Based Encoder Implementation

Implement two visitor patterns:

**Encoding Visitor (Visitable → protobuf \bytes)**
- Generate protobuf bytes directly from OTLP message objects
- Use Prost field annotations to determine encoding approach
- Two-pass algorithm:
  1. **Size calculation pass**: Build `PrecomputedSizes` containing length-delimited field sizes in traversal order
  2. **Encoding pass**: Generate output bytes using pre-calculated sizes

As an example for the first pass:

```rust
pub struct LogsDataEncodedLen {
  tag: u32,
}

impl LogsDataVisitor<PrecomputedSizes> for LogsDataEncodedLen {
    fn visit_logs_data(&mut self, mut arg: PrecomputedSizes, rs: impl LogsDataVisitable<PrecomputedSizes>) -> PrecomputedSizes {
        let my_idx = arg.len();
        arg.sizes.reserve();

        let child_idx = arg.len();
        arg = rs.accept_resource_logs(arg, ResourceEncodedLen{TAG}, ScopeLogsEncodedLen{TAG});
        let child_size = arg.get_size(child_idx);

        // sum all children
        let total_size = child_size;
        let my_size = varint_size(self.tag<<3) + varint_size(total_size) + total_size;

        arg.set_size(my_idx, my_size);
        arg
    }
}
```

Note that "TAG" needs to be replaced by the protobuf tag number of
each child, which is known in the derive package.

**Decoding Visitor (protobuf bytes → Visitor calls)**
- Parse protobuf bytes and invoke visitor methods
- Use tag numbers from field annotations for field identification
- Support incremental parsing for streaming scenarios

We will create two Visitors, one for to compute the length and one to encode bytes w/ precomputed length.

We will call the first pass visitor "{}EncodedLen" and the second pass visitor "{}EncodeBytes".

Keep in mind that the Prost implementations for encoded_len and
encode_raw are available in EXPANDED to give you an idea of the
pattern.

Note, however, that Prost uses an inefficient algorithm and that we
aim to improve upon it. Prost has no way to re-use the size computed
at each level in the traversal, so it repeatedly computes the size for
every level as it descends, which has O(n * m) complexity for depth m
and node count n. We want an O(n) algorithm which we get by computing
the sizes once.

The EncodeLen visitor new() a pre-allocated ("inner") Vec<usize>
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
