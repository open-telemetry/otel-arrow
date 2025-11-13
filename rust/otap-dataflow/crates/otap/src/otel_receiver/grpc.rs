// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

pub(crate) use impls::{
    AcceptedGrpcEncodings, BodyStream, BodyStreamError, GrpcEncoding, GrpcMessageEncoder,
    GrpcStreamingBody, MIN_COMPRESSED_CAPACITY, RequestTimeout, build_accept_encoding_header,
    grpc_encoding_token, negotiate_response_encoding, parse_grpc_accept_encoding,
    parse_grpc_encoding,
};

mod impls {
    use crate::compression::CompressionMethod;
    use crate::otap_grpc::ArrowRequestStream;
    use async_trait::async_trait;
    use bytes::{Buf, BufMut, Bytes, BytesMut};
    use flate2::Compression;
    use flate2::read::{GzDecoder, ZlibDecoder};
    use flate2::write::{GzEncoder, ZlibEncoder};
    use futures::{Stream, StreamExt};
    use http::{HeaderMap, HeaderValue};
    use otap_df_pdata::proto::opentelemetry::arrow::v1::BatchArrowRecords;
    use prost::Message;
    use std::collections::VecDeque;
    use std::io::{self, Read, Write};
    use std::mem;
    use std::pin::Pin;
    use std::time::Duration;
    use tokio::time::{Instant as TokioInstant, Sleep, sleep};
    use tonic::Status;
    use zstd::bulk::{Compressor as ZstdCompressor, Decompressor as ZstdDecompressor};

    const MIN_DECOMPRESSED_CAPACITY: usize = 8 * 1024;
    pub(crate) const MIN_COMPRESSED_CAPACITY: usize = 1024;

