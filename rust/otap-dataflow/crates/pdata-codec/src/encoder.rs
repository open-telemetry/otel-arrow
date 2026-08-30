// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Encoding extension contract and prepared-output ownership model.
//!
//! Encoders are pipeline-local, reusable instances. Simple implementations can
//! return owned [`Bytes`] from [`PdataEncoder::encode`]. Implementations with a
//! scratch buffer can override [`PdataEncoder::prepare_encode`] and let a
//! synchronous consumer inspect that buffer before deciding whether an
//! asynchronous send requires detached owned bytes.

use bytes::Bytes;
use otel_arrow_dfe_pdata::OtapArrowRecords;
use otel_arrow_dfe_pdata::otlp::ProtoBuffer;

use crate::CodecError;

/// Converts native OTAP Arrow records into complete encoded batches.
///
/// Instances are created lazily from a startup-resolved encoding plan and then
/// reused within one pipeline runtime. Every successful result must be
/// independently decodable; stream-relative dictionaries or state are outside
/// the pdata codec contract.
///
/// Implementations that only produce owned bytes need to implement [`Self::encode`].
/// Implementations with reusable storage should also override
/// [`Self::prepare_encode`] to avoid an intermediate allocation on synchronous
/// output paths.
pub trait PdataEncoder: Send {
    /// Encodes native records into an independently decodable owned batch.
    ///
    /// Ownership of `records` permits destructive or move-based encoders. The
    /// returned bytes can cross an async suspension without further detachment.
    fn encode(&mut self, records: OtapArrowRecords) -> Result<Bytes, CodecError>;

    /// Prepares output that may borrow reusable encoder storage.
    ///
    /// The default clones the records and delegates to [`Self::encode`]. A
    /// scratch-buffer implementation can return [`EncodeOutput::buffer`]. The
    /// output must be consumed or detached before another mutable codec access.
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
///
/// Use [`AsRef`] while a synchronous transport can consume the output before
/// returning control. Use [`Self::into_bytes`] before an asynchronous send; it
/// detaches the buffer contents so no borrow of codec state crosses `.await`.
/// [`Self::copy_into_bytes`] keeps reusable storage attached at the cost of a
/// copy when the output is scratch-backed.
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
