// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// Private adapter tests remain colocated logically without exposing implementation internals.
macro_rules! postgresql_module_tests {
    (framing) => {
        mod framing_tests {
            use super::*;
            use std::{collections::VecDeque, future::Future};
            use tokio::io::{AsyncReadExt, AsyncWriteExt};
            use tokio_postgres::{Connection, tls::NoTlsStream};

            fn frame_limits() -> FrameLimits {
                FrameLimits::new(10 * 1024 * 1024).expect("configured receive budget")
            }

            fn frame(tag: u8, body: &[u8]) -> Vec<u8> {
                let mut bytes = vec![tag];
                bytes.extend_from_slice(&((body.len() + 4) as i32).to_be_bytes());
                bytes.extend_from_slice(body);
                bytes
            }

            struct Fragmented {
                bytes: Vec<u8>,
                consumed: Rc<Cell<usize>>,
                fragment: usize,
                pending: bool,
                error_at_end: Option<io::ErrorKind>,
            }

            impl AsyncRead for Fragmented {
                fn poll_read(
                    self: Pin<&mut Self>,
                    cx: &mut Context<'_>,
                    output: &mut ReadBuf<'_>,
                ) -> Poll<io::Result<()>> {
                    let this = self.get_mut();
                    if this.pending {
                        this.pending = false;
                        cx.waker().wake_by_ref();
                        return Poll::Pending;
                    }
                    this.pending = true;
                    let start = this.consumed.get();
                    if start == this.bytes.len()
                        && let Some(kind) = this.error_at_end
                    {
                        return Poll::Ready(Err(io::Error::from(kind)));
                    }
                    let count = output
                        .remaining()
                        .min(this.fragment)
                        .min(this.bytes.len() - start);
                    output.put_slice(&this.bytes[start..start + count]);
                    this.consumed.set(start + count);
                    Poll::Ready(Ok(()))
                }
            }

            fn fragmented(bytes: Vec<u8>, fragment: usize) -> BoundedBackend<Fragmented> {
                fragmented_with_limits(bytes, fragment, frame_limits())
            }

            fn fragmented_with_limits(
                bytes: Vec<u8>,
                fragment: usize,
                limits: FrameLimits,
            ) -> BoundedBackend<Fragmented> {
                BoundedBackend::new(
                    Fragmented {
                        bytes,
                        consumed: Rc::default(),
                        fragment,
                        pending: true,
                        error_at_end: None,
                    },
                    ReceiveFailure::default(),
                    limits,
                )
            }

            /// Scenario: An oversized backend length arrives one byte at a time.
            /// Guarantees: Only five header bytes are read; no body or header reaches the driver.
            #[tokio::test]
            async fn oversized_header_rejected_before_body() {
                let mut bytes = vec![b'D'];
                bytes.extend_from_slice(&(frame_limits().data_row as i32).to_be_bytes());
                bytes.extend_from_slice(b"unread private row");
                let mut stream = fragmented(bytes, 1);
                let mut output = [0; 64];
                let error = stream.read(&mut output).await.expect_err("oversized frame");
                assert_eq!(error.kind(), io::ErrorKind::InvalidData);
                assert_eq!(stream.failure.get(), Some(Error::Limit));
                assert_eq!(stream.inner.consumed.get(), 5);
                assert_eq!(output, [0; 64]);
                assert!(!error.to_string().contains("private"));
                assert!(stream.read(&mut output).await.is_err());
                assert_eq!(stream.inner.consumed.get(), 5);
            }

            /// Scenario: An oversized header and body bytes are already available in the socket.
            /// Guarantees: Fixed read-ahead stays within eight KiB and no unchecked header reaches the driver.
            #[tokio::test]
            async fn socket_buffer_keeps_oversized_frames_bounded() {
                let mut bytes = vec![b'D'];
                bytes.extend_from_slice(&(frame_limits().data_row as i32).to_be_bytes());
                bytes.resize(READ_BUFFER_BYTES * 2, 0);
                let source = fragmented(bytes, usize::MAX).inner;
                let consumed = source.consumed.clone();
                let failure = ReceiveFailure::default();
                let mut stream = BoundedBackend::new(
                    tokio::io::BufReader::with_capacity(READ_BUFFER_BYTES, source),
                    failure.clone(),
                    frame_limits(),
                );
                let mut output = [0; 64];
                assert!(stream.read(&mut output).await.is_err());
                assert_eq!(failure.get(), Some(Error::Limit));
                assert!(consumed.get() <= READ_BUFFER_BYTES);
                assert_eq!(output, [0; 64]);
            }

            /// Scenario: A backend announces invalid signed lengths, including negative and short lengths.
            /// Guarantees: Malformed headers are rejected explicitly without exposing their bytes.
            #[tokio::test]
            async fn malformed_lengths_rejected() {
                for length in [-1i32, i32::MIN, 0, 3] {
                    let mut bytes = vec![b'D'];
                    bytes.extend_from_slice(&length.to_be_bytes());
                    let mut stream = fragmented(bytes, 2);
                    let mut output = [0; 8];
                    let error = stream.read(&mut output).await.expect_err("invalid length");
                    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
                    assert_eq!(stream.failure.get(), Some(Error::Database));
                    assert_eq!(output, [0; 8]);
                }
            }

            /// Scenario: Standard authentication, metadata, row, notice, and completion frames are fragmented.
            /// Guarantees: Bytes survive pending reads and small destinations without crossing a frame boundary.
            #[tokio::test]
            async fn standard_frames_preserve_boundaries() {
                let frames = [
                    frame(b'R', &0i32.to_be_bytes()),
                    frame(b'S', b"server_version\0\x31\x35\0"),
                    frame(b'T', &[0, 0]),
                    frame(b'D', &[0, 1, 0, 0, 0, 1, b'x']),
                    frame(b'N', b"SNOTICE\0Mignored\0\0"),
                    frame(b's', &[]),
                    frame(b'Z', b"I"),
                ];
                let expected = frames.concat();
                for fragment in [1, 2, 7, usize::MAX] {
                    for capacity in [1, 4, 64] {
                        let mut stream = fragmented(expected.clone(), fragment);
                        let mut actual = Vec::new();
                        let mut boundary = 0;
                        for expected_frame in &frames {
                            boundary += expected_frame.len();
                            while actual.len() < boundary {
                                let mut output = vec![0; capacity];
                                let count = stream.read(&mut output).await.expect("frame bytes");
                                assert!(count > 0);
                                actual.extend_from_slice(&output[..count]);
                                assert!(actual.len() <= boundary, "read crossed frame boundary");
                            }
                        }
                        assert_eq!(actual, expected);
                        assert_eq!(stream.read(&mut [0; 8]).await.expect("EOF"), 0);
                        assert_eq!(stream.failure.get(), None);
                    }
                }
            }

            /// Scenario: A backend frame is exactly the inclusive configured wire-size limit.
            /// Guarantees: The complete legal frame streams through without an additional helper body allocation.
            #[tokio::test]
            async fn inclusive_frame_limit_passes() {
                let expected = frame(b'D', &vec![0; frame_limits().data_row - 5]);
                let mut stream = fragmented(expected, 4096);
                let mut total = 0;
                let mut output = [0; 4096];
                loop {
                    let read = stream.read(&mut output).await.expect("bounded frame");
                    if read == 0 {
                        break;
                    }
                    total += read;
                }
                assert_eq!(total, frame_limits().data_row);
                assert_eq!(stream.failure.get(), None);
            }

            /// Scenario: The same DataRow is received under different configured byte budgets.
            /// Guarantees: Wire overhead is explicit, inclusive bounds adapt, and rejection reads only the header.
            #[tokio::test]
            async fn configured_frame_limits_account_for_wire_overhead() {
                for budget in [1, 1024, 10 * 1024 * 1024] {
                    let limits = FrameLimits::new(budget).expect("limits");
                    assert_eq!(limits.data_row, budget as usize + 5 + 2 + 4 * 128);
                    assert_eq!(limits.raw_row(), budget as usize + 2 + 4 * 128);
                    let bytes = frame(b'D', &vec![0; limits.data_row - 4]);
                    let mut rejected = fragmented_with_limits(bytes.clone(), 1, limits);
                    let mut output = Vec::new();
                    assert!(rejected.read_to_end(&mut output).await.is_err());
                    assert!(output.is_empty());
                    assert_eq!(rejected.failure.get(), Some(Error::Limit));
                    assert_eq!(rejected.inner.consumed.get(), 5);
                    let larger = FrameLimits::new(budget + 1).expect("larger");
                    let mut accepted = fragmented_with_limits(bytes.clone(), 4096, larger);
                    let _ = accepted
                        .read_to_end(&mut output)
                        .await
                        .expect("larger budget");
                    assert_eq!(output, bytes);
                }
                assert!(FrameLimits::new(u64::MAX).is_err());
            }

            /// Scenario: A tiny row budget is smaller than startup metadata and an advertised DataRow.
            /// Guarantees: Only non-DataRow frames receive the bounded control allowance; rows cannot borrow it.
            #[tokio::test]
            async fn control_allowance_does_not_raise_data_row_limit() {
                let limits = FrameLimits::new(1).expect("tiny budget");
                assert_eq!(limits.control, CONTROL_FRAME_BYTES);
                for tag in [b'R', b'S', b'T', b'N', b'D'] {
                    let bytes = frame(tag, &vec![0; CONTROL_FRAME_BYTES - 5]);
                    let mut stream = fragmented_with_limits(bytes.clone(), 4096, limits);
                    let mut output = Vec::new();
                    if tag == b'D' {
                        assert!(stream.read_to_end(&mut output).await.is_err());
                        assert_eq!(stream.inner.consumed.get(), 5);
                    } else {
                        let _ = stream
                            .read_to_end(&mut output)
                            .await
                            .expect("control allowance");
                        assert_eq!(output, bytes);
                    }
                }
                let bytes = frame(b'T', &vec![0; CONTROL_FRAME_BYTES - 4]);
                let mut stream = fragmented_with_limits(bytes, 1, limits);
                assert!(stream.read(&mut [0; 8]).await.is_err());
                assert_eq!(stream.inner.consumed.get(), 5);
            }

            /// Scenario: The peer closes between frames, inside a header, or inside an accepted body.
            /// Guarantees: Partial header/body EOF reports retryable transport loss without fabricated bytes.
            #[tokio::test]
            async fn eof_and_truncation_are_distinct() {
                assert_eq!(
                    fragmented(Vec::new(), 1)
                        .read(&mut [0; 8])
                        .await
                        .expect("EOF"),
                    0
                );
                let complete = frame(b'D', b"row");
                for length in 1..complete.len() {
                    let mut stream = fragmented(complete[..length].to_vec(), 1);
                    let mut output = Vec::new();
                    let error = stream
                        .read_to_end(&mut output)
                        .await
                        .expect_err("truncated");
                    assert_eq!(error.kind(), io::ErrorKind::UnexpectedEof);
                    assert_eq!(stream.failure.get(), Some(Error::Unavailable));
                    if length < 5 {
                        assert!(output.is_empty());
                        assert!(error.to_string().contains("header"));
                    } else {
                        assert_eq!(output, complete[..length]);
                        assert!(error.to_string().contains("body"));
                    }
                }
            }

            /// Scenario: The underlying stream resets during header or body reception.
            /// Guarantees: The original transport error propagates without exposing incomplete headers.
            #[tokio::test]
            async fn transport_errors_propagate() {
                let complete = frame(b'D', b"row");
                for length in [0, 3, 6] {
                    let mut stream = fragmented(complete[..length].to_vec(), 2);
                    stream.inner.error_at_end = Some(io::ErrorKind::ConnectionReset);
                    let mut output = Vec::new();
                    let error = stream.read_to_end(&mut output).await.expect_err("reset");
                    assert_eq!(error.kind(), io::ErrorKind::ConnectionReset);
                    if length < 5 {
                        assert!(output.is_empty());
                    } else {
                        assert_eq!(output, complete[..length]);
                    }
                }
            }

            /// Scenario: The underlying reader reports UnexpectedEof rather than a zero-byte truncated read.
            /// Guarantees: Both header and body truncation remain retryable transport failures, not size-limit failures.
            #[tokio::test]
            async fn explicit_unexpected_eof_is_retryable() {
                let complete = frame(b'D', b"row");
                for length in [3, 6] {
                    let mut stream = fragmented(complete[..length].to_vec(), 2);
                    stream.inner.error_at_end = Some(io::ErrorKind::UnexpectedEof);
                    let mut output = Vec::new();
                    let error = stream
                        .read_to_end(&mut output)
                        .await
                        .expect_err("truncated");
                    assert_eq!(error.kind(), io::ErrorKind::UnexpectedEof);
                    assert_eq!(stream.failure.get(), Some(Error::Unavailable));
                }
            }

            /// Scenario: The wrapper writes a startup or password message and receives an empty read buffer.
            /// Guarantees: Outbound bytes are unchanged and a zero-capacity read consumes nothing.
            #[tokio::test]
            async fn writes_pass_through_and_empty_reads_do_not_consume() {
                let (client, mut server) = tokio::io::duplex(64);
                let mut stream =
                    BoundedBackend::new(client, ReceiveFailure::default(), frame_limits());
                assert_eq!(stream.read(&mut []).await.expect("empty read"), 0);
                stream.write_all(b"outbound").await.expect("write");
                stream.flush().await.expect("flush");
                stream.shutdown().await.expect("shutdown");
                let mut bytes = Vec::new();
                let _ = server.read_to_end(&mut bytes).await.expect("read outbound");
                assert_eq!(bytes, b"outbound");
            }

            // Release each scripted response only after a client write, keeping
            // startup/prepare traffic out of the portal read-ahead measurement.
            struct Peer {
                responses: VecDeque<Vec<u8>>,
                current: Vec<u8>,
                position: usize,
                consumed: Rc<Cell<usize>>,
                write_error: Option<io::ErrorKind>,
            }

            impl AsyncRead for Peer {
                fn poll_read(
                    self: Pin<&mut Self>,
                    _: &mut Context<'_>,
                    output: &mut ReadBuf<'_>,
                ) -> Poll<io::Result<()>> {
                    let this = self.get_mut();
                    if this.position == this.current.len() {
                        return Poll::Pending;
                    }
                    let count = output.remaining().min(this.current.len() - this.position);
                    output.put_slice(&this.current[this.position..this.position + count]);
                    this.position += count;
                    this.consumed.set(this.consumed.get() + count);
                    Poll::Ready(Ok(()))
                }
            }

            impl AsyncWrite for Peer {
                fn poll_write(
                    self: Pin<&mut Self>,
                    cx: &mut Context<'_>,
                    bytes: &[u8],
                ) -> Poll<io::Result<usize>> {
                    let this = self.get_mut();
                    assert_eq!(this.position, this.current.len(), "request was pipelined");
                    if this.responses.is_empty()
                        && let Some(kind) = this.write_error
                    {
                        return Poll::Ready(Err(io::Error::new(
                            kind,
                            "private transport diagnostic",
                        )));
                    }
                    this.current = this.responses.pop_front().expect("scripted response");
                    this.position = 0;
                    cx.waker().wake_by_ref();
                    Poll::Ready(Ok(bytes.len()))
                }

                fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
                    Poll::Ready(Ok(()))
                }

                fn poll_shutdown(
                    self: Pin<&mut Self>,
                    _: &mut Context<'_>,
                ) -> Poll<io::Result<()>> {
                    Poll::Ready(Ok(()))
                }
            }

            async fn drive<S: AsyncRead + AsyncWrite + Unpin, F: Future>(
                connection: &mut Connection<S, NoTlsStream>,
                future: F,
            ) -> F::Output {
                tokio::select! {
                    result = future => result,
                    result = poll_fn(|cx| connection.poll_message(cx)) => {
                        panic!("unexpected driver termination or async message: {result:?}");
                    }
                }
            }

            /// Scenario: An oversized response terminates the driver while a query awaits its channel.
            /// Guarantees: The receive latch retains Limit even though the query receives only a closed error.
            #[tokio::test]
            async fn receive_limit_survives_driver_channel_closure() {
                let mut oversized = vec![b'D'];
                oversized.extend_from_slice(&(frame_limits().data_row as i32).to_be_bytes());
                let failure = ReceiveFailure::default();
                let peer = Peer {
                    responses: VecDeque::from([
                        [frame(b'R', &0i32.to_be_bytes()), frame(b'Z', b"I")].concat(),
                        oversized,
                    ]),
                    current: Vec::new(),
                    position: 0,
                    consumed: Rc::default(),
                    write_error: None,
                };
                let (client, mut connection) = tokio_postgres::Config::new()
                    .user("test")
                    .ssl_mode(SslMode::Disable)
                    .connect_raw(
                        BoundedBackend::new(peer, failure.clone(), frame_limits()),
                        NoTls,
                    )
                    .await
                    .expect("startup");
                let query = client.batch_execute("SELECT 1");
                let driver = async move {
                    let error = poll_fn(|cx| connection.poll_message(cx))
                        .await
                        .expect("terminal message")
                        .expect_err("oversized frame");
                    drop(connection);
                    error
                };
                let (query, _) = tokio::join!(query, driver);
                assert!(query.expect_err("driver closed").is_closed());
                assert_eq!(failure.get(), Some(Error::Limit));
            }

            /// Scenario: The driver receives 300 64-KiB portal rows before the consumer polls.
            /// Guarantees: Batches remain frame-sized with fixed socket read-ahead, while a valid fetch over eight MiB is fully accepted.
            #[tokio::test]
            async fn portal_backpressure_bounds_actual_driver_batches() {
                let ready = frame(b'Z', b"T");
                let mut description = vec![0, 1];
                description.extend_from_slice(b"value\0");
                description.extend_from_slice(&0u32.to_be_bytes());
                description.extend_from_slice(&0i16.to_be_bytes());
                description.extend_from_slice(&25u32.to_be_bytes());
                description.extend_from_slice(&(-1i16).to_be_bytes());
                description.extend_from_slice(&(-1i32).to_be_bytes());
                description.extend_from_slice(&0i16.to_be_bytes());
                let mut body = vec![0, 1];
                let payload_bytes = 64 * 1024;
                body.extend_from_slice(&(payload_bytes as i32).to_be_bytes());
                body.extend_from_slice(&vec![b'x'; payload_bytes]);
                let row = frame(b'D', &body);
                let mut rows = row.repeat(300);
                rows.extend_from_slice(&frame(b's', &[]));
                rows.extend_from_slice(&ready);
                let consumed = Rc::new(Cell::new(0));
                let failure = ReceiveFailure::default();
                let peer = Peer {
                    responses: VecDeque::from([
                        [frame(b'R', &0i32.to_be_bytes()), frame(b'Z', b"I")].concat(),
                        [frame(b'C', b"BEGIN\0"), ready.clone()].concat(),
                        [
                            frame(b'1', &[]),
                            frame(b't', &[0, 0]),
                            frame(b'T', &description),
                            ready.clone(),
                        ]
                        .concat(),
                        [frame(b'2', &[]), ready].concat(),
                        rows,
                    ]),
                    current: Vec::new(),
                    position: 0,
                    consumed: consumed.clone(),
                    write_error: None,
                };
                let (mut client, mut connection) = tokio_postgres::Config::new()
                    .user("test")
                    .ssl_mode(SslMode::Disable)
                    .connect_raw(
                        BoundedBackend::new(
                            tokio::io::BufReader::with_capacity(READ_BUFFER_BYTES, peer),
                            failure.clone(),
                            frame_limits(),
                        ),
                        NoTls,
                    )
                    .await
                    .expect("startup");
                let transaction = drive(&mut connection, client.transaction())
                    .await
                    .expect("begin");
                let statement = drive(&mut connection, transaction.prepare("SELECT value"))
                    .await
                    .expect("prepare");
                let portal = drive(&mut connection, transaction.bind(&statement, &[]))
                    .await
                    .expect("bind");
                let rows = transaction
                    .query_portal_raw(&portal, 300)
                    .await
                    .expect("portal");
                tokio::pin!(rows);
                let before = consumed.get();
                // First poll dispatches Execute; subsequent polls must stall on the
                // capacity-one channel (which also grants one sender-reserved slot).
                for _ in 0..4 {
                    poll_fn(|cx| {
                        assert!(connection.poll_message(cx).is_pending());
                        Poll::Ready(())
                    })
                    .await;
                }
                let received = consumed.get() - before;
                assert!(received >= row.len());
                assert!(
                    received <= 3 * row.len() + READ_BUFFER_BYTES,
                    "read ahead {received} bytes"
                );
                let mut count = 0;
                while let Some(row) = drive(&mut connection, rows.next()).await {
                    let row = row.expect("row");
                    assert_eq!(row.get::<_, &str>(0).len(), payload_bytes);
                    count += 1;
                }
                assert_eq!(count, 300);
                assert_eq!(failure.get(), None);
            }

            /// Scenario: A native row contains a text field larger than one MiB under a two-MiB budget.
            /// Guarantees: Framing and native decoding accept it, but a smaller configured raw-row bound rejects it.
            #[tokio::test]
            async fn native_row_larger_than_one_mib_uses_configured_budget() {
                let payload = vec![b'x'; 1024 * 1024 + 1];
                let mut description = 3i16.to_be_bytes().to_vec();
                for (name, oid) in [("ts", 1184u32), ("id", 20), ("payload", 25)] {
                    description.extend_from_slice(name.as_bytes());
                    description.push(0);
                    description.extend_from_slice(&0u32.to_be_bytes());
                    description.extend_from_slice(&0i16.to_be_bytes());
                    description.extend_from_slice(&oid.to_be_bytes());
                    description.extend_from_slice(&(-1i16).to_be_bytes());
                    description.extend_from_slice(&(-1i32).to_be_bytes());
                    description.extend_from_slice(&0i16.to_be_bytes());
                }
                let mut body = 3i16.to_be_bytes().to_vec();
                for value in [
                    &0i64.to_be_bytes()[..],
                    &1i64.to_be_bytes()[..],
                    payload.as_slice(),
                ] {
                    body.extend_from_slice(&(value.len() as i32).to_be_bytes());
                    body.extend_from_slice(value);
                }
                let peer = Peer {
                    responses: VecDeque::from([
                        [frame(b'R', &0i32.to_be_bytes()), frame(b'Z', b"I")].concat(),
                        [
                            frame(b'1', &[]),
                            frame(b't', &[0, 0]),
                            frame(b'T', &description),
                            frame(b'Z', b"I"),
                        ]
                        .concat(),
                        [
                            frame(b'2', &[]),
                            frame(b'D', &body),
                            frame(b'C', b"SELECT 1\0"),
                            frame(b'Z', b"I"),
                        ]
                        .concat(),
                    ]),
                    current: Vec::new(),
                    position: 0,
                    consumed: Rc::default(),
                    write_error: None,
                };
                let limits = FrameLimits::new(2 * 1024 * 1024).expect("budget");
                let (client, mut connection) = tokio_postgres::Config::new()
                    .user("test")
                    .ssl_mode(SslMode::Disable)
                    .connect_raw(
                        BoundedBackend::new(peer, ReceiveFailure::default(), limits),
                        NoTls,
                    )
                    .await
                    .expect("startup");
                let rows = drive(&mut connection, client.query("SELECT ts, id, payload", &[]))
                    .await
                    .expect("native row");
                assert_eq!(rows.len(), 1);
                assert!(rows[0].raw_size_bytes() > 1024 * 1024);
                let plan = super::super::tests::test_validated().plan;
                let mut schema = super::super::tests::test_schema();
                schema.columns.push(super::super::query::ResultColumn {
                    name: "payload".into(),
                    source_type: "text".into(),
                    nullable: false,
                    type_modifier: -1,
                });
                schema.native_types.push(tokio_postgres::types::Type::TEXT);
                let mut cache = convert::CursorTimestampCache::default();
                let decoded = convert::row(&rows[0], &plan, &schema, &mut cache, limits.raw_row())
                    .expect("large decoded row");
                assert_eq!(
                    decoded.row.row.values[2],
                    otel_arrow_dfe_scraper::database::CellValue::String(
                        String::from_utf8(payload).expect("text")
                    )
                );
                assert!(matches!(
                    convert::row(&rows[0], &plan, &schema, &mut cache, 1024 * 1024),
                    Err(Error::Limit)
                ));
            }

            /// Scenario: A connected backend fails a subsequent query write with BrokenPipe.
            /// Guarantees: The driver error remains retryable and redacted, rather than terminating collection as a database error.
            #[tokio::test]
            async fn broken_pipe_after_handshake_is_retryable() {
                let peer = Peer {
                    responses: VecDeque::from([[
                        frame(b'R', &0i32.to_be_bytes()),
                        frame(b'Z', b"I"),
                    ]
                    .concat()]),
                    current: Vec::new(),
                    position: 0,
                    consumed: Rc::default(),
                    write_error: Some(io::ErrorKind::BrokenPipe),
                };
                let (client, mut connection) = tokio_postgres::Config::new()
                    .user("test")
                    .ssl_mode(SslMode::Disable)
                    .connect_raw(
                        BoundedBackend::new(peer, ReceiveFailure::default(), frame_limits()),
                        NoTls,
                    )
                    .await
                    .expect("startup");
                let query = client.batch_execute("SELECT 1");
                let driver = async move {
                    let error = poll_fn(|cx| connection.poll_message(cx))
                        .await
                        .expect("driver failure")
                        .expect_err("broken pipe");
                    drop(connection);
                    database(error)
                };
                let (query, error) = tokio::join!(query, driver);
                assert!(query.is_err());
                assert_eq!(error, Error::Unavailable);
                assert!(!error.to_string().contains("private"));
            }
        }
    };
    (value) => {
        mod tests {
            use super::*;

            /// Scenario: Text and bytea values exceed the former one-MiB decoder ceiling.
            /// Guarantees: Both codecs preserve all bytes; configured framing and page admission supply their limits.
            #[test]
            fn large_text_and_bytea_are_not_truncated() {
                let bytes = vec![b'x'; 1024 * 1024 + 1];
                assert_eq!(
                    decode(&Type::TEXT, &bytes).expect("text"),
                    CellValue::String(String::from_utf8(bytes.clone()).expect("UTF-8"))
                );
                assert_eq!(
                    decode(&Type::BYTEA, &bytes).expect("bytea"),
                    CellValue::Bytes(bytes)
                );
            }

            /// Scenario: Native PostgreSQL timestamps cover years before the epoch, leap days, and microsecond boundaries.
            /// Guarantees: Both native timestamp types emit exact ISO 8601 UTC text with six fractional digits and a Z suffix.
            #[test]
            fn direct_timestamp_format_matches_native_text_contract() {
                for (year, month, day) in [
                    (1, 1, 1),
                    (1960, 2, 29),
                    (2000, 1, 1),
                    (2026, 12, 31),
                    (9999, 12, 31),
                ] {
                    for micros in [0, 1, 1000, 123456, 999999] {
                        let dt = NaiveDate::from_ymd_opt(year, month, day)
                            .expect("date")
                            .and_hms_micro_opt(23, 59, 59, micros)
                            .expect("time");
                        let bytes = dt
                            .signed_duration_since(epoch().expect("epoch"))
                            .num_microseconds()
                            .expect("native range")
                            .to_be_bytes();
                        assert_eq!(
                            decode(&Type::TIMESTAMP, &bytes).expect("timestamp"),
                            CellValue::Timestamp(dt.format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string())
                        );
                        assert_eq!(
                            decode(&Type::TIMESTAMPTZ, &bytes).expect("timestamptz"),
                            CellValue::TimestampTz(dt.format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string())
                        );
                    }
                }
            }

            /// Scenario: Sorted rows share timestamps or continuously advance to a new timestamp.
            /// Guarantees: The single-entry cache preserves exact values and reuses fixed storage instead of growing with row count.
            #[test]
            fn timestamp_cache_is_exact_and_bounded() {
                let mut cache = CursorTimestampCache::default();
                let _ = cache.decode(&0i64.to_be_bytes(), 6).expect("initial");
                let capacity = cache.text.capacity();
                for micros in [-1i64, 0, 0, 1, 1, 123456, 1_000_000, 86_400_000_000] {
                    let bytes = micros.to_be_bytes();
                    let dt = timestamp(&bytes).expect("native timestamp");
                    let (cached, text) = cache.decode(&bytes, 6).expect("decoded");
                    assert_eq!(cached, dt);
                    assert_eq!(text, dt.format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string());
                    assert_eq!(cache.text.capacity(), capacity);
                }
                let micros = 1i64.to_be_bytes();
                assert!(cache.decode(&micros, 6).is_ok());
                assert!(
                    cache.decode(&micros, 3).is_err(),
                    "precision is part of the cache key"
                );
                assert!(cache.decode(&i64::MAX.to_be_bytes(), 6).is_err());
                assert!(cache.decode(&[0; 7], 6).is_err());
                assert_eq!(
                    cache.decode(&micros, 6).expect("valid").1,
                    "2000-01-01T00:00:00.000001Z"
                );
            }

            /// Scenario: A native cursor timestamp is checked without parsing its formatted string.
            /// Guarantees: Typmod precision, invalid modifiers, and leap-second rejection match checkpoint parsing.
            #[test]
            fn native_cursor_validation_preserves_precision_checks() {
                let date = NaiveDate::from_ymd_opt(2026, 1, 1).expect("date");
                for micros in [0, 1, 1000, 123000, 123001, 999999] {
                    let dt = date.and_hms_micro_opt(0, 0, 0, micros).expect("time");
                    for modifier in [-2, -1, 0, 1, 2, 3, 4, 5, 6, 7] {
                        assert_eq!(
                            validate_timestamp(dt, modifier).is_ok(),
                            cursor_time(&timestamp_text(dt).expect("formatted"), modifier).is_ok()
                        );
                    }
                }
                assert!(
                    validate_timestamp(
                        date.and_hms_nano_opt(23, 59, 59, 1_000_000_000)
                            .expect("leap second"),
                        6
                    )
                    .is_err()
                );
            }
        }
    };
    (worker) => {
        mod tests {
            use super::*;

            /// Scenario: Every page supplies fresh catalog metadata while decoder state is cached.
            /// Guarantees: Changed source/type/nullability/collation/key signatures fail without replacing the baseline.
            #[test]
            fn cached_schema_rechecks_complete_signature() {
                use super::super::tests::{mock_signature, test_validated};
                use tokio_postgres::types::Type;
                let validated = test_validated();
                let baseline = mock_signature(
                    &validated.plan,
                    &[(Type::TIMESTAMPTZ, 6, true), (Type::INT8, -1, true)],
                );
                let mut cache = None;
                let _ =
                    resolve_schema(&mut cache, &validated, baseline.clone()).expect("discovery");
                for change in 0..8 {
                    let mut changed = baseline.clone();
                    let column = changed
                        .columns
                        .get_mut(&validated.plan.projections[0])
                        .expect("column");
                    match change {
                        0 => column.oid += 1,
                        1 => column.attribute += 1,
                        2 => column.type_oid = Type::TIMESTAMP.oid(),
                        3 => column.modifier = 3,
                        4 => column.not_null = false,
                        5 => column.collation = 1,
                        6 => changed.keys.clear(),
                        _ => {
                            let _ = changed.columns.remove(&validated.plan.projections[1]);
                        }
                    }
                    assert!(matches!(
                        resolve_schema(&mut cache, &validated, changed),
                        Err(Error::Metadata)
                    ));
                    let cached = resolve_schema(&mut cache, &validated, baseline.clone())
                        .expect("original baseline retained");
                    assert_eq!(cached.signature, baseline);
                    assert_eq!(
                        cached.resolved.native_types,
                        [Type::TIMESTAMPTZ, Type::INT8]
                    );
                }
            }

            fn page_row(id: i64, width: usize) -> CursorRow {
                use otel_arrow_dfe_scraper::database::{CellValue, Cursor};
                CursorRow {
                    row: Row {
                        values: vec![CellValue::String("x".repeat(width))],
                    },
                    cursor: Cursor::composite("2026-01-01T00:00:00.000000Z".into(), id),
                }
            }

            fn measured_page_bytes(page: &QueryPage) -> u64 {
                page_metadata_bytes(page)
                    + (page.rows.capacity() * size_of::<CursorRow>()) as u64
                    + page
                        .rows
                        .iter()
                        .map(|row| row_payload_bytes(row).expect("composite"))
                        .sum::<u64>()
            }

            /// Scenario: A ten-thousand-row configuration has only enough bytes for one small row and metadata.
            /// Guarantees: Initial allocation follows actual bytes, and the rejected next row does not advance the cursor.
            #[test]
            fn tiny_page_budget_admits_real_row_without_full_page_allocation() {
                let mut page = QueryPage {
                    columns: metadata(&super::super::tests::test_schema()),
                    rows: Vec::new(),
                };
                let mut payload = page_metadata_bytes(&page);
                let limit = payload
                    + row_payload_bytes(&page_row(1, 1)).expect("row")
                    + size_of::<CursorRow>() as u64;
                assert!(limit < 1024);
                assert!(
                    push_bounded_row(&mut page.rows, &mut payload, page_row(1, 1), 10_000, limit)
                        .expect("fits")
                );
                assert_eq!(page.rows.capacity(), 1);
                assert_eq!(measured_page_bytes(&page), limit);
                let cursor = page.rows[0].cursor.clone();
                assert!(
                    !push_bounded_row(&mut page.rows, &mut payload, page_row(2, 1), 10_000, limit)
                        .expect("full")
                );
                assert_eq!(page.rows[0].cursor, cursor);
                assert_eq!(page.rows.len(), 1);
            }

            /// Scenario: Many small rows fill a configured ten-thousand-row page.
            /// Guarantees: Growth is geometric and allocated slots, cursor storage, and metadata stay charged.
            #[test]
            fn page_growth_is_bounded_and_geometric() {
                let mut page = QueryPage {
                    columns: metadata(&super::super::tests::test_schema()),
                    rows: Vec::new(),
                };
                let mut payload = page_metadata_bytes(&page);
                let limit = 10 * 1024 * 1024;
                let mut growths = 0;
                for id in 1..=10_000 {
                    let capacity = page.rows.capacity();
                    assert!(
                        push_bounded_row(
                            &mut page.rows,
                            &mut payload,
                            page_row(id, 16),
                            10_000,
                            limit
                        )
                        .expect("fits")
                    );
                    growths += usize::from(capacity != page.rows.capacity());
                }
                assert!(growths <= 13);
                assert!(page.rows.capacity() <= 10_000);
                assert_eq!(
                    measured_page_bytes(&page),
                    payload + (page.rows.capacity() * size_of::<CursorRow>()) as u64
                );
                assert!(measured_page_bytes(&page) <= limit);
                assert!(
                    !push_bounded_row(
                        &mut page.rows,
                        &mut payload,
                        page_row(10_001, 0),
                        10_000,
                        limit
                    )
                    .expect("row ceiling")
                );
            }

            /// Scenario: Geometric spare capacity would consume bytes needed by a wider next row.
            /// Guarantees: Reclaiming spare slots admits the real row without bypassing actual allocation accounting.
            #[test]
            fn spare_slots_do_not_force_premature_page_end() {
                let mut rows = Vec::with_capacity(16);
                let mut payload = 0;
                let wide = page_row(2, 1024);
                let limit = row_payload_bytes(&page_row(1, 1)).expect("small")
                    + row_payload_bytes(&wide).expect("wide")
                    + 2 * size_of::<CursorRow>() as u64;
                assert!(
                    push_bounded_row(&mut rows, &mut payload, page_row(1, 1), 10_000, limit)
                        .expect("small row")
                );
                assert!(rows.capacity() > 2);
                assert!(
                    push_bounded_row(&mut rows, &mut payload, wide, 10_000, limit)
                        .expect("wide row")
                );
                assert_eq!(rows.len(), 2);
                assert_eq!(
                    payload + (rows.capacity() * size_of::<CursorRow>()) as u64,
                    limit
                );
            }

            /// Scenario: A row retains excess cursor capacity or exceeds the customer's normalized limit.
            /// Guarantees: Heap capacity is charged and an oversized first row fails explicitly rather than yielding success.
            #[test]
            fn row_admission_charges_cursor_capacity_and_large_values() {
                use otel_arrow_dfe_scraper::database::Cursor;
                let mut row = page_row(1, 1);
                let before = row_payload_bytes(&row).expect("row");
                let mut timestamp = String::with_capacity(4096);
                timestamp.push_str("2026-01-01T00:00:00.000000Z");
                row.cursor = Cursor::composite(timestamp, 1);
                assert!(row_payload_bytes(&row).expect("cursor capacity") > before + 4000);
                assert_eq!(
                    push_bounded_row(&mut Vec::new(), &mut 0, row, 10_000, 1024),
                    Err(Error::Limit)
                );
                let mut rows = Vec::new();
                let mut payload = 0;
                assert!(
                    push_bounded_row(
                        &mut rows,
                        &mut payload,
                        page_row(1, 1024 * 1024 + 1),
                        10_000,
                        2 * 1024 * 1024
                    )
                    .expect("large row fits")
                );
                assert_eq!(rows.len(), 1);
                assert_eq!(
                    push_bounded_row(
                        &mut Vec::new(),
                        &mut 0,
                        page_row(1, 1024 * 1024 + 1),
                        10_000,
                        1024 * 1024
                    ),
                    Err(Error::Limit)
                );
            }

            /// Scenario: Native cursor positions cross equal timestamps, integer boundaries, and timestamp transitions.
            /// Guarantees: The allocation-free ordering check agrees with the shared durable-cursor comparison.
            #[test]
            fn native_cursor_order_matches_shared_order() {
                use otel_arrow_dfe_scraper::database::Cursor;
                let timestamp = convert::cursor_time("2026-01-01T00:00:00.123456Z", 6)
                    .expect("timestamp")
                    .naive_utc();
                for (previous, next) in [
                    ((timestamp, 1), (timestamp, 1)),
                    ((timestamp, 1), (timestamp, 2)),
                    ((timestamp, 0), (timestamp, -1)),
                    ((timestamp, i64::MIN), (timestamp, i64::MAX)),
                    (
                        (timestamp, i64::MAX),
                        (timestamp + chrono::Duration::microseconds(1), i64::MIN),
                    ),
                    (
                        (timestamp, 1),
                        (timestamp - chrono::Duration::microseconds(1), 2),
                    ),
                ] {
                    let cursor = |position: convert::CursorPosition| {
                        Cursor::composite(
                            position.0.format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string(),
                            position.1,
                        )
                    };
                    assert_eq!(
                        ensure_advance(&previous, &next).is_ok(),
                        cursor(next).compare(&cursor(previous)).expect("compatible")
                            == std::cmp::Ordering::Greater
                    );
                }
            }

            /// Scenario: Row sizes change from narrow to wide while pages approach their byte ceiling.
            /// Guarantees: The first fetch probes one row; subsequent fetches respect remaining bytes and the caller's configured row limit.
            #[test]
            fn fetch_size_adapts_to_observed_rows() {
                let mut size = FetchSize::default();
                let max_rows = super::super::tests::test_validated().common.max_rows();
                let budget = 8 * 1024 * 1024;
                assert_eq!(size.target(max_rows, budget), 1);
                size.observe(512);
                assert_eq!(size.target(300, budget), 300);
                assert_eq!(size.target(17, budget), 17);
                assert_eq!(size.target(max_rows, budget), max_rows);
                assert_eq!(size.target(12, budget), 12);
                size.observe(900 * 1024);
                assert_eq!(size.target(max_rows, budget), 9);
                assert_eq!(size.target(max_rows, 900 * 1024 - 1), 0);
                size.observe(1);
                assert_eq!(size.target(max_rows, budget), 9);
                assert_eq!(size.target(0, budget), 0);
            }

            /// Scenario: Sixty-four valid 900-KiB rows span several 8-MiB pages.
            /// Guarantees: Fetch estimates survive page boundaries without repeatedly requesting discarded rows.
            #[test]
            fn large_rows_do_not_repeat_full_fetches() {
                let mut size = FetchSize::default();
                let max_rows = super::super::tests::test_validated().common.max_rows();
                let row_bytes = 900 * 1024;
                let mut remaining = 64usize;
                let mut requested = 0;
                while remaining > 0 {
                    let mut used = 128 * 1024;
                    let mut page_rows = 0;
                    loop {
                        let count = size.target(
                            (max_rows - page_rows).min(remaining),
                            8 * 1024 * 1024 - used,
                        );
                        if count == 0 {
                            break;
                        }
                        requested += count;
                        for _ in 0..count {
                            size.observe(row_bytes);
                            used += row_bytes;
                            assert!(used <= 8 * 1024 * 1024);
                            page_rows += 1;
                            remaining -= 1;
                        }
                    }
                    assert!(page_rows > 0);
                }
                assert_eq!(requested, 64);
            }

            /// Scenario: A connection-driving task panics before worker cleanup joins it.
            /// Guarantees: Cleanup reports failure once, removes the session, and never polls the completed join handle again.
            #[tokio::test]
            async fn panicked_driver_cleanup_is_not_repolled() {
                tokio::task::LocalSet::new()
                    .run_until(async {
                        let driver: JoinHandle<Result<()>> = tokio::task::spawn_local(async {
                            panic!("injected driver failure");
                        });
                        let mut worker = Worker {
                            validated: super::super::tests::test_validated(),
                            session: Some(Session {
                                client: None,
                                driver,
                                statement: None,
                                catalog: None,
                                receive_failure: ReceiveFailure::default(),
                            }),
                            schema: None,
                            cancel: Rc::new(RefCell::new(None)),
                            poisoned: false,
                            principal: None,
                            fetch_size: FetchSize::default(),
                            timestamp_cache: convert::CursorTimestampCache::default(),
                        };
                        let baseline = super::super::tests::mock_signature(
                            &worker.validated.plan,
                            &[
                                (tokio_postgres::types::Type::TIMESTAMPTZ, 6, true),
                                (tokio_postgres::types::Type::INT8, -1, true),
                            ],
                        );
                        let _ =
                            resolve_schema(&mut worker.schema, &worker.validated, baseline.clone())
                                .expect("initial discovery");
                        assert_eq!(worker.close().await, Err(Error::Cleanup));
                        assert!(worker.session.is_none());
                        assert_eq!(
                            worker
                                .schema
                                .as_ref()
                                .expect("baseline survives close")
                                .signature,
                            baseline
                        );
                        assert!(worker.poisoned);
                        assert_eq!(worker.close().await, Ok(()));
                        assert!(
                            worker.poisoned,
                            "cleanup cannot clear the quarantined state"
                        );
                    })
                    .await;
            }
        }
    };
    () => {
        mod tests {
            use super::*;

            /// Scenario: PostgreSQL reports overload, transaction conflicts, timeout, or permanent failures.
            /// Guarantees: Transient SQLSTATEs and timeouts retry through the shared policy; auth/schema errors stay terminal.
            #[test]
            fn sqlstate_retry_policy() {
                for code in [
                    "53300", "08006", "57P01", "57P02", "57P03", "40001", "40P01",
                ] {
                    let error = sqlstate(code);
                    assert_eq!(error, Error::Unavailable);
                    assert!(PostgreSqlAdapter::is_retryable(&error));
                }
                let timeout = sqlstate("57014");
                assert_eq!(timeout, Error::Timeout);
                assert!(PostgreSqlAdapter::is_retryable(&timeout));
                for code in ["28P01", "42501", "42P01", "23505", "53200", "XX000"] {
                    let error = sqlstate(code);
                    assert_eq!(error, Error::Database);
                    assert!(!PostgreSqlAdapter::is_retryable(&error));
                }
                for error in [
                    Error::Limit,
                    Error::Value,
                    Error::Credential,
                    Error::Cleanup,
                ] {
                    assert!(!PostgreSqlAdapter::is_retryable(&error));
                }
            }

            /// Scenario: A worker panics after acknowledging cleanup but before its thread exits.
            /// Guarantees: A failed join remains terminal across repeated shutdown calls.
            #[tokio::test]
            async fn failed_join_remains_terminal() {
                let (commands, receiver) = mpsc::channel(1);
                let (sender, exit) = oneshot::channel();
                drop(receiver);
                let thread = std::thread::spawn(move || {
                    sender.send(Ok(())).expect("exit acknowledgment");
                    panic!("test worker exit failure");
                });
                let mut adapter = PostgreSqlAdapter {
                    credentials: crate::receivers::postgresql_receiver::tests::test_provider(),
                    commands,
                    operation: Arc::new(Operation::default()),
                    busy: Arc::new(AtomicBool::new(false)),
                    thread: Some(thread),
                    exit,
                    exit_result: None,
                    stopped: false,
                    timeout: Duration::from_secs(30),
                };
                for _ in 0..3 {
                    assert_eq!(adapter.shutdown().await, Err(Error::Cleanup));
                    assert!(!adapter.stopped);
                }
            }

            /// Scenario: The worker exits without sending its cleanup acknowledgment.
            /// Guarantees: Every shutdown attempt reports unconfirmed cleanup instead of repolling a consumed oneshot.
            #[tokio::test]
            async fn lost_exit_acknowledgment_remains_terminal() {
                let (commands, receiver) = mpsc::channel(1);
                let (sender, exit) = oneshot::channel();
                drop(receiver);
                drop(sender);
                let mut adapter = PostgreSqlAdapter {
                    credentials: crate::receivers::postgresql_receiver::tests::test_provider(),
                    commands,
                    operation: Arc::new(Operation::default()),
                    busy: Arc::new(AtomicBool::new(false)),
                    thread: None,
                    exit,
                    exit_result: None,
                    stopped: false,
                    timeout: Duration::from_secs(30),
                };
                for _ in 0..3 {
                    assert_eq!(adapter.shutdown().await, Err(Error::Cleanup));
                    assert!(!adapter.stopped);
                    assert!(adapter.begin_operation().is_err());
                }
            }
        }
    };
}

