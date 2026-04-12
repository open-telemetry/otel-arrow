use arrow::array::{RecordBatch, BooleanArray};
use arrow::row::{RowConverter, SortField};
use arrow::compute::filter_record_batch;
use std::collections::HashSet;

pub fn deduplicate_attributes(rb: &RecordBatch) -> Result<RecordBatch, String> {
    if rb.num_rows() <= 1 {
        return Ok(rb.clone());
    }

    let parent_id_arr = rb.column(0);
    let key_arr = rb.column(1);
    
    let mut row_converter = RowConverter::new(vec![
        SortField::new(parent_id_arr.data_type().clone()),
        SortField::new(key_arr.data_type().clone()),
    ]).map_err(|e| e.to_string())?;
    
    let rows = row_converter.convert_columns(&[parent_id_arr.clone(), key_arr.clone()])
        .map_err(|e| e.to_string())?;
    
    let mut seen = HashSet::with_capacity(rb.num_rows());
    let mut filter_vec = vec![false; rb.num_rows()];
    
    for i in (0..rb.num_rows()).rev() {
        let row_bytes = rows.row(i).as_ref().to_vec();
        if seen.insert(row_bytes) {
            filter_vec[i] = true;
        }
    }
    
    let filter_array = BooleanArray::from(filter_vec);
    filter_record_batch(rb, &filter_array).map_err(|e| e.to_string())
}
