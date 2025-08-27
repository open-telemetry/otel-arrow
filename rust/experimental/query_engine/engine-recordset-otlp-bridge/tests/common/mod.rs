// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use data_engine_expressions::*;
use data_engine_recordset::*;

pub(crate) fn process_records<'a, 'b, 'c, TRecords, TRecord>(
    pipeline: &'a PipelineExpression,
    engine: &'b RecordSetEngine,
    records: &mut TRecords,
) -> RecordSetEngineResults<'a, 'c, TRecord>
where
    'a: 'c,
    'b: 'c,
    TRecords: RecordSet<TRecord>,
    TRecord: Record + 'static,
{
    println!("Request:");
    println!("{records:?}");

    println!("Pipeline:");
    println!("{pipeline:?}");

    let mut batch = engine.begin_batch(pipeline).unwrap();

    let dropped_records = batch.push_records(records);

    let mut final_results = batch.flush();

    for record in dropped_records.into_iter() {
        final_results.dropped_records.push(record);
    }

    println!("Initialization:");
    if final_results.diagnostics.is_empty() {
        println!("None")
    } else {
        println!("{final_results}");
    }

    println!("Included summaries:");
    if final_results.summaries.included_summaries.is_empty() {
        println!("None")
    } else {
        for included_summary in &final_results.summaries.included_summaries {
            let diagnostics = included_summary.to_string();
            if !diagnostics.is_empty() {
                println!("{included_summary}");
            }
            println!(
                "Id: {}, GroupBy: {:?}, Aggregation: {:?}, Map: {:?}",
                included_summary.summary_id,
                included_summary.group_by_values,
                included_summary.aggregation_values,
                included_summary.map
            );
        }
    }

    println!("Dropped summaries:");
    if final_results.summaries.dropped_summaries.is_empty() {
        println!("None")
    } else {
        for dropped_summary in &final_results.summaries.dropped_summaries {
            let diagnostics = dropped_summary.to_string();
            if !diagnostics.is_empty() {
                println!("{dropped_summary}");
            }
            println!(
                "Id: {}, GroupBy: {:?}, Aggregation: {:?}, Map: {:?}",
                dropped_summary.summary_id,
                dropped_summary.group_by_values,
                dropped_summary.aggregation_values,
                dropped_summary.map
            );
        }
    }

    println!("Included records:");
    if final_results.included_records.is_empty() {
        println!("None")
    } else {
        for included_record in &final_results.included_records {
            println!("{included_record}");
            println!("{:?}", included_record.get_record());
        }
    }

    println!("Dropped records:");
    if final_results.dropped_records.is_empty() {
        println!("None")
    } else {
        for dropped_record in &final_results.dropped_records {
            println!("{dropped_record}");
            println!("{:?}", dropped_record.get_record());
        }
    }

    final_results
}