use super::{
    adapter::Error,
    adapter::Operation,
    config::PostgreSqlReceiverConfig as Config,
    query::{NativeColumn, Plan, ResolvedSchema, Signature, compile_parameters},
    value as convert,
};
use otel_arrow_dfe_engine::capability::{
    CapabilityError, CapabilityErrorSource,
    auth::{
        BasicAuthCredential,
        basic_auth_provider::{
            BasicAuthCredentialStream, BasicAuthProvider as BasicAuthCapability,
        },
    },
};
use otel_arrow_dfe_engine::local::capability::auth::basic_auth_provider::BasicAuthProvider;
use otel_arrow_dfe_scraper::database::{CellValue, DatabaseSystem, DriverAdapter};
use serde_json::{Value, json};
use std::sync::Arc;
use tokio_postgres::types::Type;

enum TestCredentials {
    Fixed,
    Pending,
    Failed,
}

fn fixed_credential() -> BasicAuthCredential {
    BasicAuthCredential::new("reader", "password").expect("fixture")
}

#[async_trait::async_trait(?Send)]
impl BasicAuthProvider for TestCredentials {
    async fn get_credential(&self) -> Result<BasicAuthCredential, CapabilityError> {
        match self {
            Self::Fixed => Ok(fixed_credential()),
            Self::Pending => futures::future::pending().await,
            Self::Failed => Err(CapabilityErrorSource::<BasicAuthCapability>::new(
                "PRIVATE_PROVIDER".into(),
            )
            .error("PRIVATE_PASSWORD")),
        }
    }

