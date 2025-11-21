// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Low-level gRPC/H2 mechanics/helpers for the experimental OTAP receiver.
//!
//! This module takes care of:
//! - negotiating compression,
//! - decoding/encoding length-prefixed frames,
//! - managing request time limits,
//! - abstracting the underlying `RecvStream` so it can be fuzzed in tests.
//!
//! Note: The implementation is heavily inspired by tonic's framing and compression stack, but
//! tailored to the single-threaded OTAP runtime.

use crate::compression::CompressionMethod;
use crate::otel_receiver::status::Status;
use async_trait::async_trait;
use bytes::{Buf, Bytes, BytesMut};
use flate2::read::{GzDecoder, ZlibDecoder};
use futures::{Stream, StreamExt};
use http::{HeaderMap, HeaderValue};
use otap_df_pdata::proto::opentelemetry::arrow::v1::BatchArrowRecords;
use prost::Message;
use std::collections::VecDeque;
use std::future::Future;
use std::io::{self, Write};
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::time::{Instant as TokioInstant, Sleep, sleep};
use zstd::bulk::Decompressor;
use crate::otel_receiver::GrpcRequestRouter;

/// Floor for compressed buffer allocations to avoid tiny vec growth.
pub(crate) const MIN_COMPRESSED_CAPACITY: usize = 8 * 1024;

#[cfg(feature = "unsafe-optimizations")]
#[inline]
fn set_bytes_len(buf: &mut BytesMut, len: usize) {
    if buf.capacity() < len {
        buf.reserve(len - buf.capacity());
    }
    // SAFETY: caller ensures bytes are fully overwritten before use.
    #[allow(unsafe_code)]
    unsafe {
        buf.set_len(len)
    }
}

#[cfg(not(feature = "unsafe-optimizations"))]
#[inline]
fn set_bytes_len(buf: &mut BytesMut, len: usize) {
    if buf.capacity() < len {
        buf.reserve(len - buf.capacity());
    }
    buf.resize(len, 0);
}

/// Parses the client's `grpc-encoding` header and enforces server policy.
///
/// Note: Non-UTF8 headers, unknown tokens, or disabled algorithms all yield a gRPC `unimplemented`
/// status so clients get immediate, well-scoped errors instead of silently falling back.
pub(crate) fn parse_grpc_encoding(
    headers: &HeaderMap,
    accepted: &AcceptedGrpcEncodings,
) -> Result<GrpcEncoding, Status> {
    // The method first validates that the `content-type` header begins with `application/grpc`.
    // This keeps HTTP/2 requests that merely mimic the header names from being accepted.
    match headers.get(http::header::CONTENT_TYPE) {
        Some(value) if value.as_bytes().starts_with(b"application/grpc") => {}
        other => {
            log::error!("Rejecting stream due to invalid content-type: {other:?}");
            return Err(Status::invalid_argument(
                "missing application/grpc content-type",
            ));
        }
    }

    // Only encodings explicitly advertised in `AcceptedGrpcEncodings` are permitted, even though
    // the parser understands additional aliases (such as `zstdarrow{n}`), the request is rejected
    // unless the server opted in to the corresponding [`CompressionMethod`].
    match headers.get("grpc-encoding") {
        None => Ok(GrpcEncoding::Identity),
        Some(value) => {
            let raw = value.to_str().map_err(|_| {
                log::error!("Non-UTF8 grpc-encoding header");
                Status::invalid_argument("invalid grpc-encoding header")
            })?;
            let trimmed = raw.trim();
            let ascii = trimmed.as_bytes();
            const PREFIX: &[u8] = b"zstdarrow";

            let encoding = if ascii.is_empty() || eq_ascii_case_insensitive(ascii, b"identity") {
                GrpcEncoding::Identity
            } else if eq_ascii_case_insensitive(ascii, b"zstd") {
                GrpcEncoding::Zstd
            } else if eq_ascii_case_insensitive(ascii, b"gzip") {
                GrpcEncoding::Gzip
            } else if eq_ascii_case_insensitive(ascii, b"deflate") {
                GrpcEncoding::Deflate
            } else if ascii.len() >= PREFIX.len()
                && starts_with_ascii_case_insensitive(ascii, PREFIX)
            {
                let tail = &ascii[PREFIX.len()..];
                if tail.len() == 1 && tail[0].is_ascii_digit() {
                    GrpcEncoding::Zstd
                } else {
                    log::error!("Unsupported grpc-encoding {}", trimmed);
                    return Err(Status::unimplemented("grpc compression not supported"));
                }
            } else {
                log::error!("Unsupported grpc-encoding {}", trimmed);
                return Err(Status::unimplemented("grpc compression not supported"));
            };

            if accepted.allows(encoding) {
                Ok(encoding)
            } else {
                log::error!(
                    "grpc-encoding {} not enabled in server configuration",
                    trimmed
                );
                Err(Status::unimplemented("grpc compression not supported"))
            }
        }
    }
}

/// Returns true when two ASCII byte slices are equal ignoring case (without allocating or
/// converting to UTF-8).
fn eq_ascii_case_insensitive(value: &[u8], expected: &[u8]) -> bool {
    value.len() == expected.len()
        && value
            .iter()
            .zip(expected)
            .all(|(lhs, rhs)| ascii_byte_eq_ignore_case(*lhs, *rhs))
}

