// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! gRPC response frame encoder with optional compression.

use crate::compression::CompressionMethod;
use crate::otel_receiver::grpc::{GrpcEncoding, MIN_COMPRESSED_CAPACITY};
use crate::otel_receiver::status::Status;
use bytes::{BufMut, Bytes, BytesMut};
use flate2::Compression;
use flate2::write::{GzEncoder, ZlibEncoder};
use prost::Message;
use std::cell::RefCell;
use std::io::{self, Write};
use std::mem;
use std::ops::{Deref, DerefMut};
use zstd::bulk::Compressor as ZstdCompressor;

#[cfg(feature = "unsafe-optimizations")]
#[inline]
fn set_vec_len(buf: &mut Vec<u8>, len: usize) {
    // SAFETY: caller guarantees capacity and that bytes are overwritten before read.
    #[allow(unsafe_code)]
    unsafe {
        buf.set_len(len)
    }
}

#[cfg(not(feature = "unsafe-optimizations"))]
#[inline]
fn set_vec_len(buf: &mut Vec<u8>, len: usize) {
    if buf.len() != len {
        buf.resize(len, 0);
    }
}

/// Per encoding pool of reusable gRPC response encoders.
///
/// Encoders are stored in small vectors keyed by `GrpcEncoding`. This avoids
/// repeatedly allocating compression buffers on the response hot path.
pub(crate) struct ResponseEncoderPool {
    inner: RefCell<EncoderSlots>,
}

/// RAII guard that returns a checked out encoder back to the pool on drop.
pub(crate) struct EncoderGuard<'a> {
    encoder: Option<GrpcResponseFrameEncoder>,
    pool: &'a ResponseEncoderPool,
    encoding: GrpcEncoding,
}

impl ResponseEncoderPool {
    pub(crate) fn new(methods: &[CompressionMethod], target_encoders: usize) -> Self {
        let pool_size = target_encoders.max(1);
        let mut slots = EncoderSlots {
            identity: Vec::with_capacity(pool_size),
            zstd: Vec::new(),
            gzip: Vec::new(),
            deflate: Vec::new(),
        };

        // Always seed identity encoder(s) since it is universally supported.
        for _ in 0..pool_size {
            slots
                .identity
                .push(GrpcResponseFrameEncoder::new(GrpcEncoding::Identity));
        }

        for method in methods {
            let (vec, encoding) = match method {
                CompressionMethod::Zstd => (&mut slots.zstd, GrpcEncoding::Zstd),
                CompressionMethod::Gzip => (&mut slots.gzip, GrpcEncoding::Gzip),
                CompressionMethod::Deflate => (&mut slots.deflate, GrpcEncoding::Deflate),
            };
            if vec.is_empty() {
                vec.reserve(pool_size);
            }
            for _ in vec.len()..pool_size {
                vec.push(GrpcResponseFrameEncoder::new(encoding));
            }
        }
        Self {
            inner: RefCell::new(slots),
        }
    }

    pub(crate) fn checkout(&self, encoding: GrpcEncoding) -> EncoderGuard<'_> {
        let mut slots = self.inner.borrow_mut();
        let encoder = match encoding {
            GrpcEncoding::Identity => slots.identity.pop(),
            GrpcEncoding::Zstd => slots.zstd.pop(),
            GrpcEncoding::Gzip => slots.gzip.pop(),
            GrpcEncoding::Deflate => slots.deflate.pop(),
        }
        .unwrap_or_else(|| GrpcResponseFrameEncoder::new(encoding));

        EncoderGuard {
            encoder: Some(encoder),
            pool: self,
            encoding,
        }
    }
}

impl<'a> Drop for EncoderGuard<'a> {
    fn drop(&mut self) {
        if let Some(encoder) = self.encoder.take() {
            let mut slots = self.pool.inner.borrow_mut();
            match self.encoding {
                GrpcEncoding::Identity => slots.identity.push(encoder),
                GrpcEncoding::Zstd => slots.zstd.push(encoder),
                GrpcEncoding::Gzip => slots.gzip.push(encoder),
                GrpcEncoding::Deflate => slots.deflate.push(encoder),
            }
        }
    }
}

impl<'a> Deref for EncoderGuard<'a> {
    type Target = GrpcResponseFrameEncoder;

    fn deref(&self) -> &Self::Target {
        self.encoder.as_ref().expect("encoder should be present")
    }
}

impl<'a> DerefMut for EncoderGuard<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.encoder.as_mut().expect("encoder should be present")
    }
}

pub(crate) struct EncoderSlots {
    identity: Vec<GrpcResponseFrameEncoder>,
    zstd: Vec<GrpcResponseFrameEncoder>,
    gzip: Vec<GrpcResponseFrameEncoder>,
    deflate: Vec<GrpcResponseFrameEncoder>,
}

/// Builds length prefixed gRPC response frames with optional compression.
pub(crate) struct GrpcResponseFrameEncoder {
    compression: GrpcEncoding,
    // Reusable buffer containing the result of the serialization of the response message.
    message_buf: BytesMut,
    // Reusable buffer for building the gRPC frame header + payload.
    frame_buf: BytesMut,
    compressed_buf: Vec<u8>,
    zstd: Option<ZstdCompressor<'static>>,
}

