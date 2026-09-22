# otel-arrow-contrib-data-engine-kql-parser

> **Experimental, pre-1.0.** This crate has no stability guarantees. Types,
> supported syntax, and behavior may change or be removed without notice,
> including in patch releases. It is published so other pre-1.0 data-engine
> crates can depend on a versioned registry package; it is not yet
> recommended for production use.

## Overview

`otel-arrow-contrib-data-engine-kql-parser` parses a subset of the Kusto Query
Language (KQL) into an
[`otel-arrow-contrib-data-engine-expressions`](https://crates.io/crates/otel-arrow-contrib-data-engine-expressions)
`PipelineExpression`. It is built on
[`otel-arrow-contrib-data-engine-parser-abstractions`](https://crates.io/crates/otel-arrow-contrib-data-engine-parser-abstractions)
and a [`pest`](https://docs.rs/pest) grammar (`src/kql.pest`) covering
tabular operators (`extend`, `where`, `project`, `summarize`, ...), scalar
and logical expressions, and the built-in KQL function surface implemented
by this crate.

The resulting `PipelineExpression` is engine-agnostic; it can be evaluated by
[`otel-arrow-contrib-data-engine-recordset`](https://crates.io/crates/otel-arrow-contrib-data-engine-recordset)
or any other compatible execution engine.

## Principal types

- `KqlParser` - implements the shared
  `otel_arrow_contrib_data_engine_parser_abstractions::Parser` trait for KQL.
- Re-exported parser-abstractions surface (`Parser`, `ParserError`,
  `ParserOptions`, `ParserResult`, `ParserMapSchema`, `ParserMapKeySchema`) so
  consumers do not need a direct dependency on
  `otel-arrow-contrib-data-engine-parser-abstractions` just to parse a query.

## Usage

```rust
use otel_arrow_contrib_data_engine_kql_parser::{KqlParser, Parser};

let result = KqlParser::parse("source | extend value = 1");
match result {
    Ok(parser_result) => {
        let _pipeline = parser_result.pipeline;
        // Evaluate the pipeline with an execution engine, e.g.
        // otel-arrow-contrib-data-engine-recordset's `RecordSetEngine`.
    }
    Err(diagnostics) => {
        // Each entry is a `ParserError` describing a parse failure.
        for diagnostic in diagnostics {
            eprintln!("{diagnostic:?}");
        }
    }
}
```

Use `KqlParser::parse_with_options` with a
`otel_arrow_contrib_data_engine_parser_abstractions::ParserOptions` to declare a
source/summary map schema, attached data names, or user-defined functions
before parsing.
