// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use arrow::compute::concat_batches;
use arrow::datatypes::DataType;
use arrow::util::pretty::print_batches;
use datafusion::execution::context::SessionContext;
use datafusion::prelude::ParquetReadOptions;

const FILTER_LOGS_BY_ATTRS_EXAMPLE: &str = "
    SELECT
        (logs._part_id || '_' || CAST(logs.id as STRING)) as unique_log_id,
        logs.time_unix_nano, 
        logs.body.str,
        logs.severity_number,
        logs.severity_text,
        log_attrs.key as attr_key,
        log_attrs.str as attr_str
    FROM 
        logs 
        INNER JOIN log_attrs
            ON logs.id = log_attrs.parent_id
            AND logs._part_id  = log_attrs._part_id
    WHERE 
        log_attrs.key = 'server.address'
        AND log_attrs.str = 'example.com'
    LIMIT 100
";

const GROUP_LOGS_BY_ATTRS_AND_TIME: &str = "
    SELECT
         date_trunc('minute', logs.time_unix_nano) as time_unix_nano_bucket,
         log_attrs.str as app_widget_id,
         count(*)
     FROM
        logs
        INNER JOIN log_attrs
            ON logs.id = log_attrs.parent_id
            AND logs._part_id  = log_attrs._part_id
    WHERE 
        log_attrs.key = 'app.widget.id'
     GROUP BY log_attrs.str, date_trunc('minute', logs.time_unix_nano)
     ORDER BY date_trunc('minute', logs.time_unix_nano) ASC
     LIMIT 100
";

#[tokio::main]
async fn main() {
    let ctx = SessionContext::new();

    let parquet_reader_opts =
        ParquetReadOptions::new().table_partition_cols(vec![("_part_id".into(), DataType::Utf8)]);

    ctx.register_parquet("logs", "/tmp/logs", parquet_reader_opts.clone())
        .await
        .unwrap();
    ctx.register_parquet("log_attrs", "/tmp/log_attrs", parquet_reader_opts.clone())
        .await
        .unwrap();

    let example_queries = [
        ("filter logs by attributes", FILTER_LOGS_BY_ATTRS_EXAMPLE),
        ("count logs by attr/time", GROUP_LOGS_BY_ATTRS_AND_TIME),
    ];

    for (example_name, query) in example_queries {
        let df = ctx.sql(query).await.unwrap();

        println!("\nexecuting example: {example_name}");
        let batches = df.collect().await.unwrap();
        println!("got {} batches ...", batches.len());

        // pretty print the result:
        if !batches.is_empty() {
            let schema = batches[0].schema();
            let result = concat_batches(&schema, batches.iter()).unwrap();
            print_batches(&[result]).unwrap();
        }
    }
}