    fn credential_stream(&self) -> BasicAuthCredentialStream {
        match self {
            Self::Fixed => Box::pin(futures::stream::iter([fixed_credential()])),
            Self::Pending | Self::Failed => Box::pin(futures::stream::pending()),
        }
    }
}

pub(super) fn test_provider() -> Box<dyn BasicAuthProvider> {
    Box::new(TestCredentials::Fixed)
}

const KEY: &str =
    "((e.ts > :last_timestamp) OR (e.ts = :last_timestamp AND e.id > :last_tie_breaker))";

fn config(sql: &str) -> Value {
    let root = std::env::current_dir().expect("working directory");
    json!({
        "source_id": "test",
        "connection": {
            "host": "LOCALHOST", "database": "fixture"
        },
        "watermark": {
            "mode":"composite",
            "timestamp":{"column":"ts","initial":"2026-10-05T00:00:00.000000Z","timezone":"UTC"},
            "tie_breaker":{"column":"id","initial":0}
        },
        "query": {
            "statement": sql,
            "interval": "60s",
            "max_rows_per_poll": 1000,
            "max_batch_bytes": "8 MiB"
        },
        "checkpoint":{"directory":root.join("checkpoint"),"on_permanent_nack":"pause"}
    })
}
fn sql(predicate: &str) -> String {
    format!("SELECT e.ts, e.id FROM public.events AS e WHERE {predicate} ORDER BY e.ts, e.id")
}
fn validate(value: &Value) -> super::adapter::Result<super::config::Validated> {
    Config::parse(value)?.validate()
}