/// Returns true if `value` starts with `prefix`, ignoring ASCII case (without allocating or
/// converting to UTF-8).
fn starts_with_ascii_case_insensitive(value: &[u8], prefix: &[u8]) -> bool {
    value.len() >= prefix.len()
        && value
            .iter()
            .zip(prefix)
            .all(|(lhs, rhs)| ascii_byte_eq_ignore_case(*lhs, *rhs))
}

/// Compares two ASCII bytes without allocating or converting to UTF-8.
fn ascii_byte_eq_ignore_case(lhs: u8, rhs: u8) -> bool {
    lhs == rhs || lhs.eq_ignore_ascii_case(&rhs)
}

/// Parses the client's `grpc-accept-encoding` header into capability flags.
pub(crate) fn parse_grpc_accept_encoding(headers: &HeaderMap) -> ClientAcceptEncodings {
    let Some(value) = headers.get("grpc-accept-encoding") else {
        return ClientAcceptEncodings::identity_only();
    };
    let raw = match value.to_str() {
        Ok(raw) => raw,
        Err(_) => return ClientAcceptEncodings::identity_only(),
    };

    let mut encodings = ClientAcceptEncodings {
        identity: false,
        zstd: false,
        gzip: false,
        deflate: false,
    };
    let mut recognized = false;

    for token in raw.split(',') {
        let trimmed = token.trim();
        if trimmed.is_empty() {
            continue;
        }
        let ascii = trimmed.as_bytes();
        if eq_ascii_case_insensitive(ascii, b"identity") {
            encodings.identity = true;
            recognized = true;
        } else if eq_ascii_case_insensitive(ascii, b"zstd") {
            encodings.zstd = true;
            recognized = true;
        } else if eq_ascii_case_insensitive(ascii, b"gzip") {
            encodings.gzip = true;
            recognized = true;
        } else if eq_ascii_case_insensitive(ascii, b"deflate") {
            encodings.deflate = true;
            recognized = true;
        }
    }

    if recognized {
        encodings
    } else {
        ClientAcceptEncodings::identity_only()
    }
}

/// Chooses the response encoding based on server preference & client support.
///
/// The caller provides the ordered list of server-supported compression methods and the client's
/// advertised capabilities. The function walks the server list in order, returning the first method
/// that the client also supports. This gives the server deterministic control over preference
/// ordering (e.g. pick `zstd` when available, otherwise fall back to `gzip`, etc.) while still
/// honoring the client's declared limits. When no overlap exists the function returns
/// `GrpcEncoding::Identity`, signaling that the response must be sent uncompressed.
pub(crate) fn negotiate_response_encoding(
    configured: &[CompressionMethod],
    client: &ClientAcceptEncodings,
) -> GrpcEncoding {
    for method in configured {
        if client.supports(*method) {
            return match method {
                CompressionMethod::Zstd => GrpcEncoding::Zstd,
                CompressionMethod::Gzip => GrpcEncoding::Gzip,
                CompressionMethod::Deflate => GrpcEncoding::Deflate,
            };
        }
    }
    GrpcEncoding::Identity
}

#[derive(Clone, Copy, Debug)]
/// Supported compression algorithms for the OTAP receiver responses.
pub enum GrpcEncoding {
    /// No compression.
    Identity,
    /// Zstd compression.
    Zstd,
    /// Gzip compression.
    Gzip,
    /// Deflate compression.
    Deflate,
    // ToDo Add support for Snappy to follow Go implementation
    // ToDo Add support for OpenZL in the future
}

#[derive(Clone, Copy)]
/// Bit-mask indicating which compression methods the server allows.
pub(crate) struct AcceptedGrpcEncodings {
    /// Whether server allows zstd for requests.
    zstd: bool,
    /// Whether server allows gzip for requests.
    gzip: bool,
    /// Whether server allows deflate for requests.
    deflate: bool,
}

impl AcceptedGrpcEncodings {
    pub(crate) fn from_methods(methods: &[CompressionMethod]) -> Self {
        let mut encodings = Self {
            zstd: false,
            gzip: false,
            deflate: false,
        };

        for method in methods {
            match method {
                CompressionMethod::Zstd => encodings.zstd = true,
                CompressionMethod::Gzip => encodings.gzip = true,
                CompressionMethod::Deflate => encodings.deflate = true,
            }
        }

        encodings
    }

    fn allows(self, encoding: GrpcEncoding) -> bool {
        match encoding {
            GrpcEncoding::Identity => true,
            GrpcEncoding::Zstd => self.zstd,
            GrpcEncoding::Gzip => self.gzip,
            GrpcEncoding::Deflate => self.deflate,
        }
    }
}

#[derive(Clone, Copy)]
/// Parsed view of the client's `grpc-accept-encoding` preference list.
pub(crate) struct ClientAcceptEncodings {
    /// Client is willing to accept identity/not-compressed responses.
    pub(crate) identity: bool,
    /// Client advertised support for zstd responses.
    pub(crate) zstd: bool,
    /// Client advertised support for gzip responses.
    pub(crate) gzip: bool,
    /// Client advertised support for deflate responses.
    pub(crate) deflate: bool,
}

impl ClientAcceptEncodings {
    fn identity_only() -> Self {
        Self {
            identity: true,
            zstd: false,
            gzip: false,
            deflate: false,
        }
    }

    fn supports(self, method: CompressionMethod) -> bool {
        match method {
            CompressionMethod::Zstd => self.zstd,
            CompressionMethod::Gzip => self.gzip,
            CompressionMethod::Deflate => self.deflate,
        }
    }
}

