# OTLP pipeline data

Status: **Development**

This package aims to resemble the OpenTelemetry Collector "pdata"
interface, but for Rust.  This package defines an OTLP pipeline data
interface based on prost::Message objects

For each OTLP message object, a set of associated types and methods
are generated through the `./derive` sub-crate in this directory.

The file `model/src/lib.rs` defines how each message type is treated,
with a few fields in each type being declared as parameters (in order
of importance), with remaining fields set through a builder
pattern. There are cases where the builder pattern is not used because
all fields are specified as parameters.

See the [tests](./tests.rs) for examples of the resulting syntax.

## Example generated code

Use `cargo expand` to see the full macro derivation, generally.

The `TracesData` message object derives the following code, for example:

```rust
                impl TracesData {
                    pub fn new<T1: Into<::prost::alloc::vec::Vec<ResourceSpans>>>(
                        resource_spans: T1,
                    ) -> Self {
                        Self {
                            resource_spans: resource_spans.into(),
                        }
                    }
                }
```
