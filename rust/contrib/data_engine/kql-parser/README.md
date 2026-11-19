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

## Supported KQL Grammar

This is a subset of KQL, not a complete Kusto implementation. The syntax below
describes the current [grammar](src/kql.pest) and its translation into expressions.
Successful parsing does not guarantee that an execution engine supports every
resulting expression or supplies the referenced source fields.

### Query Structure

A tabular query starts with a source identifier, followed by zero or more
pipe-separated operators. Statements are separated by semicolons; a final
semicolon is optional, except that scalar `let` declarations require one.
Spaces, tabs, newlines, `//` line comments, and `/* ... */` block comments are
accepted between tokens. Keywords and function names use the lowercase spellings
shown here; Boolean literals also accept `True`, `TRUE`, `False`, and `FALSE`.

```kusto
let threshold = 10;
source
| where duration > threshold
| extend duration_seconds = duration / 1000.0
| project name, duration_seconds
```

Source and variable identifiers start with an ASCII letter or `_`, followed by
ASCII letters, digits, or `_`. Field access also supports quoted keys, such as
`['service.name']`, nested paths such as `attributes.region`, and array/map
indexing such as `values[0]`, `values[-1]`, or `attributes[key]`. Index expressions
can be computed scalars. Assignment destinations must refer to a field or path
on the source, not a variable or the entire source.

### Tabular Operators

| Operator | Supported form | Purpose |
| --- | --- | --- |
| `extend` | `extend field = expression [, ...]` | Add or overwrite source fields. |
| `where` | `where predicate` | Retain records matching a logical expression. |
| `project` | `project field [, alias = expression, ...]` | Select source fields, optionally computing new fields. |
| `project-keep` | `project-keep field_or_pattern [, ...]` | Retain selected fields or paths. |
| `project-away` | `project-away field_or_pattern [, ...]` | Remove selected fields or paths. |
| `project-rename` | `project-rename new_name = old_name [, ...]` | Rename source fields. |
| `summarize` | `summarize [aggregate [, ...]] [by key [, ...]]` | Aggregate records, optionally grouped by scalar expressions. |

`project-keep` and `project-away` accept `*` patterns for top-level field names,
such as `service*`, as well as explicit accessors. Schema options can affect
field validation and pattern resolution.

The supported aggregate functions are `avg(expression)`, `count()`,
`max(expression)`, `min(expression)`, and `sum(expression)`. Aggregates and
grouping keys can be named using `name = expression`. Operators may follow
`summarize` to transform the summary fields.

```kusto
source
| summarize requests = count(), average = avg(duration) by service
| where requests > 10
| project service, requests, average
```

### Scalar and Logical Operators

| Category | Operators |
| --- | --- |
| Arithmetic | Binary `+`, `-`, `*`, `/`, `%`; unary `-`. |
| Equality | `==`, `!=`, case-insensitive `=~`, `!~`. |
| Ordering | `<`, `<=`, `>`, `>=`. |
| Boolean | `and`, `or`, `not(predicate)`. |
| Substring matching | `contains`, `contains_cs`, `!contains`, `!contains_cs`. |
| Term matching | `has`, `has_cs`, `!has`, `!has_cs`. |
| Membership | `in (expression, ...)`, `!in (...)`, `in~ (...)`, `!in~ (...)`. |
| Regular expressions | `matches regex expression`. |

String matching is case-insensitive unless the operator ends in `_cs`;
membership is case-sensitive unless it ends in `~`. Use `==` for equality:
single `=` is reserved for assignments and named arguments.

Binary precedence, from lowest to highest, is `or`, `and`, equality, ordering,
substring/term matching, membership, regular-expression matching, addition/
subtraction, then multiplication/division/modulo. Binary operators associate
left to right within each level. Parentheses override precedence.

### Literals and Types