/// Produces the `grpc-accept-encoding` header to advertise server support.
pub(crate) fn build_accept_encoding_header(methods: &[CompressionMethod]) -> HeaderValue {
    let mut tokens = Vec::with_capacity(methods.len() + 1);
    for method in methods {
        tokens.push(compression_method_token(*method));
    }
    // `identity` is always supported but least preferred.
    tokens.push("identity");
    let joined = tokens.join(",");
    HeaderValue::from_str(&joined).unwrap_or_else(|_| HeaderValue::from_static("identity"))
}

fn compression_method_token(method: CompressionMethod) -> &'static str {
    match method {
        CompressionMethod::Zstd => "zstd",
        CompressionMethod::Gzip => "gzip",
        CompressionMethod::Deflate => "deflate",
    }
}

pub(crate) fn grpc_encoding_token(encoding: GrpcEncoding) -> Option<&'static str> {
    match encoding {
        GrpcEncoding::Identity => None,
        GrpcEncoding::Zstd => Some("zstd"),
        GrpcEncoding::Gzip => Some("gzip"),
        GrpcEncoding::Deflate => Some("deflate"),
    }
}

pub(crate) type BodyStreamError = String;

/// Abstraction over the inbound h2 data stream so tests can inject fakes.
#[async_trait(?Send)]
pub(crate) trait BodyStream {
    async fn next_chunk(&mut self) -> Option<Result<Bytes, BodyStreamError>>;
    fn release_capacity(&mut self, released: usize) -> Result<(), BodyStreamError>;
}

pub(crate) struct H2BodyStream {
    inner: h2::RecvStream,
}

impl H2BodyStream {
    pub(crate) fn new(inner: h2::RecvStream) -> Self {
        Self { inner }
    }
}

#[async_trait(?Send)]
impl BodyStream for H2BodyStream {
    /// Pulls the next DATA frame chunk from h2.
    async fn next_chunk(&mut self) -> Option<Result<Bytes, BodyStreamError>> {
        self.inner
            .data()
            .await
            .map(|res| res.map_err(|err| err.to_string()))
    }

    /// Returns flow-control credits back to the peer.
    fn release_capacity(&mut self, released: usize) -> Result<(), BodyStreamError> {
        self.inner
            .flow_control()
            .release_capacity(released)
            .map_err(|err| err.to_string())
    }
}

/// Pull-based view over an h2 stream that yields decoded `BatchArrowRecords`.
pub(crate) struct GrpcStreamingBody<S = H2BodyStream> {
    recv: S,
    buffer: ChunkBuffer,
    current_frame: Option<FrameHeader>,
    finished: bool,
    encoding: GrpcEncoding,
    /// Shared reference to router so we can reach the pooled decompressor
    router: Rc<GrpcRequestRouter>,
    decompressed_buf: BytesMut,
}

#[derive(Clone, Copy)]
/// Metadata describing the next gRPC frame.
struct FrameHeader {
    length: usize,
    compressed: bool,
}

/// Simple queue of received bytes backing the gRPC frame decoder.
struct ChunkBuffer {
    chunks: VecDeque<Bytes>,
    len: usize,
}

impl ChunkBuffer {
    /// Creates an empty buffer that tracks total buffered length.
    fn new() -> Self {
        Self {
            chunks: VecDeque::new(),
            len: 0,
        }
    }

    /// Returns the number of bytes buffered across all chunks.
    fn len(&self) -> usize {
        self.len
    }

    /// Appends a chunk to the tail of the buffer without copying.
    fn push(&mut self, chunk: Bytes) {
        if chunk.is_empty() {
            return;
        }
        self.len += chunk.len();
        self.chunks.push_back(chunk);
    }

    /// Splits off `size` bytes from the front of the buffer. O(size) due to copying references.
    fn split_frame(&mut self, size: usize) -> Option<FrameBuf> {
        if size > self.len {
            return None;
        }
        if size == 0 {
            return Some(FrameBuf::new(VecDeque::new(), 0));
        }

        let mut needed = size;
        let mut parts = VecDeque::new();
        while needed > 0 {
            let mut chunk = self.chunks.pop_front()?;
            if chunk.len() > needed {
                let part = chunk.split_to(needed);
                self.len -= needed;
                parts.push_back(part);
                self.chunks.push_front(chunk);
                needed = 0;
            } else {
                needed -= chunk.len();
                self.len -= chunk.len();
                parts.push_back(chunk);
            }
        }
        Some(FrameBuf::new(parts, size))
    }
}

/// Thin Buf impl over a small deque of byte chunks.
struct FrameBuf {
    chunks: VecDeque<Bytes>,
    remaining: usize,
}

impl FrameBuf {
    /// Wraps the supplied deque and remaining length into a `FrameBuf`.
    fn new(chunks: VecDeque<Bytes>, remaining: usize) -> Self {
        Self { chunks, remaining }
    }

    /// Converts the buffered slices into a single `Bytes`, coalescing if needed.
    fn into_bytes(mut self) -> Bytes {
        match self.chunks.len() {
            0 => Bytes::new(),
            1 => self
                .chunks
                .pop_front()
                .expect("frame buffer length mismatch"),
            _ => {
                let mut buf = BytesMut::with_capacity(self.remaining);
                while let Some(chunk) = self.chunks.pop_front() {
                    buf.extend_from_slice(&chunk);
                }
                buf.freeze()
            }
        }
    }
}

impl Buf for FrameBuf {
    fn remaining(&self) -> usize {
        self.remaining
    }

    fn chunk(&self) -> &[u8] {
        self.chunks
            .front()
            .map(|bytes| bytes.as_ref())
            .unwrap_or(&[])
    }