pub(super) fn test_validated() -> super::config::Validated {
    validate(&config(&sql(KEY))).expect("worker fixture")
}

pub(super) fn test_schema() -> ResolvedSchema {
    let validated = test_validated();
    let signature = mock_signature(
        &validated.plan,
        &[(Type::TIMESTAMPTZ, 6, true), (Type::INT8, -1, true)],
    );
    ResolvedSchema::discover(&validated.plan, &validated.common, &signature)
        .expect("resolved fixture")
}

pub(super) fn mock_signature(plan: &Plan, columns: &[(Type, i32, bool)]) -> Signature {
    assert_eq!(plan.projections.len(), columns.len());
    Signature {
        columns: plan
            .projections
            .iter()
            .zip(columns)
            .enumerate()
            .map(|(index, (reference, (native, modifier, not_null)))| {
                (
                    reference.clone(),
                    NativeColumn {
                        oid: 42,
                        attribute: index as i16 + 1,
                        type_oid: native.oid(),
                        modifier: *modifier,
                        not_null: *not_null,
                        collation: 0,
                    },
                )
            })
            .collect(),
        keys: vec![(42, vec![(43, vec![1, 2])])],
    }
}

/// Scenario: Schema discovery is requested for reordered projections with quoted output aliases.
/// Guarantees: SQL names and cursor positions are derived locally without inventing native types.
#[test]
fn discovery_plan_tracks_aliases_and_cursor_positions() {
    let source = sql(KEY).replace(
        "e.ts, e.id FROM",
        "e.id AS \"ID\", e.note AS message, e.ts AS event_time FROM",
    );
    let mut cfg = config(&source);
    cfg["watermark"]["timestamp"]["column"] = json!("event_time");
    cfg["watermark"]["tie_breaker"]["column"] = json!("id");
    cfg["query"]["output"] = json!({"timestamp_column": "event_time"});
    let validated = validate(&cfg).expect("local plan");
    let plan = &validated.plan;
    assert_eq!(plan.output_names, ["ID", "message", "event_time"]);
    assert_eq!(
        (plan.timestamp, plan.tie, plan.output_timestamp),
        (2, 0, Some(2))
    );
    let signature = mock_signature(
        plan,
        &[
            (Type::INT4, -1, true),
            (Type::TEXT, -1, false),
            (Type::TIMESTAMP, 3, true),
        ],
    );
    let schema =
        ResolvedSchema::discover(plan, &validated.common, &signature).expect("native types");
    assert_eq!(
        schema.native_types,
        [Type::INT4, Type::TEXT, Type::TIMESTAMP]
    );
    assert!(schema.columns[1].nullable);
    assert_eq!(schema.columns[2].type_modifier, 3);
}

