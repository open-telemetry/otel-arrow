use arrow::{
    array::{RecordBatch, StringArray, UInt8Array, UInt16Array},
    datatypes::{DataType, Field, Schema},
};
use datafusion::{
    catalog::{
        MemTable,
        memory::{DataSourceExec, MemorySourceConfig},
    },
    error::DataFusionError,
    physical_optimizer::PhysicalOptimizerRule,
    prelude::*,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), DataFusionError> {
    let left_batch1 = RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new("id", DataType::UInt8, false),
            Field::new("a", DataType::Utf8, false),
        ])),
        vec![
            Arc::new(UInt8Array::from_iter_values([0, 1, 2, 3])),
            Arc::new(StringArray::from_iter_values(["a", "b", "c", "d"])),
        ],
    )
    .unwrap();

    let right_batch = RecordBatch::try_new(
        Arc::new(Schema::new(vec![Field::new(
            "parent_id",
            DataType::UInt8,
            false,
        )])),
        vec![Arc::new(UInt8Array::from_iter_values([0, 1]))],
    )
    .unwrap();

    let ctx = SessionContext::new();
    let left_table = MemTable::try_new(left_batch1.schema(), vec![vec![left_batch1]]).unwrap();
    let right_table = MemTable::try_new(right_batch.schema(), vec![vec![right_batch]]).unwrap();
    ctx.register_table("tab_l", Arc::new(left_table))?;
    ctx.register_table("tab_r", Arc::new(right_table))?;

    let df = ctx.table("tab_l").await?.join(
        ctx.table("tab_r").await?,
        JoinType::LeftSemi,
        &["id"],
        &["parent_id"],
        None,
    )?;

    let state = ctx.state();
    let logical_plan = state.optimize(df.logical_plan())?;
    let physical_plan = state.create_physical_plan(&logical_plan).await?;

    Ok(())
}

#[derive(Debug)]
struct AlbertOptimizer {}

impl PhysicalOptimizerRule for AlbertOptimizer {
    fn name(&self) -> &str {
        "albert"
    }

    fn schema_check(&self) -> bool {
        false
    }

    fn optimize(
        &self,
        plan: Arc<dyn datafusion::physical_plan::ExecutionPlan>,
        config: &datafusion::config::ConfigOptions,
    ) -> datafusion::error::Result<Arc<dyn datafusion::physical_plan::ExecutionPlan>> {
        todo!()
        // if let Some(ds) = plan.as_any().downcast_ref::<DataSourceExec>() {
        //     if let Some(mem) = ds.data_source().as_any().downcast_ref::<MemorySourceConfig>() {
        //         let schema = mem.original_schema();
        //         let is_left = schema.fields.len() == 2;
        //         let next_batch = if is_left {
        //             RecordBatch::try_new(
        //                 Arc::new(Schema::new(vec![
        //                     Field::new("id", DataType::UInt8, false),
        //                     Field::new("b", DataType::UInt16, false)
        //                 ])),
        //                 vec![
        //                     Arc::new(UInt8Array::from_iter_values([5, 6, 7])),
        //                     Arc::new(UInt16Array::from_iter_values([5, 1, 4])),
        //                 ]
        //             ).unwrap()
        //         } else {
        //             RecordBatch::try_new(
        //                 Arc::new(Schema::new(vec![
        //                     Field::new("parent_id", DataType::UInt8, false),
        //                 ])),
        //                 vec![
        //                     Arc::new(UInt8Array::from_iter_values([0, 1])),
        //                 ]
        //             ).unwrap()
        //         };

        //         // let next_pla
        //     }
        // }
    }
}