impl GrpcResponseFrameEncoder {
    pub(crate) fn new(compression: GrpcEncoding) -> Self {
        Self {
            compression,
            frame_buf: BytesMut::with_capacity(512),
            message_buf: BytesMut::with_capacity(512),
            compressed_buf: Vec::new(), // By default gRPC responses are uncompressed
            zstd: None,
        }
    }

    /// Serializes a protobuf message and wraps it in a gRPC frame.
    pub(crate) fn encode<M: Message>(&mut self, message: &M) -> Result<Bytes, Status> {
        // Serialize the message into the reusable buffer.
        self.message_buf.clear();
        message
            .encode(&mut self.message_buf)
            .map_err(|e| Status::internal(format!("failed to encode response: {e}")))?;
        let uncompressed = self.message_buf.split().freeze();

        // Compress & frame according to the negotiated encoding.
        match self.compression {
            GrpcEncoding::Identity => self.finish_frame(false, uncompressed.as_ref()),
            GrpcEncoding::Zstd => {
                self.compress_zstd(uncompressed.as_ref())?;
                let mut payload = mem::take(&mut self.compressed_buf);
                let result = self.finish_frame(true, payload.as_slice());
                payload.clear();
                self.compressed_buf = payload;
                result
            }
            GrpcEncoding::Gzip => {
                self.compress_gzip(uncompressed.as_ref())?;
                let mut payload = mem::take(&mut self.compressed_buf);
                let result = self.finish_frame(true, payload.as_slice());
                payload.clear();
                self.compressed_buf = payload;
                result
            }
            GrpcEncoding::Deflate => {
                self.compress_deflate(uncompressed.as_ref())?;
                let mut payload = mem::take(&mut self.compressed_buf);
                let result = self.finish_frame(true, payload.as_slice());
                payload.clear();
                self.compressed_buf = payload;
                result
            }
        }
    }

    /// Builds the 5-byte gRPC frame header plus payload.
    fn finish_frame(&mut self, compressed: bool, payload: &[u8]) -> Result<Bytes, Status> {
        let needed = 5 + payload.len();
        if self.frame_buf.capacity() < needed {
            self.frame_buf.reserve(needed - self.frame_buf.capacity());
        }
        self.frame_buf.clear();
        self.frame_buf.put_u8(u8::from(compressed));
        self.frame_buf.put_u32(payload.len() as u32);
        self.frame_buf.extend_from_slice(payload);
        Ok(self.frame_buf.split().freeze())
    }

    /// Performs zstd compression into `compressed_buf`, growing as needed.
    fn compress_zstd(&mut self, payload: &[u8]) -> Result<(), Status> {
        self.ensure_zstd_encoder()?;
        let mut required_capacity = payload.len().max(MIN_COMPRESSED_CAPACITY);
        loop {
            // Make sure the scratch buffer is large enough for the next attempt.
            if self.compressed_buf.capacity() < required_capacity {
                self.compressed_buf
                    .reserve(required_capacity - self.compressed_buf.capacity());
            }
            set_vec_len(&mut self.compressed_buf, required_capacity);
            let result = {
                // Safe because `ensure_zstd_encoder` guarantees we have an encoder.
                let encoder = self.zstd.as_mut().expect("zstd encoder must exist");
                // Compress directly into the reusable scratch buffer to avoid extra allocations.
                encoder.compress_to_buffer(payload, self.compressed_buf.as_mut_slice())
            };
            match result {
                Ok(written) => {
                    // Shrink to the actual size once compression finishes successfully.
                    self.compressed_buf.truncate(written);
                    return Ok(());
                }
                Err(err)
                    if err.kind() == io::ErrorKind::Other
                        && err.to_string().contains("Destination buffer is too small") =>
                {
                    // Double the capacity and retry when the destination buffer was insufficient.
                    required_capacity = required_capacity.checked_mul(2).ok_or_else(|| {
                        Status::internal("zstd compression failed: output too large")
                    })?;
                }
                Err(err) => {
                    // Any other compression failure aborts this response frame.
                    return Err(Status::internal(format!("zstd compression failed: {err}")));
                }
            }
        }
    }

    /// Compresses with gzip into the scratch buffer.
    fn compress_gzip(&mut self, payload: &[u8]) -> Result<(), Status> {
        self.compressed_buf.clear();
        {
            let mut encoder = GzEncoder::new(&mut self.compressed_buf, Compression::default());
            encoder
                .write_all(payload)
                .and_then(|_| encoder.try_finish())
                .map_err(|err| Status::internal(format!("gzip compression failed: {err}")))?;
        }
        Ok(())
    }

    /// Compresses with deflate into the scratch buffer.
    fn compress_deflate(&mut self, payload: &[u8]) -> Result<(), Status> {
        self.compressed_buf.clear();
        {
            let mut encoder = ZlibEncoder::new(&mut self.compressed_buf, Compression::default());
            encoder
                .write_all(payload)
                .and_then(|_| encoder.try_finish())
                .map_err(|err| Status::internal(format!("deflate compression failed: {err}")))?;
        }
        Ok(())
    }

    /// Lazily creates the zstd encoder.
    fn ensure_zstd_encoder(&mut self) -> Result<(), Status> {
        if self.zstd.is_some() {
            return Ok(());
        }
        match ZstdCompressor::new(0) {
            Ok(encoder) => {
                self.zstd = Some(encoder);
                Ok(())
            }
            Err(err) => Err(Status::internal(format!(
                "failed to initialize zstd compressor: {err}"
            ))),
        }
    }
}
