# OTAP Parquet Query Examples

_Note: this is a temporary place for these examples. We'd like these examples
to eventually live in the otap-dataflow crate, but the current latest version
of datafusion is using an older version of arrow. We'll move these once the
version of arrow in datafusion is updated.
(see [datafusion/#16799](https://github.com/apache/datafusion/issues/16799))_

Pre-requisite, create some OTAP parquet data using the parquet exporter. For
example, run the OTAP Dataflow pipeline using the example config:

```sh
# run in the otap-dataflow directory
cargo run --bin df_engine -- --pipeline configs/fake-parquet.yaml
```

Run example:

```sh
cargo run --package parquet-query-examples --example query_logs
```
