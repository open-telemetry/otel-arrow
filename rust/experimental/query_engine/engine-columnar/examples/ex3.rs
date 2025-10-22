use std::sync::Arc;

use arrow::array::{RecordBatch, StringArray, StructArray, UInt8Array};
use arrow::datatypes::{DataType, Field, Fields, Schema};
use arrow::util::pretty::print_batches;
use datafusion::catalog::MemTable;
use datafusion::common::JoinType;
use datafusion::prelude::{SessionContext, col, lit, when};

#[tokio::main]
async fn main() {
    let struct_fields = Fields::from(vec![Field::new("a", DataType::UInt8, false)]);
    let schema1 = Arc::new(Schema::new(vec![
        Field::new("id", DataType::UInt8, false),
        Field::new("name", DataType::Utf8, false),
        Field::new("s", DataType::Struct(struct_fields.clone()), false),
    ]));

    let rb1 = RecordBatch::try_new(
        schema1.clone(),
        vec![
            Arc::new(UInt8Array::from_iter_values(vec![0, 1, 2, 3])),
            Arc::new(StringArray::from_iter_values(vec!["a", "b", "c", "d"])),
            Arc::new(StructArray::new(
                struct_fields.clone(),
                vec![Arc::new(UInt8Array::from_iter_values([0, 1, 2, 3]))],
                None,
            )),
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

    let query = "select * from table1 left semi join table2 on table1.s.a == table2.parent_id";
    let batches = ctx
        .sql(query.clone())
        .await
        .unwrap()
        .collect()
        .await
        .unwrap();

    print_batches(&batches).unwrap();

    let df = ctx.sql(query.clone()).await.unwrap();
    let lp = df.logical_plan();
    println!("logical plan:\n{}", lp);
    println!("logical plan:\n{:#?}", lp);
}
