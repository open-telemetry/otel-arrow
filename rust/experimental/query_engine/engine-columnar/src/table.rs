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

#[derive(Debug)]
pub(crate) struct RecordBatchPartitionStream {
    schema: SchemaRef,
    curr_batch: Mutex<RecordBatch>,
}

impl RecordBatchPartitionStream {
    pub fn new(batch: RecordBatch) -> Self {
        let schema = batch.schema();

        Self {
            schema,
            curr_batch: Mutex::new(batch),
        }
    }

    pub fn update_batch(&self, batch: RecordBatch) {
        let mut guard = self.curr_batch.lock();
        *guard = batch
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

pub struct OneShotRecordBatchStream {
    schema: SchemaRef,
    record_batch: Option<RecordBatch>,
}

impl OneShotRecordBatchStream {
    fn new(schema: SchemaRef, record_batch: RecordBatch) -> Self {
        Self {
            schema,
            record_batch: Some(record_batch),
        }
    }
}

impl Stream for OneShotRecordBatchStream {
    type Item = datafusion::error::Result<RecordBatch>;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Ready(self.get_mut().record_batch.take().map(Ok))
    }
}

impl RecordBatchStream for OneShotRecordBatchStream {
    fn schema(&self) -> SchemaRef {
        self.schema.clone()
    }
}
