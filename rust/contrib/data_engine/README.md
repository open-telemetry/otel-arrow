# Data Engine

## Scope and repository placement

This workspace provides a general-purpose expression model, parser
abstractions, language parsers, and evaluation engines. Its architecture is
independent of OTAP and OpenTelemetry data: consumers supply their own
languages, records, and integration layers.

The OTAP Dataflow query engine is one consumer of these crates, but that
integration lives separately in
[`otel-arrow-dfe-query-engine`](../../otap-dataflow/crates/query-engine/).
It does not define the data engine's scope.

The workspace lives under `rust/contrib` in the `otel-arrow` repository while
it is developed and maintained here. Published crates therefore use the
`otel-arrow-contrib-data-engine-*` prefix. The prefix communicates current
repository stewardship and distinguishes these experimental contributed crates
from OTAP Dataflow's core crates; it does not imply an architectural dependency
on OTAP or OpenTelemetry.

## Background

This work originated in a Phase 2 `otel-arrow` deliverable:

- **Prototype for DataFusion integration with OpenTelemetry data, OTTL-transform
  feasibility study**

This folder contains work in progress to implement a 'query engine' that can:

- Take in instructions in multiple common transform languages
- Produce an intermediate language abstraction from those instructions
- Execute requested manipulations on the data

That exploration produced both this independent data-engine workspace and the
OTAP-specific consumer linked above.

## Folder structure

|Name                        |Description                                                                                        |
|----------------------------|---------------------------------------------------------------------------------------------------|
|expressions                 |Intermediate language and syntax tree for the query engine                                         |
|kql-parser                  |Parser to turn KQL queries into query engine expressions (syntax trees)                            |
|ottl-parser                 |Parser to turn OTTL queries into query engine expressions (syntax trees)                           |
|parser-abstractions         |Common parser components and implementations for common literals                                   |
|engine-recordset            |Query engine implementation which takes a syntax tree and runs over a set of records (hierarchical)|
|engine-recordset-otlp-bridge|A bridge for running the recordset engine over Protobuf encoded blobs of OTLP data                 |

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