/// Scenario: Queries have missing, ambiguous, repeated, or unqualified cursor outputs.
/// Guarantees: Invalid names and cursor provenance fail before database discovery.
#[test]
fn discovery_rejects_invalid_local_outputs() {
    for projection in [
        "e.ts",
        "e.id",
        "e.ts, e.id, e.note AS \"TS\"",
        "e.ts, e.id, e.ts AS repeated",
        "e.ts, e.id, e.id AS repeated",
        "e.ts AS other, e.id",
        "e.ts AS id, e.id",
        "ts, e.id",
        "e.*",
    ] {
        let source = sql(KEY).replace("e.ts, e.id FROM", &format!("{projection} FROM"));
        assert!(validate(&config(&source)).is_err(), "{projection}");
    }
    let mut cfg = config(&sql(KEY));
    cfg["watermark"]["tie_breaker"]["column"] = json!("ts");
    assert!(validate(&cfg).is_err(), "same cursor output");
    let mut cfg = config(&sql(KEY));
    cfg["query"]["output"] = json!({"timestamp_column": "missing"});
    assert!(validate(&cfg).is_err(), "missing event timestamp");
}

/// Scenario: Configuration includes a customer-authored result_schema field.
/// Guarantees: Manual schema fields are rejected as unknown, including nonempty arrays.
#[test]
fn manual_result_schema_is_rejected() {
    for schema in [
        json!([{"name":"ts","source_type":"timestamptz","nullable":false,"type_modifier":6}]),
        json!([]),
        Value::Null,
    ] {
        let mut cfg = config(&sql(KEY));
        cfg["query"]["result_schema"] = schema;
        assert!(Config::parse(&cfg).is_err());
        assert!(
            serde_json::from_value::<Config>(cfg)
                .err()
                .expect("unknown schema field")
                .to_string()
                .contains("unknown field `result_schema`")
        );
    }
}

/// Scenario: The catalog describes every supported native output, including nullable event time.
/// Guarantees: Discovered OIDs, typmods, names, and nullability select exact native decoders.
#[test]
fn discovery_resolves_supported_output_types() {
    let types = [
        Type::BOOL,
        Type::INT2,
        Type::INT4,
        Type::INT8,
        Type::NUMERIC,
        Type::FLOAT4,
        Type::FLOAT8,
        Type::TEXT,
        Type::VARCHAR,
        Type::BPCHAR,
        Type::BYTEA,
        Type::TIMESTAMP,
        Type::TIMESTAMPTZ,
        Type::DATE,
        Type::UUID,
        Type::JSON,
        Type::JSONB,
        Type::INTERVAL,
    ];
    let projection = (0..types.len())
        .map(|index| format!(", e.c{index}"))
        .collect::<String>();
    let source = sql(KEY).replace("e.ts, e.id FROM", &format!("e.ts, e.id{projection} FROM"));
    let mut cfg = config(&source);
    cfg["query"]["output"] = json!({"timestamp_column": "c11"});
    let validated = validate(&cfg).expect("local validation");
    let mut columns = vec![(Type::TIMESTAMPTZ, 6, true), (Type::INT8, -1, true)];
    columns.extend(types.iter().cloned().map(|native| (native, -1, false)));
    let signature = mock_signature(&validated.plan, &columns);
    let schema = ResolvedSchema::discover(&validated.plan, &validated.common, &signature)
        .expect("supported metadata");
    for (index, native) in types.iter().enumerate() {
        let column = &schema.columns[index + 2];
        assert_eq!(column.name, format!("c{index}"));
        assert_eq!(column.source_type, native.name());
        assert!(column.nullable);
        assert_eq!(column.type_modifier, -1);
        assert_eq!(&schema.native_types[index + 2], native);
    }
}

/// Scenario: Catalog discovery sees unsupported types, missing metadata, or nullable/wrongly typed cursors.
/// Guarantees: Invalid native contracts are rejected before preparing user SQL.
#[test]
fn discovery_rejects_invalid_native_contracts() {
    let source = sql(KEY).replace("e.ts, e.id FROM", "e.ts, e.id, e.note FROM");
    let validated = validate(&config(&source)).expect("local validation");
    let baseline = mock_signature(
        &validated.plan,
        &[
            (Type::TIMESTAMPTZ, 6, true),
            (Type::INT8, -1, true),
            (Type::TEXT, -1, false),
        ],
    );
    for (index, native, modifier, not_null) in [
        (0, Type::TIMESTAMPTZ, 6, false),
        (1, Type::INT8, -1, false),
        (0, Type::TEXT, -1, true),
        (1, Type::NUMERIC, -1, true),
        (2, Type::OID, -1, false),
        (2, Type::TEXT_ARRAY, -1, false),
        (0, Type::TIMESTAMP, 7, true),
        (0, Type::TIMESTAMP, -2, true),
    ] {
        let mut signature = baseline.clone();
        let column = signature
            .columns
            .get_mut(&validated.plan.projections[index])
            .expect("column");
        column.type_oid = native.oid();
        column.modifier = modifier;
        column.not_null = not_null;
        assert!(ResolvedSchema::discover(&validated.plan, &validated.common, &signature).is_err());
    }
    let mut signature = baseline.clone();
    signature
        .columns
        .get_mut(&validated.plan.projections[2])
        .expect("column")
        .type_oid = u32::MAX;
    assert!(ResolvedSchema::discover(&validated.plan, &validated.common, &signature).is_err());
    let _ = signature.columns.remove(&validated.plan.projections[2]);
    assert!(ResolvedSchema::discover(&validated.plan, &validated.common, &signature).is_err());
    let mut cfg = config(&source);
    cfg["query"]["output"] = json!({"timestamp_column": "note"});
    let event_time = validate(&cfg).expect("output name exists");
    assert!(ResolvedSchema::discover(&event_time.plan, &event_time.common, &baseline).is_err());
}

/// Scenario: Valid UTC configuration is later resolved to narrower native timestamp or integer precision.
/// Guarantees: Discovery checks exact initial-cursor representability while generic invalid UTC fails locally.
#[test]
fn discovery_checks_native_cursor_precision() {
    for (timestamp, precision, tie, native, valid) in [
        ("2026-01-01T00:00:00.123000Z", 3, 32767, Type::INT2, true),
        ("2026-01-01T00:00:00.123001Z", 3, 0, Type::INT8, false),
        ("2026-01-01T00:00:00Z", -1, 32768, Type::INT2, false),
        ("2026-01-01T00:00:00Z", 0, i32::MIN as i64, Type::INT4, true),
        (
            "2026-01-01T00:00:00Z",
            6,
            i32::MAX as i64 + 1,
            Type::INT4,
            false,
        ),
    ] {
        let mut cfg = config(&sql(KEY));
        cfg["watermark"]["timestamp"]["initial"] = json!(timestamp);
        cfg["watermark"]["tie_breaker"]["initial"] = json!(tie);
        let validated = validate(&cfg).expect("native precision not known locally");
        let signature = mock_signature(
            &validated.plan,
            &[(Type::TIMESTAMP, precision, true), (native, -1, true)],
        );
        assert_eq!(
            ResolvedSchema::discover(&validated.plan, &validated.common, &signature).is_ok(),
            valid
        );
    }
    for initial in [
        "2026-01-01T00:00:00.1234560Z",
        "2026-01-01T00:00:00.123456789Z",
        "2026-01-01T00:00:00+00:00",
        "2026-01-01 00:00:00Z",
        "2026-02-30T00:00:00Z",
    ] {
        let mut cfg = config(&sql(KEY));
        cfg["watermark"]["timestamp"]["initial"] = json!(initial);
        assert!(validate(&cfg).is_err(), "{initial}");
    }
}

/// Scenario: Both vendor identities are emitted by the existing shared mapper.
/// Guarantees: Adding PostgreSQL leaves the Oracle semantic identity unchanged.
#[test]
fn system_identity() {
    assert_eq!(DatabaseSystem::PostgreSQL.as_str(), "postgresql");
    assert_eq!(DatabaseSystem::Oracle.as_str(), "oracle.db");
}

/// Scenario: Named, custom, repeated, and native parameters occur in real token positions.
/// Guarantees: Roles are fixed and all unrelated UTF-8, CRLF, strings, comments, and casts survive byte-for-byte.
#[test]
fn parameter_compilation_preserves_bytes() {
    for source in [
        "SELECT '\u{e9}:ts', :id, :ts, :ts\r\n-- :id\n",
        "SELECT E'\\\\:ts', $$:id$$, \"x:ts\", :ts::timestamp, :id /* :ts */",
        "SELECT :ts /* a /* :id */ b */, :id",
    ] {
        let result = compile_parameters(source, "ts", "id").expect("compile");
        assert!(result.contains("$1"));
        assert!(result.contains("$2"));
        assert_eq!(
            compile_parameters(&result, "ts", "id").expect("native"),
            result
        );
    }
    assert_eq!(
        compile_parameters("SELECT '\u{e9}'\r\n, :id, :ts, :ts", "ts", "id").expect("compile"),
        "SELECT '\u{e9}'\r\n, $2, $1, $1"
    );
}

/// Scenario: Placeholders are missing, malformed, mixed, or present only in literals/comments.
/// Guarantees: No guessed mapping or accidental bind-looking text can authorize execution.
#[test]
fn invalid_parameters_fail_closed() {
    for text in [
        "SELECT :ts, $2",
        "SELECT :ts, :unknown",
        "SELECT :ts, : id",
        "SELECT :ts, :\"id\"",
        "SELECT :ts, ':id'",
        "SELECT :ts /* :id */",
        "SELECT $1, $0",
        "SELECT $1, $3",
        "SELECT $01, $2",
        "SELECT $1, ?",
        "SELECT :ts, :id;",
        "SELECT $$:ts$$, $$:id$$",
    ] {
        assert!(compile_parameters(text, "ts", "id").is_err(), "{text}");
    }
}

/// Scenario: Supported keysets, static predicates, aliases, and unique-side joins are authored.
/// Guarantees: The complete AST is admitted locally before later catalog proof.
#[test]
fn supported_sql_shapes() {
    for predicate in [
        KEY.to_owned(),
        "(e.ts, e.id) > (:last_timestamp, :last_tie_breaker)".into(),
        format!("{KEY} AND e.id >= -10"),
        format!("e.id IS NOT NULL AND ({KEY})"),
    ] {
        assert!(validate(&config(&sql(&predicate))).is_ok(), "{predicate}");
    }
    let joined = sql(KEY).replace(
        "public.events AS e",
        "public.events AS e INNER JOIN public.lookup AS l ON e.id = l.id",
    );
    assert!(validate(&config(&joined)).is_ok());
}

