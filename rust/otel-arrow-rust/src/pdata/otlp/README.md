# OTLP pipeline data

Status: **Development**

This package aims to resemble the OpenTelemetry Collector "pdata"
interface, but for Rust.  This package defines an OTLP pipeline data
interface based on prost::Message objects

For each OTLP message object, a set of additional implementation,
associated methods and traits are generated through the ./derive
sub-crate of this directory. 

This package defines OTLP-specific structs relevant for encoding OTLP
as bytes. It does this by implementing a Visitor pattern, in which:

- Every OTLP message object has a corresponding Visitable trait, which is
  the thing a Visitor will handle to visit the children of that message.
- Every OTLP message object has a corresponding Visitor trait, which is how
  to apply custom logic to OTLP-like data.
- Every OTLP message object has also:
  - NoopVisitor: a do-nothing Visitor implements all traits
  - Builder: a builder pattern for defning test OTLP message objects,
    along with a new() method covering obligatory fields.
  - EncodedLen: a Visitor for computing the sizes of an OTLP object
    to an intermediate Vec<usize>, uses Accumulator, has test-only
    pdata_size() implementation.
  - Accumulator: an impl for summing the encoded size of the children
    of an OTLP message.

The file `model/src/lib.rs` defines how each message type is treated,
with a few fields in each type being declared as parameters (in order
of importance), with remaining fields set through a builder
pattern. There are cases where the builder pattern is not used because
all fields are specified as parameters.

See the [tests](./tests.rs) for examples of the resulting syntax.

## Example generated code

Derive rules on the `TracesData` message object derive the following code.

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
                pub trait TracesDataVisitor<Argument> {
                    fn visit_traces_data(
                        &mut self,
                        arg: Argument,
                        v: impl TracesDataVisitable<Argument>,
                    ) -> Argument;
                }
                pub trait TracesDataVisitable<Argument> {
                    fn accept_traces_data(
                        &mut self,
                        arg: Argument,
                        resource_spans: impl ResourceSpansVisitor<Argument>,
                    ) -> Argument;
                }
                impl<Argument> TracesDataVisitor<Argument>
                for crate::pdata::NoopVisitor {
                    fn visit_traces_data(
                        &mut self,
                        arg: Argument,
                        _v: impl TracesDataVisitable<Argument>,
                    ) -> Argument {
                        arg
                    }
                }
                impl<Argument> TracesDataVisitable<Argument> for &TracesData {
                    fn accept_traces_data(
                        &mut self,
                        mut arg: Argument,
                        resource_spans: impl ResourceSpansVisitor<Argument>,
                    ) -> Argument {
                        let mut resource_spans = resource_spans;
                        for item in &self.resource_spans {
                            arg = resource_spans.visit_resource_spans(arg, item);
                        }
                        arg
                    }
                }
                /// EncodedLen visitor for calculating protobuf encoded size
                pub struct TracesDataEncodedLen<const TAG: u32, const OPTION: bool> {}
                impl<
                    const TAG: u32,
                    const OPTION: bool,
                > TracesDataEncodedLen<TAG, OPTION> {
                    /// Create a new EncodedLen visitor.
                    pub fn new() -> Self {
                        Self {}
                    }
                    /// Calculate the sum of direct children's encoded lengths.
                    fn children_encoded_size(
                        &mut self,
                        mut arg: crate::pdata::otlp::PrecomputedSizes,
                        mut v: impl TracesDataVisitable<
                            crate::pdata::otlp::PrecomputedSizes,
                        >,
                    ) -> (crate::pdata::otlp::PrecomputedSizes, usize) {
                        let mut total = 0;
                        let mut resource_spans = crate::pdata::otlp::Accumulate::new(ResourceSpansEncodedLen::<
                            1u32,
                            true,
                        > {});
                        arg = v.accept_traces_data(arg, &mut resource_spans);
                        total += resource_spans.total;
                        (arg, total)
                    }
                }
                impl<
                    const TAG: u32,
                    const OPTION: bool,
                > TracesDataVisitor<crate::pdata::otlp::PrecomputedSizes>
                for TracesDataEncodedLen<TAG, OPTION> {
                    fn visit_traces_data(
                        &mut self,
                        mut arg: crate::pdata::otlp::PrecomputedSizes,
                        mut v: impl TracesDataVisitable<
                            crate::pdata::otlp::PrecomputedSizes,
                        >,
                    ) -> crate::pdata::otlp::PrecomputedSizes {
                        let idx = arg.len();
                        arg.reserve();
                        let (mut arg, total_child_size) = self
                            .children_encoded_size(arg, v);
                        arg.set_size(
                            idx,
                            crate::pdata::otlp::encoders::conditional_length_delimited_size::<
                                TAG,
                                OPTION,
                            >(total_child_size),
                        );
                        arg
                    }
                }
                impl TracesData {}
                impl<
                    V: TracesDataVisitor<crate::pdata::otlp::PrecomputedSizes>,
                > TracesDataVisitor<crate::pdata::otlp::PrecomputedSizes>
                for &mut crate::pdata::otlp::Accumulate<V> {
                    fn visit_traces_data(
                        &mut self,
                        mut arg: crate::pdata::otlp::PrecomputedSizes,
                        v: impl TracesDataVisitable<crate::pdata::otlp::PrecomputedSizes>,
                    ) -> crate::pdata::otlp::PrecomputedSizes {
                        let idx = arg.len();
                        arg = self.inner.visit_traces_data(arg, v);
                        self.total += arg.get_size(idx);
                        arg
                    }
                }
```

## Copilot instructions

## Procedural macros

There are OTLP-specific procedural macros implemented in
src/pdata/otlp/derive, these are invoked while generated the code in
src/proto. The procedural macros in this file are complex and will
require debugging, and it will help us to know how to recognize them.

Errors in the derived code will show up like the following, where "X"
characters denote arbitrary errors inside the derived code.

```
error[E0XXX]: XXXXXXXXXXXXXXX
  --> rust/otel-arrow-rust/src/proto/./././opentelemetry.proto.collector.metrics.v1.rs:16:1
   |
16 | #[crate::pdata::otlp::qualified("opentelemetry.proto.collector.metrics.v1.ExportMetricsPartialSuccess")]
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ XXXXXXXXX`
```

When developing procedural macros, it is important to use "cargo
expand" to generate code for understanding the problem. Combine this
command with "grep -A N -B N pattern" will help understand the
problem because the output is many KiB.

In these development scenarios, we maintain a file named EXPANDED at
the top of the workspace created by `cargo expand > EXPANDED`. This
allows you to see the current macro expansion, which can be especially
helpful to diagnose compilation errors caused by bugs in the macros.

When you are looking to understand a type that is being generated
through macros, refer to the EXPANDED file instead of searching
through the macro library.  ALWAYS update the EXPANDED file to have
the current expansion after modifying the procedural macros.

If the derive macros are panicking, it becomes difficult to debug this
way, and a backtrace of the Rust compiler tends to be difficult to
interpret. In these cases, use `eprintln!("🚨 ...")` to make progress.

## Macro debugging cycle

We will proceed to run a two commands in one:

```
cargo expand > EXPANDED 2> EXPAND_ERRORS; cargo check 2> CHECKED
```

Inspect CHECKED for compiler errors and EXPANDED for the raw source
code produced by the derive generation logic and EXPAND_ERRORS for
compiler errors during macro processing.

When the check step succeeds, the next thing to verify are the tests:

```
cargo test > TEST_OUTPUT 2> TEST_ERROR
```

After the tests run, inspect both files to learn whether the tests
passed or failed, then continue to iterate.