| Value | Examples |
| --- | --- |
| Boolean | `true`, `false`. |
| Integer | `0`, `42`, `-42`. |
| Floating point | `1.5`, `1e3`, `real(1)`, `real(+inf)`, `real(-inf)`. |
| String | `"text"`, `'text'`; escapes include the enclosing quote, `\\`, `\n`, `\r`, `\t`, and `\uXXXX`. |
| Typed null | `bool(null)`, `long(null)`, `string(null)`, `dynamic(null)`, and other supported type names. |
| Date/time | `datetime(2024-01-01)`, `datetime("2024-01-01T00:00:00Z")`. |
| Time span | `1h`, `1.5s`, `timespan(01:02:03)`, `timespan("01:02:03")`, `timespan(2d)`. |
| Dynamic array/map | `dynamic([1, "two"])`, `dynamic({"region": "west", "values": [1, 2]})`. |
| Regular expression | `regex("pattern")`, `regex("pattern", "i")`. |

Time units include `d`, `h`, `m`, `s`, `ms`, and singular/plural forms of `day`,
`hour`, `minute`, `second`, `millisecond`, `microsecond`, and `tick`.
Dynamic arrays and maps can nest and contain scalar literals; they are not
general-purpose constructors for arbitrary computed expressions.

Type names accepted in typed nulls, function parameters, and `typeof` are
`bool`, `datetime`, `decimal`, `double`, `dynamic`, `guid`, `int`, `long`, `real`,
`regex`, `string`, and `timespan`. The expression model maps `int`/`long` to
integer, `decimal`/`double`/`real` to double, and `guid` to string. It does not
provide separate decimal or GUID value types. `dynamic` parameters do not
impose a specific scalar type.

`extract_json` can convert its result to Boolean, date/time, double, integer,
string, or time span using `typeof`; `typeof(dynamic)` leaves the result
unconverted, while `typeof(regex)` is rejected during translation.

### Built-in Scalar Functions

In the signatures below, square brackets denote optional arguments and `...`
denotes additional comma-separated arguments, not literal KQL syntax.

| Category | Supported signatures |
| --- | --- |
| Conditional | `iff(predicate, then, else)`, `iif(predicate, then, else)`, `case(predicate, value, ..., else)`, `coalesce(first, second, ...)`. |
| Conversion | `tobool(value)`, `toint(value)`, `tolong(value)`, `tofloat(value)`, `toreal(value)`, `todouble(value)`, `tostring(value)`, `todatetime(value)`, `totimespan(value)`. |
| String | `strlen(value)`, `replace_string(text, lookup, replacement)`, `substring(text, start [, length])`, `strcat(value, ...)`, `strcat_delim(delimiter, value, ...)`, `extract(pattern, capture_group, text)`. |
| Parsing | `parse_json(value)`, `parse_regex(pattern [, flags])`, `extract_json(path, text [, typeof(type)])`. |
| Array | `array_concat(array, ...)`. |
| Mathematical | `bin(value, size)`. |
| Temporal | `now([offset])`. |
| Logical | `not(predicate)`. |
| Type inspection | `gettype(value)`; `typeof(type)` is supported as the optional type argument of `extract_json`. |

### Variables and User-defined Functions

Scalar variables use `let name = expression;`. Scalar functions use
`let name = (parameter:type, ...) { ... }`; their bodies may contain local
scalar `let` declarations followed by an optional scalar return expression.
Scalar parameters may have statically resolvable default values. Calls accept
positional arguments and named arguments using `parameter = expression`.

```kusto
let add = (value:long, increment:long = 1) {
    let result = value + increment;
    result
};
source | extend next_value = add(value, increment = 2)
```

The parser also accepts wildcard tabular parameters such as `record:(*)`,
represented as mutable map parameters. Explicit tabular schemas such as
`record:(name:string)` are recognized by the grammar but rejected during
translation. Function bodies do not support tabular pipelines.

### Limits and Diagnostics

Operators outside the list above, including `join`, `union`, `sort`, `take`,
`distinct`, and `mv-expand`, are not implemented by this grammar. Neither are
tabular `let` bindings or arbitrary Kusto built-in functions. Additional
functions must be declared in the query or registered through `ParserOptions`.

The parser checks more than grammar: identifiers, argument counts and types,
assignment targets, and configured schemas can produce diagnostics even when
the text matches a grammar rule. `ParserError` reports syntax errors,
unsupported syntax, or KQL-style diagnostics with query locations. For the
implementation and executable examples, see [query parsing](src/query_expression.rs),
[tabular expressions](src/tabular_expressions.rs), and
[scalar expressions](src/scalar_expression.rs), including their test modules.