/// Scenario: Exact OR keysets occur alone, in parentheses, among filters, and with an inner join.
/// Guarantees: Only the keyset AST becomes a tuple comparison; all other SQL and the page limit match.
#[test]
fn keyset_optimization_preserves_query_tree() {
    use sqlparser::{dialect::PostgreSqlDialect, parser::Parser};

    let tuple = "((e.ts, e.id) > (:last_timestamp, :last_tie_breaker))";
    for predicate in [
        KEY.to_owned(),
        format!("(({KEY}))"),
        format!("e.id >= -10 AND {KEY}"),
        format!("{KEY} AND e.id IS NOT NULL"),
        format!("e.id >= -10 AND (({KEY})) AND e.id < 42"),
        format!("{KEY} AND e.note = 'caf\u{e9} :last_timestamp it''s $2'"),
    ] {
        for joined in [false, true] {
            let mut source = sql(&predicate);
            if joined {
                source = source.replace(
                    "public.events AS e",
                    "public.events AS e INNER JOIN public.lookup AS l ON e.id = l.id",
                );
            }
            let expected_source = source.replace(KEY, tuple);
            let original = validate(&config(&source)).expect("OR query");
            let native = validate(&config(&expected_source)).expect("tuple query");
            let expected = format!(
                "{} LIMIT {}",
                compile_parameters(&expected_source, "last_timestamp", "last_tie_breaker")
                    .expect("parameters"),
                original.common.max_rows()
            );
            let parse =
                |text: &str| Parser::parse_sql(&PostgreSqlDialect {}, text).expect("query tree");
            assert_eq!(parse(&original.plan.sql), parse(&expected), "{source}");
            assert_eq!(parse(&original.plan.sql), parse(&native.plan.sql));
            assert_eq!(original.common.sql(), source);
            assert_eq!(original.plan.referenced, native.plan.referenced);
            assert_eq!(original.plan.filters, native.plan.filters);
        }
    }
}

/// Scenario: Quoted aliases, cursor columns, and UTF-8 or escaped strings surround a nested OR keyset.
/// Guarantees: AST rewriting retains identifier spelling, literal values, projections, filters, and order.
#[test]
fn keyset_optimization_preserves_quoted_identifiers() {
    use sqlparser::{dialect::PostgreSqlDialect, parser::Parser};

    let source = concat!(
        "SELECT \"Event\".\"Ts\" AS ts, \"Event\".\"Id\" AS id ",
        "FROM \"public\".\"Events\" AS \"Event\" ",
        "WHERE \"Event\".\"Note\" = 'it''s :last_timestamp' AND ",
        "(((\"Event\".\"Ts\") > (:last_timestamp)) OR ",
        "(((\"Event\".\"Ts\") = (:last_timestamp)) AND ",
        "((\"Event\".\"Id\") > (:last_tie_breaker)))) ",
        "AND \"Event\".\"Other\" = '"
    );
    let source = format!("{source}\u{e9} $2' ORDER BY \"Event\".\"Ts\" ASC, \"Event\".\"Id\"");
    let optimized = validate(&config(&source)).expect("quoted query");
    let expected = format!(
        "SELECT \"Event\".\"Ts\" AS ts, \"Event\".\"Id\" AS id \
         FROM \"public\".\"Events\" AS \"Event\" \
         WHERE \"Event\".\"Note\" = 'it''s :last_timestamp' AND \
         ((\"Event\".\"Ts\", \"Event\".\"Id\") > ($1, $2)) \
         AND \"Event\".\"Other\" = '\u{e9} $2' \
         ORDER BY \"Event\".\"Ts\" ASC, \"Event\".\"Id\" LIMIT {}",
        optimized.common.max_rows()
    );
    assert_eq!(
        Parser::parse_sql(&PostgreSqlDialect {}, &optimized.plan.sql).expect("optimized"),
        Parser::parse_sql(&PostgreSqlDialect {}, &expected).expect("expected")
    );
    assert_eq!(optimized.common.sql(), source);
}

/// Scenario: A bounded query projects 128 columns with additional filter and join references.
/// Guarantees: Catalog requests can deduplicate each relation's complete metadata without losing provenance.
#[test]
fn wide_query_tracks_all_catalog_references() {
    let projection = std::iter::once("e.ts".to_owned())
        .chain(std::iter::once("e.id".to_owned()))
        .chain((0..126).map(|i| format!("e.c{i}")))
        .collect::<Vec<_>>()
        .join(", ");
    let source = format!(
        "SELECT {projection} FROM public.events AS e \
         INNER JOIN public.lookup AS l ON e.id = l.id \
         WHERE {KEY} AND e.c0 > 0 AND e.extra IS NOT NULL AND l.enabled = TRUE \
         ORDER BY e.ts, e.id"
    );
    let validated = validate(&config(&source)).expect("wide query");
    let plan = validated.plan;
    assert_eq!(plan.projections.len(), 128);
    assert_eq!(plan.referenced.len(), 131);
    for (alias, expected) in [("e", 129), ("l", 2)] {
        assert_eq!(
            plan.referenced.iter().filter(|c| c.alias == alias).count(),
            expected
        );
    }
    assert_eq!(plan.relations[1].join[0].0.alias, "e");
    assert_eq!(plan.relations[1].join[0].1.alias, "l");
    assert!(
        plan.sql
            .ends_with(&format!(" LIMIT {}", validated.common.max_rows()))
    );
}

/// Scenario: Legal SQL outside the experiment or an OR-bypass weakens the keyset contract.
/// Guarantees: Rejected statements never reach native prepare, including all extra query/select clauses.
#[test]
fn unsafe_and_unsupported_sql_rejected() {
    let base = sql(KEY);
    for text in [
        sql(&format!("{KEY} OR TRUE")),
        sql(&format!("{KEY} AND ({KEY})")),
        sql(&format!(
            "{KEY} AND (e.ts, e.id) > (:last_timestamp, :last_tie_breaker)"
        )),
        sql("e.ts >= :last_timestamp OR (e.ts = :last_timestamp AND e.id > :last_tie_breaker)"),
        sql("e.ts > :last_timestamp OR (e.ts = :last_timestamp AND e.id >= :last_tie_breaker)"),
        sql("e.ts > :last_timestamp OR (e.id > :last_tie_breaker AND e.ts = :last_timestamp)"),
        sql("e.ts > :last_timestamp AND e.id > :last_tie_breaker"),
        base.replace("e.ts, e.id FROM", "e.ts, abs(e.id) AS id FROM"),
        base.replace("SELECT", "SELECT DISTINCT"),
        base.replace(" FROM", " INTO public.copy FROM"),
        base.replace("e.ts, e.id FROM", "* FROM"),
        base.replace("public.events AS e", "(SELECT * FROM public.events) AS e"),
        base.replace(
            "public.events AS e",
            "public.events AS e LEFT JOIN public.x AS x ON e.id=x.id",
        ),
        base.replace("ORDER BY e.ts, e.id", "ORDER BY e.id, e.ts"),
        base.replace("ORDER BY e.ts, e.id", "ORDER BY e.ts DESC, e.id"),
        base.replace("ORDER BY e.ts, e.id", "ORDER BY e.ts NULLS FIRST, e.id"),
        format!("{base} LIMIT 1"),
        format!(
            "{base} LIMIT {}",
            validate(&config(&base)).expect("valid").common.max_rows()
        ),
        format!("{base} OFFSET 1"),
        format!("{base} FETCH FIRST 1 ROW ONLY"),
        format!("{base} FOR UPDATE"),
        format!("{base}; SELECT 1"),
        format!("{base} UNION {base}"),
        base.replace("ORDER BY", "GROUP BY e.ts, e.id ORDER BY"),
        format!("WITH x AS (DELETE FROM public.events) {base}"),
        sql(&format!(
            "{KEY} AND e.id = pg_catalog.set_config('a', 'b', false)"
        )),
    ] {
        assert!(validate(&config(&text)).is_err(), "{text}");
    }
}

/// Scenario: SQL exceeds bytes, lexical tokens, or nesting limits.
/// Guarantees: Resource bounds reject before recursive AST processing.
#[test]
fn parser_budgets() {
    assert!(
        compile_parameters(
            &format!("SELECT :ts, :id {}", " ".repeat(16384)),
            "ts",
            "id"
        )
        .is_err()
    );
    assert!(
        compile_parameters(
            &format!("SELECT :ts, :id {}", "x ".repeat(4096)),
            "ts",
            "id"
        )
        .is_err()
    );
    assert!(
        compile_parameters(
            &format!("SELECT {}:ts{}, :id", "(".repeat(17), ")".repeat(17)),
            "ts",
            "id"
        )
        .is_err()
    );
}

/// Scenario: Configured intervals are at, below, above, or fractional to the whole-second bounds.
/// Guarantees: 90s and 1.5m agree; no fractional result is rounded into acceptance.
#[test]
fn interval_boundaries() {
    for (interval, allowed) in [
        ("59s", false),
        ("60s", true),
        ("90s", true),
        ("1.5m", true),
        ("24h", true),
        ("24h1s", false),
        ("60.5s", false),
    ] {
        let mut cfg = config(&sql(KEY));
        cfg["query"]["interval"] = json!(interval);
        assert_eq!(validate(&cfg).is_ok(), allowed, "{interval}");
    }
}

/// Scenario: PostgreSQL omits catch-up settings or overrides either shared budget.
/// Guarantees: Oracle's 32-page/10-second defaults apply; overrides preserve checkpoints and per-page resource bounds.
#[test]
fn catch_up_matches_oracle_defaults_and_overrides() {
    let base = config(&sql(KEY));
    let fingerprint = validate(&base).expect("default config").fingerprint;
    for (override_value, pages, millis) in [
        (None, 32, 10_000),
        (Some(json!({"max_pages": 2})), 2, 10_000),
        (Some(json!({"max_duration": "50ms"})), 32, 50),
        (
            Some(json!({"max_pages": 1024, "max_duration": "5m"})),
            1024,
            300_000,
        ),
    ] {
        let mut value = base.clone();
        if let Some(override_value) = override_value {
            value["query"]["catch_up"] = override_value;
        }
        let validated = validate(&value).expect("valid catch-up");
        assert_eq!(validated.fingerprint, fingerprint);
        let query = validated.common;
        assert_eq!(query.catch_up().max_pages, pages);
        assert_eq!(
            query.catch_up().max_duration,
            std::time::Duration::from_millis(millis)
        );
        assert_eq!(query.fetch_size_rows(), 300);
        assert_eq!(query.max_rows(), 1000);
        assert_eq!(query.max_batch_bytes(), 8 * 1024 * 1024);
        assert_eq!(query.max_normalized_bytes(), 8 * 1024 * 1024);
    }
}

/// Scenario: PostgreSQL supplies catch-up values outside Oracle's shared limits or adds unknown fields.
/// Guarantees: Invalid budgets are rejected instead of disabling limits or silently falling back.
#[test]
fn invalid_catch_up_settings_are_rejected() {
    for catch_up in [
        json!({"max_pages": 0}),
        json!({"max_pages": 1025}),
        json!({"max_duration": "0ms"}),
        json!({"max_duration": "5m1ms"}),
        json!({"unlimited": true}),
        Value::Null,
    ] {
        let mut value = config(&sql(KEY));
        value["query"]["catch_up"] = catch_up;
        assert!(validate(&value).is_err());
    }
}

/// Scenario: PostgreSQL omits or overrides the Oracle-style fetch-size setting.
/// Guarantees: The default remains 300, valid overrides reach the compiled query, and changing fetch size preserves checkpoint identity.
#[test]
fn fetch_size_defaults_and_overrides() {
    let base = config(&sql(KEY));
    let default = validate(&base).expect("default");
    assert_eq!(default.common.fetch_size_rows(), 300);
    for rows in [1, 17, 300, 1000] {
        let mut value = base.clone();
        value["query"]["fetch_size_rows"] = json!(rows);
        let configured = validate(&value).expect("configured fetch size");
        assert_eq!(configured.common.fetch_size_rows(), rows);
        assert_eq!(configured.fingerprint, default.fingerprint);
    }
}

/// Scenario: Fetch size is zero, exceeds the configured page size, or has an invalid type.
/// Guarantees: Invalid values fail explicitly rather than being clamped or silently replaced with a default.
#[test]
fn invalid_fetch_sizes_are_rejected() {
    for rows in [
        json!(0),
        json!(1001),
        json!(10000),
        json!(-1),
        json!(1.5),
        json!("300"),
        Value::Null,
    ] {
        let mut value = config(&sql(KEY));
        value["query"]["fetch_size_rows"] = rows;
        assert!(validate(&value).is_err());
    }
}