    fn advance(&mut self, mut cnt: usize) {
        assert!(cnt <= self.remaining);
        self.remaining -= cnt;
        while cnt > 0 {
            let Some(front_len) = self.chunks.front().map(|b| b.len()) else {
                break;
            };
            if cnt < front_len {
                if let Some(front) = self.chunks.front_mut() {
                    front.advance(cnt);
                }
                break;
            } else {
                cnt -= front_len;
                let _ = self.chunks.pop_front();
            }
        }
    }
}

impl GrpcStreamingBody<H2BodyStream> {
    pub(crate) fn new(recv: h2::RecvStream, encoding: GrpcEncoding, router: Rc<GrpcRequestRouter>) -> Self {
        Self::with_stream(H2BodyStream::new(recv), encoding, router)
    }
}

impl<S> GrpcStreamingBody<S>
where
    S: BodyStream,
{
    fn with_stream(recv: S, encoding: GrpcEncoding, router: Rc<GrpcRequestRouter>) -> Self {
        Self {
            recv,
            buffer: ChunkBuffer::new(),
            current_frame: None,
            finished: false,
            encoding,
            router,
            decompressed_buf: BytesMut::with_capacity(128 * 1024),
        }
    }

    /// Pulls the next chunk from the underlying transport into our buffer.
    /// Complexity: O(1) push per chunk.  Because we run on the single-threaded
    /// runtime, `self.finished` is only toggled here.
    async fn fill_buffer(&mut self) -> Result<(), Status> {
        if self.finished {
            return Ok(());
        }
        match self.recv.next_chunk().await {
            Some(Ok(bytes)) => {
                let chunk_len = bytes.len();
                self.buffer.push(bytes);
                if let Err(err) = self.recv.release_capacity(chunk_len) {
                    log::debug!("release_capacity failed: {err}");
                }
                Ok(())
            }
            Some(Err(err)) => Err(Status::internal(format!("stream error: {err}"))),
            None => {
                self.finished = true;
                Ok(())
            }
        }
    }

    /// Reassembles the next gRPC frame payload, including the compression flag bit.
    async fn next_payload(&mut self) -> Result<Option<(bool, Bytes)>, Status> {
        loop {
            if self.current_frame.is_none() {
                if self.buffer.len() < 5 {
                    if self.finished {
                        return Ok(None);
                    }
                    self.fill_buffer().await?;
                    continue;
                }
                let header = self
                    .buffer
                    .split_frame(5)
                    .expect("buffer len checked above")
                    .into_bytes();
                let compressed = header[0] == 1;
                let len = u32::from_be_bytes([header[1], header[2], header[3], header[4]]) as usize;
                self.current_frame = Some(FrameHeader {
                    length: len,
                    compressed,
                });
            }

            if let Some(header) = self.current_frame.take() {
                if self.buffer.len() < header.length {
                    if self.finished {
                        log::error!("Stream ended before full gRPC frame was received");
                        return Err(Status::internal("truncated gRPC frame"));
                    }
                    self.fill_buffer().await?;
                    self.current_frame = Some(header);
                    continue;
                }

                let payload = self
                    .buffer
                    .split_frame(header.length)
                    .expect("buffer len checked above")
                    .into_bytes();
                return Ok(Some((header.compressed, payload)));
            }
        }
    }

    /// Makes sure the scratch buffer is large enough for the decoded payload.
    /// Includes heuristics to shrink the buffer if it remains excessively large
    /// for small payloads, preventing memory leaks in long-lived connections.
    fn reserve_decompressed_capacity(&mut self, payload_len: usize) {
        let current_capacity = self.decompressed_buf.capacity();

        // 1. Growth Path (Hot Path)
        // If we need more space, grow exponentially to amortize allocation costs.
        if current_capacity < payload_len {
            let required = payload_len
                .saturating_sub(current_capacity)
                .max(current_capacity); // Double the capacity (exponential growth)

            self.decompressed_buf.reserve(required);
            return;
        }

        // 2. Shrink Path (Cold Path - Optimization)
        // Only trigger if:
        // - The buffer is empty.
        // - We are holding a large amount of memory (e.g. > 4MB).
        // - The current requirement is tiny compared to capacity (e.g. < 1/8th).
        const EXCESSIVE_MEMORY_THRESHOLD: usize = 4 * 1024 * 1024; // 4MB

        if self.decompressed_buf.is_empty()
            && current_capacity > EXCESSIVE_MEMORY_THRESHOLD
            && payload_len < (current_capacity / 8)
        {
            // Shrink to a "Safe Baseline" (e.g. 1MB). This prevents thrashing
            // if the traffic fluctuates between 10KB and 800KB.
            const BASELINE_CAPACITY: usize = 1024 * 1024; // 1MB

            // We allocate a new buffer. The old one is dropped, releasing memory to the OS/Allocator.
            // We take the max of payload_len and Baseline to ensure we cover the current packet.
            let new_capacity = std::cmp::max(payload_len, BASELINE_CAPACITY);

            // Only proceed if we are actually saving significant memory
            if new_capacity < (current_capacity / 2) {
                self.decompressed_buf = BytesMut::with_capacity(new_capacity);
            }
        }
    }

    /// Dispatches to the appropriate decompressor for the negotiated encoding.
    fn decompress(&mut self, payload: Bytes) -> Result<Bytes, Status> {
        match self.encoding {
            GrpcEncoding::Identity => {
                log::error!("Received compressed frame but grpc-encoding=identity");
                Err(Status::unimplemented("message compression not negotiated"))
            }
            GrpcEncoding::Zstd => self.decompress_zstd(payload),
            GrpcEncoding::Gzip => self.decompress_gzip(payload),
            GrpcEncoding::Deflate => self.decompress_deflate(payload),
        }
    }

    /// Performs a zstd decode, growing the buffer as needed.
    /// Complexity: amortized O(n) over the payload size because each retry doubles
    /// the buffer.
    fn decompress_zstd(&mut self, payload: Bytes) -> Result<Bytes, Status> {
        self.reserve_decompressed_capacity(payload.len());

        let mut required_capacity = self.decompressed_buf.capacity();

        loop {
            set_bytes_len(&mut self.decompressed_buf, required_capacity);
            let buffer = &mut self.decompressed_buf[..];

            // Take ownership of the shared decompressor for the duration of this call to avoid
            // overlapping borrows on `self`.
            let mut slot = self.router.zstd_decompressor.borrow_mut();
            let mut decompressor = slot
                .take()
                .unwrap_or_else(|| Decompressor::new().expect("failed to initialize zstd decompressor"));
            drop(slot);

            let result = decompressor.decompress_to_buffer(payload.as_ref(), buffer);

            // Return the decompressor to the pool for reuse.
            *self.router.zstd_decompressor.borrow_mut() = Some(decompressor);

            match result {
                Ok(written) => {
                    self.decompressed_buf.truncate(written);
                    return Ok(self.decompressed_buf.split().freeze());
                }
                Err(err)
                if err.kind() == io::ErrorKind::Other
                    && err.to_string().contains("Destination buffer is too small") =>
                    {
                        required_capacity = required_capacity.checked_mul(2).ok_or_else(|| {
                            Status::internal("zstd decompression failed: output too large")
                        })?;
                        // No clear() needed – we overwrote the whole buffer with set_len
                    }
                Err(err) => {
                    return Err(Status::internal(format!("zstd decompression failed: {err}")));
                }
            }
        }
    }

    /// Performs a gzip inflate into the scratch buffer via streaming decoder.
    fn decompress_gzip(&mut self, payload: Bytes) -> Result<Bytes, Status> {
        self.reserve_decompressed_capacity(payload.len());
        self.decompressed_buf.clear();
        let mut decoder = GzDecoder::new(payload.as_ref());
        let mut writer = BytesMutWriter::new(&mut self.decompressed_buf);
        _ = io::copy(&mut decoder, &mut writer).map_err(|err| {
            log::error!("gzip decompression failed: {err}");
            Status::internal(format!("gzip decompression failed: {err}"))
        })?;
        Ok(self.decompressed_buf.split().freeze())
    }

    /// Performs a deflate inflate into the scratch buffer via streaming decoder.
    fn decompress_deflate(&mut self, payload: Bytes) -> Result<Bytes, Status> {
        self.reserve_decompressed_capacity(payload.len());
        self.decompressed_buf.clear();
        let mut decoder = ZlibDecoder::new(payload.as_ref());
        let mut writer = BytesMutWriter::new(&mut self.decompressed_buf);
        _ = io::copy(&mut decoder, &mut writer).map_err(|err| {
            log::error!("deflate decompression failed: {err}");
            Status::internal(format!("deflate decompression failed: {err}"))
        })?;
        Ok(self.decompressed_buf.split().freeze())
    }
}

