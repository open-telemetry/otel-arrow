# otel-arrow-contrib-data-engine-parser-abstractions

> **Experimental, pre-1.0.** This crate has no stability guarantees. Types,
> traits, and behavior may change or be removed without notice, including in
> patch releases. It is published so other pre-1.0 data-engine crates can
> depend on a versioned registry package; it is not yet recommended for
> production use.

## Overview

`otel-arrow-contrib-data-engine-parser-abstractions` provides the [`pest`](https://docs.rs/pest)-based
building blocks shared by data-engine query language front ends (for
example the KQL parser). It standardizes how a language-specific parser
turns pest parse trees into
[`otel-arrow-contrib-data-engine-expressions`](https://crates.io/crates/otel-arrow-contrib-data-engine-expressions)
expression trees: source-location tracking, literal parsing helpers, parser
state/scoping, map schema declarations, and error/diagnostic types.

It does not implement any concrete grammar itself; language crates such as
[`otel-arrow-contrib-data-engine-kql-parser`] depend on it to avoid re-implementing
these cross-cutting concerns.

## Principal types

- `Parser` - the trait every language front end implements:
  `parse`/`parse_with_options` return a `ParserResult` or a list of
  `ParserError`s.
- `ParserOptions`, `ParserMapSchema`, `ParserMapKeySchema` - configure the
  source/summary map schemas, attached data names, and user-defined
  functions available while parsing a query.
- `ParserState`, `ParserStateScope`, `ParserScope`, `ParserFunction` -
  mutable parsing state (scopes, declared functions, diagnostics)
  threaded through a recursive-descent parse.
- `ParserError` - the structured error/diagnostic type returned by parsers.
- `to_query_location`, `parse_standard_bool_literal`,
  `parse_standard_null_literal`, `parse_standard_integer_literal`,
  `parse_standard_double_literal`, `parse_standard_string_literal` - helpers
  for converting pest `Pair`s into `QueryLocation`s and common literal
  scalar expressions.

## Usage

This crate is consumed by language-specific parsers, not typically used
directly by end users. A language crate implements `Parser` and uses the
shared helpers while walking its pest parse tree:

```rust,ignore
use otel_arrow_contrib_data_engine_parser_abstractions::{
    Parser, ParserOptions, ParserResult, ParserError,
};

pub struct MyLanguageParser {}

impl Parser for MyLanguageParser {
    fn parse_with_options(
        query: &str,
        options: ParserOptions,
    ) -> Result<ParserResult, Vec<ParserError>> {
        // Parse `query` with a pest grammar, using this crate's helpers
        // (`to_query_location`, `parse_standard_string_literal`, ...) to
        // build an `otel_arrow_contrib_data_engine_expressions::PipelineExpression`.
        todo!()
    }
}
```

See [`otel-arrow-contrib-data-engine-kql-parser`] for a complete example of a
language front end built on these abstractions.

[`otel-arrow-contrib-data-engine-kql-parser`]: https://crates.io/crates/otel-arrow-contrib-data-engine-kql-parser
