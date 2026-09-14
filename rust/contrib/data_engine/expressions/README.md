# otel-arrow-contrib-data-engine-expressions

> **Experimental, pre-1.0.** This crate has no stability guarantees. Types,
> traits, and behavior may change or be removed without notice, including in
> patch releases. It is published so other pre-1.0 data-engine crates can
> depend on a versioned registry package; it is not yet recommended for
> production use.

## Overview

`otel-arrow-contrib-data-engine-expressions` defines the expression and value model
shared by the OpenTelemetry Arrow "data engine" query pipeline: a small AST
for scalar, logical, transform, and summary expressions, plus the runtime
value types that parsers build and that execution engines evaluate.

This crate has no dependency on any specific query language or execution
engine. It is the common vocabulary that connects a parser front end (for
example [`otel-arrow-contrib-data-engine-kql-parser`]) to an execution engine (for
example [`otel-arrow-contrib-data-engine-recordset`]).

## Principal types

- `Expression`, `QueryLocation`, `ExpressionError` - the base expression
  trait, source-location tracking, and error type shared by every expression
  kind.
- `PipelineExpression`, `PipelineExpressionBuilder` - the parsed, immutable
  representation of a query pipeline: constants, functions,
  initializations, and the ordered list of `DataExpression`s.
- `ScalarExpression`, `LogicalExpression`, `DataExpression`,
  `TransformExpression` - the expression node families (arithmetic,
  comparisons, branching/discard, map/rename/reduce transforms, and more).
- `StaticScalarExpression` and its variants (`StringScalarExpression`,
  `IntegerScalarExpression`, `DateTimeValue`, ...) - literal values produced
  during parsing.
- `ValueAccessor` - resolves a scalar expression chain against a record or
  intermediate value at evaluation time.

## Usage

This crate is a building block and is not typically used standalone. A
parser builds a `PipelineExpression` from a query string, and an execution
engine evaluates it against records:

```rust
use otel_arrow_contrib_data_engine_expressions::{
    DataExpression, PipelineExpressionBuilder,
};

// Parsers (for example otel-arrow-contrib-data-engine-kql-parser) construct a
// PipelineExpression from a query string; engines (for example
// otel-arrow-contrib-data-engine-recordset) evaluate it against records.
let pipeline = PipelineExpressionBuilder::new("source")
    .with_expressions(Vec::<DataExpression>::new())
    .build()
    .expect("pipeline optimization succeeds");
```

See [`otel-arrow-contrib-data-engine-kql-parser`] for a parser that produces
`PipelineExpression` values, and
[`otel-arrow-contrib-data-engine-recordset`] for an engine that executes them.

[`otel-arrow-contrib-data-engine-kql-parser`]: https://crates.io/crates/otel-arrow-contrib-data-engine-kql-parser
[`otel-arrow-contrib-data-engine-recordset`]: https://crates.io/crates/otel-arrow-contrib-data-engine-recordset