struct BytesMutWriter<'a> {
    buffer: &'a mut BytesMut,
}

impl<'a> BytesMutWriter<'a> {
    fn new(buffer: &'a mut BytesMut) -> Self {
        Self { buffer }
    }
}

impl Write for BytesMutWriter<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Minimal async interface that lets the experimental receiver consume Arrow batches without
/// requiring the underlying stream to be `Send`. It mirrors the subset of `ArrowRequestStream`
/// used by this module so tests can inject local fakes.
#[async_trait(?Send)]
pub(crate) trait RequestStream {
    /// Fetches the next `BatchArrowRecords`, or `Ok(None)` when the peer half closes the stream.
    /// Implementations translate transport/protobuf failures into gRPC `Status` errors.
    async fn next_message(&mut self) -> Result<Option<BatchArrowRecords>, Status>;
}

#[async_trait(?Send)]
impl<S> RequestStream for GrpcStreamingBody<S>
where
    S: BodyStream + Unpin + 'static,
{
    /// Reassembles length-prefixed gRPC frames, handling optional compression, and yields decoded
    /// `BatchArrowRecords` to the caller.
    async fn next_message(&mut self) -> Result<Option<BatchArrowRecords>, Status> {
        let Some((compressed, payload)) = self.next_payload().await? else {
            return Ok(None);
        };

        let decoded = if compressed {
            let bytes = self.decompress(payload)?;
            BatchArrowRecords::decode(bytes)
        } else {
            BatchArrowRecords::decode(payload)
        };

        // Surface decoding failures as gRPC errors so clients know the batch was invalid.
        let message = decoded.map_err(|e| {
            log::error!("Failed to decode BatchArrowRecords: {e}");
            Status::invalid_argument(format!("failed to decode BatchArrowRecords: {e}"))
        })?;
        Ok(Some(message))
    }
}

impl<S> GrpcStreamingBody<S>
where
    S: BodyStream + Unpin + 'static,
{
    /// Returns the next raw gRPC frame payload. Compressed messages reuse the decompression
    /// scratch buffer and hand back a `Bytes` view without an extra copy.
    pub(crate) async fn next_message_bytes(&mut self) -> Result<Option<Bytes>, Status> {
        let Some((compressed, payload)) = self.next_payload().await? else {
            return Ok(None);
        };
        if compressed {
            let bytes = self.decompress(payload)?;
            Ok(Some(bytes))
        } else {
            Ok(Some(payload))
        }
    }
}

