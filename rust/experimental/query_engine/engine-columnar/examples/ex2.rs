use std::sync::Arc;

use arrow::array::{RecordBatch, StringArray, UInt8Array};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::util::pretty::print_batches;
use datafusion::catalog::MemTable;
use datafusion::common::JoinType;
use datafusion::prelude::{SessionContext, col, lit, when};

#[tokio::main]
async fn main() {
    let schema1 = Arc::new(Schema::new(vec![
        Field::new("id", DataType::UInt8, false),
        Field::new("name", DataType::Utf8, false),
    ]));

    let rb1 = RecordBatch::try_new(
        schema1.clone(),
        vec![
            Arc::new(UInt8Array::from_iter_values(vec![0, 1, 2, 3])),
            Arc::new(StringArray::from_iter_values(vec!["a", "b", "c", "d"])),
        ],
    )
    .unwrap();

    let schema2 = Arc::new(Schema::new(vec![
        Field::new("parent_id", DataType::UInt8, false),
        Field::new("key", DataType::Utf8, false),
        Field::new("val", DataType::Utf8, false),
    ]));

    let rb2 = RecordBatch::try_new(
        schema2.clone(),
        vec![
            Arc::new(UInt8Array::from_iter_values(vec![0, 1])),
            Arc::new(StringArray::from_iter_values(vec!["k1", "k2"])),
            Arc::new(StringArray::from_iter_values(vec!["v1", "v2"])),
        ],
    )
    .unwrap();

    let rb3 = RecordBatch::try_new(
        schema2.clone(),
        vec![
            Arc::new(UInt8Array::from_iter_values(vec![1, 2])),
            Arc::new(StringArray::from_iter_values(vec!["k1", "k2"])),
            Arc::new(StringArray::from_iter_values(vec!["v1", "v2"])),
        ],
    )
    .unwrap();

    let ctx = SessionContext::new();

    ctx.register_table(
        "table1",
        Arc::new(MemTable::try_new(schema1.clone(), vec![vec![rb1]]).unwrap()),
    )
    .unwrap();
    ctx.register_table(
        "table2",
        Arc::new(MemTable::try_new(schema2.clone(), vec![vec![rb2]]).unwrap()),
    )
    .unwrap();
    ctx.register_table(
        "table3",
        Arc::new(MemTable::try_new(schema2.clone(), vec![vec![rb3]]).unwrap()),
    )
    .unwrap();

    // Here is an example where dropping the `mark` column works as expected.
    // By way of a motivating example, imagine I am trying to do the following:
    // Add `ok = true` column where there's an associated record in table2
    let results = ctx
        .table("table1")
        .await
        .unwrap()
        .join(
            ctx.table("table2").await.unwrap(),
            JoinType::LeftMark,
            &["id"],
            &["parent_id"],
            None,
        )
        .unwrap()
        .with_column(
            "ok",
            when(col("mark").eq(lit(true)), lit(true)).end().unwrap(),
        )
        .unwrap()
        // this works as expected, drops the mark column
        .drop_columns(&["mark"])
        .unwrap()
        .collect()
        .await
        .unwrap();

    println!("result 1");
    print_batches(&results).unwrap();
    // prints:
    // result 1
    // +----+------+------+
    // | id | name | ok   |
    // +----+------+------+
    // | 0  | a    | true |
    // | 1  | b    | true |
    // | 2  | c    |      |
    // | 3  | d    |      |
    // +----+------+------+

    // Here is an example where I'm not able to drop the `mark` column. Say for example
    // that I'm trying to do the following:
    // Add `ok = true` column where there's an associated record in table2 and table3
    let results = ctx
        .table("table1")
        .await
        .unwrap()
        .join(
            ctx.table("table2").await.unwrap(),
            JoinType::LeftMark,
            &["id"],
            &["parent_id"],
            None,
        )
        .unwrap()
        .join(
            ctx.table("table3").await.unwrap(),
            JoinType::LeftMark,
            &["id"],
            &["parent_id"],
            None,
        )
        .unwrap()
        .with_column(
            "ok",
            when(
                col("table2.mark")
                    .eq(lit(true))
                    .and(col("table3.mark").eq(lit(true))),
                lit(true),
            )
            .end()
            .unwrap(),
        )
        .unwrap()
        // this doesn't work:
        // .drop_columns(&["mark"])
        // neither does this
        .drop_columns(&["table2.mark", "table3.mark"])
        .unwrap()
        .collect()
        .await
        .unwrap();

    println!("result 2");
    print_batches(&results).unwrap()
    // prints:
    // result 2
    // +----+------+-------+-------+------+
    // | id | name | mark  | mark  | ok   |
    // +----+------+-------+-------+------+
    // | 0  | a    | true  | false |      |
    // | 1  | b    | true  | true  | true |
    // | 2  | c    | false | true  |      |
    // | 3  | d    | false | false |      |
    // +----+------+-------+-------+------+
}
