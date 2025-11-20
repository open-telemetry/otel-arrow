// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use arrow::array::RecordBatch;
use arrow::datatypes::SchemaRef;
use datafusion::execution::{RecordBatchStream, SendableRecordBatchStream, TaskContext};
use datafusion::physical_plan::streaming::PartitionStream;
use futures_core::Stream;
use parking_lot::Mutex;

/// A partition stream that serves a single, updateable RecordBatch to DataFusion.
///
/// This allows external code to provide new data to a DataFusion ExecutionPlan between
/// executions. Each call to `execute()` snapshots the current batch.
///
/// This implements [`PartitionStream`] which means it can be used alongside
/// [`StreamingTable`](datafusion::catalog::streaming::StreamingTable).
#[derive(Debug)]
pub(crate) struct RecordBatchPartitionStream {
    schema: SchemaRef,
    curr_batch: Mutex<Option<RecordBatch>>,
}

impl RecordBatchPartitionStream {
    /// Create a new instance of [`RecordBatchPartitionStream`]
    // TODO remove allow(unused) eventually this is constructed during planning
    #[allow(unused)]
    pub fn new(schema: SchemaRef) -> Self {
        Self {
            schema,
            curr_batch: Mutex::new(None),
        }
    }

    /// Updates the batch that will be returned by future executions.
    ///
    /// This does not affect any streams that are currently executing - they'll continue to use the
    /// batch they captured when execute() was called.
    pub fn update_batch(&self, batch: RecordBatch) {
        let mut guard = self.curr_batch.lock();
        *guard = Some(batch)
    }
}

impl PartitionStream for RecordBatchPartitionStream {
    fn schema(&self) -> &SchemaRef {
        &self.schema
    }

    fn execute(&self, _ctx: Arc<TaskContext>) -> SendableRecordBatchStream {
        let rb = self.curr_batch.lock().clone();
        let stream = OneShotRecordBatchStream::new(self.schema.clone(), rb);
        Box::pin(stream)
    }
}

/// A stream that yields exactly one RecordBatch then completes.
///
/// Used to wrap a single batch for DataFusion's streaming execution model. The batch is moved out
/// on first poll, subsequent polls return None.
pub struct OneShotRecordBatchStream {
    schema: SchemaRef,
    record_batch: Option<RecordBatch>,
}

impl OneShotRecordBatchStream {
    fn new(schema: SchemaRef, record_batch: Option<RecordBatch>) -> Self {
        Self {
            schema,
            record_batch,
        }
    }
}

impl Stream for OneShotRecordBatchStream {
    type Item = datafusion::error::Result<RecordBatch>;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Take the batch on first poll, return None thereafter
        Poll::Ready(self.get_mut().record_batch.take().map(Ok))
    }
}

impl RecordBatchStream for OneShotRecordBatchStream {
    fn schema(&self) -> SchemaRef {
        self.schema.clone()
    }
}
