# Query Abstraction

One of the mentioned key deliverables in Phase 2 of otel-arrow is:

- **Prototype for DataFusion integration with OpenTelemetry data, OTTL-transform
  feasibility study**

As part of the Transform effort, we want to understand how standard telemetry
data manipulations translate into DataFusion operations. While DataFusion
[supports SQL out of the
box](https://datafusion.apache.org/user-guide/features.html), we believe
customers may want the ability to plug-and-play their own data manipulation
language on top of DataFusion.

OpenTelemetry Collector users may already be aware of the [OpenTelemetry
Transformation Language (or
OTTL)](https://github.com/open-telemetry/opentelemetry-collector-contrib/tree/main/pkg/ottl)
which may be used in various `processors` to shape data in certain ways.
Exploratory work with DataFusion should support OTTL statements as an input to
corresponding DataFusion operations.

In order to make sure this work is generalizable we've chosen a another query
language, [Kusto Query Language (or
KQL)](https://learn.microsoft.com/kusto/query/?view=microsoft-fabric), to
support side by side.

Support for multiple query languages implies a shared Intermediate Language (IL)
or abstraction layer which may help keep DataFusion interaction relatively
language-agnostic.

## Intermediate Language Abstraction

The immediate exploration of a possible IL examines 3 different languages and a
plugin model to support other languages in the future.

While SQL is the native language of choice for DataFusion and OTTL is the
standard for OpenTelemetry, KQL is more vendor-specific and could eventually
belong in a currently non-existant `otel-arrow-contrib` repository as this
project develops.

| Language | DataFusion built-in Support | Abstraction Plugin Status |
|----------|-----------------------------|---------------------------|
| SQL      | :white_check_mark:          | not immediately in scope* |
| OTTL     | :x:                         | :construction:            |
| KQL      | :x:                         | :construction:            |

\* It is not clear if the scope of work should include translation of SQL into
an IL. It is highly possible we can re-use the native
[datafusion-sql-parser-rs](https://github.com/apache/datafusion-sqlparser-rs)
Parser and/or the [datafusion sql
module](https://github.com/apache/datafusion/tree/main/datafusion/sql) instead
to avoid re-implementing existing work.

To illustrate how these 3 languages may intersect in their data shaping,
consider the following examples of data filtering:

```sql
-- SQL filtering operation
SELECT * FROM MyTable
WHERE Foo = 'bar';
```

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
MyTable
| where Foo == "bar"
```

All of these operations try to accomplish the same goal. In DataFusion, this
operation may be represented as the following Rust code using
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
