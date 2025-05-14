# AttributeStore vs AttributeStore2

This document explains the differences between the two implementations of `AttributeStore` and provides instructions on how to run benchmarks to compare their performance.

## Implementation Differences

### AttributeStore (Original)

The original `AttributeStore<T>` implementation:

- Uses a `HashMap<T, Vec<KeyValue>>` to store attributes
- Copies data from the RecordBatch into new `KeyValue` objects in memory
- Provides attributes directly as slices of a `Vec<KeyValue>`
- Allocates new memory for each unique parent ID and its attributes
- More memory usage but potentially faster attribute access

### AttributeStore2 (New, Reference-based)

The new `AttributeStore2<T>` implementation:

- Maintains a reference to the original RecordBatch
- Builds linked lists using indices to navigate attributes with the same parent ID
- Uses a combination of:
  - `HashMap<T, usize>` to store the first index for each parent ID
  - `Vec<Option<usize>>` to store "next pointers" for the linked list of attributes
- Creates `KeyValue` objects on-demand during iteration rather than storing them
- Less memory usage but potentially more CPU overhead when accessing attributes
- Uses an iterator interface for attribute access rather than returning slices

## Memory Efficiency

`AttributeStore2` should be more memory-efficient because it doesn't duplicate the data from the RecordBatch:

- Original: Stores a complete copy of all attributes in new `KeyValue` objects
- New: Only stores indices and a reference to the RecordBatch

## Access Patterns

- Both implementations support parent ID lookups
- Both implementations support delta parent ID lookups
- `AttributeStore` returns attributes as slices (`&[KeyValue]`)
- `AttributeStore2` returns attributes as iterators (`AttributeIterator<T>`)

## Running Benchmarks

Benchmarks are implemented using the Criterion crate to compare:

1. Creation time (how long it takes to build each store)
2. Lookup performance (how fast attributes can be accessed by ID)
3. Delta ID lookup performance (how fast attributes can be accessed with delta encoding)

To run the benchmarks:

```
cargo bench --bench attribute_store_bench
```

This will generate benchmark results and graphs in the `target/criterion` directory.

## Memory Usage Comparison

The benchmarks include a memory usage comparison function that estimates the memory overhead of each implementation. While not run automatically as part of the benchmarks, you can call it directly to see the memory usage differences:

```rust
use otel_arrow_rust::otlp::attributes::store::AttributeStore;
use otel_arrow_rust::otlp::attributes::store2::AttributeStore2;

fn main() {
    memory_usage_comparison();
}
```
