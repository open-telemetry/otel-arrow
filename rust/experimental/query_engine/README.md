# Query Engine

One of the mentioned key deliverables in Phase 2 of otel-arrow is:

- **Prototype for DataFusion integration with OpenTelemetry data, OTTL-transform
  feasibility study**

This folder contains work in progress to implement a 'query engine' that can:

- Take in instructions in multiple common transform languages
- Produce an intermediate language abstraction from those instructions
- Execute requested manipulations on the data

The eventual vision of this work is using this 'query engine' as a processor in
an OTAP pipeline likely leveraging DataFusion for query execution.

## Folder structure

|Name            |Description                                                                              |
|----------------|-----------------------------------------------------------------------------------------|
|expressions     |Intermediate language and syntax tree for the query engine                               |
|kql-parser      |Parser to turn KQL queries into query engine expressions (syntax trees)                  |
|ottl-parser     |Parser to turn OTTL queries into query engine expressions (syntax trees)                 |
|parser-generics |Common parser components and implementations for common literals                         |
|engine-columnar |Query engine implementation which takes a syntax tree and runs over columnar data (arrow)|
|engine-recordset|Query engine implementation which takes a syntax tree and runs over set or records (otlp)|

## Intermediate Language Abstraction

The immediate exploration of an IL should focus on 2 languages that can both
produce the same internal query engine expressions.

OpenTelemetry Collector users may already be aware of the [OpenTelemetry
Transformation Language (or
OTTL)](https://github.com/open-telemetry/opentelemetry-collector-contrib/tree/main/pkg/ottl)
which may be used in various `processors` to shape data in certain ways.

In order to make sure this work is generalizable we've chosen another query
language, [Kusto Query Language (or
KQL)](https://learn.microsoft.com/kusto/query/?view=microsoft-fabric), to
support side by side.

To illustrate how these 2 languages may intersect in their data shaping,
consider the following examples of data filtering:

```yml
# OTTL filtering operation in a Collector pipeline
processors:
  filter:
    logs:
      log_record:
        - 'Foo == "bar"'
```

```kql
// KQL filtering operation
source
| where Foo == "bar"
```

These operations accomplish the same goal. In DataFusion, this operation may be
represented as the following Rust code using
[DataFrame.filter](https://docs.rs/datafusion/latest/datafusion/dataframe/struct.DataFrame.html#method.filter).

```rust
df.filter(col("Foo").eq("bar"))?;
```

A potential IL representation for this concept may be something like the
following (in Rust objects, loosely using DataFusion
[logical_expr](https://docs.rs/datafusion/latest/datafusion/logical_expr/index.html)
and
[expr](https://docs.rs/datafusion/latest/datafusion/logical_expr/expr/index.html)
concepts to suggest object/enum names).

```rust
LogicalExpression::Filter(
    BinaryExpression::Equals(
        Expression::Identifier("Foo"),
        Expression::Literal("bar")
    )
)
```
