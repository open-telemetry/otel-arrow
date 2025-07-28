use core::fmt::Debug;

use crate::{data_engine::DataEngineState, error::Error};

pub trait DataRecordBatch<T>: Debug {
    fn drain<S: DataEngineState, F>(&mut self, state: &mut S, action: F) -> Result<(), Error>
    where
        F: Fn(&mut S, T) -> Result<(), Error>;
}