/// Utility wrapper that enforces per-request idle deadlines.
///
/// Each inbound OTAP Arrow request shares the single-threaded runtime with other tasks. To prevent
/// a stalled client from tying up resources indefinitely, `RequestTimeout` arms a `tokio::time::Sleep`
/// whenever we poll the status stream and cancels/reset it as soon as new data arrives. If the timer
/// elapses before the stream yields another item we abort the request with `DEADLINE_EXCEEDED`,
/// mirroring the behaviour of tonic's server stack.
pub(crate) struct RequestTimeout {
    duration: Option<Duration>,
    sleep: Option<Pin<Box<Sleep>>>,
}

impl RequestTimeout {
    pub(crate) fn new(duration: Option<Duration>) -> Self {
        Self {
            duration,
            sleep: None,
        }
    }

    pub(crate) async fn next_with<S, T>(&mut self, stream: &mut S) -> Result<Option<T>, ()>
    where
        S: Stream<Item = T> + Unpin,
    {
        futures::future::poll_fn(|cx| self.poll_next_with(cx, stream)).await
    }

    /// Awaits the provided future while enforcing the configured timeout.
    pub(crate) async fn with_future<F, T>(&mut self, future: F) -> Result<T, ()>
    where
        F: Future<Output = T>,
    {
        futures::pin_mut!(future);
        futures::future::poll_fn(|cx| self.poll_with_future(cx, future.as_mut())).await
    }

    pub(crate) fn poll_next_with<S, T>(
        &mut self,
        cx: &mut Context<'_>,
        stream: &mut S,
    ) -> Poll<Result<Option<T>, ()>>
    where
        S: Stream<Item = T> + Unpin,
    {
        if self.duration.is_none() {
            return StreamExt::poll_next_unpin(stream, cx).map(Ok);
        }

        self.ensure_sleep();
        if let Some(sleep) = self.sleep.as_mut() {
            if sleep.as_mut().poll(cx).is_ready() {
                return Poll::Ready(Err(()));
            }
        }

        match StreamExt::poll_next_unpin(stream, cx) {
            Poll::Ready(item) => {
                if let (Some(duration), Some(sleep)) = (self.duration, self.sleep.as_mut()) {
                    sleep.as_mut().reset(TokioInstant::now() + duration);
                }
                Poll::Ready(Ok(item))
            }
            Poll::Pending => Poll::Pending,
        }
    }

    pub(crate) fn poll_with_future<T>(
        &mut self,
        cx: &mut Context<'_>,
        mut future: Pin<&mut impl Future<Output = T>>,
    ) -> Poll<Result<T, ()>> {
        if self.duration.is_none() {
            return future.as_mut().poll(cx).map(Ok);
        }

        self.ensure_sleep();
        if let Some(sleep) = self.sleep.as_mut() {
            if sleep.as_mut().poll(cx).is_ready() {
                return Poll::Ready(Err(()));
            }
        }

        match future.as_mut().poll(cx) {
            Poll::Ready(out) => {
                if let (Some(duration), Some(sleep)) = (self.duration, self.sleep.as_mut()) {
                    sleep.as_mut().reset(TokioInstant::now() + duration);
                }
                Poll::Ready(Ok(out))
            }
            Poll::Pending => Poll::Pending,
        }
    }