    pub(crate) fn parse_grpc_encoding(
        headers: &HeaderMap,
        accepted: &AcceptedGrpcEncodings,
    ) -> Result<GrpcEncoding, Status> {
        match headers.get(http::header::CONTENT_TYPE) {
            Some(value) if value.as_bytes().starts_with(b"application/grpc") => {}
            other => {
                log::error!("Rejecting stream due to invalid content-type: {other:?}");
                return Err(Status::invalid_argument(
                    "missing application/grpc content-type",
                ));
            }
        }
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

                let encoding = if ascii.is_empty() || eq_ascii_case_insensitive(ascii, b"identity")
                {
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

    fn eq_ascii_case_insensitive(value: &[u8], expected: &[u8]) -> bool {
        value.len() == expected.len()
            && value
                .iter()
                .zip(expected)
                .all(|(lhs, rhs)| ascii_byte_eq_ignore_case(*lhs, *rhs))
    }

    fn starts_with_ascii_case_insensitive(value: &[u8], prefix: &[u8]) -> bool {
        value.len() >= prefix.len()
            && value
                .iter()
                .zip(prefix)
                .all(|(lhs, rhs)| ascii_byte_eq_ignore_case(*lhs, *rhs))
    }

    fn ascii_byte_eq_ignore_case(lhs: u8, rhs: u8) -> bool {
        lhs == rhs || lhs.eq_ignore_ascii_case(&rhs)
    }

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
    pub(crate) enum GrpcEncoding {
        Identity,
        Zstd,
        Gzip,
        Deflate,
        // ToDo Add support for Snappy to follow Go implementation
    }

    #[derive(Clone, Copy)]
    pub(crate) struct AcceptedGrpcEncodings {
        zstd: bool,
        gzip: bool,
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
    pub(crate) struct ClientAcceptEncodings {
        pub(crate) identity: bool,
        pub(crate) zstd: bool,
        pub(crate) gzip: bool,
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

    #[async_trait]
    pub(crate) trait BodyStream: Send {
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

    #[async_trait]
    impl BodyStream for H2BodyStream {
        async fn next_chunk(&mut self) -> Option<Result<Bytes, BodyStreamError>> {
            self.inner
                .data()
                .await
                .map(|res| res.map_err(|err| err.to_string()))
        }

        fn release_capacity(&mut self, released: usize) -> Result<(), BodyStreamError> {
            self.inner
                .flow_control()
                .release_capacity(released)
                .map_err(|err| err.to_string())
        }
    }

    pub(crate) struct GrpcStreamingBody<S = H2BodyStream> {
        recv: S,
        buffer: ChunkBuffer,
        current_frame: Option<FrameHeader>,
        finished: bool,
        encoding: GrpcEncoding,
        zstd: Option<ZstdDecompressor<'static>>,
        decompressed_buf: Vec<u8>,
    }

    #[derive(Clone, Copy)]
    struct FrameHeader {
        length: usize,
        compressed: bool,
    }

    struct ChunkBuffer {
        chunks: VecDeque<Bytes>,
        len: usize,
    }

    impl ChunkBuffer {
        fn new() -> Self {
            Self {
                chunks: VecDeque::new(),
                len: 0,
            }
        }

        fn len(&self) -> usize {
            self.len
        }

        fn push(&mut self, chunk: Bytes) {
            if chunk.is_empty() {
                return;
            }
            self.len += chunk.len();
            self.chunks.push_back(chunk);
        }

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

    struct FrameBuf {
        chunks: VecDeque<Bytes>,
        remaining: usize,
    }

    impl FrameBuf {
        fn new(chunks: VecDeque<Bytes>, remaining: usize) -> Self {
            Self { chunks, remaining }
        }

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
        pub(crate) fn new(recv: h2::RecvStream, encoding: GrpcEncoding) -> Self {
            Self::with_stream(H2BodyStream::new(recv), encoding)
        }
    }

    impl<S> GrpcStreamingBody<S>
    where
        S: BodyStream,
    {
        pub(crate) fn with_stream(recv: S, encoding: GrpcEncoding) -> Self {
            Self {
                recv,
                buffer: ChunkBuffer::new(),
                current_frame: None,
                finished: false,
                encoding,
                zstd: None,
                decompressed_buf: Vec::new(),
            }
        }

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
        fn reserve_decompressed_capacity(&mut self, payload_len: usize) {
            let required_capacity = payload_len.saturating_mul(2).max(MIN_DECOMPRESSED_CAPACITY);
            if self.decompressed_buf.capacity() < required_capacity {
                self.decompressed_buf
                    .reserve(required_capacity - self.decompressed_buf.capacity());
            }
        }

        fn decompress(&mut self, payload: Bytes) -> Result<&[u8], Status> {
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

        fn decompress_zstd(&mut self, payload: Bytes) -> Result<&[u8], Status> {
            self.ensure_zstd_decompressor()?;
            let mut required_capacity = self
                .decompressed_buf
                .capacity()
                .max(payload.len().saturating_mul(2))
                .max(MIN_DECOMPRESSED_CAPACITY);

            loop {
                if self.decompressed_buf.capacity() < required_capacity {
                    self.decompressed_buf
                        .reserve(required_capacity - self.decompressed_buf.capacity());
                }
                self.decompressed_buf.clear();
                let result = {
                    let decompressor = self
                        .zstd
                        .as_mut()
                        .expect("decompressor must be initialized");
                    decompressor.decompress_to_buffer(payload.as_ref(), &mut self.decompressed_buf)
                };
                match result {
                    Ok(_) => return Ok(self.decompressed_buf.as_slice()),
                    Err(err) => {
                        let err_msg = err.to_string();
                        if err.kind() == io::ErrorKind::Other
                            && err_msg.contains("Destination buffer is too small")
                        {
                            required_capacity =
                                required_capacity.checked_mul(2).ok_or_else(|| {
                                    log::error!(
                                        "zstd decompression failed: required buffer overflow"
                                    );
                                    Status::internal("zstd decompression failed: output too large")
                                })?;
                            continue;
                        }
                        log::error!("zstd decompression failed: {err_msg}");
                        return Err(Status::internal(format!(
                            "zstd decompression failed: {err_msg}"
                        )));
                    }
                }
            }
        }

        fn decompress_gzip(&mut self, payload: Bytes) -> Result<&[u8], Status> {
            self.reserve_decompressed_capacity(payload.len());
            self.decompressed_buf.clear();
            let mut decoder = GzDecoder::new(payload.as_ref());
            let _ = decoder
                .read_to_end(&mut self.decompressed_buf)
                .map_err(|err| {
                    log::error!("gzip decompression failed: {err}");
                    Status::internal(format!("gzip decompression failed: {err}"))
                })?;
            Ok(self.decompressed_buf.as_slice())
        }

        fn decompress_deflate(&mut self, payload: Bytes) -> Result<&[u8], Status> {
            self.reserve_decompressed_capacity(payload.len());
            self.decompressed_buf.clear();
            let mut decoder = ZlibDecoder::new(payload.as_ref());
            let _ = decoder
                .read_to_end(&mut self.decompressed_buf)
                .map_err(|err| {
                    log::error!("deflate decompression failed: {err}");
                    Status::internal(format!("deflate decompression failed: {err}"))
                })?;
            Ok(self.decompressed_buf.as_slice())
        }

        fn ensure_zstd_decompressor(&mut self) -> Result<(), Status> {
            if self.zstd.is_some() {
                return Ok(());
            }
            match ZstdDecompressor::new() {
                Ok(decoder) => {
                    self.zstd = Some(decoder);
                    Ok(())
                }
                Err(err) => {
                    log::error!("Failed to construct zstd decompressor: {err}");
                    Err(Status::internal(format!(
                        "failed to initialize zstd decompressor: {err}"
                    )))
                }
            }
        }
    }

    #[async_trait]
    impl<S> ArrowRequestStream for GrpcStreamingBody<S>
    where
        S: BodyStream + Unpin + 'static,
    {
        async fn next_message(&mut self) -> Result<Option<BatchArrowRecords>, Status> {
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
                    let len =
                        u32::from_be_bytes([header[1], header[2], header[3], header[4]]) as usize;
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
                        .expect("buffer len checked above");
                    let decoded = if header.compressed {
                        let bytes = self.decompress(payload.into_bytes())?;
                        BatchArrowRecords::decode(bytes)
                    } else {
                        BatchArrowRecords::decode(payload)
                    };
                    let message = decoded.map_err(|e| {
                        log::error!("Failed to decode BatchArrowRecords: {e}");
                        Status::invalid_argument(format!("failed to decode BatchArrowRecords: {e}"))
                    })?;
                    return Ok(Some(message));
                }
            }
        }
    }

    pub(crate) struct GrpcMessageEncoder {
        compression: GrpcEncoding,
        frame_buf: BytesMut,
        message_buf: BytesMut,
        compressed_buf: Vec<u8>,
        zstd: Option<ZstdCompressor<'static>>,
    }

    impl GrpcMessageEncoder {
        pub(crate) fn new(compression: GrpcEncoding) -> Self {
            Self {
                compression,
                frame_buf: BytesMut::with_capacity(512),
                message_buf: BytesMut::with_capacity(512),
                compressed_buf: Vec::new(),
                zstd: None,
            }
        }

        pub(crate) fn encode<M: Message>(&mut self, message: &M) -> Result<Bytes, Status> {
            self.message_buf.clear();
            message
                .encode(&mut self.message_buf)
                .map_err(|e| Status::internal(format!("failed to encode response: {e}")))?;
            let uncompressed = self.message_buf.split().freeze();

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

        fn compress_zstd(&mut self, payload: &[u8]) -> Result<(), Status> {
            self.ensure_zstd_encoder()?;
            let mut required_capacity = payload.len().max(MIN_COMPRESSED_CAPACITY);
            loop {
                if self.compressed_buf.len() != required_capacity {
                    self.compressed_buf.resize(required_capacity, 0);
                }
                let result = {
                    let encoder = self.zstd.as_mut().expect("zstd encoder must exist");
                    encoder.compress_to_buffer(payload, self.compressed_buf.as_mut_slice())
                };
                match result {
                    Ok(written) => {
                        self.compressed_buf.truncate(written);
                        return Ok(());
                    }
                    Err(err)
                        if err.kind() == io::ErrorKind::Other
                            && err.to_string().contains("Destination buffer is too small") =>
                    {
                        required_capacity = required_capacity.checked_mul(2).ok_or_else(|| {
                            log::error!("zstd compression failed: required buffer overflow");
                            Status::internal("zstd compression failed: output too large")
                        })?;
                    }
                    Err(err) => {
                        log::error!("zstd compression failed: {err}");
                        return Err(Status::internal(format!("zstd compression failed: {err}")));
                    }
                }
            }
        }

        fn compress_gzip(&mut self, payload: &[u8]) -> Result<(), Status> {
            self.compressed_buf.clear();
            {
                let mut encoder = GzEncoder::new(&mut self.compressed_buf, Compression::default());
                encoder
                    .write_all(payload)
                    .and_then(|_| encoder.try_finish())
                    .map_err(|err| {
                        log::error!("gzip compression failed: {err}");
                        Status::internal(format!("gzip compression failed: {err}"))
                    })?;
            }
            Ok(())
        }

        fn compress_deflate(&mut self, payload: &[u8]) -> Result<(), Status> {
            self.compressed_buf.clear();
            {
                let mut encoder =
                    ZlibEncoder::new(&mut self.compressed_buf, Compression::default());
                encoder
                    .write_all(payload)
                    .and_then(|_| encoder.try_finish())
                    .map_err(|err| {
                        log::error!("deflate compression failed: {err}");
                        Status::internal(format!("deflate compression failed: {err}"))
                    })?;
            }
            Ok(())
        }

        fn ensure_zstd_encoder(&mut self) -> Result<(), Status> {
            if self.zstd.is_some() {
                return Ok(());
            }
            match ZstdCompressor::new(0) {
                Ok(encoder) => {
                    self.zstd = Some(encoder);
                    Ok(())
                }
                Err(err) => {
                    log::error!("Failed to construct zstd compressor: {err}");
                    Err(Status::internal(format!(
                        "failed to initialize zstd compressor: {err}"
                    )))
                }
            }
        }
    }

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

        fn arm(&mut self) {
            if let Some(duration) = self.duration {
                if self.sleep.is_none() {
                    self.sleep = Some(Box::pin(sleep(duration)));
                }
            }
        }

        fn reset(&mut self) {
            if let (Some(duration), Some(sleep)) = (self.duration, self.sleep.as_mut()) {
                sleep.as_mut().reset(TokioInstant::now() + duration);
            }
        }

        pub(crate) async fn next_with<S, T>(&mut self, stream: &mut S) -> Result<Option<T>, ()>
        where
            S: Stream<Item = T> + Unpin,
        {
            match self.duration {
                None => Ok(stream.next().await),
                Some(_) => {
                    self.arm();
                    let sleep = self
                        .sleep
                        .as_mut()
                        .expect("sleep must be armed when timeout is configured");
                    tokio::select! {
                        _ = sleep.as_mut() => Err(()),
                        item = stream.next() => {
                            self.reset();
                            Ok(item)
                        }
                    }
                }
            }
        }
    }
}