/// Scenario: Required page budgets are supplied while operational settings are omitted or changed.
/// Guarantees: Oracle defaults, shared bounds, configured SQL limits, and durable identity all agree.
#[test]
fn configurable_page_budgets_and_defaults() {
    let mut base = config(&sql(KEY));
    let _ = base["query"]
        .as_object_mut()
        .expect("query")
        .remove("interval");
    let default = validate(&base).expect("defaults");
    assert_eq!(default.common.timeout(), std::time::Duration::from_secs(30));
    assert_eq!(
        default.common.interval(),
        std::time::Duration::from_secs(60)
    );
    assert_eq!(default.common.fetch_size_rows(), 300);
    for (rows, fetch, bytes, timeout) in [
        (1, 1, json!(1), "1s"),
        (10_000, 10_000, json!("10 MiB"), "5m"),
        (10_000, 300, json!(10 * 1024 * 1024), "30s"),
        (10_000, 300, json!("256 MiB"), "90s"),
    ] {
        let mut value = base.clone();
        value["query"]["max_rows_per_poll"] = json!(rows);
        value["query"]["fetch_size_rows"] = json!(fetch);
        value["query"]["max_batch_bytes"] = bytes;
        value["query"]["timeout"] = json!(timeout);
        let configured = validate(&value).expect("configured budgets");
        assert_eq!(configured.common.max_rows(), rows);
        assert_eq!(configured.common.fetch_size_rows(), fetch);
        assert_eq!(
            configured.common.max_normalized_bytes(),
            configured.config.query.max_batch_bytes
        );
        assert_eq!(
            configured.common.max_batch_bytes(),
            configured.config.query.max_batch_bytes
        );
        assert_eq!(configured.common.timeout(), configured.config.query.timeout);
        assert!(configured.plan.sql.ends_with(&format!(" LIMIT {rows}")));
        assert_eq!(configured.fingerprint, default.fingerprint);
    }
    let mut string_bytes = base.clone();
    string_bytes["query"]["max_batch_bytes"] = json!("10 MiB");
    let mut numeric_bytes = string_bytes.clone();
    numeric_bytes["query"]["max_batch_bytes"] = json!(10 * 1024 * 1024);
    assert_eq!(
        validate(&string_bytes)
            .expect("string bytes")
            .common
            .max_batch_bytes(),
        validate(&numeric_bytes)
            .expect("numeric bytes")
            .common
            .max_batch_bytes()
    );
}

/// Scenario: Required budgets are omitted or malformed, outside shared bounds, or inconsistent with fetch size.
/// Guarantees: No implicit row/byte defaults, clamping, fractional timeout, or disabled limits are accepted.
#[test]
fn invalid_page_budgets_and_timeouts_are_rejected() {
    let base = config(&sql(KEY));
    for field in ["max_rows_per_poll", "max_batch_bytes"] {
        let mut value = base.clone();
        let _ = value["query"].as_object_mut().expect("query").remove(field);
        assert!(validate(&value).is_err(), "missing {field}");
    }
    for (field, invalid) in [
        (
            "max_rows_per_poll",
            vec![
                json!(0),
                json!(-1),
                json!(1.5),
                json!(10_001),
                json!("1000"),
                Value::Null,
            ],
        ),
        (
            "max_batch_bytes",
            vec![
                json!(0),
                json!(-1),
                json!(1.5),
                json!(256 * 1024 * 1024 + 1),
                json!("257 MiB"),
                json!("invalid"),
                Value::Null,
            ],
        ),
        (
            "timeout",
            vec![
                json!("0s"),
                json!("-1s"),
                json!("500ms"),
                json!("1.5s"),
                json!("301s"),
                json!(30),
                Value::Null,
            ],
        ),
    ] {
        for invalid in invalid {
            let mut value = base.clone();
            value["query"][field] = invalid.clone();
            assert!(validate(&value).is_err(), "{field}: {invalid}");
        }
    }
    let mut value = base;
    value["query"]["max_rows_per_poll"] = json!(299);
    assert!(validate(&value).is_err(), "default fetch exceeds page");
    value["query"]["max_rows_per_poll"] = json!(10_000);
    value["query"]["fetch_size_rows"] = json!(10_001);
    assert!(validate(&value).is_err(), "shared fetch ceiling");
}

/// Scenario: An optional event timestamp is NULL or outside OTLP's unsigned nanosecond range.
/// Guarantees: PostgreSQL values reach the shared mapper unchanged and use observation time without changing the cursor.
#[test]
fn optional_event_time_uses_shared_fallback() {
    use otel_arrow_dfe_pdata::{
        PayloadData,
        otlp::OtlpProtoBytes,
        proto::opentelemetry::{common::v1::any_value, logs::v1::LogsData},
    };
    use otel_arrow_dfe_scraper::database::{
        ColumnMetadata, Cursor, CursorRow, OutputConfig, QueryPage, Row, encode_page,
    };
    use prost::Message;
    let epoch = chrono::NaiveDate::from_ymd_opt(2000, 1, 1)
        .expect("epoch")
        .and_hms_opt(0, 0, 0)
        .expect("midnight");
    let mut values = vec![CellValue::Null];
    for year in [1960, 3000] {
        let time = chrono::NaiveDate::from_ymd_opt(year, 1, 1)
            .expect("date")
            .and_hms_opt(0, 0, 0)
            .expect("time");
        let bytes = time
            .signed_duration_since(epoch)
            .num_microseconds()
            .expect("micros")
            .to_be_bytes();
        values.push(convert::decode(&Type::TIMESTAMPTZ, &bytes).expect("native timestamp"));
    }
    for value in values {
        let is_null = matches!(value, CellValue::Null);
        let expected = match &value {
            CellValue::Null => None,
            CellValue::TimestampTz(text) => Some(any_value::Value::StringValue(text.clone())),
            _ => panic!("timestamp fixture"),
        };
        let cursor = Cursor::composite("2026-10-05T00:00:00Z".into(), 1);
        let encoded = encode_page(
            QueryPage {
                columns: vec![ColumnMetadata {
                    name: "occurred".into(),
                    source_type: "timestamptz".into(),
                    nullable: true,
                }],
                rows: vec![CursorRow {
                    row: Row {
                        values: vec![value],
                    },
                    cursor: cursor.clone(),
                }],
            },
            DatabaseSystem::PostgreSQL,
            "fixture",
            &OutputConfig {
                timestamp_column: Some("occurred".into()),
                validation_columns: vec![],
            },
            123,
            1024,
        )
        .expect("mapping")
        .expect("one row");
        assert_eq!(encoded.row_count, 1);
        assert_eq!(encoded.candidate, cursor);
        assert_eq!(encoded.event_time_fallbacks, usize::from(!is_null));
        let PayloadData::OtlpBytes(OtlpProtoBytes::ExportLogsRequest(bytes)) =
            encoded.pdata.payload().into_data()
        else {
            panic!("OTLP logs");
        };
        let logs = LogsData::decode(bytes).expect("logs");
        let record = &logs.resource_logs[0].scope_logs[0].log_records[0];
        assert_eq!(record.time_unix_nano, 123);
        let Some(any_value::Value::KvlistValue(body)) =
            record.body.as_ref().and_then(|body| body.value.as_ref())
        else {
            panic!("structured row");
        };
        assert_eq!(
            body.values[0]
                .value
                .as_ref()
                .and_then(|value| value.value.clone()),
            expected
        );
    }
}

/// Scenario: A PostgreSQL cursor is configured using UTC ISO 8601 text or another timestamp notation.
/// Guarantees: Only the T/Z form is accepted, with exact native precision rather than offset conversion or truncation.
#[test]
fn cursor_configuration_requires_iso_utc() {
    for (initial, valid) in [
        ("2026-01-01T00:00:00Z", true),
        ("2026-01-01T00:00:00.123456Z", true),
        ("2026-01-01T00:00:00.123456000Z", false),
        ("2026-01-01 00:00:00", false),
        ("2026-01-01 00:00:00Z", false),
        ("2026-01-01T00:00:00+00:00", false),
        ("2026-01-01T05:30:00+05:30", false),
        ("2026-01-01T00:00:00", false),
        ("2026-01-01", false),
        ("2026-01-01T00:00:00.123456789Z", false),
    ] {
        let mut value = config(&sql(KEY));
        value["watermark"]["timestamp"]["initial"] = json!(initial);
        assert_eq!(validate(&value).is_ok(), valid, "{initial}");
    }
}

/// Scenario: PostgreSQL's console example obtains both credentials through the shared provider.
/// Guarantees: The example references username/password files and preserves the required capability binding.
#[test]
fn console_example_uses_shared_file_credentials() {
    let value: Value = serde_yaml::from_str(include_str!(
        "../../../../../configs/postgresql-console.yaml"
    ))
    .expect("example YAML");
    let pipeline = &value["groups"]["default"]["pipelines"]["main"];
    let provider = &pipeline["extensions"]["pg-credentials"];
    assert_eq!(
        provider["type"],
        "urn:otel:extension:flat_file_user_pass_auth"
    );
    assert!(provider["config"].get("username").is_none());
    assert_eq!(
        provider["config"]["username_file"],
        "${env:PG_USERNAME_FILE:-/run/secrets/postgresql/username}"
    );
    assert_eq!(
        provider["config"]["password_secret_file"],
        "${env:PG_PASSWORD_FILE:-/run/secrets/postgresql/password}"
    );
    assert_eq!(
        pipeline["nodes"]["pg"]["capabilities"]["basic_auth_provider"],
        "pg-credentials"
    );
    let query = &pipeline["nodes"]["pg"]["config"]["query"];
    assert_eq!(query["max_rows_per_poll"], 10_000);
    assert_eq!(query["max_batch_bytes"], "10 MiB");
    assert_eq!(query["timeout"], "30s");
    assert!(query.get("result_schema").is_none());
    let parsed = Config::parse(&pipeline["nodes"]["pg"]["config"]).expect("receiver YAML");
    assert_eq!(parsed.query.max_batch_bytes, 10 * 1024 * 1024);
}

/// Scenario: Operators add unsupported fields or omit required lower-bound information.
/// Guarantees: Strict objects reject unsafe settings without printing their values.
#[test]
fn strict_configuration() {
    let base = config(&sql(KEY));
    for pointer in [
        "/connection/password",
        "/connection/tls",
        "/query/output/attributes",
        "/query/queries",
        "/checkpoint/unknown",
        "/unknown",
    ] {
        let mut cfg = base.clone();
        let (parent, key) = pointer.rsplit_once('/').expect("pointer");
        if parent == "/query/output" {
            cfg["query"]["output"] = json!({});
        }
        let _ = cfg
            .pointer_mut(parent)
            .expect("parent")
            .as_object_mut()
            .expect("object")
            .insert(key.into(), json!("diagnostic-canary"));
        assert!(validate(&cfg).is_err(), "{pointer}");
    }
    for field in ["initial", "timezone"] {
        let mut cfg = base.clone();
        let _ = cfg["watermark"]["timestamp"]
            .as_object_mut()
            .expect("object")
            .remove(field);
        assert!(validate(&cfg).is_err());
    }
    for host in [
        "postgres://user:pass@host/db",
        "a,b",
        "/socket",
        "a b",
        "a@b",
        "-bad",
    ] {
        let mut cfg = base.clone();
        cfg["connection"]["host"] = json!(host);
        assert!(validate(&cfg).is_err());
    }
}

/// Scenario: Operators tune polling, edit source semantics, or author equivalent keysets.
/// Guarantees: Auto-only identity hashes original SQL, excludes policy tuning, and changes with source semantics.
#[test]
fn discovery_checkpoint_identity_is_policy_stable() {
    let source = sql(KEY);
    let cfg = config(&source);
    let validated = validate(&cfg).expect("automatic config");
    let original = &validated.fingerprint;
    let configured = &validated.config;
    let identity = serde_json::to_vec(&(
        "postgresql/v4-auto",
        "postgresql",
        &configured.connection.host,
        configured.connection.port,
        &configured.connection.database,
        &source,
        "named-parameters/v2",
        &configured.watermark,
        &configured.query.output,
        "UTC/microseconds/native-v2",
    ))
    .expect("auto identity");
    assert_eq!(original, &blake3::hash(&identity).to_hex().to_string());
    let tuple = validate(&config(&sql(
        "((e.ts, e.id) > (:last_timestamp, :last_tie_breaker))",
    )))
    .expect("tuple");
    assert_eq!(validated.plan.sql, tuple.plan.sql);
    assert_ne!(original, &tuple.fingerprint);
    for (field, value) in [
        ("interval", json!("90s")),
        ("timeout", json!("60s")),
        ("max_rows_per_poll", json!(2000)),
        ("max_batch_bytes", json!("16 MiB")),
        ("fetch_size_rows", json!(1)),
        ("catch_up", json!({"max_pages": 1, "max_duration": "50ms"})),
    ] {
        let mut tuned = cfg.clone();
        tuned["query"][field] = value;
        assert_eq!(
            &validate(&tuned).expect("policy edit").fingerprint,
            original
        );
    }
    let mut tuned = cfg.clone();
    tuned["checkpoint"]["on_permanent_nack"] = json!("retry");
    assert_eq!(
        &validate(&tuned).expect("checkpoint policy").fingerprint,
        original
    );
    for (pointer, value) in [
        ("/connection/host", json!("other-host")),
        ("/connection/database", json!("other_db")),
        ("/watermark/tie_breaker/initial", json!(-1)),
        (
            "/query/statement",
            json!(source.replace("SELECT", "SELECT ")),
        ),
    ] {
        let mut changed = cfg.clone();
        *changed.pointer_mut(pointer).expect("field") = value;
        assert_ne!(
            &validate(&changed).expect("source edit").fingerprint,
            original
        );
    }
    let mut changed = cfg;
    changed["query"]["output"] = json!({"timestamp_column": "ts"});
    assert_ne!(
        &validate(&changed).expect("output edit").fingerprint,
        original
    );
}

