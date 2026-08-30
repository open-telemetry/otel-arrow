// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use bytes::Bytes;
use otel_arrow_dfe_pdata::OtapArrowRecords;
use otel_arrow_dfe_pdata::otlp::ProtoBuffer;

use crate::CodecError;

/// Reusable encoder configured once for one pipeline output plan.
pub trait PdataEncoder: Send {
    /// Encodes native records into an independently decodable owned batch.
    fn encode(&mut self, records: OtapArrowRecords) -> Result<Bytes, CodecError>;

    /// Prepares output that may borrow reusable encoder storage.
    fn prepare_encode<'a>(
        &'a mut self,
        records: &mut OtapArrowRecords,
    ) -> Result<EncodeOutput<'a>, CodecError> {
        self.encode(records.clone()).map(EncodeOutput::bytes)
    }
}

enum OutputStorage<'a> {
    Bytes(Bytes),
    Buffer(BufferOutput<'a>),
}

struct BufferOutput<'a> {
    buffer: &'a mut ProtoBuffer,
    max_retained_capacity: usize,
}

impl Drop for BufferOutput<'_> {
    fn drop(&mut self) {
        self.buffer.retain_capacity(self.max_retained_capacity);
    }
}

/// Prepared encoded output backed by owned bytes or reusable scratch storage.
pub struct EncodeOutput<'a>(OutputStorage<'a>);

impl<'a> EncodeOutput<'a> {
    /// Wraps independently owned bytes.
    #[must_use]
    pub fn bytes(bytes: Bytes) -> Self {
        Self(OutputStorage::Bytes(bytes))
    }

    /// Borrows reusable codec storage for synchronous consumption.
    #[must_use]
    pub fn buffer(buffer: &'a mut ProtoBuffer, max_retained_capacity: usize) -> Self {
        Self(OutputStorage::Buffer(BufferOutput {
            buffer,
            max_retained_capacity,
        }))
    }

    /// Detaches bytes for an asynchronous send without copying their contents.
    #[must_use]
    pub fn into_bytes(self) -> Bytes {
        match self.0 {
            OutputStorage::Bytes(bytes) => bytes,
            OutputStorage::Buffer(output) => {
                let (bytes, capacity) = output.buffer.take_into_bytes();
                output
                    .buffer
                    .ensure_capacity(capacity.min(output.max_retained_capacity));
                bytes
            }
        }
    }

    /// Keeps scratch attached and copies only when output uses that scratch.
    #[must_use]
    pub fn copy_into_bytes(self) -> Bytes {
        match self.0 {
            OutputStorage::Bytes(bytes) => bytes,
            OutputStorage::Buffer(output) => Bytes::copy_from_slice(output.buffer.as_ref()),
        }
    }
}

impl AsRef<[u8]> for EncodeOutput<'_> {
    fn as_ref(&self) -> &[u8] {
        match &self.0 {
            OutputStorage::Bytes(bytes) => bytes,
            OutputStorage::Buffer(output) => output.buffer.as_ref(),
        }
    }
}
