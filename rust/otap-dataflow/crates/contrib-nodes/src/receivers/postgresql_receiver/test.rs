// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// Private adapter tests remain colocated logically without exposing implementation internals.
macro_rules! postgresql_module_tests {
    (framing) => {
        mod framing_tests {
            use super::*;
            use tokio::io::AsyncReadExt;

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
                BoundedBackend::new(
                    Fragmented {
                        bytes,
                        consumed: Rc::default(),
                        fragment,
                        pending: true,
                        error_at_end: None,
                    },
                    ReceiveFailure::default(),
                    frame_limits(),
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

            /// Scenario: The peer closes, resets, or reports UnexpectedEof at each header/body offset.
            /// Guarantees: Clean EOF, retryable truncation, original errors, and checked prefixes stay distinct.
            #[tokio::test]
            async fn eof_truncation_and_transport_errors() {
                let complete = frame(b'D', b"row");
                for fragment in [1, 2] {
                    for failure in [
                        None,
                        Some(io::ErrorKind::ConnectionReset),
                        Some(io::ErrorKind::UnexpectedEof),
                    ] {
                        for length in 0..complete.len() {
                            let mut stream = fragmented(complete[..length].to_vec(), fragment);
                            stream.inner.error_at_end = failure;
                            let mut output = Vec::new();
                            let result = stream.read_to_end(&mut output).await;
                            assert_eq!(
                                output,
                                if length < 5 {
                                    &[][..]
                                } else {
                                    &complete[..length]
                                }
                            );
                            if failure.is_none() && length == 0 {
                                assert_eq!(result.expect("clean EOF"), 0);
                                assert_eq!(stream.failure.get(), None);
                                continue;
                            }
                            let error = result.expect_err("interrupted frame");
                            let kind = failure.unwrap_or(io::ErrorKind::UnexpectedEof);
                            assert_eq!(error.kind(), kind);
                            if kind == io::ErrorKind::UnexpectedEof {
                                assert_eq!(stream.failure.get(), Some(Error::Unavailable));
                            }
                            if failure.is_none() {
                                assert!(error.to_string().contains(if length < 5 {
                                    "header"
                                } else {
                                    "body"
                                }));
                            }
                        }
                    }
                }
            }
        }
    };
    (worker) => {
        mod tests {
            use super::*;

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
                    "53300", "08006", "55P03", "57P01", "57P02", "57P03", "57P05", "40001", "40P01",
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
        }
    };
}

use super::{
    adapter::Error,
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
use otel_arrow_dfe_scraper::database::DriverAdapter;
use serde_json::{Value, json};
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
        "connection": {"host": "LOCALHOST", "database": "fixture"},
        "watermark": {
            "mode":"composite",
            "timestamp":{"column":"ts","initial":"2026-10-05T00:00:00.000000Z","timezone":"UTC"},
            "tie_breaker":{"column":"id","initial":0}
        },
        "query": {
            "statement": sql, "interval": "60s",
            "max_rows_per_poll": 1000, "max_batch_bytes": "8 MiB"
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

/// Scenario: Catalog discovery sees unsupported types, missing metadata, or nullable/wrongly typed cursors.
/// Guarantees: Invalid native contracts are rejected before preparing user SQL.
#[test]
fn discovery_rejects_invalid_native_contracts() {
    let source = sql(KEY).replace("e.ts, e.id FROM", "e.ts, e.id, e.note FROM");
    let validated = validate(&config(&source)).expect("local validation");
    assert_eq!(
        validated.common.output().timestamp_column.as_deref(),
        Some("ts")
    );
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
    for source in [
        format!("SELECT :ts, :id {}", " ".repeat(16384)),
        format!("SELECT :ts, :id {}", "x ".repeat(4096)),
        format!("SELECT {}:ts{}, :id", "(".repeat(17), ")".repeat(17)),
    ] {
        assert!(compile_parameters(&source, "ts", "id").is_err());
    }
}

/// Scenario: Required budgets are omitted, malformed, outside bounds, or inconsistent with fetch size.
/// Guarantees: No implicit row/byte defaults, clamping, or disabled limits are accepted.
#[test]
fn invalid_page_budgets_are_rejected() {
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
    assert_eq!(
        validate(&config(&sql(KEY)))
            .expect("fixed timeout")
            .common
            .timeout(),
        std::time::Duration::from_secs(30)
    );
}

/// Scenario: Operators add unsupported fields or omit required lower-bound information.
/// Guarantees: Strict objects reject unsafe settings without printing their values.
#[test]
fn strict_configuration() {
    let base = config(&sql(KEY));
    for mode in ["snapshot", "scalar"] {
        let mut cfg = base.clone();
        cfg["watermark"]["mode"] = json!(mode);
        assert!(validate(&cfg).is_err(), "{mode}");
    }
    for pointer in [
        "/connection/password",
        "/connection/tls",
        "/query/output",
        "/query/queries",
        "/query/timeout",
        "/checkpoint/unknown",
        "/unknown",
    ] {
        let mut cfg = base.clone();
        let (parent, key) = pointer.rsplit_once('/').expect("pointer");
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
    for host in ["******host/db", "a,b", "/socket", "a b", "a@b", "-bad"] {
        let mut cfg = base.clone();
        cfg["connection"]["host"] = json!(host);
        assert!(validate(&cfg).is_err());
    }
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

fn number(weight: i16, sign: u16, scale: u16, digits: &[u16]) -> Vec<u8> {
    let mut bytes = (digits.len() as i16).to_be_bytes().to_vec();
    bytes.extend_from_slice(&weight.to_be_bytes());
    bytes.extend_from_slice(&sign.to_be_bytes());
    bytes.extend_from_slice(&scale.to_be_bytes());
    for digit in digits {
        bytes.extend_from_slice(&digit.to_be_bytes());
    }
    bytes
}

/// Scenario: NUMERIC contains signed integers, fractions, scale, malformed groups, or extreme weights.
/// Guarantees: Base-10000 decoding stays exact and bounded without float conversion or scale loss.
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