fn number(weight: i16, sign: u16, scale: u16, digits: &[u16]) -> Vec<u8> {
    let mut bytes = vec![];
    bytes.extend_from_slice(&(digits.len() as i16).to_be_bytes());
    bytes.extend_from_slice(&weight.to_be_bytes());
    bytes.extend_from_slice(&sign.to_be_bytes());
    bytes.extend_from_slice(&scale.to_be_bytes());
    for d in digits {
        bytes.extend_from_slice(&d.to_be_bytes());
    }
    bytes
}

/// Scenario: NUMERIC encodes large exact integers, preserved dscale, fractions, and malformed groups.
/// Guarantees: No float rounding, silent scale truncation, special-value acceptance, or huge allocation occurs.
#[test]
fn exact_numeric() {
    for (bytes, expected) in [
        (
            number(3, 0, 0, &[9007, 1992, 5474, 993]),
            "9007199254740993",
        ),
        (number(0, 0x4000, 4, &[12, 3400]), "-12.3400"),
        (number(-2, 0, 8, &[1]), "0.00000001"),
        (number(0, 0, 3, &[]), "0.000"),
        (number(1, 0, 0, &[1]), "10000"),
    ] {
        assert_eq!(convert::numeric(&bytes).expect("numeric"), expected);
    }
    for bytes in [
        number(0, 0xC000, 0, &[]),
        number(0, 0, 0, &[10000]),
        number(-1, 0, 2, &[1234]),
        number(32767, 0, 0, &[1]),
        vec![0; 7],
    ] {
        assert!(convert::numeric(&bytes).is_err());
    }
    assert_eq!(
        convert::numeric(&number(4095, 0, 0, &[1000]))
            .expect("boundary")
            .len(),
        16384
    );
    assert!(convert::numeric(&number(4096, 0, 0, &[1])).is_err());
}

/// Scenario: Temporal cursors and native integer ties approach their precision/range boundaries.
/// Guarantees: Native binding cannot silently truncate submicroseconds, typmod digits, leap seconds, or integers.
#[test]
fn cursor_precision_and_range() {
    for (text, precision, valid) in [
        ("2026-10-05T00:00:00.123000Z", 3, true),
        ("2026-10-05T00:00:00.123001Z", 3, false),
        ("2026-10-05T00:00:00.123456789Z", 6, false),
        ("2026-10-05T00:00:00.123456000Z", 6, false),
        ("2026-02-30T00:00:00Z", 6, false),
        ("2026-10-05T00:00:60Z", 6, false),
        ("infinity", 6, false),
    ] {
        assert_eq!(convert::cursor_time(text, precision).is_ok(), valid);
    }
    assert!(convert::check_tie(32767, "int2").is_ok());
    assert!(convert::check_tie(32768, "int2").is_err());
    assert!(convert::check_tie(i32::MIN as i64, "int4").is_ok());
    assert!(convert::check_tie(i32::MIN as i64 - 1, "int4").is_err());
    assert!(convert::check_tie(i64::MAX, "int8").is_ok());
}

/// Scenario: Supported exact codecs and unsupported/nonfinite source values are decoded.
/// Guarantees: JSON lexemes, bytes, padding and interval units survive; invalid data never stringifies.
#[test]
fn exact_values() {
    let json = br#"{"n":9007199254740993,"x":1.2345678901234567890123456789}"#;
    assert_eq!(
        convert::decode(&Type::JSON, json).expect("json"),
        CellValue::String(std::str::from_utf8(json).expect("utf8").into())
    );
    assert_eq!(
        convert::decode(&Type::BYTEA, &[0, 255]).expect("bytes"),
        CellValue::Bytes(vec![0, 255])
    );
    assert_eq!(
        convert::decode(&Type::BPCHAR, b"x   ").expect("padding"),
        CellValue::String("x   ".into())
    );
    let interval = [
        (-9i64).to_be_bytes().to_vec(),
        2i32.to_be_bytes().to_vec(),
        (-1i32).to_be_bytes().to_vec(),
    ]
    .concat();
    assert_eq!(
        convert::decode(&Type::INTERVAL, &interval).expect("interval"),
        CellValue::Interval("months=-1;days=2;microseconds=-9".into())
    );
    assert_eq!(
        convert::decode(&Type::UUID, &[0; 16]).expect("uuid"),
        CellValue::String("00000000-0000-0000-0000-000000000000".into())
    );
    for (ty, bytes) in [
        (Type::JSONB, vec![2, b'{', b'}']),
        (Type::TEXT, vec![255]),
        (Type::FLOAT8, f64::NAN.to_be_bytes().to_vec()),
        (Type::OID, vec![0; 4]),
        (Type::TIMESTAMP, i64::MAX.to_be_bytes().to_vec()),
    ] {
        assert!(convert::decode(&ty, &bytes).is_err());
    }
}

/// Scenario: Cancellation is requested before a connection or client token exists.
/// Guarantees: The operation generation is independently cancellable and later work observes it.
#[test]
fn disconnected_cancellation() {
    let first = Operation::default();
    first.cancel();
    assert_eq!(first.check(), Err(Error::Cancelled));
    assert!(Operation::default().check().is_ok());
    assert!(!super::adapter::PostgreSqlAdapter::is_retryable(
        &Error::Cleanup
    ));
}

/// Scenario: Diagnostics format typed errors whose underlying inputs could contain secrets.
/// Guarantees: Only static bounded messages are published, without a native source chain.
#[test]
fn redacted_errors() {
    use std::error::Error as _;
    for error in [
        Error::Config,
        Error::Sql,
        Error::Metadata,
        Error::Value,
        Error::Limit,
        Error::Credential,
        Error::Database,
        Error::Unavailable,
        Error::Cancelled,
        Error::Timeout,
        Error::Cleanup,
    ] {
        assert!(error.to_string().len() < 100);
        assert!(error.source().is_none());
    }
}

/// Scenario: A worker receives shutdown without ever connecting to a database.
/// Guarantees: Exit is acknowledged, the thread is joined, and repeated shutdown is idempotent.
#[tokio::test]
async fn disconnected_worker_shutdown() {
    let mut adapter =
        super::adapter::PostgreSqlAdapter::new(test_validated(), test_provider()).expect("worker");
    tokio::time::timeout(std::time::Duration::from_secs(2), adapter.shutdown())
        .await
        .expect("bounded shutdown")
        .expect("confirmed exit");
    adapter.shutdown().await.expect("idempotent shutdown");
    assert!(adapter.begin_operation().is_err());
}

/// Scenario: A provider waits indefinitely before the first database connection and shutdown cancels its operation.
/// Guarantees: The credential wait ends promptly and the idle worker can be joined without opening a connection.
#[tokio::test]
async fn provider_wait_is_cancellable() {
    use otel_arrow_dfe_scraper::database::DriverCancellation;
    let validated = test_validated();
    let query = validated.common.clone();
    let mut adapter =
        super::adapter::PostgreSqlAdapter::new(validated, Box::new(TestCredentials::Pending))
            .expect("worker");
    let cancellation = adapter.begin_operation().expect("operation");
    {
        let operation = adapter.validate_query(&query);
        tokio::pin!(operation);
        assert!(futures::poll!(&mut operation).is_pending());
        cancellation.cancel().await.expect("cancel");
        assert!(matches!(
            tokio::time::timeout(std::time::Duration::from_secs(1), operation)
                .await
                .expect("prompt cancel"),
            Err(Error::Cancelled),
        ));
    }
    adapter.shutdown().await.expect("worker joined");
}

/// Scenario: A provider never produces credentials and no cancellation arrives.
/// Guarantees: The bounded acquisition wait fails explicitly rather than stalling the receiver indefinitely.
#[tokio::test(start_paused = true)]
async fn provider_wait_has_a_deadline() {
    let validated = test_validated();
    let query = validated.common.clone();
    let mut adapter =
        super::adapter::PostgreSqlAdapter::new(validated, Box::new(TestCredentials::Pending))
            .expect("worker");
    _ = adapter.begin_operation().expect("operation");
    assert!(matches!(
        adapter.validate_query(&query).await,
        Err(Error::Credential)
    ));
    adapter.shutdown().await.expect("worker joined");
}

/// Scenario: A credential capability returns sensitive provider details in its error.
/// Guarantees: PostgreSQL exposes only a terminal credential category with no secret text or source chain.
#[tokio::test]
async fn provider_errors_are_redacted() {
    use std::error::Error as _;
    let validated = test_validated();
    let query = validated.common.clone();
    let mut adapter =
        super::adapter::PostgreSqlAdapter::new(validated, Box::new(TestCredentials::Failed))
            .expect("worker");
    _ = adapter.begin_operation().expect("operation");
    let error = adapter
        .validate_query(&query)
        .await
        .expect_err("credential error");
    assert_eq!(error, Error::Credential);
    assert!(!super::adapter::PostgreSqlAdapter::is_retryable(&error));
    assert!(error.source().is_none());
    assert!(!format!("{error:?} {error}").contains("PRIVATE"));
    adapter.shutdown().await.expect("worker joined");
}

/// Scenario: PostgreSQL configuration supplies a TLS block, old inline credentials, or a scalar watermark.
/// Guarantees: Unsupported transport/authentication fields and cursor modes fail rather than being ignored or reinterpreted.
#[test]
fn unsupported_transport_and_cursor_modes_fail_closed() {
    for (field, value) in [
        ("tls", json!({"ca_file": "ca.pem"})),
        ("sslmode", json!("require")),
        ("username", json!("user")),
        ("password_file", json!("password")),
    ] {
        let mut cfg = config(&sql(KEY));
        cfg["connection"][field] = value;
        assert!(validate(&cfg).is_err());
    }
    let mut cfg = config(&sql(KEY));
    cfg["watermark"] = json!({"mode":"scalar","column":"id","bind":"last_id","initial":{"type":"int64","value":0}});
    assert!(validate(&cfg).is_err());
}

/// Scenario: A PostgreSQL receiver is constructed without a credential capability.
/// Guarantees: The required binding is reported before source leases or database connections are acquired.
#[test]
fn factory_requires_basic_auth_provider() {
    use otel_arrow_dfe_engine::testing::{receiver::TestRuntime, test_node};
    let runtime = TestRuntime::<otel_arrow_dfe_otap::pdata::OtapPdata>::new();
    let result = (super::POSTGRESQL_RECEIVER.create)(
        otel_arrow_dfe_engine::context::ControllerContext::new(
            otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle::new(),
        )
        .pipeline_context_with("group".into(), "pipeline".into(), 0, 1, 0),
        test_node("postgresql"),
        Arc::new(
            otel_arrow_dfe_config::node::NodeUserConfig::new_receiver_config(
                super::POSTGRESQL_RECEIVER_URN,
            ),
        ),
        runtime.config(),
        &otel_arrow_dfe_engine::capability::registry::Capabilities::empty(),
    );
    assert!(
        matches!(result, Err(otel_arrow_dfe_config::error::Error::InvalidUserConfig { error }) if error.contains("basic_auth_provider"))
    );
}

/// Scenario: A caller supplies a scalar cursor to a composite PostgreSQL query.
/// Guarantees: The current shared Cursor API is checked before credential or database work begins.
#[tokio::test]
async fn scalar_cursor_cannot_execute_as_composite() {
    use otel_arrow_dfe_scraper::database::{Cursor, ScalarValue};
    let validated = test_validated();
    let query = validated.common.clone();
    let mut adapter =
        super::adapter::PostgreSqlAdapter::new(validated, test_provider()).expect("worker");
    assert!(matches!(
        adapter
            .execute(&query, &Cursor::Scalar(ScalarValue::Int64(1)))
            .await,
        Err(Error::Value),
    ));
    adapter.shutdown().await.expect("worker joined");
}
