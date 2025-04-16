# OTLP pipeline data

Status: **Development**

This package defines an OTLP pipeline data interface based on
prost::Message objects.  The package layout:

- `mod.rs`: Core module that re-exports the procedural macros and
  defines the main OTLP message traits.
- `tests.rs`: Contains test cases for the OTLP pipeline data
  functionality.
- `derive/`: A procedural macro crate that provides derive macros for
  OTLP message types:
  - `src/lib.rs`: Implements the `Message` procedural macro for
    deriving OTLP interfaces.
- `model/`: A crate that provides model definitions for OTLP types:
  - `src/lib.rs`: Contains type definitions and metadata structures
    for OTLP messages.

## Design

This package aims to resemble the OpenTelemetry Collector "pdata"
interface, but for Rust.  Like the `pdatagen` command from that
repository (Golang), this package includes a model definition that
simplifies a few details, especially related to "oneof" features in
the protocol.

The file `model/src/lib.rs` defines how each message type is treated,
with a few fields in each type being declared as parameters (in order
of importance), with remaining fields set through a builder
pattern. There are cases where the builder pattern is not used because
all fields are specified as parameters.

Oneof cases are expanded into one constructor per field. Since OTLP
has no more than one oneof field per message, this leads to a simple
pattern. The most important example is AnyValue, and in this case
there are constructors named new_string, new_int, new_double, etc.

See the [tests](./tests.rs) for examples of the resulting syntax.

## Under development

This package is subject to change, still under development.