    fn ensure_sleep(&mut self) {
        if let Some(duration) = self.duration {
            if self.sleep.is_none() {
                self.sleep = Some(Box::pin(sleep(duration)));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compression::CompressionMethod;
    use crate::otel_receiver::encoder::GrpcResponseFrameEncoder;
    use async_trait::async_trait;
    use bytes::{BufMut, Bytes, BytesMut};
    use flate2::Compression;
    use flate2::read::{GzDecoder, ZlibDecoder};
    use flate2::write::{GzEncoder, ZlibEncoder};
    use http::{HeaderMap, HeaderValue};
    use otap_df_pdata::proto::opentelemetry::arrow::v1::{BatchArrowRecords, BatchStatus};
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};
    use std::collections::VecDeque;
    use std::io::{Read, Write};
    use std::sync::{Arc, Mutex};
    use tokio::sync::mpsc;
    use tokio::task::yield_now;
    use tokio::time::Duration;
    use tokio_stream::wrappers::UnboundedReceiverStream;
    use zstd::bulk::Compressor as ZstdCompressor;

    fn base_headers() -> HeaderMap {
        let mut headers = HeaderMap::new();
        let _ = headers.insert(
            http::header::CONTENT_TYPE,
            HeaderValue::from_static("application/grpc"),
        );
        headers
    }

    #[test]
    fn test_parse_grpc_encoding_variants() {
        let accepted = AcceptedGrpcEncodings::from_methods(&[
            CompressionMethod::Zstd,
            CompressionMethod::Gzip,
        ]);
        let mut headers = base_headers();
        let _ = headers.insert("grpc-encoding", HeaderValue::from_static("zstd"));
        assert!(matches!(
            parse_grpc_encoding(&headers, &accepted),
            Ok(GrpcEncoding::Zstd)
        ));

        let _ = headers.insert("grpc-encoding", HeaderValue::from_static("gzip"));
        assert!(matches!(
            parse_grpc_encoding(&headers, &accepted),
            Ok(GrpcEncoding::Gzip)
        ));

        let _ = headers.insert("grpc-encoding", HeaderValue::from_static("zstdarrow1"));
        assert!(matches!(
            parse_grpc_encoding(&headers, &accepted),
            Ok(GrpcEncoding::Zstd)
        ));
    }

    #[test]
    fn test_parse_grpc_encoding_respects_config() {
        let accepted = AcceptedGrpcEncodings::from_methods(&[CompressionMethod::Deflate]);
        let mut headers = base_headers();
        let _ = headers.insert("grpc-encoding", HeaderValue::from_static("gzip"));
        assert!(parse_grpc_encoding(&headers, &accepted).is_err());
    }

    #[test]
    fn test_parse_grpc_accept_encoding() {
        let mut headers = HeaderMap::new();
        let _ = headers.insert(
            "grpc-accept-encoding",
            HeaderValue::from_static("gzip,zstd, identity "),
        );
        let parsed = parse_grpc_accept_encoding(&headers);
        assert!(parsed.identity);
        assert!(parsed.zstd);
        assert!(parsed.gzip);
        assert!(!parsed.deflate);
    }

    #[test]
    fn test_negotiate_response_encoding_prefers_config_order() {
        let mut client_headers = HeaderMap::new();
        let _ = client_headers.insert(
            "grpc-accept-encoding",
            HeaderValue::from_static("zstd,gzip"),
        );
        let client = parse_grpc_accept_encoding(&client_headers);
        let cfg = vec![CompressionMethod::Gzip, CompressionMethod::Zstd];
        assert!(matches!(
            negotiate_response_encoding(&cfg, &client),
            GrpcEncoding::Gzip
        ));

        let cfg = vec![CompressionMethod::Zstd];
        assert!(matches!(
            negotiate_response_encoding(&cfg, &client),
            GrpcEncoding::Zstd
        ));
    }

    #[test]
    fn test_build_accept_encoding_header_includes_identity() {
        let value =
            build_accept_encoding_header(&[CompressionMethod::Zstd, CompressionMethod::Gzip]);
        assert_eq!(value.to_str().unwrap(), "zstd,gzip,identity");
    }

    #[tokio::test]
    async fn request_timeout_triggers_after_inactivity() {
        let mut timeout = RequestTimeout::new(Some(Duration::from_millis(50)));
        let (tx, rx) = mpsc::unbounded_channel::<Result<&'static str, ()>>();
        let mut stream = UnboundedReceiverStream::new(rx);

        let _producer = tokio::spawn(async move {
            let _ = tx.send(Ok("first"));
            sleep(Duration::from_millis(10)).await;
            let _ = tx.send(Ok("second"));
            sleep(Duration::from_millis(200)).await;
            let _ = tx.send(Ok("third"));
        });

        assert!(timeout.next_with(&mut stream).await.unwrap().is_some());
        sleep(Duration::from_millis(15)).await;
        assert!(timeout.next_with(&mut stream).await.unwrap().is_some());
        assert!(timeout.next_with(&mut stream).await.is_err());
    }

    #[tokio::test]
    async fn request_timeout_disabled_when_unset() {
        let mut timeout = RequestTimeout::new(None);
        let (tx, rx) = mpsc::unbounded_channel::<Result<&'static str, ()>>();
        let mut stream = UnboundedReceiverStream::new(rx);

        let _producer = tokio::spawn(async move {
            sleep(Duration::from_millis(30)).await;
            let _ = tx.send(Ok("done"));
        });

        sleep(Duration::from_millis(35)).await;
        let next = timeout.next_with(&mut stream).await.unwrap();
        assert!(next.is_some());
    }

    #[test]
    fn test_grpc_message_encoder_identity_frame_layout() {
        let mut encoder = GrpcResponseFrameEncoder::new(GrpcEncoding::Identity);
        let message = BatchStatus {
            batch_id: 42,
            status_code: 7,
            status_message: "ok".to_string(),
        };
        let encoded = encoder.encode(&message).expect("identity encode");
        assert_eq!(encoded[0], 0);
        let len = u32::from_be_bytes(encoded[1..5].try_into().unwrap()) as usize;
        assert_eq!(len, encoded.len() - 5);
        assert_eq!(
            encoded[5..],
            message.encode_to_vec(),
            "payload matches prost encoding"
        );
    }

    #[test]
    fn test_grpc_message_encoder_gzip_round_trip() {
        let mut encoder = GrpcResponseFrameEncoder::new(GrpcEncoding::Gzip);
        let message = BatchStatus {
            batch_id: 99,
            status_code: 14,
            status_message: "compressed".to_string(),
        };
        let encoded = encoder.encode(&message).expect("gzip encode");
        assert_eq!(encoded[0], 1);
        let len = u32::from_be_bytes(encoded[1..5].try_into().unwrap()) as usize;
        assert_eq!(len, encoded.len() - 5);
        let mut decoder = GzDecoder::new(&encoded[5..]);
        let mut decompressed = Vec::new();
        let _ = decoder.read_to_end(&mut decompressed).expect("gunzip");
        assert_eq!(decompressed, message.encode_to_vec());
    }

    #[test]
    fn test_grpc_message_encoder_deflate_round_trip() {
        let mut encoder = GrpcResponseFrameEncoder::new(GrpcEncoding::Deflate);
        let message = BatchStatus {
            batch_id: 7,
            status_code: 3,
            status_message: "deflated".to_string(),
        };
        let encoded = encoder.encode(&message).expect("deflate encode");
        assert_eq!(encoded[0], 1);
        let len = u32::from_be_bytes(encoded[1..5].try_into().unwrap()) as usize;
        assert_eq!(len, encoded.len() - 5);
        let mut decoder = ZlibDecoder::new(&encoded[5..]);
        let mut decompressed = Vec::new();
        let _ = decoder.read_to_end(&mut decompressed).expect("inflate");
        assert_eq!(decompressed, message.encode_to_vec());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn grpc_streaming_body_randomized_frames() {
        async fn run_case(encoding: GrpcEncoding, seed: u64) {
            let mut rng = StdRng::seed_from_u64(seed);
            for iteration in 0..32 {
                let frame_count = rng.random_range(1..=8);
                let mut expected_ids = Vec::with_capacity(frame_count);
                let mut chunk_queue: VecDeque<Result<Bytes, &'static str>> = VecDeque::new();
                let mut expected_release = 0usize;

                for frame_idx in 0..frame_count {
                    let batch_id = (iteration * 100 + frame_idx) as i64;
                    expected_ids.push(batch_id);
                    let batch = BatchArrowRecords {
                        batch_id,
                        ..Default::default()
                    };

                    let frame = build_body_frame(&batch, encoding);
                    for chunk in split_frame_into_chunks(frame, &mut rng) {
                        expected_release += chunk.len();
                        chunk_queue.push_back(Ok(chunk));
                    }
                }

                let (stream, state_handle) = MockRecvStream::new(chunk_queue);
                let mut body = GrpcStreamingBody::with_stream(stream, encoding);
                let mut observed_ids = Vec::new();
                while let Some(batch) = body
                    .next_message()
                    .await
                    .expect("fuzzer should decode batches")
                {
                    observed_ids.push(batch.batch_id);
                }
                drop(body);

                assert_eq!(
                    observed_ids, expected_ids,
                    "encoding {:?} iteration {}",
                    encoding, iteration
                );
                let released = state_handle
                    .lock()
                    .expect("state lock poisoned")
                    .released_bytes;
                assert_eq!(
                    released, expected_release,
                    "flow control release mismatch for {:?}",
                    encoding
                );
            }
        }

        run_case(GrpcEncoding::Identity, 0x1111).await;
        run_case(GrpcEncoding::Gzip, 0x2222).await;
        run_case(GrpcEncoding::Deflate, 0x3333).await;
        run_case(GrpcEncoding::Zstd, 0x4444).await;
    }

    fn build_body_frame(batch: &BatchArrowRecords, encoding: GrpcEncoding) -> Bytes {
        let payload = batch.encode_to_vec();
        let (compressed, encoded_payload) = match encoding {
            GrpcEncoding::Identity => (false, payload),
            GrpcEncoding::Gzip => (true, compress_payload_gzip(&payload)),
            GrpcEncoding::Deflate => (true, compress_payload_deflate(&payload)),
            GrpcEncoding::Zstd => (true, compress_payload_zstd(&payload)),
        };
        let mut frame = BytesMut::with_capacity(5 + encoded_payload.len());
        frame.put_u8(u8::from(compressed));
        frame.put_u32(encoded_payload.len() as u32);
        frame.extend_from_slice(&encoded_payload);
        frame.freeze()
    }

    fn compress_payload_gzip(payload: &[u8]) -> Vec<u8> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(payload).expect("gzip write");
        encoder.finish().expect("gzip finish")
    }

    fn compress_payload_deflate(payload: &[u8]) -> Vec<u8> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(payload).expect("deflate write");
        encoder.finish().expect("deflate finish")
    }

    fn compress_payload_zstd(payload: &[u8]) -> Vec<u8> {
        let mut encoder = ZstdCompressor::new(0).expect("zstd encoder");
        let mut buffer = vec![0u8; payload.len().max(MIN_COMPRESSED_CAPACITY)];
        let written = encoder
            .compress_to_buffer(payload, buffer.as_mut_slice())
            .expect("zstd compress");
        buffer.truncate(written);
        buffer
    }

    fn split_frame_into_chunks(frame: Bytes, rng: &mut StdRng) -> Vec<Bytes> {
        let mut offset = 0;
        let mut chunks = Vec::new();
        while offset < frame.len() {
            let remaining = frame.len() - offset;
            let max_chunk = remaining.clamp(1, 64);
            let step = rng.random_range(1..=max_chunk);
            chunks.push(frame.slice(offset..offset + step));
            offset += step;
        }
        chunks
    }

    struct MockStreamState {
        released_bytes: usize,
    }

    struct MockRecvStream {
        chunks: VecDeque<Result<Bytes, &'static str>>,
        state: Arc<Mutex<MockStreamState>>,
    }

    impl MockRecvStream {
        fn new(
            chunks: VecDeque<Result<Bytes, &'static str>>,
        ) -> (Self, Arc<Mutex<MockStreamState>>) {
            let state = Arc::new(Mutex::new(MockStreamState { released_bytes: 0 }));
            (
                Self {
                    chunks,
                    state: state.clone(),
                },
                state,
            )
        }
    }

    #[async_trait(?Send)]
    impl BodyStream for MockRecvStream {
        async fn next_chunk(&mut self) -> Option<Result<Bytes, BodyStreamError>> {
            yield_now().await;
            self.chunks
                .pop_front()
                .map(|res| res.map_err(|err| err.to_string()))
        }

        fn release_capacity(&mut self, released: usize) -> Result<(), BodyStreamError> {
            if let Ok(mut state) = self.state.lock() {
                state.released_bytes += released;
            }
            Ok(())
        }
    }
}
