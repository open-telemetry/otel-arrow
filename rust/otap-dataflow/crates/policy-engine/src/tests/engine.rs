// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::borrow::Cow;
use std::sync::Arc;

use arrow::array::{
    Array, ArrayRef, BinaryArray, BooleanArray, DictionaryArray, FixedSizeBinaryArray,
    Float64Array, Int32Array, Int64Array, RecordBatch, StringArray, StructArray, UInt8Array,
    UInt16Array,
};
use arrow::buffer::{Buffer, NullBuffer, OffsetBuffer};
use arrow::datatypes::{DataType, Field, Schema, UInt8Type, UInt16Type};
use otel_arrow_dfe_pdata::otap::{Logs, OtapArrowRecords};
use otel_arrow_dfe_pdata::otlp::attributes::AttributeValueType;
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_dfe_pdata::schema::{FieldExt, consts};
use regex_syntax::hir::Hir;

use crate::bitset::{BitsetBufferPool, DenseBitset};
use crate::columnar::eval_anyval_bytes_eq_into;
use crate::model::{
    Action, AttributePath, CustomPolicy, LogAdd, LogAttributeSelector, LogFieldSelector,
    LogFilterPolicy, LogMatcher, LogRecordField, LogRedact, LogRemove, LogRename,
    LogTargetSelector, LogTransformPolicy, MetricFilterPolicy, NumericValue, Policy,
    PolicyDefinition, PolicyError, Predicate, SamplingConfig, SamplingMode, ScalarValue,
    ScopeField, SpanSamplingPolicy,
};
use crate::otap_ops::{ExtractedValue, LogsBatchView, extract_anyval_from_attr_batch};
use crate::string_index::{extract_glob_ngrams, extract_required_ngrams};
use crate::{EvaluationScratch, PolicyEngine, PolicyMatchCount};

use super::helpers::{
    build_attr_batch, build_custom_test_batch, compile_engine, decode_otlp_logs, encode_cbor,
    encode_otlp_logs, parse_hir,
};

/// Scenario: Evaluate a drop policy (`severity_number < 9`) alongside a keep policy (`service.name == "payment"`) on mixed DEBUG/INFO log rows.
/// Guarantees: Rows matching both keep and drop survive (`ACTION_KEEP` overrides `ACTION_DROP`), non-exempt DEBUG rows are dropped, and `RowSelection` accessors match the kept indices.
#[test]
fn test_keep_overrides_drop_precedence() {
    // Row 0: DEBUG (sev=5), service="payment" -> matches both Drop(sev < 9) and Keep(service == "payment") -> MUST BE KEPT
    // Row 1: DEBUG (sev=5), service="cart"    -> matches Drop(sev < 9) only -> MUST BE DROPPED
    // Row 2: INFO  (sev=9), service="cart"    -> matches neither -> MUST BE KEPT
    let batch = build_custom_test_batch(
        vec![Some("debug payment"), Some("debug cart"), Some("info cart")],
        vec![5, 5, 9],
        vec!["DEBUG", "DEBUG", "INFO"],
        vec![[1u8; 16]; 3],
        vec![[1u8; 8]; 3],
        vec![
            (0, "service.name", ScalarValue::String("payment".into())),
            (1, "service.name", ScalarValue::String("cart".into())),
            (2, "service.name", ScalarValue::String("cart".into())),
        ],
        vec![],
    );

    let policies = vec![
        Policy::log_filter(
            "drop_low_severity",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::SeverityNumber),
                predicate: Predicate::Lt(NumericValue::Int(9)),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "keep_payment_service",
            vec![LogMatcher {
                field: LogFieldSelector::LogAttribute(AttributePath::single("service.name")),
                predicate: Predicate::Equals(ScalarValue::String("payment".into())),
                negate: false,
            }],
            Action::Keep,
        ),
    ];

    let expected = DenseBitset::from_indices([0, 2]);
    let engine = compile_engine(&policies);
    let kept = engine
        .evaluate_filter(&batch, &mut EvaluationScratch::new(), None)
        .unwrap();
    assert_eq!(kept.len(), expected.len());
    assert!(!kept.is_empty());
    assert!(kept.contains(0));
    assert!(!kept.contains(1));
    assert_eq!(
        kept.iter().collect::<Vec<_>>(),
        expected.iter().collect::<Vec<_>>()
    );
    assert_eq!(
        kept.into_bitmap(),
        expected,
        "PolicyEngine failed ACTION_KEEP > ACTION_DROP precedence"
    );
}

/// Scenario: Evaluate filter predicates covering `$` trailing-newline anchoring, cross-row `StringArray` isolation, 64-bit integer precision above `2^53`, non-zero vs. all-zero `trace_id`, and CBOR nested map/array predicates.
/// Guarantees: Regex `$` rejects trailing `\n`, `contains` never matches across adjacent rows, `i64` vs. `f64` preserves exact precision above `2^53`, all-zero IDs are treated as absent, and nested CBOR paths and array membership evaluate accurately.
#[test]
fn test_all_9_predicates_and_edge_cases() {
    // Tests:
    // 1. RE2 `$` vs trailing `\n`: `^ERROR$` matches `"ERROR"` (row 0) but MUST NOT match `"ERROR\n"` (row 1).
    // 2. Non-dictionary StringArray row boundary isolation: row 0 `"foo"` and row 1 `"bar"` must NOT match `contains: "oba"`.
    // 3. Exact 64-bit integer precision above 2^53: 9_007_199_254_740_993 (2^53 + 1) > 9_007_199_254_740_992.0 (2^53 as f64).
    // 4. trace_id / span_id lowercase hex and all-zero byte absence (`[0u8; 16]` has exists == false).
    // 5. CBOR nested AttributePath (`["http", "request", "method"]`) and array `contains`.
    let mut trace0 = [0u8; 16];
    trace0[0] = 0xab;
    trace0[15] = 0xcd;
    let trace_zero = [0u8; 16];

    let nested_map = ciborium::Value::Map(vec![(
        ciborium::Value::Text("request".into()),
        ciborium::Value::Map(vec![(
            ciborium::Value::Text("method".into()),
            ciborium::Value::Text("POST".into()),
        )]),
    )]);
    let tags_array = ciborium::Value::Array(vec![
        ciborium::Value::Text("canary".into()),
        ciborium::Value::Text("pci-dss".into()),
    ]);

    let batch = build_custom_test_batch(
        vec![Some("foo"), Some("bar"), Some("ERROR\n"), Some("ERROR")],
        vec![9, 9, 17, 17],
        vec!["INFO", "INFO", "ERROR", "ERROR"],
        vec![trace0, trace_zero, trace_zero, trace0],
        vec![[0u8; 8]; 4],
        vec![
            (0, "big_int", ScalarValue::Int(9_007_199_254_740_993)), // 2^53 + 1
            (1, "big_int", ScalarValue::Int(9_007_199_254_740_992)), // 2^53
            (
                2,
                "raw_bytes",
                ScalarValue::Bytes(vec![0xde, 0xad, 0xbe, 0xef]),
            ),
        ],
        vec![(0, "http", nested_map), (3, "tags", tags_array)],
    );

    // 1. `^ERROR$` must drop row 3 ("ERROR") and keep row 2 ("ERROR\n")
    let p_re_dollar = vec![Policy::log_filter(
        "drop_exact_error_line",
        vec![LogMatcher {
            field: LogFieldSelector::RecordField(LogRecordField::Body),
            predicate: Predicate::Regex("^ERROR$".into()),
            negate: false,
        }],
        Action::Drop,
    )];
    let engine = compile_engine(&p_re_dollar);
    let kept = engine
        .evaluate_filter(&batch, &mut EvaluationScratch::new(), None)
        .unwrap()
        .into_bitmap();
    assert_eq!(
        kept,
        DenseBitset::from_indices([0, 1, 2]),
        "PolicyEngine failed RE2 $ vs trailing newline"
    );

    // 2. Row boundary isolation: `contains("oba")` on ["foo", "bar", ...] must match 0 rows (keep all 4 rows)
    let p_boundary = vec![Policy::log_filter(
        "drop_cross_row_oba",
        vec![LogMatcher {
            field: LogFieldSelector::RecordField(LogRecordField::Body),
            predicate: Predicate::Contains(ScalarValue::String("oba".into())),
            negate: false,
        }],
        Action::Drop,
    )];
    let engine = compile_engine(&p_boundary);
    let kept = engine
        .evaluate_filter(&batch, &mut EvaluationScratch::new(), None)
        .unwrap()
        .into_bitmap();
    assert_eq!(
        kept,
        DenseBitset::from_indices([0, 1, 2, 3]),
        "PolicyEngine falsely matched across adjacent StringArray rows"
    );

    // 3. Cross-type numeric precision above 2^53: Gt(Double(9_007_199_254_740_992.0)) matches only row 0 (2^53 + 1), NOT row 1 (2^53)
    let p_precision = vec![Policy::log_filter(
        "drop_above_2_53",
        vec![LogMatcher {
            field: LogFieldSelector::LogAttribute(AttributePath::single("big_int")),
            predicate: Predicate::Gt(NumericValue::Double(9_007_199_254_740_992.0)),
            negate: false,
        }],
        Action::Drop,
    )];
    let engine = compile_engine(&p_precision);
    let kept = engine
        .evaluate_filter(&batch, &mut EvaluationScratch::new(), None)
        .unwrap()
        .into_bitmap();
    assert_eq!(
        kept,
        DenseBitset::from_indices([1, 2, 3]),
        "PolicyEngine failed 2^53 cross-type i64/f64 precision test"
    );

    // 4. trace_id exists & hex match: rows 0 and 3 have non-zero trace0, rows 1 and 2 have all-zero trace_id
    let p_trace_exists = vec![Policy::log_filter(
        "drop_when_trace_id_exists",
        vec![LogMatcher {
            field: LogFieldSelector::RecordField(LogRecordField::TraceId),
            predicate: Predicate::Exists,
            negate: false,
        }],
        Action::Drop,
    )];
    let engine = compile_engine(&p_trace_exists);
    let kept = engine
        .evaluate_filter(&batch, &mut EvaluationScratch::new(), None)
        .unwrap()
        .into_bitmap();
    assert_eq!(
        kept,
        DenseBitset::from_indices([1, 2]),
        "PolicyEngine failed all-zero trace_id absence semantics"
    );

    // 5. CBOR nested map path ["http", "request", "method"] == "POST" (drops row 0) and array contains "pci-dss" (drops row 3)
    let p_cbor = vec![
        Policy::log_filter(
            "drop_nested_post",
            vec![LogMatcher {
                field: LogFieldSelector::LogAttribute(AttributePath::nested([
                    "http", "request", "method",
                ])),
                predicate: Predicate::Equals(ScalarValue::String("POST".into())),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "drop_pci_tag",
            vec![LogMatcher {
                field: LogFieldSelector::LogAttribute(AttributePath::single("tags")),
                predicate: Predicate::Contains(ScalarValue::String("pci-dss".into())),
                negate: false,
            }],
            Action::Drop,
        ),
    ];
    let engine = compile_engine(&p_cbor);
    let kept = engine
        .evaluate_filter(&batch, &mut EvaluationScratch::new(), None)
        .unwrap()
        .into_bitmap();
    assert_eq!(
        kept,
        DenseBitset::from_indices([1, 2]),
        "PolicyEngine failed CBOR nested path or array contains"
    );
}

/// Scenario: Run `extract_required_ngrams` on parsed regex HIR nodes covering literals, captures, concats, duplicates, character classes, anchors, bounded/unbounded repetitions, and alternations.
/// Guarantees: Mandatory contiguous byte substrings (`len >= 2`, clamped for long repetitions) are extracted and deduplicated, optional branches yield no n-grams, and alternations retain only n-grams common to every branch.
#[test]
fn test_extract_required_ngrams_all_hir_kinds() {
    // Pure literal >= 2 bytes and < 2 bytes
    assert_eq!(
        extract_required_ngrams(&parse_hir("hello")),
        vec![b"hello".to_vec()]
    );
    assert!(extract_required_ngrams(&parse_hir("a")).is_empty());

    // Pure literal inside Capture and Concat of pure literals
    assert_eq!(
        extract_required_ngrams(&parse_hir("(abcd)")),
        vec![b"abcd".to_vec()]
    );
    assert_eq!(
        extract_required_ngrams(&parse_hir("(ab)(cd)")),
        vec![b"abcd".to_vec()]
    );

    // Duplicate n-grams are deduplicated
    assert_eq!(
        extract_required_ngrams(&parse_hir("tok.*tok")),
        vec![b"tok".to_vec()]
    );

    // Empty, Class, and Look produce no n-grams
    assert!(extract_required_ngrams(&parse_hir("")).is_empty());
    assert!(extract_required_ngrams(&parse_hir("[a-z0-9]+")).is_empty());
    assert!(extract_required_ngrams(&parse_hir("^$")).is_empty());
    assert!(extract_required_ngrams(&parse_hir(r"\b")).is_empty());

    // Capture wrapping non-pure HIR
    assert_eq!(
        extract_required_ngrams(&parse_hir("(alpha.*beta)")),
        vec![b"alpha".to_vec(), b"beta".to_vec()]
    );

    // Repetition: min == 0, min >= 1 pure literal (short, normal, clamped > 16), non-pure literal
    assert!(extract_required_ngrams(&parse_hir("(abc)*")).is_empty());
    assert!(extract_required_ngrams(&parse_hir("(abc)?")).is_empty());
    assert!(extract_required_ngrams(&parse_hir("x{1,3}")).is_empty());
    assert_eq!(
        extract_required_ngrams(&parse_hir("a{2,5}")),
        vec![b"aa".to_vec()]
    );
    assert_eq!(
        extract_required_ngrams(&parse_hir("(ab){2,}")),
        vec![b"abab".to_vec()]
    );
    assert_eq!(
        extract_required_ngrams(&parse_hir("a{20,30}")),
        vec![vec![b'a'; 16]]
    );
    assert_eq!(
        extract_required_ngrams(&parse_hir("(ab){20,}")),
        vec![b"ab".repeat(16)]
    );
    assert_eq!(
        extract_required_ngrams(&parse_hir(r"(err\d+)+")),
        vec![b"err".to_vec()]
    );

    // Concat: short literal cleared before wildcard, trailing literal >= 2 and < 2
    assert_eq!(
        extract_required_ngrams(&parse_hir("x.*tail")),
        vec![b"tail".to_vec()]
    );
    assert_eq!(
        extract_required_ngrams(&parse_hir("head.*x")),
        vec![b"head".to_vec()]
    );

    // Alternation: common n-gram across non-prefix-factorable branches, disjoint 3-branch break
    assert_eq!(
        extract_required_ngrams(&parse_hir("(a.*tok|b.*tok)")),
        vec![b"tok".to_vec()]
    );
    assert!(extract_required_ngrams(&parse_hir("(aaa.*1|bbb.*2|ccc.*3)")).is_empty());
    assert!(extract_required_ngrams(&Hir::alternation(vec![])).is_empty());
}

/// Scenario: Compile and evaluate overlapping multi-ngram regexes, shared n-gram postings, indexed `contains` (`>= 2` bytes), fallback 1-byte `contains`, and fallback zero-ngram regexes.
/// Guarantees: The inverted n-gram index dispatches multi-ngram regexes only when all required literals hit, evaluates fallback patterns without dropping matches, and filters the expected row set.
#[test]
fn test_string_index_multigram_fallback_and_dedup() {
    let batch = build_custom_test_batch(
        vec![
            // repeated n-gram "err", hits "err.*timeout" & contains("timeout")
            Some("err err timeout here"),
            Some("alpha only"),             // 1/3 required n-grams
            Some("alpha beta"),             // 2/3 required n-grams
            Some("gamma beta alpha"),       // 3/3 n-grams in wrong order -> MetaRegex rejects
            Some("alpha 1 beta 2 gamma"),   // 3/3 n-grams in order -> matches
            Some("98765"),                  // matches zero-ngram fallback regex ^\d+$
            Some("has z char"),             // matches 1-byte fallback contains("z")
            Some("err connection refused"), // shared n-gram "err" matches second policy
            Some("clean log"),              // matches nothing
        ],
        vec![9; 9],
        vec!["INFO"; 9],
        vec![[1u8; 16]; 9],
        vec![[1u8; 8]; 9],
        vec![],
        vec![],
    );

    let policies = vec![
        // Multi-ngram regex (3 required n-grams)
        Policy::log_filter(
            "drop_alpha_beta_gamma",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Regex("alpha.*beta.*gamma".into()),
                negate: false,
            }],
            Action::Drop,
        ),
        // Two regexes sharing the same n-gram "err" (tests intern_ngram cache + posting list)
        Policy::log_filter(
            "drop_err_timeout",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Regex("err.*timeout".into()),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "drop_err_refused",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Regex("err.*refused".into()),
                negate: false,
            }],
            Action::Drop,
        ),
        // Indexed Contains (>= 2 bytes) that actually matches
        Policy::log_filter(
            "drop_contains_timeout",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Contains(ScalarValue::String("timeout".into())),
                negate: false,
            }],
            Action::Drop,
        ),
        // Fallback Contains (< 2 bytes)
        Policy::log_filter(
            "drop_contains_z",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Contains(ScalarValue::String("z".into())),
                negate: false,
            }],
            Action::Drop,
        ),
        // Fallback Regex (0 required n-grams)
        Policy::log_filter(
            "drop_digits_only",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Regex(r"^\d+$".into()),
                negate: false,
            }],
            Action::Drop,
        ),
    ];

    let engine = compile_engine(&policies);

    // Rows 0, 4, 5, 6, 7 are dropped; rows 1, 2, 3, 8 are kept
    let kept = engine
        .evaluate_filter(&batch, &mut EvaluationScratch::new(), None)
        .unwrap()
        .into_bitmap();
    assert_eq!(kept, DenseBitset::from_indices([1, 2, 3, 8]));
}

/// Scenario: Compile a policy batch mixing shared filter/transform predicates, empty attribute paths, invalid filter/transform/redact regexes, and empty transform operation lists.
/// Guarantees: Identical predicates across valid filter and transform rules are deduplicated into one predicate ID, while every invalid policy is rejected atomically and reported in `CompileOutput::errors`.
#[test]
fn test_engine_compile_edge_cases_and_errors() {
    let disabled = Policy {
        id: "disabled".into(),
        name: "disabled".into(),
        description: String::new(),
        enabled: false,
        labels: Vec::new(),
        definition: PolicyDefinition::LogFilter(LogFilterPolicy {
            matches: vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Regex("[unclosed_ignored".into()),
                negate: false,
            }],
            action: Action::Drop,
        }),
    };
    let unspecified = Policy::log_filter(
        "unspec",
        vec![LogMatcher {
            field: LogFieldSelector::RecordField(LogRecordField::Body),
            predicate: Predicate::Exists,
            negate: false,
        }],
        Action::Unspecified,
    );
    let empty_matches = Policy::log_filter("empty", vec![], Action::Drop);
    let metric = Policy {
        id: "metric".into(),
        name: "metric".into(),
        description: "metric filter placeholder".into(),
        enabled: true,
        labels: vec![("team".into(), ScalarValue::String("obs".into()))],
        definition: PolicyDefinition::MetricFilter(MetricFilterPolicy {
            action: Action::Drop,
        }),
    };
    let span_sampling = Policy {
        id: "span_sampling".into(),
        name: "span_sampling".into(),
        description: String::new(),
        enabled: true,
        labels: Vec::new(),
        definition: PolicyDefinition::SpanSampling(SpanSamplingPolicy {
            sampling: Some(SamplingConfig {
                percentage: Some(50.0),
                mode: Some(SamplingMode::Equalizing),
                sampling_precision: Some(4),
            }),
        }),
    };
    let custom = Policy {
        id: "custom".into(),
        name: "custom".into(),
        description: String::new(),
        enabled: true,
        labels: Vec::new(),
        definition: PolicyDefinition::CustomPolicy(CustomPolicy {
            type_url: "type.googleapis.com/example.CustomPolicy".into(),
            value: vec![1, 2, 3],
        }),
    };

    // Identical predicate across filter & transform rules exercises pred_dedup & field_index_map
    let shared_matcher = LogMatcher {
        field: LogFieldSelector::LogAttribute(AttributePath::single("env")),
        predicate: Predicate::Equals(ScalarValue::String("prod".into())),
        negate: false,
    };
    let f1 = Policy::log_filter("f1", vec![shared_matcher.clone()], Action::Drop);
    let f1_dup = Policy::log_filter("f1_dup", vec![shared_matcher.clone()], Action::Drop);
    let f2 = Policy::log_filter(
        "f2",
        vec![LogMatcher {
            negate: true,
            ..shared_matcher.clone()
        }],
        Action::Keep,
    );
    let tx = Policy::log_transform(
        "tx1",
        LogTransformPolicy {
            matches: vec![shared_matcher],
            rename: vec![],
            add: vec![LogAdd {
                target: LogTargetSelector::LogAttribute(AttributePath::single("tagged")),
                value: ScalarValue::Bool(true),
                upsert: true,
            }],
            remove: vec![],
            redact: vec![],
        },
    );
    // Empty attribute paths and empty segments are rejected per policy, collecting every error in one pass
    let empty_paths = Policy::log_filter(
        "empty_paths",
        vec![
            LogMatcher {
                field: LogFieldSelector::LogAttribute(AttributePath { path: vec![] }),
                predicate: Predicate::Exists,
                negate: false,
            },
            LogMatcher {
                field: LogFieldSelector::ResourceAttribute(AttributePath { path: vec![] }),
                predicate: Predicate::Exists,
                negate: false,
            },
            LogMatcher {
                field: LogFieldSelector::ScopeAttribute(AttributePath { path: vec![] }),
                predicate: Predicate::Exists,
                negate: false,
            },
            LogMatcher {
                field: LogFieldSelector::LogAttribute(AttributePath::nested(["http", ""])),
                predicate: Predicate::Exists,
                negate: false,
            },
            LogMatcher {
                field: LogFieldSelector::BodyField(AttributePath::single("")),
                predicate: Predicate::Exists,
                negate: false,
            },
        ],
        Action::Drop,
    );

    let out = PolicyEngine::compile(&[
        disabled,
        unspecified,
        empty_matches,
        metric,
        span_sampling,
        custom,
        f1,
        f1_dup,
        f2,
        tx,
        empty_paths,
    ])
    .expect("compile succeeds with isolated per-policy errors");
    // Drop rules f1 and f1_dup deduplicate `env == "prod"` into 1 drop predicate; keep rule f2
    // and transform tx each get 1 predicate in their own stage plan.
    assert_eq!(out.engine.num_drop_predicates(), 1);
    assert_eq!(out.engine.num_keep_predicates(), 1);
    assert_eq!(out.engine.num_filter_predicates(), 2);
    assert_eq!(out.engine.num_transform_predicates(), 1);
    assert_eq!(out.engine.num_predicates(), 3);

    let err_ids: Vec<&str> = out.errors.iter().map(|e| e.policy_id.as_str()).collect();
    assert!(!err_ids.contains(&"disabled"));
    assert!(err_ids.contains(&"unspec"));
    assert!(err_ids.contains(&"empty"));
    assert!(err_ids.contains(&"metric"));
    assert!(err_ids.contains(&"span_sampling"));
    assert!(err_ids.contains(&"custom"));
    assert_eq!(
        out.errors
            .iter()
            .filter(|e| e.policy_id == "empty_paths")
            .count(),
        5,
        "all 5 empty attribute paths / empty segments should be reported in one pass"
    );
    assert!(out.errors[0].to_string().contains(&out.errors[0].policy_id));

    // Invalid regex in filter matcher is isolated per policy
    let bad_filter = Policy::log_filter(
        "bad_f",
        vec![LogMatcher {
            field: LogFieldSelector::RecordField(LogRecordField::Body),
            predicate: Predicate::Regex("[bad".into()),
            negate: false,
        }],
        Action::Drop,
    );
    let out = PolicyEngine::compile(&[bad_filter]).expect("isolated compile");
    assert_eq!(out.engine.num_predicates(), 0);
    assert_eq!(out.errors.len(), 1);
    assert_eq!(out.errors[0].policy_id, "bad_f");
    assert!(out.errors[0].message.contains("matches[0]"));

    // Invalid regex in transform matcher + empty operations both reported
    let bad_tx_matcher = Policy::log_transform(
        "bad_tx_m",
        LogTransformPolicy {
            matches: vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Regex("[bad".into()),
                negate: false,
            }],
            rename: vec![],
            add: vec![],
            remove: vec![],
            redact: vec![],
        },
    );
    let out = PolicyEngine::compile(&[bad_tx_matcher]).expect("isolated compile");
    assert_eq!(out.engine.num_predicates(), 0);
    assert_eq!(out.errors.len(), 2);
    assert!(out.errors.iter().all(|e| e.policy_id == "bad_tx_m"));

    // Invalid regex in transform redact is isolated per policy
    let bad_tx_redact = Policy::log_transform(
        "bad_tx_r",
        LogTransformPolicy {
            matches: vec![],
            rename: vec![],
            add: vec![],
            remove: vec![],
            redact: vec![LogRedact {
                target: LogTargetSelector::RecordField(LogRecordField::Body),
                replacement: None,
                regex: Some("[bad".into()),
            }],
        },
    );
    let out = PolicyEngine::compile(&[bad_tx_redact]).expect("isolated compile");
    assert_eq!(out.errors.len(), 1);
    assert_eq!(out.errors[0].policy_id, "bad_tx_r");
    assert!(out.errors[0].message.contains("redact[0].regex"));
}

/// Scenario: Evaluate filter and transform policies across missing-column batches, all 7 `Body` value types, root/scope/resource/scope-attr selectors, zero/empty root/scope absence checks (`Exists`/`Lt`/`Equals("")`), and a transform after dropping row 0.
/// Guarantees: Missing columns do not error, `SeverityNumber == 0` and empty root/scope strings behave as absent for `Exists`/`Lt`/`Equals("")`, shared-key filter/transform evaluators execute independently, and transforms apply to the exact shifted row index after earlier rows are dropped.
#[test]
fn test_engine_all_selectors_types_and_missing_columns() {
    // 1. Minimal batch missing BODY, TraceId, SpanId, EventName, SeverityNumber columns
    let minimal_root = RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new(consts::ID, DataType::UInt16, false).with_plain_encoding(),
            Field::new(consts::SEVERITY_TEXT, DataType::Utf8, true),
        ])),
        vec![
            Arc::new(UInt16Array::from(vec![0u16, 1, 2])),
            Arc::new(StringArray::from(vec![
                Some("hello world"),
                Some("exact"),
                None,
            ])),
        ],
    )
    .unwrap();
    let mut minimal_batch = OtapArrowRecords::Logs(Logs::default());
    minimal_batch
        .set(ArrowPayloadType::Logs, minimal_root)
        .unwrap();

    let missing_cols_engine = compile_engine(&[
        // Exercises eval_str_only (Exists + str_equals_map + string_index) on SeverityText
        Policy::log_filter(
            "eq_exact",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
                predicate: Predicate::Equals(ScalarValue::String("exact".into())),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "contains_world",
            vec![
                LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
                    predicate: Predicate::Contains(ScalarValue::String("world".into())),
                    negate: false,
                },
                LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
                    predicate: Predicate::Exists,
                    negate: false,
                },
            ],
            Action::Drop,
        ),
        // Missing BODY column for both Body and BodyField selectors
        Policy::log_filter(
            "body_exists",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Exists,
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "bf_exists",
            vec![LogMatcher {
                field: LogFieldSelector::BodyField(AttributePath::single("x")),
                predicate: Predicate::Exists,
                negate: false,
            }],
            Action::Drop,
        ),
        // Missing TraceId column
        Policy::log_filter(
            "missing_trace",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::TraceId),
                predicate: Predicate::Exists,
                negate: false,
            }],
            Action::Drop,
        ),
    ]);
    assert_eq!(
        missing_cols_engine
            .evaluate_filter(&minimal_batch, &mut EvaluationScratch::new(), None)
            .unwrap()
            .into_bitmap(),
        DenseBitset::from_indices([2])
    );

    // 2. Rich batch testing AnyValColumns (null struct, null type, Empty, Str, Int, Double, Bool,
    //    Bytes, Map, Slice), ScopeField (Name, Version, SchemaUrl), SpanId, SeverityText,
    //    EventName, SeverityNumber (0 absent), BodyField nested map, ResourceAttrs & ScopeAttrs
    //    shared across multiple rows!
    let body_map = encode_cbor(&ciborium::Value::Map(vec![(
        ciborium::Value::Text("req".into()),
        ciborium::Value::Map(vec![(
            ciborium::Value::Text("code".into()),
            ciborium::Value::Integer(500.into()),
        )]),
    )]));
    let body_slice = encode_cbor(&ciborium::Value::Array(vec![
        ciborium::Value::Text("a".into()),
        ciborium::Value::Integer(42.into()),
    ]));

    let body_struct = StructArray::new(
        vec![
            Arc::new(Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false)),
            Arc::new(Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true)),
            Arc::new(Field::new(consts::ATTRIBUTE_INT, DataType::Int64, true)),
            Arc::new(Field::new(
                consts::ATTRIBUTE_DOUBLE,
                DataType::Float64,
                true,
            )),
            Arc::new(Field::new(consts::ATTRIBUTE_BOOL, DataType::Boolean, true)),
            Arc::new(Field::new(consts::ATTRIBUTE_BYTES, DataType::Binary, true)),
            Arc::new(Field::new(consts::ATTRIBUTE_SER, DataType::Binary, true)),
        ]
        .into(),
        vec![
            Arc::new(UInt8Array::from(vec![
                AttributeValueType::Str as u8,    // row 0: masked by struct validity
                255,                              // row 1: invalid type discriminant
                AttributeValueType::Empty as u8,  // row 2: Empty
                AttributeValueType::Int as u8,    // row 3: Int(100)
                AttributeValueType::Double as u8, // row 4: Double(12.5)
                AttributeValueType::Double as u8, // row 5: Double(NaN)
                AttributeValueType::Bool as u8,   // row 6: Bool(true)
                AttributeValueType::Bytes as u8,  // row 7: Bytes([0xde, 0xad, 0xbe, 0xef])
                AttributeValueType::Map as u8,    // row 8: CBOR Map {req: {code: 500}}
                AttributeValueType::Slice as u8,  // row 9: CBOR Slice ["a", 42]
            ])),
            Arc::new(StringArray::from(vec![
                Some("hidden"),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ])),
            Arc::new(Int64Array::from(vec![
                None,
                None,
                None,
                Some(100),
                None,
                None,
                None,
                None,
                None,
                None,
            ])),
            Arc::new(Float64Array::from(vec![
                None,
                None,
                None,
                None,
                Some(12.5),
                Some(f64::NAN),
                None,
                None,
                None,
                None,
            ])),
            Arc::new(BooleanArray::from(vec![
                None,
                None,
                None,
                None,
                None,
                None,
                Some(true),
                None,
                None,
                None,
            ])),
            Arc::new(BinaryArray::from_opt_vec(vec![
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(&[0xde, 0xad, 0xbe, 0xef]),
                None,
                None,
            ])),
            Arc::new(BinaryArray::from_opt_vec(vec![
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(body_map.as_slice()),
                Some(body_slice.as_slice()),
            ])),
        ],
        Some(NullBuffer::from(vec![
            false, true, true, true, true, true, true, true, true, true,
        ])),
    );

    // Rows 0, 1 share resource_id=0 and scope_id=0; rows 2..10 have resource_id=1, scope_id=1
    let res_ids = vec![0u16, 0, 1, 1, 1, 1, 1, 1, 1, 1];
    let scope_ids = vec![0u16, 0, 1, 1, 1, 1, 1, 1, 1, 1];
    let res_struct = StructArray::new(
        vec![Arc::new(
            Field::new(consts::ID, DataType::UInt16, false).with_plain_encoding(),
        )]
        .into(),
        vec![Arc::new(UInt16Array::from(res_ids))],
        None,
    );
    let scope_struct = StructArray::new(
        vec![
            Arc::new(Field::new(consts::ID, DataType::UInt16, false).with_plain_encoding()),
            Arc::new(Field::new(consts::NAME, DataType::Utf8, true)),
            Arc::new(Field::new(consts::VERSION, DataType::Utf8, true)),
        ]
        .into(),
        vec![
            Arc::new(UInt16Array::from(scope_ids)),
            Arc::new(StringArray::from(vec![
                "scope.alpha",
                "",
                "scope.beta",
                "scope.beta",
                "scope.beta",
                "scope.beta",
                "scope.beta",
                "scope.beta",
                "scope.beta",
                "scope.beta",
            ])),
            Arc::new(StringArray::from(vec![
                "1.0", "", "2.0", "2.0", "2.0", "2.0", "2.0", "2.0", "2.0", "2.0",
            ])),
        ],
        None,
    );

    let mut span_nz = [0u8; 8];
    span_nz[0] = 0x12;
    span_nz[7] = 0xef;
    let span_zero = [0u8; 8];
    let span_slices = [
        span_nz.as_slice(),
        span_zero.as_slice(),
        span_zero.as_slice(),
        span_zero.as_slice(),
        span_zero.as_slice(),
        span_zero.as_slice(),
        span_zero.as_slice(),
        span_zero.as_slice(),
        span_zero.as_slice(),
        span_zero.as_slice(),
    ];

    let root_schema = Arc::new(Schema::new(vec![
        Field::new(consts::ID, DataType::UInt16, false).with_plain_encoding(),
        Field::new(consts::BODY, body_struct.data_type().clone(), true),
        Field::new(consts::SEVERITY_NUMBER, DataType::Int32, true),
        Field::new(consts::SEVERITY_TEXT, DataType::Utf8, true),
        Field::new(consts::EVENT_NAME, DataType::Utf8, true),
        Field::new(consts::SPAN_ID, DataType::FixedSizeBinary(8), true),
        Field::new(consts::SCHEMA_URL, DataType::Utf8, true),
        Field::new(consts::RESOURCE, res_struct.data_type().clone(), true),
        Field::new(consts::SCOPE, scope_struct.data_type().clone(), true),
    ]));
    let root_rb = RecordBatch::try_new(
        root_schema,
        vec![
            Arc::new(UInt16Array::from((0..10u16).collect::<Vec<_>>())),
            Arc::new(body_struct),
            Arc::new(Int32Array::from(vec![0, 5, 9, 13, 17, 21, 0, 9, 9, 9])),
            Arc::new(StringArray::from(vec![
                "", "DEBUG", "INFO", "WARN", "ERROR", "FATAL", "", "INFO", "INFO", "INFO",
            ])),
            Arc::new(StringArray::from(vec![
                "",
                "evt.login",
                "evt.ok",
                "evt.ok",
                "evt.ok",
                "evt.ok",
                "",
                "evt.ok",
                "evt.ok",
                "evt.ok",
            ])),
            Arc::new(FixedSizeBinaryArray::try_from_iter(span_slices.into_iter()).unwrap()),
            Arc::new(StringArray::from(vec![
                "https://schema.v1",
                "",
                "https://schema.v2",
                "https://schema.v2",
                "https://schema.v2",
                "https://schema.v2",
                "https://schema.v2",
                "https://schema.v2",
                "https://schema.v2",
                "https://schema.v2",
            ])),
            Arc::new(res_struct),
            Arc::new(scope_struct),
        ],
    )
    .unwrap();

    let res_attrs = build_attr_batch(
        vec![(0, "service.name", ScalarValue::String("billing".into()))],
        vec![],
    );
    let scope_attrs = build_attr_batch(
        vec![(0, "instr.lib", ScalarValue::String("otel-rs".into()))],
        vec![],
    );
    // LogAttrs with Empty value, out-of-range parent_id, nested CBOR, and flat string
    let nested_map = encode_cbor(&ciborium::Value::Map(vec![(
        ciborium::Value::Text("inner".into()),
        ciborium::Value::Text("hit".into()),
    )]));
    let log_attrs = RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new(consts::PARENT_ID, DataType::UInt16, false).with_plain_encoding(),
            Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
            Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
            Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
            Field::new(consts::ATTRIBUTE_SER, DataType::Binary, true),
        ])),
        vec![
            Arc::new(UInt16Array::from(vec![
                0u16, // Empty value
                99,   // out-of-range parent_id
                2,    // CBOR Map
                3,    // flat string
            ])),
            Arc::new(StringArray::from(vec![
                "shared", "shared", "shared", "shared",
            ])),
            Arc::new(UInt8Array::from(vec![
                AttributeValueType::Empty as u8,
                AttributeValueType::Str as u8,
                AttributeValueType::Map as u8,
                AttributeValueType::Str as u8,
            ])),
            Arc::new(StringArray::from(vec![
                None,
                Some("ignored"),
                None,
                Some("flat"),
            ])),
            Arc::new(BinaryArray::from_opt_vec(vec![
                None,
                None,
                Some(nested_map.as_slice()),
                None,
            ])),
        ],
    )
    .unwrap();

    let mut batch = OtapArrowRecords::Logs(Logs::default());
    batch.set(ArrowPayloadType::Logs, root_rb).unwrap();
    batch.set(ArrowPayloadType::LogAttrs, log_attrs).unwrap();
    batch
        .set(ArrowPayloadType::ResourceAttrs, res_attrs)
        .unwrap();
    batch
        .set(ArrowPayloadType::ScopeAttrs, scope_attrs)
        .unwrap();

    // Evaluate every Body ExtractedValue variant (Int, Double, NaN, Bool, Bytes, Map, Slice)
    let body_preds = vec![
        // Row 3: Body == Int(100) and Gte(100), Lte(100)
        Policy::log_filter(
            "int_eq",
            vec![
                LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::Body),
                    predicate: Predicate::Equals(ScalarValue::Int(100)),
                    negate: false,
                },
                LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::Body),
                    predicate: Predicate::Gte(NumericValue::Int(100)),
                    negate: false,
                },
                LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::Body),
                    predicate: Predicate::Lte(NumericValue::Int(100)),
                    negate: false,
                },
            ],
            Action::Drop,
        ),
        // Row 4: Double(12.5) with Gt/Gte/Lt/Lte; row 5 is NaN (must not match)
        Policy::log_filter(
            "dbl_range",
            vec![
                LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::Body),
                    predicate: Predicate::Equals(ScalarValue::Double(12.5)),
                    negate: false,
                },
                LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::Body),
                    predicate: Predicate::Gt(NumericValue::Double(10.0)),
                    negate: false,
                },
                LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::Body),
                    predicate: Predicate::Gte(NumericValue::Double(12.5)),
                    negate: false,
                },
                LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::Body),
                    predicate: Predicate::Lt(NumericValue::Double(13.0)),
                    negate: false,
                },
                LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::Body),
                    predicate: Predicate::Lte(NumericValue::Double(12.5)),
                    negate: false,
                },
            ],
            Action::Drop,
        ),
        // Row 6: Body == Bool(true)
        Policy::log_filter(
            "bool_eq",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Equals(ScalarValue::Bool(true)),
                negate: false,
            }],
            Action::Drop,
        ),
        // Row 7: Body == Bytes and Contains(Bytes)
        Policy::log_filter(
            "bytes_match",
            vec![
                LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::Body),
                    predicate: Predicate::Equals(ScalarValue::Bytes(vec![0xde, 0xad, 0xbe, 0xef])),
                    negate: false,
                },
                LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::Body),
                    predicate: Predicate::Contains(ScalarValue::Bytes(vec![0xad, 0xbe])),
                    negate: false,
                },
            ],
            Action::Drop,
        ),
        // Row 8: Body is Map -> BodyField(["req", "code"]) == 500 & Body Exists
        Policy::log_filter(
            "body_map_nested",
            vec![
                LogMatcher {
                    field: LogFieldSelector::BodyField(AttributePath::nested(["req", "code"])),
                    predicate: Predicate::Equals(ScalarValue::Int(500)),
                    negate: false,
                },
                LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::Body),
                    predicate: Predicate::Exists,
                    negate: false,
                },
            ],
            Action::Drop,
        ),
        // Row 9: Body is Slice -> Contains(Int(42))
        Policy::log_filter(
            "body_slice",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Contains(ScalarValue::Int(42)),
                negate: false,
            }],
            Action::Drop,
        ),
    ];
    let eng_body = compile_engine(&body_preds);
    // Rows 3, 4, 6, 7, 8, 9 dropped; rows 0 (null struct), 1 (null type), 2 (Empty), 5 (NaN) kept
    assert_eq!(
        eng_body
            .evaluate_filter(&batch, &mut EvaluationScratch::new(), None)
            .unwrap()
            .into_bitmap(),
        DenseBitset::from_indices([0, 1, 2, 5])
    );

    // Evaluate SeverityText, EventName, SpanId, ScopeField::{Name, Version, SchemaUrl}
    let root_eng = compile_engine(&[
        Policy::log_filter(
            "sev_text",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
                predicate: Predicate::Equals(ScalarValue::String("WARN".into())),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "evt_name",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::EventName),
                predicate: Predicate::Contains(ScalarValue::String("login".into())),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "span_id",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::SpanId),
                predicate: Predicate::Equals(ScalarValue::String(hex::encode(span_nz))),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "scope_name",
            vec![LogMatcher {
                field: LogFieldSelector::ScopeField(ScopeField::Name),
                predicate: Predicate::Equals(ScalarValue::String("scope.alpha".into())),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "scope_ver",
            vec![LogMatcher {
                field: LogFieldSelector::ScopeField(ScopeField::Version),
                predicate: Predicate::Equals(ScalarValue::String("1.0".into())),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "schema_url",
            vec![LogMatcher {
                field: LogFieldSelector::ScopeField(ScopeField::SchemaUrl),
                predicate: Predicate::Regex(r"schema\.v1$".into()),
                negate: false,
            }],
            Action::Drop,
        ),
        // Exercises SeverityNumber == 0 absent semantics and exact Int equality on root field
        Policy::log_filter(
            "sev_num_fatal",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::SeverityNumber),
                predicate: Predicate::Equals(ScalarValue::Int(21)),
                negate: false,
            }],
            Action::Drop,
        ),
    ]);
    // Drops rows 0 (span_id/scope), 1 (evt.login), 3 (WARN), 5 (sev=21); rows 0 and 6 have sev=0
    assert_eq!(
        root_eng
            .evaluate_filter(&batch, &mut EvaluationScratch::new(), None)
            .unwrap()
            .into_bitmap(),
        DenseBitset::from_indices([2, 4, 6, 7, 8, 9])
    );

    // Evaluate ResourceAttrs & ScopeAttrs matching BOTH rows 0 and 1 sharing resource_id/scope_id=0
    let res_eng = compile_engine(&[Policy::log_filter(
        "res_billing",
        vec![LogMatcher {
            field: LogFieldSelector::ResourceAttribute(AttributePath::single("service.name")),
            predicate: Predicate::Equals(ScalarValue::String("billing".into())),
            negate: false,
        }],
        Action::Drop,
    )]);
    assert_eq!(
        res_eng
            .evaluate_filter(&batch, &mut EvaluationScratch::new(), None)
            .unwrap()
            .into_bitmap(),
        DenseBitset::from_indices([2, 3, 4, 5, 6, 7, 8, 9])
    );

    let scope_eng = compile_engine(&[Policy::log_filter(
        "scope_lib",
        vec![LogMatcher {
            field: LogFieldSelector::ScopeAttribute(AttributePath::single("instr.lib")),
            predicate: Predicate::Equals(ScalarValue::String("otel-rs".into())),
            negate: false,
        }],
        Action::Drop,
    )]);
    assert_eq!(
        scope_eng
            .evaluate_filter(&batch, &mut EvaluationScratch::new(), None)
            .unwrap()
            .into_bitmap(),
        DenseBitset::from_indices([2, 3, 4, 5, 6, 7, 8, 9])
    );

    // Same top-level key "shared" split across a filter-only evaluator (["shared"]) and a
    // transform-only evaluator (["shared", "inner"]) + filter-only root vs transform-only root
    // exercises should_run() skipping inside both evaluate_predicates and evaluate_attr_table!
    let split_eng = compile_engine(&[
        Policy::log_filter(
            "drop_shared_flat",
            vec![
                LogMatcher {
                    field: LogFieldSelector::LogAttribute(AttributePath::single("shared")),
                    predicate: Predicate::Equals(ScalarValue::String("flat".into())),
                    negate: false,
                },
                LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
                    predicate: Predicate::Equals(ScalarValue::String("WARN".into())),
                    negate: false,
                },
            ],
            Action::Drop,
        ),
        Policy::log_transform(
            "tag_nested_shared",
            LogTransformPolicy {
                matches: vec![
                    LogMatcher {
                        field: LogFieldSelector::LogAttribute(AttributePath::nested([
                            "shared", "inner",
                        ])),
                        predicate: Predicate::Equals(ScalarValue::String("hit".into())),
                        negate: false,
                    },
                    LogMatcher {
                        field: LogFieldSelector::RecordField(LogRecordField::EventName),
                        predicate: Predicate::Equals(ScalarValue::String("evt.ok".into())),
                        negate: false,
                    },
                ],
                rename: vec![],
                add: vec![LogAdd {
                    target: LogTargetSelector::LogAttribute(AttributePath::single("tx_hit")),
                    value: ScalarValue::Bool(true),
                    upsert: true,
                }],
                remove: vec![],
                redact: vec![],
            },
        ),
    ]);
    // Drops only row 3 ("flat" + "WARN"); transform tags row 2 (CBOR {"inner": "hit"} + "evt.ok")
    assert_eq!(
        split_eng
            .evaluate_filter(&batch, &mut EvaluationScratch::new(), None)
            .unwrap()
            .into_bitmap(),
        DenseBitset::from_indices([0, 1, 2, 4, 5, 6, 7, 8, 9])
    );
    let mut split_stats = split_eng.new_match_stats();
    let out = split_eng
        .evaluate(
            &batch,
            &mut EvaluationScratch::new(),
            Some(&mut split_stats),
        )
        .unwrap();
    assert_eq!(out.kept_rows, 9);
    assert_eq!(out.dropped_rows, 1);
    assert_eq!(split_stats.evaluated_rows(), 10);
    assert_eq!(
        split_stats.counts()[1],
        PolicyMatchCount { hits: 1, misses: 0 }
    );

    // Verify absent semantics for SeverityNumber == 0 and empty string root/scope fields
    // so removing `v != 0` or `!s.is_empty()` checks fails the suite.
    let sev_exists = compile_engine(&[Policy::log_filter(
        "drop_sev_exists",
        vec![LogMatcher {
            field: LogFieldSelector::RecordField(LogRecordField::SeverityNumber),
            predicate: Predicate::Exists,
            negate: false,
        }],
        Action::Drop,
    )]);
    assert_eq!(
        sev_exists
            .evaluate_filter(&batch, &mut EvaluationScratch::new(), None)
            .unwrap()
            .into_bitmap(),
        DenseBitset::from_indices([0, 6])
    );
    let sev_lt9 = compile_engine(&[Policy::log_filter(
        "drop_sev_lt_9",
        vec![LogMatcher {
            field: LogFieldSelector::RecordField(LogRecordField::SeverityNumber),
            predicate: Predicate::Lt(NumericValue::Int(9)),
            negate: false,
        }],
        Action::Drop,
    )]);
    // Only row 1 (sev=5) is dropped; rows 0 and 6 have sev=0 (absent) and must not match Lt(9).
    assert_eq!(
        sev_lt9
            .evaluate_filter(&batch, &mut EvaluationScratch::new(), None)
            .unwrap()
            .into_bitmap(),
        DenseBitset::from_indices([0, 2, 3, 4, 5, 6, 7, 8, 9])
    );

    for (id, field, expected_absent_rows) in [
        (
            "sev_text",
            LogFieldSelector::RecordField(LogRecordField::SeverityText),
            vec![0u32, 6],
        ),
        (
            "evt_name",
            LogFieldSelector::RecordField(LogRecordField::EventName),
            vec![0u32, 6],
        ),
        (
            "scope_name",
            LogFieldSelector::ScopeField(ScopeField::Name),
            vec![1u32],
        ),
        (
            "scope_ver",
            LogFieldSelector::ScopeField(ScopeField::Version),
            vec![1u32],
        ),
        (
            "schema_url",
            LogFieldSelector::ScopeField(ScopeField::SchemaUrl),
            vec![1u32],
        ),
    ] {
        let exists_eng = compile_engine(&[Policy::log_filter(
            format!("drop_{id}_exists"),
            vec![LogMatcher {
                field: field.clone(),
                predicate: Predicate::Exists,
                negate: false,
            }],
            Action::Drop,
        )]);
        assert_eq!(
            exists_eng
                .evaluate_filter(&batch, &mut EvaluationScratch::new(), None)
                .unwrap()
                .into_bitmap(),
            DenseBitset::from_indices(expected_absent_rows),
            "empty string must be treated as absent for Exists on {id}"
        );

        let eq_empty_eng = compile_engine(&[Policy::log_filter(
            format!("drop_{id}_empty"),
            vec![LogMatcher {
                field,
                predicate: Predicate::Equals(ScalarValue::String(String::new())),
                negate: false,
            }],
            Action::Drop,
        )]);
        assert_eq!(
            eq_empty_eng
                .evaluate_filter(&batch, &mut EvaluationScratch::new(), None)
                .unwrap()
                .into_bitmap(),
            DenseBitset::from_indices(0..10u32),
            "empty string must be treated as absent for Equals(\"\") on {id}"
        );
    }

    // Verify transforms target the shifted post-filter row index when an earlier row (row 0) is dropped.
    let shifted_eng = compile_engine(&[
        Policy::log_filter(
            "drop_row0_span",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::SpanId),
                predicate: Predicate::Exists,
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "drop_row3_shared_flat",
            vec![
                LogMatcher {
                    field: LogFieldSelector::LogAttribute(AttributePath::single("shared")),
                    predicate: Predicate::Equals(ScalarValue::String("flat".into())),
                    negate: false,
                },
                LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
                    predicate: Predicate::Equals(ScalarValue::String("WARN".into())),
                    negate: false,
                },
            ],
            Action::Drop,
        ),
        Policy::log_transform(
            "tag_shifted_row2",
            LogTransformPolicy {
                matches: vec![
                    LogMatcher {
                        field: LogFieldSelector::LogAttribute(AttributePath::nested([
                            "shared", "inner",
                        ])),
                        predicate: Predicate::Equals(ScalarValue::String("hit".into())),
                        negate: false,
                    },
                    LogMatcher {
                        field: LogFieldSelector::RecordField(LogRecordField::EventName),
                        predicate: Predicate::Equals(ScalarValue::String("evt.ok".into())),
                        negate: false,
                    },
                ],
                rename: vec![],
                add: vec![LogAdd {
                    target: LogTargetSelector::LogAttribute(AttributePath::single("tx_hit")),
                    value: ScalarValue::Bool(true),
                    upsert: true,
                }],
                remove: vec![],
                redact: vec![],
            },
        ),
    ]);
    // Dropping rows 0 and 3 leaves original rows [1, 2, 4, 5, 6, 7, 8, 9], shifting row 2 to index 1.
    let shifted_out = shifted_eng
        .evaluate(&batch, &mut EvaluationScratch::new(), None)
        .unwrap();
    assert_eq!(shifted_out.kept_rows, 8);
    assert_eq!(shifted_out.dropped_rows, 2);
    let shifted_records = shifted_out.records.expect("surviving rows");
    let shifted_view = LogsBatchView::try_new(&shifted_records).unwrap().unwrap();
    assert_eq!(shifted_view.num_rows, 8);
    assert_eq!(
        shifted_view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::nested(["shared", "inner"])),
            1
        ),
        Some(ExtractedValue::Str("hit".into()))
    );
    for r in 0..shifted_view.num_rows {
        let expected_tx = (r == 1).then_some(ExtractedValue::Bool(true));
        assert_eq!(
            shifted_view.extract_field_at_row(
                &LogFieldSelector::LogAttribute(AttributePath::single("tx_hit")),
                r
            ),
            expected_tx,
            "unexpected tx_hit on shifted row {r}"
        );
    }
}

/// Scenario: Run `evaluate_filter` and `evaluate` across empty batches, empty engines, transform-only engines, all-dropped and partial-drop batches, and multi-matcher conjunctions with leading and trailing negated matchers.
/// Guarantees: All-dropped batches return `records: None`, fast paths preserve row counts, and negated matchers complement predicate bitmaps accurately regardless of matcher position.
#[test]
fn test_engine_negation_conjunction_and_evaluate_lifecycle() {
    let batch = build_custom_test_batch(
        vec![Some("a"), Some("b"), Some("c")],
        vec![5, 9, 17],
        vec!["DEBUG", "INFO", "ERROR"],
        vec![[1u8; 16]; 3],
        vec![[1u8; 8]; 3],
        vec![
            (0, "keep_me", ScalarValue::Bool(true)),
            (1, "keep_me", ScalarValue::Bool(false)),
        ],
        vec![],
    );

    // 1. Empty / missing Logs batch in evaluate_filter and evaluate
    let empty_batch = OtapArrowRecords::Logs(Logs::default());
    let any_engine = compile_engine(&[Policy::log_filter(
        "drop_info",
        vec![LogMatcher {
            field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
            predicate: Predicate::Equals(ScalarValue::String("INFO".into())),
            negate: false,
        }],
        Action::Drop,
    )]);
    assert!(
        any_engine
            .evaluate_filter(&empty_batch, &mut EvaluationScratch::new(), None)
            .unwrap()
            .is_empty()
    );
    let empty_out = any_engine
        .evaluate(&empty_batch, &mut EvaluationScratch::new(), None)
        .unwrap();
    assert!(empty_out.records.is_some());
    assert_eq!(empty_out.kept_rows, 0);
    assert_eq!(empty_out.dropped_rows, 0);

    // 2. Empty engine (no filters, no transforms): fast path in evaluate_filter and evaluate
    let noop_engine = compile_engine(&[]);
    assert_eq!(
        noop_engine
            .evaluate_filter(&batch, &mut EvaluationScratch::new(), None)
            .unwrap()
            .into_bitmap(),
        DenseBitset::from_indices([0, 1, 2])
    );
    let noop_out = noop_engine
        .evaluate(&batch, &mut EvaluationScratch::new(), None)
        .unwrap();
    assert_eq!(noop_out.kept_rows, 3);
    assert_eq!(noop_out.dropped_rows, 0);

    // 3. No filters + conditional transform (fast path with predicates!)
    let tx_only = compile_engine(&[Policy::log_transform(
        "add_tag_on_error",
        LogTransformPolicy {
            matches: vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
                predicate: Predicate::Equals(ScalarValue::String("ERROR".into())),
                negate: false,
            }],
            rename: vec![],
            add: vec![LogAdd {
                target: LogTargetSelector::LogAttribute(AttributePath::single("alert")),
                value: ScalarValue::Bool(true),
                upsert: true,
            }],
            remove: vec![],
            redact: vec![],
        },
    )]);
    let mut tx_only_stats = tx_only.new_match_stats();
    let tx_out = tx_only
        .evaluate(
            &batch,
            &mut EvaluationScratch::new(),
            Some(&mut tx_only_stats),
        )
        .unwrap();
    assert_eq!(tx_out.kept_rows, 3);
    assert_eq!(tx_out.dropped_rows, 0);
    assert_eq!(
        tx_only_stats.counts()[0],
        PolicyMatchCount { hits: 1, misses: 0 }
    );

    // 4. Filter dropping ALL rows -> records == None
    let drop_all = compile_engine(&[Policy::log_filter(
        "drop_everything",
        vec![LogMatcher {
            field: LogFieldSelector::RecordField(LogRecordField::Body),
            predicate: Predicate::Exists,
            negate: false,
        }],
        Action::Drop,
    )]);
    let dropped_out = drop_all
        .evaluate(&batch, &mut EvaluationScratch::new(), None)
        .unwrap();
    assert!(dropped_out.records.is_none());
    assert_eq!(dropped_out.kept_rows, 0);
    assert_eq!(dropped_out.dropped_rows, 3);

    // 5. Partial drop WITHOUT transforms
    let partial_out = any_engine
        .evaluate(&batch, &mut EvaluationScratch::new(), None)
        .unwrap();
    assert!(partial_out.records.is_some());
    assert_eq!(partial_out.kept_rows, 2);
    assert_eq!(partial_out.dropped_rows, 1);

    // 6. Negation (first matcher negated, subsequent matcher negated, subsequent positive, and
    //    empty-accumulator short-circuit break)
    let neg_engine = compile_engine(&[
        // Drop rows where NOT ERROR AND NOT DEBUG -> drops row 1 (INFO)
        Policy::log_filter(
            "neg_first_and_second",
            vec![
                LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
                    predicate: Predicate::Equals(ScalarValue::String("ERROR".into())),
                    negate: true,
                },
                LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
                    predicate: Predicate::Equals(ScalarValue::String("DEBUG".into())),
                    negate: true,
                },
                LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::Body),
                    predicate: Predicate::Equals(ScalarValue::String("b".into())),
                    negate: false,
                },
            ],
            Action::Drop,
        ),
        // Short-circuit: first matcher yields empty bitmap so loop breaks on 2nd matcher
        Policy::log_filter(
            "short_circuit",
            vec![
                LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
                    predicate: Predicate::Equals(ScalarValue::String("NONEXISTENT".into())),
                    negate: false,
                },
                LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::Body),
                    predicate: Predicate::Exists,
                    negate: false,
                },
            ],
            Action::Drop,
        ),
    ]);
    assert_eq!(
        neg_engine
            .evaluate_filter(&batch, &mut EvaluationScratch::new(), None)
            .unwrap()
            .into_bitmap(),
        DenseBitset::from_indices([0, 2])
    );

    // 7. Filter keeping all rows + conditional transform (filter_otap_logs_by_bitmap shortcut)
    let keep_all_with_tx = compile_engine(&[
        Policy::log_filter(
            "drop_none",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
                predicate: Predicate::Equals(ScalarValue::String("FATAL".into())),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_transform(
            "tag_keep_me",
            LogTransformPolicy {
                matches: vec![LogMatcher {
                    field: LogFieldSelector::LogAttribute(AttributePath::single("keep_me")),
                    predicate: Predicate::Equals(ScalarValue::Bool(true)),
                    negate: false,
                }],
                rename: vec![],
                add: vec![LogAdd {
                    target: LogTargetSelector::LogAttribute(AttributePath::single("marked")),
                    value: ScalarValue::Int(1),
                    upsert: true,
                }],
                remove: vec![],
                redact: vec![],
            },
        ),
    ]);
    let mut kept_all_stats = keep_all_with_tx.new_match_stats();
    let kept_all_out = keep_all_with_tx
        .evaluate(
            &batch,
            &mut EvaluationScratch::new(),
            Some(&mut kept_all_stats),
        )
        .unwrap();
    assert_eq!(kept_all_out.kept_rows, 3);
    assert_eq!(kept_all_out.dropped_rows, 0);
    assert_eq!(
        kept_all_stats.counts()[1],
        PolicyMatchCount { hits: 1, misses: 0 }
    );
}

/// Scenario: Compile duplicate policies covering all finite predicate variants (including `-0.0` vs `+0.0` and wildcard-free `Glob` vs `Equals`), compile non-finite policy doubles (`NaN`, `+Inf`, `-Inf`), and evaluate runtime `NaN` and `-0.0` log values.
/// Guarantees: Canonical predicate keys deduplicate `-0.0`/`+0.0` and wildcard-free `Glob`/`Equals`, non-finite policy doubles are rejected as compile errors (`policy.md`), and runtime `NaN` log values never match finite comparisons while `-0.0 == +0.0`.
#[test]
fn test_canonical_predicate_key_dedup_all_variants_and_floats() {
    let field = LogFieldSelector::LogAttribute(AttributePath::single("x"));
    let quiet_nan_alt = f64::from_bits(0x7ff8_0000_0000_0001);

    let mk = |id: &str, pred: Predicate| {
        Policy::log_filter(
            id,
            vec![LogMatcher {
                field: field.clone(),
                predicate: pred,
                negate: false,
            }],
            Action::Drop,
        )
    };

    // Each unique predicate appears twice (with -0.0/+0.0 and wildcard-free Glob/Equals canonicalized)
    let policies = vec![
        mk("p_exists_1", Predicate::Exists),
        mk("p_exists_2", Predicate::Exists),
        mk(
            "p_eq_str_1",
            Predicate::Equals(ScalarValue::String("a".into())),
        ),
        mk(
            "p_eq_str_2",
            Predicate::Equals(ScalarValue::String("a".into())),
        ),
        // Wildcard-free Glob("a") canonicalizes into Equals(String("a"))
        mk("p_glob_exact", Predicate::Glob("a".into())),
        // Wildcard Glob deduplicates with itself
        mk("p_glob_wild_1", Predicate::Glob("foo*bar".into())),
        mk("p_glob_wild_2", Predicate::Glob("foo*bar".into())),
        mk("p_eq_int_1", Predicate::Equals(ScalarValue::Int(0))),
        mk("p_eq_int_2", Predicate::Equals(ScalarValue::Int(0))),
        mk("p_eq_bool_1", Predicate::Equals(ScalarValue::Bool(true))),
        mk("p_eq_bool_2", Predicate::Equals(ScalarValue::Bool(true))),
        mk("p_eq_bool_f", Predicate::Equals(ScalarValue::Bool(false))),
        mk(
            "p_eq_bytes_1",
            Predicate::Equals(ScalarValue::Bytes(vec![1, 2])),
        ),
        mk(
            "p_eq_bytes_2",
            Predicate::Equals(ScalarValue::Bytes(vec![1, 2])),
        ),
        // +0.0 and -0.0 collapse to the same canonical key
        mk("p_eq_pos_zero", Predicate::Equals(ScalarValue::Double(0.0))),
        mk(
            "p_eq_neg_zero",
            Predicate::Equals(ScalarValue::Double(-0.0)),
        ),
        // Non-zero finite float stays distinct from 0.0
        mk("p_eq_dbl_1", Predicate::Equals(ScalarValue::Double(1.5))),
        mk("p_eq_dbl_2", Predicate::Equals(ScalarValue::Double(1.5))),
        // Regex dedup
        mk("p_re_1", Predicate::Regex("^foo.*bar$".into())),
        mk("p_re_2", Predicate::Regex("^foo.*bar$".into())),
        // Range operators (Int and Double, including -0.0/+0.0 canonicalization)
        mk("p_gt_int_1", Predicate::Gt(NumericValue::Int(10))),
        mk("p_gt_int_2", Predicate::Gt(NumericValue::Int(10))),
        mk("p_gt_zero_p", Predicate::Gt(NumericValue::Double(0.0))),
        mk("p_gt_zero_n", Predicate::Gt(NumericValue::Double(-0.0))),
        mk("p_gte_1", Predicate::Gte(NumericValue::Double(2.5))),
        mk("p_gte_2", Predicate::Gte(NumericValue::Double(2.5))),
        mk("p_lt_1", Predicate::Lt(NumericValue::Int(5))),
        mk("p_lt_2", Predicate::Lt(NumericValue::Int(5))),
        mk("p_lte_1", Predicate::Lte(NumericValue::Double(-3.5))),
        mk("p_lte_2", Predicate::Lte(NumericValue::Double(-3.5))),
        // Contains dedup
        mk(
            "p_con_str_1",
            Predicate::Contains(ScalarValue::String("sub".into())),
        ),
        mk(
            "p_con_str_2",
            Predicate::Contains(ScalarValue::String("sub".into())),
        ),
        mk(
            "p_con_bytes_1",
            Predicate::Contains(ScalarValue::Bytes(vec![0xaa])),
        ),
        mk(
            "p_con_bytes_2",
            Predicate::Contains(ScalarValue::Bytes(vec![0xaa])),
        ),
    ];

    let engine = compile_engine(&policies);
    // 17 unique canonical predicates:
    // Exists, Equals(Str, Int(0), Bool(true), Bool(false), Bytes, Double(0.0), Double(1.5)),
    // Glob("foo*bar"), Regex, Gt(Int(10), Double(0.0)), Gte(2.5), Lt(5), Lte(-3.5),
    // Contains(Str("sub")), Contains(Bytes([0xaa]))
    assert_eq!(engine.num_predicates(), 17);

    // Non-finite doubles (NaN, -NaN, quiet NaN, +Inf, -Inf) in policies MUST be compile errors.
    let non_finite_out = PolicyEngine::compile(&[
        mk("eq_nan", Predicate::Equals(ScalarValue::Double(f64::NAN))),
        mk(
            "eq_qnan",
            Predicate::Equals(ScalarValue::Double(quiet_nan_alt)),
        ),
        mk("gt_inf", Predicate::Gt(NumericValue::Double(f64::INFINITY))),
        mk(
            "lt_ninf",
            Predicate::Lt(NumericValue::Double(f64::NEG_INFINITY)),
        ),
        mk(
            "contains_nan",
            Predicate::Contains(ScalarValue::Double(f64::NAN)),
        ),
    ])
    .unwrap();
    assert_eq!(non_finite_out.engine.num_predicates(), 0);
    assert_eq!(non_finite_out.errors.len(), 5);

    // Runtime NaN in log data still evaluates to false for all finite comparisons, while -0.0 == +0.0.
    let batch = build_custom_test_batch(
        vec![Some("row0"), Some("row1")],
        vec![9, 9],
        vec!["INFO", "INFO"],
        vec![[1u8; 16]; 2],
        vec![[1u8; 8]; 2],
        vec![
            (0, "x", ScalarValue::Double(f64::NAN)),
            (1, "x", ScalarValue::Double(0.0)),
        ],
        vec![],
    );
    let zero_engine = compile_engine(&[
        mk("zero_p", Predicate::Equals(ScalarValue::Double(0.0))),
        mk("zero_n", Predicate::Equals(ScalarValue::Double(-0.0))),
    ]);
    assert_eq!(zero_engine.num_predicates(), 1);
    // Row 1 has x == 0.0 -> dropped; row 0 has NaN -> kept
    assert_eq!(
        zero_engine
            .evaluate_filter(&batch, &mut EvaluationScratch::new(), None)
            .unwrap()
            .into_bitmap(),
        DenseBitset::from_indices([0])
    );
}

/// Scenario: Evaluate `Exists`, `Equals`, and nested-path `Exists` predicates on a log attribute whose `AttributeValueType` is `Empty`.
/// Guarantees: Top-level `Exists` matches the present `Empty` attribute key, whereas value predicates (`Equals`) and nested-path `Exists` do not match.
#[test]
fn test_empty_attribute_matches_exists_not_equals() {
    let root_rb = RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new(consts::ID, DataType::UInt16, false).with_plain_encoding(),
            Field::new(consts::SEVERITY_TEXT, DataType::Utf8, true),
        ])),
        vec![
            Arc::new(UInt16Array::from(vec![0u16, 1])),
            Arc::new(StringArray::from(vec!["INFO", "INFO"])),
        ],
    )
    .unwrap();
    let attrs_rb = RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new(consts::PARENT_ID, DataType::UInt16, false).with_plain_encoding(),
            Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
            Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
        ])),
        vec![
            Arc::new(UInt16Array::from(vec![0u16])),
            Arc::new(StringArray::from(vec!["present_empty"])),
            Arc::new(UInt8Array::from(vec![AttributeValueType::Empty as u8])),
        ],
    )
    .unwrap();
    let mut batch = OtapArrowRecords::Logs(Logs::default());
    batch.set(ArrowPayloadType::Logs, root_rb).unwrap();
    batch.set(ArrowPayloadType::LogAttrs, attrs_rb).unwrap();

    // Top-level Exists matches row 0 (key present with Empty AnyValue), keeps row 1.
    let exists_engine = compile_engine(&[Policy::log_filter(
        "drop_exists",
        vec![LogMatcher {
            field: LogFieldSelector::LogAttribute(AttributePath::single("present_empty")),
            predicate: Predicate::Exists,
            negate: false,
        }],
        Action::Drop,
    )]);
    assert_eq!(
        exists_engine
            .evaluate_filter(&batch, &mut EvaluationScratch::new(), None)
            .unwrap()
            .into_bitmap(),
        DenseBitset::from_indices([1])
    );

    // Value predicates (Equals) and nested path Exists must NOT match an Empty attribute.
    let non_match_engine = compile_engine(&[
        Policy::log_filter(
            "drop_eq_empty_str",
            vec![LogMatcher {
                field: LogFieldSelector::LogAttribute(AttributePath::single("present_empty")),
                predicate: Predicate::Equals(ScalarValue::String(String::new())),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "drop_nested_exists",
            vec![LogMatcher {
                field: LogFieldSelector::LogAttribute(AttributePath::nested([
                    "present_empty",
                    "child",
                ])),
                predicate: Predicate::Exists,
                negate: false,
            }],
            Action::Drop,
        ),
    ]);
    assert_eq!(
        non_match_engine
            .evaluate_filter(&batch, &mut EvaluationScratch::new(), None)
            .unwrap()
            .into_bitmap(),
        DenseBitset::from_indices([0, 1])
    );

    // Nested CBOR null (`0xf6`) on both LogAttribute and BodyField evaluates Exists == true and
    // value predicates == false per policy.md ("exists").
    let nested_null_map = ciborium::Value::Map(vec![(
        ciborium::Value::Text("empty_child".into()),
        ciborium::Value::Null,
    )]);
    let nested_null_bytes = encode_cbor(&nested_null_map);
    let mut nested_batch = build_custom_test_batch(
        vec![Some("r0"), Some("r1")],
        vec![9, 9],
        vec!["INFO", "INFO"],
        vec![[1u8; 16]; 2],
        vec![[1u8; 8]; 2],
        vec![],
        vec![(0, "ctx", nested_null_map)],
    );
    // Replace row 0 body with a CBOR Map containing {"empty_child": null}
    let body_struct = StructArray::new(
        vec![
            Arc::new(Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false)),
            Arc::new(Field::new(consts::ATTRIBUTE_SER, DataType::Binary, true)),
        ]
        .into(),
        vec![
            Arc::new(UInt8Array::from(vec![
                AttributeValueType::Map as u8,
                AttributeValueType::Empty as u8,
            ])),
            Arc::new(BinaryArray::from(vec![
                Some(nested_null_bytes.as_slice()),
                None,
            ])),
        ],
        None,
    );
    let orig_root = nested_batch.get(ArrowPayloadType::Logs).unwrap().clone();
    let mut fields: Vec<Arc<Field>> = orig_root.schema().fields().iter().cloned().collect();
    let mut cols: Vec<ArrayRef> = orig_root.columns().to_vec();
    let body_idx = orig_root.schema().index_of(consts::BODY).unwrap();
    fields[body_idx] = Arc::new(Field::new(
        consts::BODY,
        body_struct.data_type().clone(),
        true,
    ));
    cols[body_idx] = Arc::new(body_struct);
    nested_batch
        .set(
            ArrowPayloadType::Logs,
            RecordBatch::try_new(Arc::new(Schema::new(fields)), cols).unwrap(),
        )
        .unwrap();

    for (id, field) in [
        (
            "nested_attr_null",
            LogFieldSelector::LogAttribute(AttributePath::nested(["ctx", "empty_child"])),
        ),
        (
            "nested_body_null",
            LogFieldSelector::BodyField(AttributePath::single("empty_child")),
        ),
    ] {
        let ex = compile_engine(&[Policy::log_filter(
            format!("drop_{id}_exists"),
            vec![LogMatcher {
                field: field.clone(),
                predicate: Predicate::Exists,
                negate: false,
            }],
            Action::Drop,
        )]);
        assert_eq!(
            ex.evaluate_filter(&nested_batch, &mut EvaluationScratch::new(), None)
                .unwrap()
                .into_bitmap(),
            DenseBitset::from_indices([1]),
            "{id}: nested CBOR null must satisfy Exists"
        );
        let eq = compile_engine(&[Policy::log_filter(
            format!("drop_{id}_eq_empty"),
            vec![LogMatcher {
                field,
                predicate: Predicate::Equals(ScalarValue::String(String::new())),
                negate: false,
            }],
            Action::Drop,
        )]);
        assert_eq!(
            eq.evaluate_filter(&nested_batch, &mut EvaluationScratch::new(), None)
                .unwrap()
                .into_bitmap(),
            DenseBitset::from_indices([0, 1]),
            "{id}: nested CBOR null must NOT satisfy value predicates"
        );
    }
}

/// Scenario: Run filter and redaction regexes containing `\d`, `\D`, `\w`, `\W`, `\s`, `\S`, `\b`, `\B`, `.`, `(?i)`, and `\p{Greek}` against ASCII and Unicode inputs (Arabic-Indic digits, accented Latin, Kelvin sign, NBSP, vertical tab, Greek).
/// Guarantees: Shorthand classes and word boundaries follow RE2 ASCII-only definitions in both filtering and redaction while preserving UTF-8 `.` and explicit Unicode property classes.
#[test]
fn test_re2_ascii_shorthand_classes_in_filter_and_redact() {
    // Verify RE2 ASCII shorthand classes (\d, \D, \w, \W, \s, \S, \b, \B) in both filtering
    // and redaction while preserving UTF-8 dot and explicit Unicode property classes (\p{...}).
    let batch = build_custom_test_batch(
        vec![
            Some("7"),                    // 0: ASCII digit
            Some("\u{0661}"),             // 1: Arabic-Indic digit 1 (NOT RE2 \d)
            Some("\u{00e9}"),             // 2: Latin small letter e with acute (NOT RE2 \w)
            Some("\u{212a}"),             // 3: Kelvin sign (case-folds to 'k' in RE2 (?i)\w)
            Some("\u{00a0}"),             // 4: NBSP (NOT RE2 \s)
            Some("\x0b"),                 // 5: vertical tab (NOT RE2 \s)
            Some("\t"),                   // 6: ASCII tab (IS RE2 \s)
            Some("\u{03b1}"),             // 7: Greek alpha (matches . and \p{Greek})
            Some("\u{00e9}foo"),          // 8: \bfoo\b matches under RE2 ASCII word boundary
            Some("id=123 raw=\u{0661}2"), // 9: redaction target
        ],
        vec![9; 10],
        vec!["INFO"; 10],
        vec![[1u8; 16]; 10],
        vec![[1u8; 8]; 10],
        vec![],
        vec![],
    );

    // 1. ^\d$ drops row 0 ("7") and keeps row 1 (Arabic-Indic digit \u{0661}).
    let re_digit = compile_engine(&[Policy::log_filter(
        "drop_digit",
        vec![LogMatcher {
            field: LogFieldSelector::RecordField(LogRecordField::Body),
            predicate: Predicate::Regex(r"^\d$".into()),
            negate: false,
        }],
        Action::Drop,
    )]);
    assert_eq!(
        re_digit
            .evaluate_filter(&batch, &mut EvaluationScratch::new(), None)
            .unwrap()
            .into_bitmap(),
        DenseBitset::from_indices([1, 2, 3, 4, 5, 6, 7, 8, 9])
    );

    // 2. Bracketed [\d] matches row 0 only; ^\D$ matches single non-ASCII-digit chars (1..=7).
    let re_bracket_digit = compile_engine(&[Policy::log_filter(
        "drop_bracket_digit",
        vec![LogMatcher {
            field: LogFieldSelector::RecordField(LogRecordField::Body),
            predicate: Predicate::Regex(r"^[\d]$".into()),
            negate: false,
        }],
        Action::Drop,
    )]);
    assert_eq!(
        re_bracket_digit
            .evaluate_filter(&batch, &mut EvaluationScratch::new(), None)
            .unwrap()
            .into_bitmap(),
        DenseBitset::from_indices([1, 2, 3, 4, 5, 6, 7, 8, 9])
    );
    let re_not_digit = compile_engine(&[Policy::log_filter(
        "drop_non_digit",
        vec![LogMatcher {
            field: LogFieldSelector::RecordField(LogRecordField::Body),
            predicate: Predicate::Regex(r"^\D$".into()),
            negate: false,
        }],
        Action::Drop,
    )]);
    assert_eq!(
        re_not_digit
            .evaluate_filter(&batch, &mut EvaluationScratch::new(), None)
            .unwrap()
            .into_bitmap(),
        DenseBitset::from_indices([0, 8, 9])
    );

    // 3. In RE2, `(?i)^\w$` and `(?i)^[\w]$` both match row 0 ("7") and row 3 (Kelvin \u{212a},
    //    which case-folds to ASCII 'k') while rejecting row 2 (\u{00e9}); `(?i)^\W$` is the exact
    //    complement of `(?i)^\w$` on single-character inputs.
    for pat in [r"(?i)^\w$", r"(?i)^[\w]$"] {
        let re_word = compile_engine(&[Policy::log_filter(
            "drop_word",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Regex(pat.into()),
                negate: false,
            }],
            Action::Drop,
        )]);
        assert_eq!(
            re_word
                .evaluate_filter(&batch, &mut EvaluationScratch::new(), None)
                .unwrap()
                .into_bitmap(),
            DenseBitset::from_indices([1, 2, 4, 5, 6, 7, 8, 9]),
            "RE2 parity mismatch for {pat}"
        );
    }
    let re_not_word = compile_engine(&[Policy::log_filter(
        "drop_not_word",
        vec![LogMatcher {
            field: LogFieldSelector::RecordField(LogRecordField::Body),
            predicate: Predicate::Regex(r"(?i)^\W$".into()),
            negate: false,
        }],
        Action::Drop,
    )]);
    assert_eq!(
        re_not_word
            .evaluate_filter(&batch, &mut EvaluationScratch::new(), None)
            .unwrap()
            .into_bitmap(),
        DenseBitset::from_indices([0, 3, 8, 9])
    );

    // 4. ^\s$ matches row 6 (\t) only, NOT row 4 (NBSP) or row 5 (\x0b).
    let re_space = compile_engine(&[Policy::log_filter(
        "drop_space",
        vec![LogMatcher {
            field: LogFieldSelector::RecordField(LogRecordField::Body),
            predicate: Predicate::Regex(r"^\s$".into()),
            negate: false,
        }],
        Action::Drop,
    )]);
    assert_eq!(
        re_space
            .evaluate_filter(&batch, &mut EvaluationScratch::new(), None)
            .unwrap()
            .into_bitmap(),
        DenseBitset::from_indices([0, 1, 2, 3, 4, 5, 7, 8, 9])
    );

    // 5. \p{Greek} and UTF-8 `.` still match multi-byte UTF-8 row 7 (\u{03b1}).
    let re_greek = compile_engine(&[Policy::log_filter(
        "drop_greek",
        vec![LogMatcher {
            field: LogFieldSelector::RecordField(LogRecordField::Body),
            predicate: Predicate::Regex(r"^\p{Greek}$".into()),
            negate: false,
        }],
        Action::Drop,
    )]);
    assert_eq!(
        re_greek
            .evaluate_filter(&batch, &mut EvaluationScratch::new(), None)
            .unwrap()
            .into_bitmap(),
        DenseBitset::from_indices([0, 1, 2, 3, 4, 5, 6, 8, 9])
    );

    // 6. \bfoo\b matches row 8 (\u{00e9}foo) because \u{00e9} is a non-word character in RE2.
    let re_boundary = compile_engine(&[Policy::log_filter(
        "drop_word_boundary",
        vec![LogMatcher {
            field: LogFieldSelector::RecordField(LogRecordField::Body),
            predicate: Predicate::Regex(r"\bfoo\b".into()),
            negate: false,
        }],
        Action::Drop,
    )]);
    assert_eq!(
        re_boundary
            .evaluate_filter(&batch, &mut EvaluationScratch::new(), None)
            .unwrap()
            .into_bitmap(),
        DenseBitset::from_indices([0, 1, 2, 3, 4, 5, 6, 7, 9])
    );

    // 7. Redaction uses the same RE2 \d rules: ASCII digits redacted, Arabic-Indic \u{0661} kept.
    let redact_engine = compile_engine(&[Policy::log_transform(
        "redact_ascii_digits",
        LogTransformPolicy {
            matches: vec![],
            rename: vec![],
            add: vec![],
            remove: vec![],
            redact: vec![LogRedact {
                target: LogTargetSelector::RecordField(LogRecordField::Body),
                replacement: Some("X".into()),
                regex: Some(r"\d+".into()),
            }],
        },
    )]);
    let out = redact_engine
        .evaluate(&batch, &mut EvaluationScratch::new(), None)
        .unwrap();
    let out_records = out.records.expect("kept");
    let view = LogsBatchView::try_new(&out_records).unwrap().unwrap();
    assert_eq!(
        view.extract_field_at_row(&LogFieldSelector::RecordField(LogRecordField::Body), 1),
        Some(ExtractedValue::Str(Cow::Borrowed("\u{0661}")))
    );
    assert_eq!(
        view.extract_field_at_row(&LogFieldSelector::RecordField(LogRecordField::Body), 9),
        Some(ExtractedValue::Str(Cow::Borrowed("id=X raw=\u{0661}X")))
    );
}

/// Scenario: Evaluate `Equals` (canonical lowercase hex, uppercase hex, wrong-length, all-zero, empty `""`), `Contains`, `Glob`, and `Regex` predicates on present, all-zero, null, and missing-column `TraceId` and `SpanId` fields.
/// Guarantees: Exact canonical lowercase hex matches via pre-decoded raw bytes, uppercase/wrong-length/all-zero hex literals do not match, and absent/null/all-zero/missing-column IDs evaluate `Exists` to `false` while reading `""` for string predicates (`policy.md` "Identifier Fields").
#[test]
fn test_trace_and_span_id_raw_byte_equals_and_hex_regex() {
    let trace_a = [
        0x4b, 0xf9, 0x2f, 0x35, 0x77, 0xb3, 0x4d, 0xa6, 0xa3, 0xce, 0x92, 0x9d, 0x0e, 0x0e, 0x47,
        0x36,
    ];
    // Contains hex letters `a`/`b` so `.to_uppercase()` genuinely differs from canonical lowercase.
    let trace_b = [0xab; 16];
    let span_a = [0x00, 0xf0, 0x67, 0xaa, 0x0b, 0xa9, 0x02, 0xb7];
    let span_b = [0xcd; 8];
    let batch = build_custom_test_batch(
        vec![Some("row0"), Some("row1"), Some("row2")],
        vec![9; 3],
        vec!["INFO"; 3],
        vec![trace_a, trace_b, [0u8; 16]],
        vec![span_a, span_b, [0u8; 8]],
        vec![],
        vec![],
    );

    // Isolated falsifiable check: uppercase hex, wrong-length hex, and all-zero hex match NOTHING.
    let non_canonical_engine = compile_engine(&[
        Policy::log_filter(
            "uppercase_trace_b_no_match",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::TraceId),
                predicate: Predicate::Equals(ScalarValue::String(
                    "ABABABABABABABABABABABABABABABAB".into(),
                )),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "short_trace_no_match",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::TraceId),
                predicate: Predicate::Equals(ScalarValue::String("abab".into())),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "zero_trace_no_match",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::TraceId),
                predicate: Predicate::Equals(ScalarValue::String(
                    "00000000000000000000000000000000".into(),
                )),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "uppercase_span_b_no_match",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::SpanId),
                predicate: Predicate::Equals(ScalarValue::String("CDCDCDCDCDCDCDCD".into())),
                negate: false,
            }],
            Action::Drop,
        ),
    ]);
    assert_eq!(
        non_canonical_engine
            .evaluate_filter(&batch, &mut EvaluationScratch::new(), None)
            .unwrap()
            .into_bitmap(),
        DenseBitset::from_indices([0, 1, 2])
    );

    // Exact canonical lowercase hex matches row 0; hex regex matches row 1.
    let eq_engine = compile_engine(&[
        Policy::log_filter(
            "drop_trace_a",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::TraceId),
                predicate: Predicate::Equals(ScalarValue::String(
                    "4bf92f3577b34da6a3ce929d0e0e4736".into(),
                )),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "span_regex_drop_row1",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::SpanId),
                predicate: Predicate::Regex("^cdcd".into()),
                negate: false,
            }],
            Action::Drop,
        ),
    ]);
    assert_eq!(
        eq_engine
            .evaluate_filter(&batch, &mut EvaluationScratch::new(), None)
            .unwrap()
            .into_bitmap(),
        DenseBitset::from_indices([2])
    );

    // Absent/all-zero TraceId/SpanId (row 2) reads "" for string predicates while Exists == false.
    for (name, pred) in [
        (
            "eq_empty",
            Predicate::Equals(ScalarValue::String(String::new())),
        ),
        (
            "contains_empty",
            Predicate::Contains(ScalarValue::String(String::new())),
        ),
        ("glob_empty", Predicate::Glob(String::new())),
        ("glob_star", Predicate::Glob("*".into())),
        ("regex_empty", Predicate::Regex("^$".into())),
    ] {
        for field in [
            LogFieldSelector::RecordField(LogRecordField::TraceId),
            LogFieldSelector::RecordField(LogRecordField::SpanId),
        ] {
            let eng = compile_engine(&[Policy::log_filter(
                format!("drop_{name}"),
                vec![LogMatcher {
                    field,
                    predicate: pred.clone(),
                    negate: false,
                }],
                Action::Drop,
            )]);
            let kept = eng
                .evaluate_filter(&batch, &mut EvaluationScratch::new(), None)
                .unwrap()
                .into_bitmap();
            let expected = match name {
                "contains_empty" | "glob_star" => DenseBitset::new(),
                _ => DenseBitset::from_indices([0, 1]),
            };
            assert_eq!(kept, expected, "absent ID mismatch for {name}");
        }
    }

    // Missing trace_id / span_id columns completely: every row reads "" for Equals("") / Regex("^$").
    let no_id_root = RecordBatch::try_new(
        Arc::new(Schema::new(vec![Field::new(
            consts::SEVERITY_TEXT,
            DataType::Utf8,
            true,
        )])),
        vec![Arc::new(StringArray::from(vec!["INFO", "WARN"]))],
    )
    .unwrap();
    let mut no_id_batch = OtapArrowRecords::Logs(Logs::default());
    no_id_batch.set(ArrowPayloadType::Logs, no_id_root).unwrap();
    let missing_col_eng = compile_engine(&[Policy::log_filter(
        "drop_missing_trace_empty",
        vec![LogMatcher {
            field: LogFieldSelector::RecordField(LogRecordField::TraceId),
            predicate: Predicate::Equals(ScalarValue::String(String::new())),
            negate: false,
        }],
        Action::Drop,
    )]);
    assert!(
        missing_col_eng
            .evaluate_filter(&no_id_batch, &mut EvaluationScratch::new(), None)
            .unwrap()
            .is_empty()
    );
}

/// Scenario: Compile negated drop policies using `LogRecordField::Unspecified`, `ScopeField::Unspecified`, and empty `BodyField` paths.
/// Guarantees: Invalid selectors are rejected at compile time with per-policy errors and kept inert so a negated invalid selector cannot accidentally drop all logs.
#[test]
fn test_compile_rejects_negated_unspecified_selector_keeps_all_logs() {
    let batch = build_custom_test_batch(
        vec![Some("log-0"), Some("log-1"), Some("log-2")],
        vec![9, 9, 9],
        vec!["INFO", "INFO", "INFO"],
        vec![[1u8; 16]; 3],
        vec![[1u8; 8]; 3],
        vec![],
        vec![],
    );

    let policies = vec![
        Policy::log_filter(
            "drop_neg_unspec_record",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Unspecified),
                predicate: Predicate::Exists,
                negate: true,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "drop_neg_unspec_scope",
            vec![LogMatcher {
                field: LogFieldSelector::ScopeField(ScopeField::Unspecified),
                predicate: Predicate::Exists,
                negate: true,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "drop_neg_empty_body_field",
            vec![LogMatcher {
                field: LogFieldSelector::BodyField(AttributePath { path: vec![] }),
                predicate: Predicate::Exists,
                negate: true,
            }],
            Action::Drop,
        ),
    ];

    let out = PolicyEngine::compile(&policies).expect("compile");
    assert_eq!(out.errors.len(), 3);
    assert_eq!(out.errors[0].policy_id, "drop_neg_unspec_record");
    assert_eq!(out.errors[1].policy_id, "drop_neg_unspec_scope");
    assert_eq!(out.errors[2].policy_id, "drop_neg_empty_body_field");
    assert_eq!(out.engine.num_predicates(), 0);

    // Every input log must survive because all offending policies are kept inert
    let kept = out
        .engine
        .evaluate_filter(&batch, &mut EvaluationScratch::new(), None)
        .unwrap()
        .into_bitmap();
    assert_eq!(kept, DenseBitset::from_indices([0, 1, 2]));
}

/// Scenario: Compile a mixed batch containing invalid filter regexes, empty policy IDs, duplicate policy IDs, and valid filter and transform policies.
/// Guarantees: Invalid and duplicate-ID policies are reported in `CompileOutput::errors` without leaking predicates into `num_predicates()`, while valid policies in the same batch still execute normally.
#[test]
fn test_compile_isolates_invalid_policy_while_valid_policies_run() {
    let batch = build_custom_test_batch(
        vec![Some("drop-me"), Some("secret=abc ok")],
        vec![9, 9],
        vec!["INFO", "INFO"],
        vec![[1u8; 16]; 2],
        vec![[1u8; 8]; 2],
        vec![],
        vec![],
    );

    let policies = vec![
        // Invalid filter regex alongside valid matcher[0] must not pollute num_predicates()
        Policy::log_filter(
            "bad_filter_atomic",
            vec![
                LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
                    predicate: Predicate::Equals(ScalarValue::String("INFO".into())),
                    negate: false,
                },
                LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::Body),
                    predicate: Predicate::Regex("([unterminated".into()),
                    negate: false,
                },
            ],
            Action::Drop,
        ),
        // Empty policy ID
        Policy::log_filter(
            "",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Exists,
                negate: false,
            }],
            Action::Drop,
        ),
        // Duplicate policy IDs: both rejected for commutativity
        Policy::log_filter(
            "dup_id",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Equals(ScalarValue::String("secret=abc ok".into())),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "dup_id",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Equals(ScalarValue::String("secret=abc ok".into())),
                negate: false,
            }],
            Action::Keep,
        ),
        // Valid filter drops row 0
        Policy::log_filter(
            "valid_drop",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Equals(ScalarValue::String("drop-me".into())),
                negate: false,
            }],
            Action::Drop,
        ),
        // Valid redaction still loads and runs on surviving row 1
        Policy::log_transform(
            "valid_redact",
            LogTransformPolicy {
                matches: vec![],
                rename: vec![],
                add: vec![],
                remove: vec![],
                redact: vec![LogRedact {
                    target: LogTargetSelector::RecordField(LogRecordField::Body),
                    replacement: Some("secret=[MASK]".into()),
                    regex: Some("secret=[a-z]+".into()),
                }],
            },
        ),
    ];

    let out = PolicyEngine::compile(&policies).expect("compile");
    // Only the single predicate from `valid_drop` should be registered
    assert_eq!(out.engine.num_predicates(), 1);
    let err_ids: Vec<&str> = out.errors.iter().map(|e| e.policy_id.as_str()).collect();
    assert!(err_ids.contains(&"bad_filter_atomic"));
    assert!(err_ids.contains(&""));
    assert_eq!(err_ids.iter().filter(|&&id| id == "dup_id").count(), 2);

    let res = out
        .engine
        .evaluate(&batch, &mut EvaluationScratch::new(), None)
        .unwrap();
    assert_eq!(res.kept_rows, 1);
    assert_eq!(res.dropped_rows, 1);
    let out_records = res.records.unwrap();
    let view = LogsBatchView::try_new(&out_records).unwrap().unwrap();
    assert_eq!(
        view.extract_field_at_row(&LogFieldSelector::RecordField(LogRecordField::Body), 0),
        Some(ExtractedValue::Str(Cow::Borrowed("secret=[MASK] ok")))
    );
}

/// Scenario: Compile valid and invalid transform policies covering nested `LogAttribute`/`BodyField`, read-only `TraceId`/`SpanId`, `SeverityNumber` range `1..=24`, static `LogAdd` type checks, non-string `LogRedact`, empty `LogRedact.regex`, overlapping ancestor/descendant `LogRename`, non-finite `LogAdd` doubles, empty paths/segments, and `RecordField::Unspecified`.
/// Guarantees: Valid transform policies compile cleanly while every normative `policy.md` compile-time violation is rejected atomically with per-policy errors.
#[test]
fn test_compile_validates_transform_targets_and_rejects_invalid() {
    let policies = vec![
        Policy::log_transform(
            "nested_redact",
            LogTransformPolicy {
                matches: vec![],
                rename: vec![],
                add: vec![],
                remove: vec![],
                redact: vec![LogRedact {
                    target: LogTargetSelector::LogAttribute(AttributePath::nested([
                        "http", "token",
                    ])),
                    replacement: None,
                    regex: None,
                }],
            },
        ),
        Policy::log_transform(
            "body_field_add",
            LogTransformPolicy {
                matches: vec![],
                rename: vec![],
                add: vec![LogAdd {
                    target: LogTargetSelector::BodyField(AttributePath::single("user")),
                    value: ScalarValue::String("x".into()),
                    upsert: true,
                }],
                remove: vec![],
                redact: vec![],
            },
        ),
        Policy::log_transform(
            "nested_rename",
            LogTransformPolicy {
                matches: vec![],
                rename: vec![LogRename {
                    from: LogAttributeSelector::LogAttribute(AttributePath::nested(["a", "b"])),
                    to: LogAttributeSelector::BodyField(AttributePath::single("c")),
                    upsert: true,
                }],
                add: vec![],
                remove: vec![],
                redact: vec![],
            },
        ),
        // Same-path rename (["a"] -> ["a"]) is a valid runtime no-op; cross-root ["a"] -> BodyField(["a", "b"]) is valid.
        Policy::log_transform(
            "same_path_and_cross_root_rename_ok",
            LogTransformPolicy {
                matches: vec![],
                rename: vec![
                    LogRename {
                        from: LogAttributeSelector::LogAttribute(AttributePath::single("a")),
                        to: LogAttributeSelector::LogAttribute(AttributePath::single("a")),
                        upsert: false,
                    },
                    LogRename {
                        from: LogAttributeSelector::LogAttribute(AttributePath::single("a")),
                        to: LogAttributeSelector::BodyField(AttributePath::nested(["a", "b"])),
                        upsert: true,
                    },
                ],
                add: vec![LogAdd {
                    target: LogTargetSelector::RecordField(LogRecordField::SeverityNumber),
                    value: ScalarValue::Int(24),
                    upsert: true,
                }],
                remove: vec![LogRemove {
                    target: LogAttributeSelector::BodyField(AttributePath::single("tmp")),
                }],
                redact: vec![],
            },
        ),
        // Invalid: read-only TraceId/SpanId in LogAdd and LogRedact
        Policy::log_transform(
            "bad_readonly_ids",
            LogTransformPolicy {
                matches: vec![],
                rename: vec![],
                add: vec![
                    LogAdd {
                        target: LogTargetSelector::RecordField(LogRecordField::TraceId),
                        value: ScalarValue::String("0123456789abcdef0123456789abcdef".into()),
                        upsert: true,
                    },
                    LogAdd {
                        target: LogTargetSelector::RecordField(LogRecordField::SpanId),
                        value: ScalarValue::String("0123456789abcdef".into()),
                        upsert: true,
                    },
                ],
                remove: vec![],
                redact: vec![
                    LogRedact {
                        target: LogTargetSelector::RecordField(LogRecordField::SeverityNumber),
                        replacement: None,
                        regex: None,
                    },
                    LogRedact {
                        target: LogTargetSelector::RecordField(LogRecordField::TraceId),
                        replacement: None,
                        regex: None,
                    },
                    LogRedact {
                        target: LogTargetSelector::RecordField(LogRecordField::SpanId),
                        replacement: None,
                        regex: None,
                    },
                ],
            },
        ),
        // Invalid: static LogAdd type/range/finite violations + empty redact regex + overlapping rename
        Policy::log_transform(
            "bad_static_constraints",
            LogTransformPolicy {
                matches: vec![],
                rename: vec![
                    LogRename {
                        from: LogAttributeSelector::LogAttribute(AttributePath::single("a")),
                        to: LogAttributeSelector::LogAttribute(AttributePath::nested(["a", "b"])),
                        upsert: true,
                    },
                    LogRename {
                        from: LogAttributeSelector::BodyField(AttributePath::nested(["x", "y"])),
                        to: LogAttributeSelector::BodyField(AttributePath::single("x")),
                        upsert: true,
                    },
                ],
                add: vec![
                    LogAdd {
                        target: LogTargetSelector::RecordField(LogRecordField::SeverityText),
                        value: ScalarValue::Int(1),
                        upsert: true,
                    },
                    LogAdd {
                        target: LogTargetSelector::RecordField(LogRecordField::EventName),
                        value: ScalarValue::Bool(true),
                        upsert: true,
                    },
                    LogAdd {
                        target: LogTargetSelector::RecordField(LogRecordField::SeverityNumber),
                        value: ScalarValue::Int(0),
                        upsert: true,
                    },
                    LogAdd {
                        target: LogTargetSelector::RecordField(LogRecordField::SeverityNumber),
                        value: ScalarValue::Int(25),
                        upsert: true,
                    },
                    LogAdd {
                        target: LogTargetSelector::LogAttribute(AttributePath::single("d")),
                        value: ScalarValue::Double(f64::INFINITY),
                        upsert: true,
                    },
                ],
                remove: vec![],
                redact: vec![LogRedact {
                    target: LogTargetSelector::RecordField(LogRecordField::Body),
                    replacement: None,
                    regex: Some(String::new()),
                }],
            },
        ),
        // Invalid empty paths, empty segments, and unspecified record fields
        Policy::log_transform(
            "empty_paths_and_unspecified",
            LogTransformPolicy {
                matches: vec![],
                rename: vec![LogRename {
                    from: LogAttributeSelector::LogAttribute(AttributePath { path: vec![] }),
                    to: LogAttributeSelector::BodyField(AttributePath::nested(["a", ""])),
                    upsert: true,
                }],
                add: vec![LogAdd {
                    target: LogTargetSelector::RecordField(LogRecordField::Unspecified),
                    value: ScalarValue::String("x".into()),
                    upsert: true,
                }],
                remove: vec![LogRemove {
                    target: LogAttributeSelector::BodyField(AttributePath { path: vec![] }),
                }],
                redact: vec![LogRedact {
                    target: LogTargetSelector::LogAttribute(AttributePath { path: vec![] }),
                    replacement: None,
                    regex: None,
                }],
            },
        ),
    ];

    let out = PolicyEngine::compile(&policies).expect("compile");
    for ok_id in [
        "nested_redact",
        "body_field_add",
        "nested_rename",
        "same_path_and_cross_root_rename_ok",
    ] {
        assert!(
            !out.errors.iter().any(|e| e.policy_id == ok_id),
            "unexpected compile error for {ok_id}: {:?}",
            out.errors
        );
    }
    assert_eq!(
        out.errors
            .iter()
            .filter(|e| e.policy_id == "bad_readonly_ids")
            .count(),
        5
    );
    assert_eq!(
        out.errors
            .iter()
            .filter(|e| e.policy_id == "bad_static_constraints")
            .count(),
        8
    );
    assert_eq!(
        out.errors
            .iter()
            .filter(|e| e.policy_id == "empty_paths_and_unspecified")
            .count(),
        5
    );
}

/// Scenario: Evaluate identical filter policies over plain `StringArray` batches and `UInt8`/`UInt16` dictionary-encoded root, `body.str`, and `LogAttrs` batches containing repeated strings, empty strings, and null keys/values.
/// Guarantees: Dictionary-memoized evaluation produces the exact same kept row selection as plain string evaluation across `UInt8` and `UInt16` dictionaries and all null-encoding edge cases.
#[test]
fn test_dictionary_memoization_matches_plain_strings_u8_and_u16() {
    #[derive(Clone, Copy)]
    enum StrEncoding {
        Plain,
        DictU8,
        DictU16,
    }

    fn make_str_col(
        enc: StrEncoding,
        rows: &[Option<&str>],
        include_null_dict_val: bool,
    ) -> ArrayRef {
        match enc {
            StrEncoding::Plain => Arc::new(StringArray::from(rows.to_vec())),
            StrEncoding::DictU8 => {
                let mut unique: Vec<Option<&str>> = Vec::new();
                if include_null_dict_val {
                    unique.push(None);
                }
                let mut keys: Vec<Option<u8>> = Vec::with_capacity(rows.len());
                for (i, r) in rows.iter().enumerate() {
                    match r {
                        None => {
                            // Alternate between null dict key and valid key pointing to null dict value
                            if include_null_dict_val && i % 2 == 1 {
                                keys.push(Some(0));
                            } else {
                                keys.push(None);
                            }
                        }
                        Some(s) => {
                            let idx = if let Some(pos) = unique.iter().position(|u| *u == Some(*s))
                            {
                                pos
                            } else {
                                let pos = unique.len();
                                unique.push(Some(*s));
                                pos
                            };
                            keys.push(Some(idx as u8));
                        }
                    }
                }
                Arc::new(
                    DictionaryArray::<UInt8Type>::try_new(
                        UInt8Array::from(keys),
                        Arc::new(StringArray::from(unique)),
                    )
                    .unwrap(),
                )
            }
            StrEncoding::DictU16 => {
                let mut unique: Vec<Option<&str>> = Vec::new();
                if include_null_dict_val {
                    unique.push(None);
                }
                let mut keys: Vec<Option<u16>> = Vec::with_capacity(rows.len());
                for (i, r) in rows.iter().enumerate() {
                    match r {
                        None => {
                            if include_null_dict_val && i % 2 == 1 {
                                keys.push(Some(0));
                            } else {
                                keys.push(None);
                            }
                        }
                        Some(s) => {
                            let idx = if let Some(pos) = unique.iter().position(|u| *u == Some(*s))
                            {
                                pos
                            } else {
                                let pos = unique.len();
                                unique.push(Some(*s));
                                pos
                            };
                            keys.push(Some(idx as u16));
                        }
                    }
                }
                Arc::new(
                    DictionaryArray::<UInt16Type>::try_new(
                        UInt16Array::from(keys),
                        Arc::new(StringArray::from(unique)),
                    )
                    .unwrap(),
                )
            }
        }
    }

    fn build_batch(enc: StrEncoding) -> OtapArrowRecords {
        // OTAP schema requires `min_key_size: U16` for `body.str` and `LogAttrs.str`,
        // while `severity_text`, `event_name`, `scope.name`, and `LogAttrs.key` allow `U8` or `U16`.
        let anyval_str_enc = match enc {
            StrEncoding::Plain => StrEncoding::Plain,
            StrEncoding::DictU8 | StrEncoding::DictU16 => StrEncoding::DictU16,
        };
        // 8 log rows with repeated values, empty strings, nulls, and mixed Int body
        let body_types = UInt8Array::from(vec![
            AttributeValueType::Str as u8,
            AttributeValueType::Str as u8,
            AttributeValueType::Str as u8,
            AttributeValueType::Str as u8,
            AttributeValueType::Str as u8,
            AttributeValueType::Str as u8,
            AttributeValueType::Int as u8,
            AttributeValueType::Empty as u8,
        ]);
        let body_str = make_str_col(
            anyval_str_enc,
            &[
                Some("ERROR auth failed user=1"),
                Some("ERROR auth failed user=1"),
                Some(""),
                None,
                None,
                Some("INFO ok"),
                None,
                None,
            ],
            true,
        );
        let body_int = Arc::new(Int64Array::from(vec![
            None,
            None,
            None,
            None,
            None,
            None,
            Some(42),
            None,
        ]));
        let body_struct = StructArray::new(
            vec![
                Arc::new(Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false)),
                Arc::new(Field::new(
                    consts::ATTRIBUTE_STR,
                    body_str.data_type().clone(),
                    true,
                )),
                Arc::new(Field::new(consts::ATTRIBUTE_INT, DataType::Int64, true)),
            ]
            .into(),
            vec![Arc::new(body_types), body_str, body_int],
            None,
        );

        let sev_text = make_str_col(
            enc,
            &[
                Some("DEBUG"),
                Some("DEBUG"),
                Some(""),
                None,
                None,
                Some("WARN"),
                Some("ERROR"),
                Some("INFO"),
            ],
            true,
        );
        let event_name = make_str_col(
            enc,
            &[
                Some("auth.fail"),
                Some("auth.fail"),
                Some(""),
                Some("keep.evt"),
                None,
                Some("other"),
                None,
                Some("other"),
            ],
            true,
        );
        let scope_name = make_str_col(
            enc,
            &[
                Some("svc.auth"),
                Some("svc.auth"),
                Some(""),
                Some("svc.core"),
                Some("svc.core"),
                None,
                Some("svc.auth"),
                None,
            ],
            true,
        );
        let scope_struct = StructArray::new(
            vec![
                Arc::new(Field::new(consts::ID, DataType::UInt16, false).with_plain_encoding()),
                Arc::new(Field::new(
                    consts::NAME,
                    scope_name.data_type().clone(),
                    true,
                )),
            ]
            .into(),
            vec![
                Arc::new(UInt16Array::from(vec![0u16, 0, 1, 1, 1, 0, 0, 1])),
                scope_name,
            ],
            None,
        );
        let res_struct = StructArray::new(
            vec![Arc::new(
                Field::new(consts::ID, DataType::UInt16, false).with_plain_encoding(),
            )]
            .into(),
            vec![Arc::new(UInt16Array::from(vec![0u16, 0, 0, 0, 1, 1, 1, 1]))],
            None,
        );

        let root_rb = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::ID, DataType::UInt16, false).with_plain_encoding(),
                Field::new(consts::BODY, body_struct.data_type().clone(), true),
                Field::new(consts::SEVERITY_TEXT, sev_text.data_type().clone(), true),
                Field::new(consts::EVENT_NAME, event_name.data_type().clone(), true),
                Field::new(consts::RESOURCE, res_struct.data_type().clone(), true),
                Field::new(consts::SCOPE, scope_struct.data_type().clone(), true),
            ])),
            vec![
                Arc::new(UInt16Array::from(vec![0u16, 1, 2, 3, 4, 5, 6, 7])),
                Arc::new(body_struct),
                sev_text,
                event_name,
                Arc::new(res_struct),
                Arc::new(scope_struct),
            ],
        )
        .unwrap();

        // LogAttrs: repeated keys (watched + unwatched), repeated values, null str payload, empty string, Int attr
        let attr_keys = make_str_col(
            enc,
            &[
                Some("env"),
                Some("env"),
                Some("env"),
                Some("env"),
                Some("unwatched"),
                Some("unwatched"),
                Some("flag"),
                Some("count"),
            ],
            true,
        );
        let attr_str = make_str_col(
            anyval_str_enc,
            &[
                Some("prod"),
                Some("prod"),
                Some(""),
                None,
                Some("ignored"),
                Some("ignored"),
                None,
                None,
            ],
            true,
        );
        let log_attrs = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false).with_plain_encoding(),
                Field::new(consts::ATTRIBUTE_KEY, attr_keys.data_type().clone(), true),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(consts::ATTRIBUTE_STR, attr_str.data_type().clone(), true),
                Field::new(consts::ATTRIBUTE_INT, DataType::Int64, true),
            ])),
            vec![
                Arc::new(UInt16Array::from(vec![0u16, 1, 2, 3, 4, 5, 6, 7])),
                attr_keys,
                Arc::new(UInt8Array::from(vec![
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Empty as u8,
                    AttributeValueType::Int as u8,
                ])),
                attr_str,
                Arc::new(Int64Array::from(vec![
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(7),
                ])),
            ],
        )
        .unwrap();

        let mut batch = OtapArrowRecords::Logs(Logs::default());
        batch.set(ArrowPayloadType::Logs, root_rb).unwrap();
        batch.set(ArrowPayloadType::LogAttrs, log_attrs).unwrap();
        batch
    }

    let plain_batch = build_batch(StrEncoding::Plain);
    let u8_batch = build_batch(StrEncoding::DictU8);
    let u16_batch = build_batch(StrEncoding::DictU16);

    let test_policy_sets: Vec<Vec<Policy>> = vec![
        // Body regex + empty string equals + Int equals
        vec![
            Policy::log_filter(
                "drop_body_re",
                vec![LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::Body),
                    predicate: Predicate::Regex(r"auth failed user=\d+".into()),
                    negate: false,
                }],
                Action::Drop,
            ),
            Policy::log_filter(
                "drop_body_empty_str",
                vec![LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::Body),
                    predicate: Predicate::Equals(ScalarValue::String(String::new())),
                    negate: false,
                }],
                Action::Drop,
            ),
            Policy::log_filter(
                "drop_body_int",
                vec![LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::Body),
                    predicate: Predicate::Equals(ScalarValue::Int(42)),
                    negate: false,
                }],
                Action::Drop,
            ),
        ],
        // SeverityText & EventName & ScopeField::Name (empty string must NOT match Exists!)
        vec![
            Policy::log_filter(
                "drop_sev_exists",
                vec![LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
                    predicate: Predicate::Exists,
                    negate: false,
                }],
                Action::Drop,
            ),
            Policy::log_filter(
                "keep_evt",
                vec![LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::EventName),
                    predicate: Predicate::Equals(ScalarValue::String("keep.evt".into())),
                    negate: false,
                }],
                Action::Keep,
            ),
            Policy::log_filter(
                "keep_scope",
                vec![LogMatcher {
                    field: LogFieldSelector::ScopeField(ScopeField::Name),
                    predicate: Predicate::Contains(ScalarValue::String("core".into())),
                    negate: false,
                }],
                Action::Keep,
            ),
        ],
        // LogAttrs: Exists (including null-str row 3 and Empty row 6!), Equals(""), Regex, and Int
        vec![
            Policy::log_filter(
                "drop_env_exists",
                vec![LogMatcher {
                    field: LogFieldSelector::LogAttribute(AttributePath::single("env")),
                    predicate: Predicate::Exists,
                    negate: false,
                }],
                Action::Drop,
            ),
            Policy::log_filter(
                "keep_env_empty",
                vec![LogMatcher {
                    field: LogFieldSelector::LogAttribute(AttributePath::single("env")),
                    predicate: Predicate::Equals(ScalarValue::String(String::new())),
                    negate: false,
                }],
                Action::Keep,
            ),
            Policy::log_filter(
                "drop_flag_exists",
                vec![LogMatcher {
                    field: LogFieldSelector::LogAttribute(AttributePath::single("flag")),
                    predicate: Predicate::Exists,
                    negate: false,
                }],
                Action::Drop,
            ),
            Policy::log_filter(
                "drop_count_gt",
                vec![LogMatcher {
                    field: LogFieldSelector::LogAttribute(AttributePath::single("count")),
                    predicate: Predicate::Gt(NumericValue::Int(5)),
                    negate: false,
                }],
                Action::Drop,
            ),
        ],
    ];

    for policies in &test_policy_sets {
        let engine = compile_engine(policies);
        let expected = engine
            .evaluate_filter(&plain_batch, &mut EvaluationScratch::new(), None)
            .unwrap();
        assert_eq!(
            engine
                .evaluate_filter(&u8_batch, &mut EvaluationScratch::new(), None)
                .unwrap(),
            expected
        );
        assert_eq!(
            engine
                .evaluate_filter(&u16_batch, &mut EvaluationScratch::new(), None)
                .unwrap(),
            expected
        );
    }

    // Spot-check specific expectations on the 3rd policy set (LogAttrs):
    // Rows 0, 1 (env=prod), 6 (flag=Empty -> Exists), 7 (count=7) dropped;
    // Rows 2 and 3 (env="" and env=Str(null) -> default "") kept by keep_env_empty;
    // Rows 4, 5 kept (only unwatched attr).
    let attr_engine = compile_engine(&test_policy_sets[2]);
    assert_eq!(
        attr_engine
            .evaluate_filter(&u8_batch, &mut EvaluationScratch::new(), None)
            .unwrap()
            .into_bitmap(),
        DenseBitset::from_indices([2, 3, 4, 5])
    );
}

/// Scenario: Evaluate filter policies over `UInt16` dictionaries exceeding the 1024-entry dense threshold (`body.str`, `severity_text`, `LogAttrs.key`, and `LogAttrs.str`).
/// Guarantees: Both `DictStrEvalCache` and `DictKeyLookupCache` switch cleanly to the sparse `AHashMap` path and match all expected rows without missing entries or panicking.
#[test]
fn test_dictionary_memoization_sparse_u16_and_root_columns() {
    // Exercise > 1024 dictionary entries to cover the Sparse (AHashMap) cache path for both
    // DictStrEvalCache (root SeverityText, StructArray body.str, and attr values) and
    // DictKeyLookupCache (attr keys).
    let dict_vals: Vec<String> = (0..1_100).map(|i| format!("val_{i:04}")).collect();
    let dict_val_refs: Vec<&str> = dict_vals.iter().map(String::as_str).collect();
    let body_str_dict: ArrayRef = Arc::new(
        DictionaryArray::<UInt16Type>::try_new(
            UInt16Array::from(vec![Some(1050u16), Some(1050), Some(5), None]),
            Arc::new(StringArray::from(dict_val_refs.clone())),
        )
        .unwrap(),
    );
    let body_struct = StructArray::new(
        vec![
            Arc::new(Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false)),
            Arc::new(Field::new(
                consts::ATTRIBUTE_STR,
                body_str_dict.data_type().clone(),
                true,
            )),
        ]
        .into(),
        vec![
            Arc::new(UInt8Array::from(vec![AttributeValueType::Str as u8; 4])),
            body_str_dict,
        ],
        None,
    );
    let sev_text_dict: ArrayRef = Arc::new(
        DictionaryArray::<UInt16Type>::try_new(
            UInt16Array::from(vec![Some(5u16), Some(5), Some(1050), Some(5)]),
            Arc::new(StringArray::from(dict_val_refs.clone())),
        )
        .unwrap(),
    );

    let key_vals: Vec<String> = (0..1_100)
        .map(|i| {
            if i == 1099 {
                "watched_key".to_owned()
            } else if i == 1098 {
                "watched_key_2".to_owned()
            } else {
                format!("k_{i:04}")
            }
        })
        .collect();
    let key_val_refs: Vec<&str> = key_vals.iter().map(String::as_str).collect();
    let attr_key_dict: ArrayRef = Arc::new(
        DictionaryArray::<UInt16Type>::try_new(
            UInt16Array::from(vec![1099u16, 1099, 1098, 10]),
            Arc::new(StringArray::from(key_val_refs)),
        )
        .unwrap(),
    );
    let attr_str_dict: ArrayRef = Arc::new(
        DictionaryArray::<UInt16Type>::try_new(
            UInt16Array::from(vec![Some(1050u16), Some(1050), Some(1050), Some(5)]),
            Arc::new(StringArray::from(dict_val_refs)),
        )
        .unwrap(),
    );

    let root_rb = RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new(consts::ID, DataType::UInt16, false).with_plain_encoding(),
            Field::new(consts::BODY, body_struct.data_type().clone(), true),
            Field::new(
                consts::SEVERITY_TEXT,
                sev_text_dict.data_type().clone(),
                true,
            ),
        ])),
        vec![
            Arc::new(UInt16Array::from(vec![0u16, 1, 2, 3])),
            Arc::new(body_struct),
            sev_text_dict,
        ],
    )
    .unwrap();
    let log_attrs = RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new(consts::PARENT_ID, DataType::UInt16, false).with_plain_encoding(),
            Field::new(
                consts::ATTRIBUTE_KEY,
                attr_key_dict.data_type().clone(),
                false,
            ),
            Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
            Field::new(
                consts::ATTRIBUTE_STR,
                attr_str_dict.data_type().clone(),
                true,
            ),
        ])),
        vec![
            Arc::new(UInt16Array::from(vec![3u16, 3, 2, 3])),
            attr_key_dict,
            Arc::new(UInt8Array::from(vec![AttributeValueType::Str as u8; 4])),
            attr_str_dict,
        ],
    )
    .unwrap();

    let mut batch = OtapArrowRecords::Logs(Logs::default());
    batch.set(ArrowPayloadType::Logs, root_rb).unwrap();
    batch.set(ArrowPayloadType::LogAttrs, log_attrs).unwrap();

    // Two watched keys (`key_map.len() == 2` with `dict_len > 1024`) forces `DictKeyLookupCache`,
    // and `Contains`/`Regex` on `SeverityText` forces root `DictStrEvalCache::Sparse`.
    let engine = compile_engine(&[
        Policy::log_filter(
            "drop_body_sparse_dict",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Regex(r"^val_1050$".into()),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "drop_sev_sparse_dict",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
                predicate: Predicate::Contains(ScalarValue::String("1050".into())),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "drop_attr_sparse_dict",
            vec![LogMatcher {
                field: LogFieldSelector::LogAttribute(AttributePath::single("watched_key")),
                predicate: Predicate::Regex(r"^val_1050$".into()),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "drop_attr2_sparse_dict",
            vec![LogMatcher {
                field: LogFieldSelector::LogAttribute(AttributePath::single("watched_key_2")),
                predicate: Predicate::Equals(ScalarValue::String("nonexistent".into())),
                negate: false,
            }],
            Action::Drop,
        ),
    ]);

    // Rows 0 and 1 dropped by body.str regex; row 2 dropped by severity_text contains; row 3 dropped by watched_key
    let kept = engine
        .evaluate_filter(&batch, &mut EvaluationScratch::new(), None)
        .unwrap();
    assert!(kept.is_empty());
}

/// Scenario: Encode all-default `AnyValue` entries (`""`, `0`, `0.0`, `b""`) via `pdata::encode_logs_otap_batch` (`default_values_optional: true`), verify value columns are omitted, and run filter predicates plus sibling transforms.
/// Guarantees: Predicates match every omitted default value according to `ResolvedAnyValue::resolve` semantics, and sibling transforms preserve all omitted-default attributes and bodies across OTLP export.
#[test]
fn test_default_values_optional_omitted_columns_match_and_survive_transforms() {
    use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{
        AnyValue, InstrumentationScope, KeyValue, any_value::Value,
    };
    use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::{
        LogRecord, LogsData, ResourceLogs, ScopeLogs,
    };
    use otel_arrow_dfe_pdata::proto::opentelemetry::resource::v1::Resource;

    // Encode all-default AnyValues via pdata's real `encode_logs_otap_batch` (`default_values_optional: true`),
    // which omits all-default `str`/`int`/`double`/`bytes` columns from `Logs.body` and `LogAttrs`.
    let logs_data = LogsData::new(vec![ResourceLogs::new(
        Resource::default(),
        vec![ScopeLogs::new(
            InstrumentationScope::default(),
            vec![LogRecord {
                body: Some(AnyValue {
                    value: Some(Value::StringValue(String::new())),
                }),
                attributes: vec![
                    KeyValue {
                        key: "s_empty".into(),
                        value: Some(AnyValue {
                            value: Some(Value::StringValue(String::new())),
                        }),
                    },
                    KeyValue {
                        key: "i_zero".into(),
                        value: Some(AnyValue {
                            value: Some(Value::IntValue(0)),
                        }),
                    },
                    KeyValue {
                        key: "d_zero".into(),
                        value: Some(AnyValue {
                            value: Some(Value::DoubleValue(0.0)),
                        }),
                    },
                    KeyValue {
                        key: "b_empty".into(),
                        value: Some(AnyValue {
                            value: Some(Value::BytesValue(Vec::new())),
                        }),
                    },
                ],
                ..Default::default()
            }],
        )],
    )]);

    let otap_batch = encode_otlp_logs(&logs_data);
    // Sanity-check that pdata actually omitted the all-default columns
    let attrs_rb = otap_batch.get(ArrowPayloadType::LogAttrs).unwrap();
    assert!(attrs_rb.column_by_name(consts::ATTRIBUTE_STR).is_none());
    assert!(attrs_rb.column_by_name(consts::ATTRIBUTE_INT).is_none());
    assert!(attrs_rb.column_by_name(consts::ATTRIBUTE_DOUBLE).is_none());
    assert!(attrs_rb.column_by_name(consts::ATTRIBUTE_BYTES).is_none());

    // Predicates must match every omitted default value (`""`, `0`, `0.0`, `b""`)
    let match_engine = compile_engine(&[Policy::log_filter(
        "match_all_defaults",
        vec![
            LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Equals(ScalarValue::String(String::new())),
                negate: false,
            },
            LogMatcher {
                field: LogFieldSelector::LogAttribute(AttributePath::single("s_empty")),
                predicate: Predicate::Equals(ScalarValue::String(String::new())),
                negate: false,
            },
            LogMatcher {
                field: LogFieldSelector::LogAttribute(AttributePath::single("i_zero")),
                predicate: Predicate::Equals(ScalarValue::Int(0)),
                negate: false,
            },
            LogMatcher {
                field: LogFieldSelector::LogAttribute(AttributePath::single("d_zero")),
                predicate: Predicate::Equals(ScalarValue::Double(0.0)),
                negate: false,
            },
            LogMatcher {
                field: LogFieldSelector::LogAttribute(AttributePath::single("b_empty")),
                predicate: Predicate::Equals(ScalarValue::Bytes(Vec::new())),
                negate: false,
            },
        ],
        Action::Drop,
    )]);
    assert!(
        match_engine
            .evaluate_filter(&otap_batch, &mut EvaluationScratch::new(), None)
            .unwrap()
            .is_empty()
    );

    // A sibling transform touching LogAttrs and Body:
    // - LogAttribute("s_empty") holds `""`, which IS present on an attribute -> upsert=false does NOT overwrite.
    // - LogRecord.body holds `""`, which is treated as absent on first-class Body -> upsert=false fills it.
    let tx_engine = compile_engine(&[Policy::log_transform(
        "add_sibling",
        LogTransformPolicy {
            matches: vec![],
            rename: vec![],
            add: vec![
                LogAdd {
                    target: LogTargetSelector::LogAttribute(AttributePath::single("added")),
                    value: ScalarValue::String("ok".into()),
                    upsert: true,
                },
                LogAdd {
                    target: LogTargetSelector::LogAttribute(AttributePath::single("s_empty")),
                    value: ScalarValue::String("should_not_overwrite_attr".into()),
                    upsert: false,
                },
                LogAdd {
                    target: LogTargetSelector::RecordField(LogRecordField::Body),
                    value: ScalarValue::String("filled_empty_body".into()),
                    upsert: false,
                },
            ],
            remove: vec![],
            redact: vec![],
        },
    )]);
    let out = tx_engine
        .evaluate(&otap_batch, &mut EvaluationScratch::new(), None)
        .unwrap();
    let exported = decode_otlp_logs(out.records.unwrap());
    let rec = &exported.resource_logs[0].scope_logs[0].log_records[0];
    assert_eq!(
        rec.body,
        Some(AnyValue {
            value: Some(Value::StringValue("filled_empty_body".into())),
        })
    );
    let attrs: std::collections::BTreeMap<&str, &Option<AnyValue>> = rec
        .attributes
        .iter()
        .map(|kv| (kv.key.as_str(), &kv.value))
        .collect();
    assert_eq!(
        attrs.get("s_empty").copied().cloned().flatten(),
        Some(AnyValue {
            value: Some(Value::StringValue(String::new())),
        })
    );
    assert_eq!(
        attrs.get("i_zero").copied().cloned().flatten(),
        Some(AnyValue {
            value: Some(Value::IntValue(0)),
        })
    );
    assert_eq!(
        attrs.get("d_zero").copied().cloned().flatten(),
        Some(AnyValue {
            value: Some(Value::DoubleValue(0.0)),
        })
    );
    assert_eq!(
        attrs.get("b_empty").copied().cloned().flatten(),
        Some(AnyValue {
            value: Some(Value::BytesValue(Vec::new())),
        })
    );
    assert_eq!(
        attrs.get("added").copied().cloned().flatten(),
        Some(AnyValue {
            value: Some(Value::StringValue("ok".into())),
        })
    );
}

/// Scenario: Build a batch where row 1 has null `resource` and `scope` `StructArray` entries whose child buffers still hold non-null placeholder IDs and strings matching row 0.
/// Guarantees: `LogsBatchView` and `PolicyEngine` gate child reads on parent struct validity so null `resource`/`scope` rows return `None` and never inherit another row's attributes or scope fields.
#[test]
fn test_null_resource_and_scope_struct_rows_ignore_child_placeholders() {
    // Row 0 has valid resource/scope structs; row 1 has NULL structs whose underlying Arrow child
    // buffers still carry non-null placeholder values (id = 0, name = "leaked", version = "1.0").
    let res_struct = StructArray::new(
        vec![Arc::new(
            Field::new(consts::ID, DataType::UInt16, false).with_plain_encoding(),
        )]
        .into(),
        vec![Arc::new(UInt16Array::from(vec![0u16, 0]))],
        Some(NullBuffer::from(vec![true, false])),
    );
    let scope_name: ArrayRef = Arc::new(
        DictionaryArray::<UInt8Type>::try_new(
            UInt8Array::from(vec![0u8, 0]),
            Arc::new(StringArray::from(vec!["leaked.scope"])),
        )
        .unwrap(),
    );
    let scope_ver: ArrayRef = Arc::new(StringArray::from(vec!["1.0", "1.0"]));
    let scope_struct = StructArray::new(
        vec![
            Arc::new(Field::new(consts::ID, DataType::UInt16, false).with_plain_encoding()),
            Arc::new(Field::new(
                consts::NAME,
                scope_name.data_type().clone(),
                true,
            )),
            Arc::new(Field::new(consts::VERSION, DataType::Utf8, true)),
        ]
        .into(),
        vec![
            Arc::new(UInt16Array::from(vec![0u16, 0])),
            scope_name,
            scope_ver,
        ],
        Some(NullBuffer::from(vec![true, false])),
    );

    let root_rb = RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new(consts::ID, DataType::UInt16, false).with_plain_encoding(),
            Field::new(consts::RESOURCE, res_struct.data_type().clone(), true),
            Field::new(consts::SCOPE, scope_struct.data_type().clone(), true),
        ])),
        vec![
            Arc::new(UInt16Array::from(vec![0u16, 1])),
            Arc::new(res_struct),
            Arc::new(scope_struct),
        ],
    )
    .unwrap();
    let res_attrs = build_attr_batch(
        vec![(0, "service.name", ScalarValue::String("checkout".into()))],
        vec![],
    );
    let scope_attrs = build_attr_batch(
        vec![(0, "lib.owner", ScalarValue::String("core".into()))],
        vec![],
    );

    let mut batch = OtapArrowRecords::Logs(Logs::default());
    batch.set(ArrowPayloadType::Logs, root_rb).unwrap();
    batch
        .set(ArrowPayloadType::ResourceAttrs, res_attrs)
        .unwrap();
    batch
        .set(ArrowPayloadType::ScopeAttrs, scope_attrs)
        .unwrap();

    // Direct LogsBatchView checks: row 1 must see None for all resource/scope fields & IDs
    let view = LogsBatchView::try_new(&batch).unwrap().unwrap();
    assert_eq!(view.row_to_resource_id(0), Some(0));
    assert_eq!(view.row_to_resource_id(1), None);
    assert_eq!(view.row_to_scope_id(0), Some(0));
    assert_eq!(view.row_to_scope_id(1), None);
    assert_eq!(
        view.extract_field_at_row(&LogFieldSelector::ScopeField(ScopeField::Name), 1),
        None
    );
    assert_eq!(
        view.extract_field_at_row(&LogFieldSelector::ScopeField(ScopeField::Version), 1),
        None
    );

    // Filter evaluation: each policy must match ONLY row 0 and keep row 1
    let selectors = vec![
        LogFieldSelector::ResourceAttribute(AttributePath::single("service.name")),
        LogFieldSelector::ScopeAttribute(AttributePath::single("lib.owner")),
        LogFieldSelector::ScopeField(ScopeField::Name),
        LogFieldSelector::ScopeField(ScopeField::Version),
    ];
    for (idx, selector) in selectors.into_iter().enumerate() {
        let engine = compile_engine(&[Policy::log_filter(
            format!("drop_{idx}"),
            vec![LogMatcher {
                field: selector,
                predicate: Predicate::Exists,
                negate: false,
            }],
            Action::Drop,
        )]);
        assert_eq!(
            engine
                .evaluate_filter(&batch, &mut EvaluationScratch::new(), None)
                .unwrap()
                .into_bitmap(),
            DenseBitset::from_indices([1])
        );
    }
}

/// Scenario: Compile filter and redaction policies containing an over-length regex (> 4096 bytes) and a bounded repetition pattern exceeding the 256 KiB Thompson NFA size cap alongside a valid policy.
/// Guarantees: Oversized filter and redaction regexes are rejected cleanly during compilation without allocating unbounded automata, while valid sibling policies still compile.
#[test]
fn test_compile_enforces_regex_length_and_nfa_size_bounds() {
    let oversized_len = Policy::log_filter(
        "too_long",
        vec![LogMatcher {
            field: LogFieldSelector::RecordField(LogRecordField::Body),
            predicate: Predicate::Regex("a".repeat(5000)),
            negate: false,
        }],
        Action::Drop,
    );
    // Large bounded repetition triggers Thompson NFA size limit (256 KiB)
    let oversized_nfa = Policy::log_filter(
        "too_big_nfa",
        vec![LogMatcher {
            field: LogFieldSelector::RecordField(LogRecordField::Body),
            predicate: Predicate::Regex("a{50000}".into()),
            negate: false,
        }],
        Action::Drop,
    );
    let oversized_redact_nfa = Policy::log_transform(
        "too_big_redact_nfa",
        LogTransformPolicy {
            matches: vec![],
            rename: vec![],
            add: vec![],
            remove: vec![],
            redact: vec![LogRedact {
                target: LogTargetSelector::RecordField(LogRecordField::Body),
                replacement: None,
                regex: Some("a{50000}".into()),
            }],
        },
    );
    let valid = Policy::log_filter(
        "still_valid",
        vec![LogMatcher {
            field: LogFieldSelector::RecordField(LogRecordField::Body),
            predicate: Predicate::Regex("ok".into()),
            negate: false,
        }],
        Action::Drop,
    );

    let out = PolicyEngine::compile(&[oversized_len, oversized_nfa, oversized_redact_nfa, valid])
        .expect("compile isolates oversized regexes");
    let err_ids: Vec<&str> = out.errors.iter().map(|e| e.policy_id.as_str()).collect();
    assert!(err_ids.contains(&"too_long"));
    assert!(err_ids.contains(&"too_big_nfa"));
    assert!(err_ids.contains(&"too_big_redact_nfa"));
    assert_eq!(out.engine.num_predicates(), 1);
}

/// Scenario: Reuse one `EvaluationScratch` across two compiled engines (with different field and n-gram counts) and two log batches, then pass `Metrics` and `Traces` batches to `evaluate` and `evaluate_filter`.
/// Guarantees: Scratch buffers reset cleanly across batches and policy reloads with zero cross-batch leakage, and non-`Logs` signals fail fast with `PolicyError::Evaluation` while leaving the caller's borrowed batch intact.
#[test]
fn test_evaluation_scratch_reuse_and_signal_mismatch_errors() {
    use otel_arrow_dfe_pdata::otap::{Metrics, Traces};

    let mut scratch = EvaluationScratch::new();
    let batch_a = build_custom_test_batch(
        vec![Some("alpha error"), Some("beta info")],
        vec![17, 9],
        vec!["ERROR", "INFO"],
        vec![[1u8; 16]; 2],
        vec![[1u8; 8]; 2],
        vec![(0, "env", ScalarValue::String("prod".into()))],
        vec![],
    );
    let batch_b = build_custom_test_batch(
        vec![Some("gamma warn"), Some("alpha error")],
        vec![13, 17],
        vec!["WARN", "ERROR"],
        vec![[2u8; 16]; 2],
        vec![[2u8; 8]; 2],
        vec![(1, "env", ScalarValue::String("prod".into()))],
        vec![],
    );

    // First engine with 1 regex predicate; second engine reloads with 2 fields + different n-gram counts
    let eng1 = compile_engine(&[Policy::log_filter(
        "drop_alpha",
        vec![LogMatcher {
            field: LogFieldSelector::RecordField(LogRecordField::Body),
            predicate: Predicate::Regex("alpha.*error".into()),
            negate: false,
        }],
        Action::Drop,
    )]);
    let eng2 = compile_engine(&[
        Policy::log_filter(
            "drop_alpha_and_prod",
            vec![
                LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::Body),
                    predicate: Predicate::Regex("alpha.*error".into()),
                    negate: false,
                },
                LogMatcher {
                    field: LogFieldSelector::LogAttribute(AttributePath::single("env")),
                    predicate: Predicate::Regex("^prod$".into()),
                    negate: false,
                },
            ],
            Action::Drop,
        ),
        Policy::log_transform(
            "add_tag",
            LogTransformPolicy {
                matches: vec![],
                rename: vec![],
                add: vec![LogAdd {
                    target: LogTargetSelector::LogAttribute(AttributePath::single("checked")),
                    value: ScalarValue::Bool(true),
                    upsert: true,
                }],
                remove: vec![],
                redact: vec![],
            },
        ),
    ]);

    let out_a = eng1.evaluate(&batch_a, &mut scratch, None).unwrap();
    assert_eq!(out_a.kept_rows, 1);
    assert_eq!(out_a.dropped_rows, 1);

    // Reuse the same scratch across a policy reload (`eng2`) and a second batch (`batch_b`)
    let mut stats_b = eng2.new_match_stats();
    let out_b = eng2
        .evaluate(&batch_b, &mut scratch, Some(&mut stats_b))
        .unwrap();
    assert_eq!(out_b.kept_rows, 1);
    assert_eq!(out_b.dropped_rows, 1);
    assert_eq!(stats_b.evaluated_rows(), 2);
    assert_eq!(
        stats_b.iter(&eng2).unwrap().collect::<Vec<_>>(),
        vec![
            (
                "drop_alpha_and_prod",
                PolicyMatchCount { hits: 1, misses: 0 }
            ),
            ("add_tag", PolicyMatchCount { hits: 1, misses: 0 }),
        ]
    );

    // Metrics and Traces must fail fast with PolicyError::Evaluation while leaving caller's batch intact
    let metrics_batch = OtapArrowRecords::Metrics(Metrics::default());
    let traces_batch = OtapArrowRecords::Traces(Traces::default());
    assert!(matches!(
        eng2.evaluate(&metrics_batch, &mut scratch, None),
        Err(PolicyError::Evaluation(_))
    ));
    assert!(matches!(
        eng2.evaluate_filter(&metrics_batch, &mut scratch, None),
        Err(PolicyError::Evaluation(_))
    ));
    assert!(matches!(
        eng2.evaluate(&traces_batch, &mut scratch, None),
        Err(PolicyError::Evaluation(_))
    ));
    assert!(matches!(
        eng2.evaluate_filter(&traces_batch, &mut scratch, None),
        Err(PolicyError::Evaluation(_))
    ));
    // Caller still owns both original batches for nack handling
    assert!(matches!(metrics_batch, OtapArrowRecords::Metrics(_)));
    assert!(matches!(traces_batch, OtapArrowRecords::Traces(_)));
}

/// Scenario: Evaluate root-only and log-attribute-only engines against a full batch with LogAttrs, ResourceAttrs, and ScopeAttrs, plus a mixed filter/transform engine with 1 filter predicate and 4 transform predicates.
/// Guarantees: `LogsBatchView` never builds the `log_id_to_row` index when unused, and the filter stage allocates predicate bitmaps only for filter predicates rather than transform predicates.
#[test]
fn test_lazy_batch_view_indexing_and_split_stage_predicates() {
    let batch = build_custom_test_batch(
        vec![Some("alpha"), Some("beta")],
        vec![9, 17],
        vec!["INFO", "ERROR"],
        vec![[1u8; 16]; 2],
        vec![[1u8; 8]; 2],
        vec![(0, "env", ScalarValue::String("prod".into()))],
        vec![],
    );

    // 1. Root-only filter must not build the log_id_to_row lookup index.
    let root_only = compile_engine(&[Policy::log_filter(
        "drop_err",
        vec![LogMatcher {
            field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
            predicate: Predicate::Equals(ScalarValue::String("ERROR".into())),
            negate: false,
        }],
        Action::Drop,
    )]);
    let view = LogsBatchView::try_new(&batch).unwrap().unwrap();
    assert!(!view.is_log_index_built());
    let bitmaps =
        root_only.evaluate_filter_predicates_for_test(&view, &mut EvaluationScratch::new());
    assert_eq!(bitmaps.len(), 1);
    assert!(!view.is_log_index_built());

    // 2. LogAttribute-only filter builds log_id_to_row on demand.
    let log_attr_only = compile_engine(&[Policy::log_filter(
        "drop_prod",
        vec![LogMatcher {
            field: LogFieldSelector::LogAttribute(AttributePath::single("env")),
            predicate: Predicate::Equals(ScalarValue::String("prod".into())),
            negate: false,
        }],
        Action::Drop,
    )]);
    let view2 = LogsBatchView::try_new(&batch).unwrap().unwrap();
    let _ =
        log_attr_only.evaluate_filter_predicates_for_test(&view2, &mut EvaluationScratch::new());
    assert!(view2.is_log_index_built());

    // 3. Mixed engine: 1 filter predicate + 4 transform predicates -> filter stage allocates 1 bitmap.
    let mixed = compile_engine(&[
        Policy::log_filter(
            "drop_err",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
                predicate: Predicate::Equals(ScalarValue::String("ERROR".into())),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_transform(
            "tx_many",
            LogTransformPolicy {
                matches: vec![
                    LogMatcher {
                        field: LogFieldSelector::RecordField(LogRecordField::Body),
                        predicate: Predicate::Contains(ScalarValue::String("alpha".into())),
                        negate: false,
                    },
                    LogMatcher {
                        field: LogFieldSelector::RecordField(LogRecordField::SeverityNumber),
                        predicate: Predicate::Gte(NumericValue::Int(9)),
                        negate: false,
                    },
                    LogMatcher {
                        field: LogFieldSelector::LogAttribute(AttributePath::single("env")),
                        predicate: Predicate::Equals(ScalarValue::String("prod".into())),
                        negate: false,
                    },
                    LogMatcher {
                        field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
                        predicate: Predicate::Exists,
                        negate: false,
                    },
                ],
                rename: vec![],
                add: vec![LogAdd {
                    target: LogTargetSelector::LogAttribute(AttributePath::single("tagged")),
                    value: ScalarValue::Bool(true),
                    upsert: true,
                }],
                remove: vec![],
                redact: vec![],
            },
        ),
    ]);
    assert_eq!(mixed.num_filter_predicates(), 1);
    assert_eq!(mixed.num_transform_predicates(), 4);
    let view3 = LogsBatchView::try_new(&batch).unwrap().unwrap();
    let filter_bms =
        mixed.evaluate_filter_predicates_for_test(&view3, &mut EvaluationScratch::new());
    assert_eq!(filter_bms.len(), 1);
    assert!(!view3.is_log_index_built());
}

/// Scenario: Evaluate resource-attribute and scope-attribute conjunctions (positive+positive, positive+negated, all-negated De Morgan, cross-domain with row fields, empty ID-space short-circuit, and missing attribute table) against a 6-row batch including null resource structs and null resource IDs.
/// Guarantees: Resource and scope predicates evaluate and combine in compact ID space with exact row-level semantics, including preserving null-struct/null-ID rows under negated matchers and rejecting them under positive matchers.
#[test]
fn test_resource_and_scope_id_space_conjunctions_and_negations() {
    // 6 log rows:
    //   row 0: res=0, scope=10, sev="INFO"
    //   row 1: res=1, scope=10, sev="ERROR"
    //   row 2: res=1, scope=20, sev="INFO"
    //   row 3: res=2, scope=20, sev="ERROR"
    //   row 4: res=NULL struct, scope=10, sev="INFO"
    //   row 5: res.id=NULL, scope=NULL struct, sev="INFO"
    let res_struct = StructArray::new(
        vec![Arc::new(
            Field::new(consts::ID, DataType::UInt16, true).with_plain_encoding(),
        )]
        .into(),
        vec![Arc::new(UInt16Array::from(vec![
            Some(0),
            Some(1),
            Some(1),
            Some(2),
            Some(0), // masked by outer struct null
            None,    // null resource.id
        ]))],
        Some(NullBuffer::from(vec![true, true, true, true, false, true])),
    );
    let scope_struct = StructArray::new(
        vec![Arc::new(
            Field::new(consts::ID, DataType::UInt16, true).with_plain_encoding(),
        )]
        .into(),
        vec![Arc::new(UInt16Array::from(vec![
            Some(10),
            Some(10),
            Some(20),
            Some(20),
            Some(10),
            Some(10), // masked by outer struct null
        ]))],
        Some(NullBuffer::from(vec![true, true, true, true, true, false])),
    );
    let root_schema = Arc::new(Schema::new(vec![
        Field::new(consts::ID, DataType::UInt16, false).with_plain_encoding(),
        Field::new(consts::SEVERITY_TEXT, DataType::Utf8, true),
        Field::new(consts::RESOURCE, res_struct.data_type().clone(), true),
        Field::new(consts::SCOPE, scope_struct.data_type().clone(), true),
    ]));
    let root_rb = RecordBatch::try_new(
        root_schema,
        vec![
            Arc::new(UInt16Array::from(vec![0u16, 1, 2, 3, 4, 5])),
            Arc::new(StringArray::from(vec![
                "INFO", "ERROR", "INFO", "ERROR", "INFO", "INFO",
            ])),
            Arc::new(res_struct),
            Arc::new(scope_struct),
        ],
    )
    .unwrap();

    // ResourceAttrs:
    //   res 0: service.name="api", env="prod"
    //   res 1: service.name="api", env="staging"
    //   res 2: service.name="worker", env="prod"
    let attr_schema = Arc::new(Schema::new(vec![
        Field::new(consts::PARENT_ID, DataType::UInt16, false).with_plain_encoding(),
        Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
        Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
        Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
    ]));
    let res_attrs = RecordBatch::try_new(
        attr_schema.clone(),
        vec![
            Arc::new(UInt16Array::from(vec![0u16, 0, 1, 1, 2, 2])),
            Arc::new(StringArray::from(vec![
                "service.name",
                "env",
                "service.name",
                "env",
                "service.name",
                "env",
            ])),
            Arc::new(UInt8Array::from(vec![AttributeValueType::Str as u8; 6])),
            Arc::new(StringArray::from(vec![
                "api", "prod", "api", "staging", "worker", "prod",
            ])),
        ],
    )
    .unwrap();
    // ScopeAttrs:
    //   scope 10: team="core"
    //   scope 20: team="edge"
    let scope_attrs = RecordBatch::try_new(
        attr_schema,
        vec![
            Arc::new(UInt16Array::from(vec![10u16, 20])),
            Arc::new(StringArray::from(vec!["team", "team"])),
            Arc::new(UInt8Array::from(vec![AttributeValueType::Str as u8; 2])),
            Arc::new(StringArray::from(vec!["core", "edge"])),
        ],
    )
    .unwrap();

    let mut records = OtapArrowRecords::Logs(Logs::default());
    records
        .set(ArrowPayloadType::Logs, root_rb.clone())
        .unwrap();
    records
        .set(ArrowPayloadType::ResourceAttrs, res_attrs)
        .unwrap();
    records
        .set(ArrowPayloadType::ScopeAttrs, scope_attrs)
        .unwrap();

    let mut scratch = EvaluationScratch::new();

    // 1. Positive + negated resource conjunction: service.name == "api" AND NOT env == "prod"
    //    Matches only resource 1 -> rows 1 and 2 dropped; rows 0, 3, 4, 5 kept.
    let eng_pos_neg = compile_engine(&[Policy::log_filter(
        "drop_api_nonprod",
        vec![
            LogMatcher {
                field: LogFieldSelector::ResourceAttribute(AttributePath::single("service.name")),
                predicate: Predicate::Equals(ScalarValue::String("api".into())),
                negate: false,
            },
            LogMatcher {
                field: LogFieldSelector::ResourceAttribute(AttributePath::single("env")),
                predicate: Predicate::Equals(ScalarValue::String("prod".into())),
                negate: true,
            },
        ],
        Action::Drop,
    )]);
    let kept = eng_pos_neg
        .evaluate_filter(&records, &mut scratch, None)
        .unwrap();
    assert_eq!(kept.iter().collect::<Vec<_>>(), vec![0, 3, 4, 5]);
    // Verify raw resource-ID bitmap stores resource IDs (not log rows)
    let view = LogsBatchView::try_new(&records).unwrap().unwrap();
    let raw_bms = eng_pos_neg.evaluate_filter_predicates_for_test(&view, &mut scratch);
    assert_eq!(raw_bms[0].iter().collect::<Vec<_>>(), vec![0, 1]);
    assert_eq!(raw_bms[1].iter().collect::<Vec<_>>(), vec![0, 2]);

    // 2. All-negated resource matchers (De Morgan): NOT service.name == "worker" AND NOT env == "staging"
    //    Excludes res 2 and res 1; matches res 0 (row 0) AND null-struct (row 4) AND null-id (row 5)!
    //    Drops rows 0, 4, 5 -> keeps rows 1, 2, 3.
    let eng_all_neg = compile_engine(&[Policy::log_filter(
        "drop_all_neg",
        vec![
            LogMatcher {
                field: LogFieldSelector::ResourceAttribute(AttributePath::single("service.name")),
                predicate: Predicate::Equals(ScalarValue::String("worker".into())),
                negate: true,
            },
            LogMatcher {
                field: LogFieldSelector::ResourceAttribute(AttributePath::single("env")),
                predicate: Predicate::Equals(ScalarValue::String("staging".into())),
                negate: true,
            },
        ],
        Action::Drop,
    )]);
    let kept = eng_all_neg
        .evaluate_filter(&records, &mut scratch, None)
        .unwrap();
    assert_eq!(kept.iter().collect::<Vec<_>>(), vec![1, 2, 3]);

    // 3. Cross-domain conjunction: res.service.name == "api" AND scope.team == "edge" AND sev == "INFO"
    //    Only row 2 (res=1, scope=20, INFO) matches. Also verify transform stage with cross-domain matchers.
    let eng_cross = compile_engine(&[
        Policy::log_filter(
            "drop_cross",
            vec![
                LogMatcher {
                    field: LogFieldSelector::ResourceAttribute(AttributePath::single(
                        "service.name",
                    )),
                    predicate: Predicate::Equals(ScalarValue::String("api".into())),
                    negate: false,
                },
                LogMatcher {
                    field: LogFieldSelector::ScopeAttribute(AttributePath::single("team")),
                    predicate: Predicate::Equals(ScalarValue::String("edge".into())),
                    negate: false,
                },
                LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
                    predicate: Predicate::Equals(ScalarValue::String("INFO".into())),
                    negate: false,
                },
            ],
            Action::Drop,
        ),
        Policy::log_transform(
            "tag_worker_edge",
            LogTransformPolicy {
                matches: vec![
                    LogMatcher {
                        field: LogFieldSelector::ResourceAttribute(AttributePath::single(
                            "service.name",
                        )),
                        predicate: Predicate::Equals(ScalarValue::String("worker".into())),
                        negate: false,
                    },
                    LogMatcher {
                        field: LogFieldSelector::ScopeAttribute(AttributePath::single("team")),
                        predicate: Predicate::Equals(ScalarValue::String("edge".into())),
                        negate: false,
                    },
                ],
                rename: vec![],
                add: vec![LogAdd {
                    target: LogTargetSelector::LogAttribute(AttributePath::single("matched_cross")),
                    value: ScalarValue::Bool(true),
                    upsert: true,
                }],
                remove: vec![],
                redact: vec![],
            },
        ),
    ]);
    let mut cross_stats = eng_cross.new_match_stats();
    let out = eng_cross
        .evaluate(&records, &mut scratch, Some(&mut cross_stats))
        .unwrap();
    assert_eq!(out.dropped_rows, 1);
    assert_eq!(out.kept_rows, 5);
    assert_eq!(
        cross_stats.iter(&eng_cross).unwrap().collect::<Vec<_>>(),
        vec![
            ("drop_cross", PolicyMatchCount { hits: 1, misses: 0 }),
            ("tag_worker_edge", PolicyMatchCount { hits: 1, misses: 0 }),
        ]
    );

    // 4. Mutually exclusive positive resource matchers -> empty ID-space short-circuit drops 0 rows.
    let eng_empty = compile_engine(&[Policy::log_filter(
        "drop_impossible",
        vec![
            LogMatcher {
                field: LogFieldSelector::ResourceAttribute(AttributePath::single("service.name")),
                predicate: Predicate::Equals(ScalarValue::String("worker".into())),
                negate: false,
            },
            LogMatcher {
                field: LogFieldSelector::ResourceAttribute(AttributePath::single("env")),
                predicate: Predicate::Equals(ScalarValue::String("staging".into())),
                negate: false,
            },
        ],
        Action::Drop,
    )]);
    let kept = eng_empty
        .evaluate_filter(&records, &mut scratch, None)
        .unwrap();
    assert_eq!(kept.iter().collect::<Vec<_>>(), vec![0, 1, 2, 3, 4, 5]);

    // 5. Missing ResourceAttrs table: positive drops 0 rows, negated drops all 6 rows.
    let mut no_res_attrs = OtapArrowRecords::Logs(Logs::default());
    no_res_attrs.set(ArrowPayloadType::Logs, root_rb).unwrap();
    let kept_pos = eng_pos_neg
        .evaluate_filter(&no_res_attrs, &mut scratch, None)
        .unwrap();
    assert_eq!(kept_pos.iter().collect::<Vec<_>>(), vec![0, 1, 2, 3, 4, 5]);
    let kept_neg = eng_all_neg
        .evaluate_filter(&no_res_attrs, &mut scratch, None)
        .unwrap();
    assert!(kept_neg.is_empty());
}

/// Scenario: Evaluate mixed keep/drop filters, overlapping drop policies, keep-only filters, multi-op and no-op transforms, match-all transforms, zero-match policies, disabled/invalid policies, `evaluate_filter`, multi-batch accumulation, `reset()`, reload mismatches, and non-Logs errors with `Option<&mut PolicyMatchStats>`.
/// Guarantees: Per-policy `hit`/`miss` outcomes and `evaluated_rows` match `policy.md` ("Match Outcome Semantics") exactly, transform hits count matching surviving records (never dropped records or per-op mutations), disabled and compile-error policies are excluded, zero-match valid policies remain present, and mismatched or error calls fail fast without corrupting stats.
#[test]
fn test_per_policy_match_stats_semantics_and_lifecycle() {
    use otel_arrow_dfe_pdata::otap::Metrics;

    // 5 log rows:
    //   row 0: sev=DEBUG, env=prod, keep_flag=true   -> matches drop_debug, drop_env_prod, keep_flagged (kept -> keep hit, both drops miss; tx_multi hit, tx_noop hit, tx_all hit)
    //   row 1: sev=DEBUG, env=prod                   -> matches drop_debug, drop_env_prod (dropped -> both drops hit; transforms do NOT run)
    //   row 2: sev=INFO,  env=prod                   -> matches drop_env_prod only (dropped -> drop_env_prod hit; transforms do NOT run)
    //   row 3: sev=INFO,  keep_flag=true             -> matches keep_flagged only (kept -> keep_flagged hit; tx_all hit)
    //   row 4: sev=WARN                              -> matches NO filter policy (kept; tx_all hit)
    let batch = build_custom_test_batch(
        vec![Some("r0"), Some("r1"), Some("r2"), Some("r3"), Some("r4")],
        vec![5, 5, 9, 9, 13],
        vec!["DEBUG", "DEBUG", "INFO", "INFO", "WARN"],
        vec![[1u8; 16]; 5],
        vec![[1u8; 8]; 5],
        vec![
            (0, "env", ScalarValue::String("prod".into())),
            (0, "keep_flag", ScalarValue::Bool(true)),
            (0, "existing", ScalarValue::String("orig".into())),
            (1, "env", ScalarValue::String("prod".into())),
            (2, "env", ScalarValue::String("prod".into())),
            (3, "keep_flag", ScalarValue::Bool(true)),
        ],
        vec![],
    );

    let mut disabled_policy = Policy::log_filter(
        "disabled_drop",
        vec![LogMatcher {
            field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
            predicate: Predicate::Equals(ScalarValue::String("WARN".into())),
            negate: false,
        }],
        Action::Drop,
    );
    disabled_policy.enabled = false;

    let invalid_policy = Policy::log_filter("invalid_empty_matches", vec![], Action::Drop);

    let compile_out = PolicyEngine::compile(&[
        disabled_policy,
        invalid_policy,
        Policy::log_filter(
            "drop_debug",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
                predicate: Predicate::Equals(ScalarValue::String("DEBUG".into())),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "drop_env_prod",
            vec![LogMatcher {
                field: LogFieldSelector::LogAttribute(AttributePath::single("env")),
                predicate: Predicate::Equals(ScalarValue::String("prod".into())),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "keep_flagged",
            vec![LogMatcher {
                field: LogFieldSelector::LogAttribute(AttributePath::single("keep_flag")),
                predicate: Predicate::Equals(ScalarValue::Bool(true)),
                negate: false,
            }],
            Action::Keep,
        ),
        Policy::log_filter(
            "never_matches",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
                predicate: Predicate::Equals(ScalarValue::String("FATAL".into())),
                negate: false,
            }],
            Action::Drop,
        ),
        // 3-op transform matching DEBUG: row 0 survives (1 hit, NOT 3!), row 1 dropped (0 hit)
        Policy::log_transform(
            "tx_multi",
            LogTransformPolicy {
                matches: vec![LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
                    predicate: Predicate::Equals(ScalarValue::String("DEBUG".into())),
                    negate: false,
                }],
                rename: vec![],
                add: vec![
                    LogAdd {
                        target: LogTargetSelector::LogAttribute(AttributePath::single("a")),
                        value: ScalarValue::Int(1),
                        upsert: true,
                    },
                    LogAdd {
                        target: LogTargetSelector::LogAttribute(AttributePath::single("b")),
                        value: ScalarValue::Int(2),
                        upsert: true,
                    },
                    LogAdd {
                        target: LogTargetSelector::LogAttribute(AttributePath::single("c")),
                        value: ScalarValue::Int(3),
                        upsert: true,
                    },
                ],
                remove: vec![],
                redact: vec![],
            },
        ),
        // No-op transform (upsert: false on existing key): still counts 1 policy hit on row 0!
        Policy::log_transform(
            "tx_noop",
            LogTransformPolicy {
                matches: vec![LogMatcher {
                    field: LogFieldSelector::LogAttribute(AttributePath::single("existing")),
                    predicate: Predicate::Exists,
                    negate: false,
                }],
                rename: vec![],
                add: vec![LogAdd {
                    target: LogTargetSelector::LogAttribute(AttributePath::single("existing")),
                    value: ScalarValue::String("overwritten".into()),
                    upsert: false,
                }],
                remove: vec![],
                redact: vec![],
            },
        ),
        // Match-all transform (empty matches): counts all 3 surviving rows
        Policy::log_transform(
            "tx_all",
            LogTransformPolicy {
                matches: vec![],
                rename: vec![],
                add: vec![LogAdd {
                    target: LogTargetSelector::LogAttribute(AttributePath::single("seen")),
                    value: ScalarValue::Bool(true),
                    upsert: true,
                }],
                remove: vec![],
                redact: vec![],
            },
        ),
    ])
    .unwrap();
    assert_eq!(compile_out.errors.len(), 1);
    let eng = compile_out.engine;

    // Disabled and invalid policies excluded; zero-match valid policy included.
    assert_eq!(
        eng.policy_ids(),
        &[
            "drop_debug",
            "drop_env_prod",
            "keep_flagged",
            "never_matches",
            "tx_all",
            "tx_multi",
            "tx_noop",
        ]
    );

    let mut scratch = EvaluationScratch::new();
    let mut stats = eng.new_match_stats();

    let out = eng
        .evaluate(&batch, &mut scratch, Some(&mut stats))
        .unwrap();
    assert_eq!(out.kept_rows, 3);
    assert_eq!(out.dropped_rows, 2);
    assert_eq!(stats.evaluated_rows(), 5);
    assert_eq!(
        stats.iter(&eng).unwrap().collect::<Vec<_>>(),
        vec![
            ("drop_debug", PolicyMatchCount { hits: 1, misses: 1 }),
            ("drop_env_prod", PolicyMatchCount { hits: 2, misses: 1 }),
            ("keep_flagged", PolicyMatchCount { hits: 2, misses: 0 }),
            ("never_matches", PolicyMatchCount { hits: 0, misses: 0 }),
            ("tx_all", PolicyMatchCount { hits: 3, misses: 0 }),
            ("tx_multi", PolicyMatchCount { hits: 1, misses: 0 }),
            ("tx_noop", PolicyMatchCount { hits: 1, misses: 0 }),
        ]
    );

    // Accumulate a second batch on the same stats instance -> counts double.
    let _ = eng
        .evaluate(&batch, &mut scratch, Some(&mut stats))
        .unwrap();
    assert_eq!(stats.evaluated_rows(), 10);
    assert_eq!(stats.counts()[0], PolicyMatchCount { hits: 2, misses: 2 });
    assert_eq!(stats.counts()[4], PolicyMatchCount { hits: 6, misses: 0 });

    // Non-Logs error leaves stats untouched.
    let metrics_batch = OtapArrowRecords::Metrics(Metrics::default());
    assert!(
        eng.evaluate(&metrics_batch, &mut scratch, Some(&mut stats))
            .is_err()
    );
    assert_eq!(stats.evaluated_rows(), 10);

    // reset() clears all counts cleanly without reallocating.
    stats.reset();
    assert_eq!(stats.evaluated_rows(), 0);
    assert!(
        stats
            .counts()
            .iter()
            .all(|c| *c == PolicyMatchCount { hits: 0, misses: 0 })
    );

    // evaluate_filter records denominator + filter outcomes (transform counts stay 0).
    let sel = eng
        .evaluate_filter(&batch, &mut scratch, Some(&mut stats))
        .unwrap();
    assert_eq!(sel.len(), 3);
    assert_eq!(stats.evaluated_rows(), 5);
    assert_eq!(
        stats.iter(&eng).unwrap().collect::<Vec<_>>(),
        vec![
            ("drop_debug", PolicyMatchCount { hits: 1, misses: 1 }),
            ("drop_env_prod", PolicyMatchCount { hits: 2, misses: 1 }),
            ("keep_flagged", PolicyMatchCount { hits: 2, misses: 0 }),
            ("never_matches", PolicyMatchCount { hits: 0, misses: 0 }),
            ("tx_all", PolicyMatchCount { hits: 0, misses: 0 }),
            ("tx_multi", PolicyMatchCount { hits: 0, misses: 0 }),
            ("tx_noop", PolicyMatchCount { hits: 0, misses: 0 }),
        ]
    );

    // Reloaded engine (even with identical policy list) rejects stale stats from prior engine.
    let reloaded = compile_engine(&[Policy::log_filter(
        "drop_debug",
        vec![LogMatcher {
            field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
            predicate: Predicate::Equals(ScalarValue::String("DEBUG".into())),
            negate: false,
        }],
        Action::Drop,
    )]);
    assert!(
        reloaded
            .evaluate(&batch, &mut scratch, Some(&mut stats))
            .is_err()
    );
    assert!(
        reloaded
            .evaluate_filter(&batch, &mut scratch, Some(&mut stats))
            .is_err()
    );
    assert!(stats.iter(&reloaded).is_err());
    assert_eq!(stats.evaluated_rows(), 5);

    // Empty-filter engine still counts evaluated_rows in both evaluate_filter and evaluate,
    // and keep-only engine records keep hits with 0 drops.
    let empty_eng = compile_engine(&[]);
    let mut empty_stats = empty_eng.new_match_stats();
    let _ = empty_eng
        .evaluate_filter(&batch, &mut scratch, Some(&mut empty_stats))
        .unwrap();
    assert_eq!(empty_stats.evaluated_rows(), 5);
    let _ = empty_eng
        .evaluate(&batch, &mut scratch, Some(&mut empty_stats))
        .unwrap();
    assert_eq!(empty_stats.evaluated_rows(), 10);

    let keep_only_eng = compile_engine(&[Policy::log_filter(
        "keep_info",
        vec![LogMatcher {
            field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
            predicate: Predicate::Equals(ScalarValue::String("INFO".into())),
            negate: false,
        }],
        Action::Keep,
    )]);
    let mut keep_stats = keep_only_eng.new_match_stats();
    let keep_out = keep_only_eng
        .evaluate(&batch, &mut scratch, Some(&mut keep_stats))
        .unwrap();
    assert_eq!(keep_out.kept_rows, 5);
    assert_eq!(keep_out.dropped_rows, 0);
    assert_eq!(keep_stats.evaluated_rows(), 5);
    assert_eq!(
        keep_stats.iter(&keep_only_eng).unwrap().collect::<Vec<_>>(),
        vec![("keep_info", PolicyMatchCount { hits: 2, misses: 0 })]
    );

    // All-dropped batch: transform with empty matches records 0 hits.
    let drop_all_with_tx = compile_engine(&[
        Policy::log_filter(
            "drop_all",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Exists,
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_transform(
            "tx_all",
            LogTransformPolicy {
                matches: vec![],
                rename: vec![],
                add: vec![LogAdd {
                    target: LogTargetSelector::LogAttribute(AttributePath::single("x")),
                    value: ScalarValue::Bool(true),
                    upsert: true,
                }],
                remove: vec![],
                redact: vec![],
            },
        ),
    ]);
    let mut all_drop_stats = drop_all_with_tx.new_match_stats();
    let drop_all_out = drop_all_with_tx
        .evaluate(&batch, &mut scratch, Some(&mut all_drop_stats))
        .unwrap();
    assert!(drop_all_out.records.is_none());
    assert_eq!(all_drop_stats.evaluated_rows(), 5);
    assert_eq!(
        all_drop_stats
            .iter(&drop_all_with_tx)
            .unwrap()
            .collect::<Vec<_>>(),
        vec![
            ("drop_all", PolicyMatchCount { hits: 5, misses: 0 }),
            ("tx_all", PolicyMatchCount { hits: 0, misses: 0 }),
        ]
    );
}

/// Scenario: Evaluate keep-all, keep-only, drop-all, and partial keep+drop (including negated matchers, LogAttrs, and ResourceAttrs) policy sets with `stats = None` (short-circuit enabled) vs. `stats = Some(&mut stats)` (exact stats mode).
/// Guarantees: Short-circuiting skips the entire `Drop` plan when `Keep` covers all rows (proven by `LogsBatchView::is_log_index_built() == false`) and skips `Keep` plans when no `Drop` rules exist, while producing identical row selections and exact per-policy `hit`/`miss` stats when `stats` is `Some`.
#[test]
fn test_opt_in_filter_short_circuiting_and_stats_parity() {
    let mut scratch = EvaluationScratch::new();

    // 1. Keep-all short-circuit: Keep inspects root SeverityText ("INFO" on all rows),
    //    Drop inspects LogAttribute("noisy").
    let all_info = build_custom_test_batch(
        vec![Some("a"), Some("b"), Some("c")],
        vec![9, 9, 9],
        vec!["INFO", "INFO", "INFO"],
        vec![[1u8; 16]; 3],
        vec![[1u8; 8]; 3],
        vec![
            (0, "noisy", ScalarValue::Bool(true)),
            (2, "noisy", ScalarValue::Bool(true)),
        ],
        vec![],
    );
    let keep_all_eng = compile_engine(&[
        Policy::log_filter(
            "keep_info",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
                predicate: Predicate::Equals(ScalarValue::String("INFO".into())),
                negate: false,
            }],
            Action::Keep,
        ),
        Policy::log_filter(
            "drop_noisy",
            vec![LogMatcher {
                field: LogFieldSelector::LogAttribute(AttributePath::single("noisy")),
                predicate: Predicate::Equals(ScalarValue::Bool(true)),
                negate: false,
            }],
            Action::Drop,
        ),
    ]);

    // stats = None -> keep_exempt covers all 3 rows -> drop_plan skipped -> log_id_to_row never built.
    let view_sc = LogsBatchView::try_new(&all_info).unwrap().unwrap();
    let kept_sc = keep_all_eng.evaluate_filter_view_for_test(&view_sc, &mut scratch, None);
    assert_eq!(kept_sc.iter().collect::<Vec<_>>(), vec![0, 1, 2]);
    assert!(
        !view_sc.is_log_index_built(),
        "drop_plan must be skipped when Keep already covers all rows and stats is None"
    );

    // stats = Some -> drop_plan runs so drop_noisy records exact misses.
    let view_stats = LogsBatchView::try_new(&all_info).unwrap().unwrap();
    let mut stats = keep_all_eng.new_match_stats();
    let kept_stats =
        keep_all_eng.evaluate_filter_view_for_test(&view_stats, &mut scratch, Some(&mut stats));
    assert_eq!(kept_stats, kept_sc);
    assert!(view_stats.is_log_index_built());
    assert_eq!(
        stats.iter(&keep_all_eng).unwrap().collect::<Vec<_>>(),
        vec![
            ("keep_info", PolicyMatchCount { hits: 3, misses: 0 }),
            ("drop_noisy", PolicyMatchCount { hits: 0, misses: 2 }),
        ]
    );

    // 2. Keep-only engine (no Drop rules): stats = None skips keep_plan entirely.
    let keep_only_eng = compile_engine(&[Policy::log_filter(
        "keep_noisy",
        vec![LogMatcher {
            field: LogFieldSelector::LogAttribute(AttributePath::single("noisy")),
            predicate: Predicate::Equals(ScalarValue::Bool(true)),
            negate: false,
        }],
        Action::Keep,
    )]);
    let view_ko_sc = LogsBatchView::try_new(&all_info).unwrap().unwrap();
    let kept_ko_sc = keep_only_eng.evaluate_filter_view_for_test(&view_ko_sc, &mut scratch, None);
    assert_eq!(kept_ko_sc.iter().collect::<Vec<_>>(), vec![0, 1, 2]);
    assert!(!view_ko_sc.is_log_index_built());

    let view_ko_stats = LogsBatchView::try_new(&all_info).unwrap().unwrap();
    let mut ko_stats = keep_only_eng.new_match_stats();
    let kept_ko_stats = keep_only_eng.evaluate_filter_view_for_test(
        &view_ko_stats,
        &mut scratch,
        Some(&mut ko_stats),
    );
    assert_eq!(kept_ko_stats, kept_ko_sc);
    assert!(view_ko_stats.is_log_index_built());
    assert_eq!(
        ko_stats.iter(&keep_only_eng).unwrap().collect::<Vec<_>>(),
        vec![("keep_noisy", PolicyMatchCount { hits: 2, misses: 0 })]
    );
    // Also exercise public evaluate_filter and evaluate fast path for keep-only + stats=None.
    let sel_ko = keep_only_eng
        .evaluate_filter(&all_info, &mut scratch, None)
        .unwrap();
    assert_eq!(sel_ko.len(), 3);
    let out_ko = keep_only_eng
        .evaluate(&all_info, &mut scratch, None)
        .unwrap();
    assert_eq!(out_ko.kept_rows, 3);
    assert_eq!(out_ko.dropped_rows, 0);

    // 3. Partial Keep + multiple Drop rules (positive, negated, LogAttrs, and drop-all break):
    //    row 0: ERROR, tag="a" (kept by keep_err)
    //    row 1: WARN,  tag="a" (undecided -> dropped by drop_neg_ok: NOT body == "ok")
    //    row 2: INFO,  body="ok", tag="drop_me" (undecided -> dropped by drop_tag)
    //    row 3: INFO,  body="ok", tag="keep_implicit" (undecided -> survives)
    //    row 4: DEBUG, body="x" (undecided -> dropped by drop_neg_ok)
    let mixed = build_custom_test_batch(
        vec![Some("x"), Some("x"), Some("ok"), Some("ok"), Some("x")],
        vec![17, 13, 9, 9, 5],
        vec!["ERROR", "WARN", "INFO", "INFO", "DEBUG"],
        vec![[1u8; 16]; 5],
        vec![[1u8; 8]; 5],
        vec![
            (0, "tag", ScalarValue::String("a".into())),
            (1, "tag", ScalarValue::String("a".into())),
            (2, "tag", ScalarValue::String("drop_me".into())),
            (3, "tag", ScalarValue::String("keep_implicit".into())),
        ],
        vec![],
    );
    let partial_eng = compile_engine(&[
        Policy::log_filter(
            "keep_err",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
                predicate: Predicate::Equals(ScalarValue::String("ERROR".into())),
                negate: false,
            }],
            Action::Keep,
        ),
        Policy::log_filter(
            "drop_neg_ok",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Equals(ScalarValue::String("ok".into())),
                negate: true,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "drop_tag",
            vec![LogMatcher {
                field: LogFieldSelector::LogAttribute(AttributePath::single("tag")),
                predicate: Predicate::Equals(ScalarValue::String("drop_me".into())),
                negate: false,
            }],
            Action::Drop,
        ),
    ]);
    let sel_sc = partial_eng
        .evaluate_filter(&mixed, &mut scratch, None)
        .unwrap();
    let mut partial_stats = partial_eng.new_match_stats();
    let sel_st = partial_eng
        .evaluate_filter(&mixed, &mut scratch, Some(&mut partial_stats))
        .unwrap();
    assert_eq!(sel_sc, sel_st);
    assert_eq!(sel_sc.iter().collect::<Vec<_>>(), vec![0, 3]);
    assert_eq!(
        partial_stats
            .iter(&partial_eng)
            .unwrap()
            .collect::<Vec<_>>(),
        vec![
            ("keep_err", PolicyMatchCount { hits: 1, misses: 0 }),
            ("drop_neg_ok", PolicyMatchCount { hits: 2, misses: 1 }),
            ("drop_tag", PolicyMatchCount { hits: 1, misses: 0 }),
        ]
    );

    // 4. Drop-all break across multiple Drop rules: Drop #1 drops all undecided rows -> Drop #2 skipped.
    let drop_all_multi = compile_engine(&[
        Policy::log_filter(
            "keep_err",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
                predicate: Predicate::Equals(ScalarValue::String("ERROR".into())),
                negate: false,
            }],
            Action::Keep,
        ),
        Policy::log_filter(
            "drop_first_all",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Exists,
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "drop_second_redundant",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
                predicate: Predicate::Equals(ScalarValue::String("DEBUG".into())),
                negate: false,
            }],
            Action::Drop,
        ),
    ]);
    let sel_da_sc = drop_all_multi
        .evaluate_filter(&mixed, &mut scratch, None)
        .unwrap();
    let mut da_stats = drop_all_multi.new_match_stats();
    let sel_da_st = drop_all_multi
        .evaluate_filter(&mixed, &mut scratch, Some(&mut da_stats))
        .unwrap();
    assert_eq!(sel_da_sc, sel_da_st);
    assert_eq!(sel_da_sc.iter().collect::<Vec<_>>(), vec![0]);
    assert_eq!(
        da_stats.iter(&drop_all_multi).unwrap().collect::<Vec<_>>(),
        vec![
            ("keep_err", PolicyMatchCount { hits: 1, misses: 0 }),
            ("drop_first_all", PolicyMatchCount { hits: 4, misses: 1 }),
            (
                "drop_second_redundant",
                PolicyMatchCount { hits: 1, misses: 0 }
            ),
        ]
    );
}

/// Scenario: Reuse one `EvaluationScratch` across stages and reloaded engines where only one of `(num_ngrams, num_patterns)` changes (e.g. `(2, 1)` -> `(2, 2)` and `(1, 1)` -> `(2, 1)`), and also across a `u32::MAX` generation counter wraparound.
/// Guarantees: `StringScanScratch` never collides with stale generation stamps and never produces false negatives across stage transitions, policy reloads, or `u32` wraparound.
#[test]
fn test_string_scan_scratch_generation_monotonicity_across_stages_reloads_and_wraparound() {
    let batch = build_custom_test_batch(
        vec![Some("alpha beta"), Some("alpha only"), Some("beta only")],
        vec![9; 3],
        vec!["INFO"; 3],
        vec![[1u8; 16]; 3],
        vec![[1u8; 8]; 3],
        vec![],
        vec![],
    );

    // Keep stage has 2 n-grams ("alpha", "beta") and 1 pattern (`alpha.*beta`).
    // Drop stage has the SAME 2 n-grams ("alpha", "beta") and 2 patterns (`contains("alpha")`, `contains("beta")`).
    // Previously `StringScanScratch::prepare` reset `cur_gen = 0` when `pat_seen_gen.len()` changed
    // from 1 to 2 while leaving `ngram_seen_gen` stamped with 1..=3, silently skipping matches!
    let eng = compile_engine(&[
        Policy::log_filter(
            "keep_both",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Regex("alpha.*beta".into()),
                negate: false,
            }],
            Action::Keep,
        ),
        Policy::log_filter(
            "drop_alpha",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Contains(ScalarValue::String("alpha".into())),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "drop_beta",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Contains(ScalarValue::String("beta".into())),
                negate: false,
            }],
            Action::Drop,
        ),
    ]);

    let mut scratch = EvaluationScratch::new();
    let kept = eng
        .evaluate_filter(&batch, &mut scratch, None)
        .unwrap()
        .into_bitmap();
    // Row 0 kept by keep_both; rows 1 and 2 MUST be dropped by drop_alpha and drop_beta!
    assert_eq!(kept, DenseBitset::from_indices([0]));

    // Reload with different num_ngrams (1) and same num_patterns (2): must still match accurately.
    let reloaded = compile_engine(&[
        Policy::log_filter(
            "drop_alpha_re1",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Regex("alpha.*only".into()),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "drop_alpha_re2",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Regex("^alpha".into()),
                negate: false,
            }],
            Action::Drop,
        ),
    ]);
    let kept_reloaded = reloaded
        .evaluate_filter(&batch, &mut scratch, None)
        .unwrap()
        .into_bitmap();
    assert_eq!(kept_reloaded, DenseBitset::from_indices([2]));

    // Force u32::MAX wraparound on scratch[0] and verify next evaluation still matches all rows cleanly.
    scratch.set_string_scratch_gen_for_test(0, u32::MAX);
    let kept_wrap = eng
        .evaluate_filter(&batch, &mut scratch, None)
        .unwrap()
        .into_bitmap();
    assert_eq!(kept_wrap, DenseBitset::from_indices([0]));
}

/// Scenario: Filter logs using `Predicate::Glob` covering exact-match fast paths, n-gram-indexed wildcards, fallback single-char/wildcard-only globs, `TraceId`/`SpanId` globs, Unicode `?`, newlines, and non-string type mismatches.
/// Guarantees: `extract_glob_ngrams` extracts mandatory literal runs and `PolicyEngine` evaluates all `Predicate::Glob` forms identically to the `policy.md` specification.
#[test]
fn test_predicate_glob_filtering_index_and_trace_span_ids() {
    assert_eq!(
        extract_glob_ngrams("http.*.v?.*status"),
        vec![b"http.".to_vec(), b".v".to_vec(), b"status".to_vec()]
    );
    assert!(extract_glob_ngrams("*?a?b*").is_empty());

    let trace0 = [0xde, 0xad, 0xbe, 0xef, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
    let batch = build_custom_test_batch(
        vec![
            Some("http.request.ok"),   // 0: matches indexed glob "http.*.ok"
            Some("http.request.fail"), // 1: misses "http.*.ok"
            Some("a\u{03b1}c"),        // 2: matches fallback glob "a?c"
            Some("a.b+c"),             // 3: matches exact wildcard-free glob "a.b+c"
            Some("line1\nline2"),      // 4: matches "line1*line2" across newline
            Some("HTTP.request.ok"),   // 5: case-sensitive miss for "http.*.ok"
        ],
        vec![9; 6],
        vec!["INFO"; 6],
        vec![
            trace0, [0u8; 16], [0u8; 16], [0u8; 16], [0u8; 16], [0u8; 16],
        ],
        vec![[0u8; 8]; 6],
        vec![(1, "num", ScalarValue::Int(42))],
        vec![],
    );

    let eng = compile_engine(&[
        Policy::log_filter(
            "drop_indexed_glob",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Glob("http.*.ok".into()),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "drop_fallback_glob",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Glob("a?c".into()),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "drop_exact_glob",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Glob("a.b+c".into()),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "drop_newline_glob",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Glob("line1*line2".into()),
                negate: false,
            }],
            Action::Drop,
        ),
        // Non-string attribute must NOT match Glob("*")
        Policy::log_filter(
            "drop_non_str_glob",
            vec![LogMatcher {
                field: LogFieldSelector::LogAttribute(AttributePath::single("num")),
                predicate: Predicate::Glob("*".into()),
                negate: false,
            }],
            Action::Drop,
        ),
        // Keep row 0 via TraceId glob
        Policy::log_filter(
            "keep_trace_glob",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::TraceId),
                predicate: Predicate::Glob("deadbeef*01".into()),
                negate: false,
            }],
            Action::Keep,
        ),
    ]);
    let kept = eng
        .evaluate_filter(&batch, &mut EvaluationScratch::new(), None)
        .unwrap()
        .into_bitmap();
    // Row 0 kept by TraceId glob; rows 1 and 5 unmatched by drop globs; rows 2, 3, 4 dropped.
    assert_eq!(kept, DenseBitset::from_indices([0, 1, 5]));
}

/// Scenario: Evaluate `Exists`, `BodyField` write initialization, `LogAdd(upsert=false)`, and `LogRedact` against log records whose `LogRecord.body` is empty string `""` (both `StructArray` and plain `StringArray` encodings).
/// Guarantees: Empty-string `Body == ""` is treated as absent for `Exists`, `BodyField` map initialization, `LogAdd(upsert=false)`, and `LogRedact` no-op per `policy.md`.
#[test]
fn test_empty_string_body_absent_semantics_for_exists_and_transforms() {
    // 1. StructArray Body with Str(""), Str("real"), and null
    let struct_batch = build_custom_test_batch(
        vec![Some(""), Some("real"), None],
        vec![9; 3],
        vec!["INFO"; 3],
        vec![[1u8; 16]; 3],
        vec![[1u8; 8]; 3],
        vec![],
        vec![],
    );
    // 2. Dictionary-encoded StructArray Body (`Dictionary<UInt16, Utf8>`) with "", "real", and null
    let dict_body_str = DictionaryArray::<UInt16Type>::try_new(
        UInt16Array::from(vec![Some(0u16), Some(1), None]),
        Arc::new(StringArray::from(vec!["", "real"])),
    )
    .unwrap();
    let dict_body_struct = StructArray::new(
        vec![
            Arc::new(Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false)),
            Arc::new(Field::new(
                consts::ATTRIBUTE_STR,
                dict_body_str.data_type().clone(),
                true,
            )),
        ]
        .into(),
        vec![
            Arc::new(UInt8Array::from(vec![
                AttributeValueType::Str as u8,
                AttributeValueType::Str as u8,
                AttributeValueType::Empty as u8,
            ])),
            Arc::new(dict_body_str),
        ],
        None,
    );
    let dict_root = RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new(consts::ID, DataType::UInt16, false).with_plain_encoding(),
            Field::new(consts::BODY, dict_body_struct.data_type().clone(), true),
        ])),
        vec![
            Arc::new(UInt16Array::from(vec![0u16, 1, 2])),
            Arc::new(dict_body_struct),
        ],
    )
    .unwrap();
    let mut dict_batch = OtapArrowRecords::Logs(Logs::default());
    dict_batch.set(ArrowPayloadType::Logs, dict_root).unwrap();

    // Exists must treat "" as absent on both plain and dictionary-encoded StructArray Body
    let exists_eng = compile_engine(&[Policy::log_filter(
        "drop_body_exists",
        vec![LogMatcher {
            field: LogFieldSelector::RecordField(LogRecordField::Body),
            predicate: Predicate::Exists,
            negate: false,
        }],
        Action::Drop,
    )]);
    for (label, batch) in [("struct", &struct_batch), ("dict", &dict_batch)] {
        assert_eq!(
            exists_eng
                .evaluate_filter(batch, &mut EvaluationScratch::new(), None)
                .unwrap()
                .into_bitmap(),
            DenseBitset::from_indices([0, 2]),
            "{label}: Body == \"\" must evaluate Exists to false"
        );
    }

    // LogRedact on Body == "" must be a no-op (not replace "" with "[REDACTED]")
    let redact_eng = compile_engine(&[Policy::log_transform(
        "redact_body",
        LogTransformPolicy {
            matches: vec![],
            rename: vec![],
            add: vec![],
            remove: vec![],
            redact: vec![LogRedact {
                target: LogTargetSelector::RecordField(LogRecordField::Body),
                replacement: Some("[REDACTED]".into()),
                regex: None,
            }],
        },
    )]);
    for (label, batch) in [("struct", &struct_batch), ("dict", &dict_batch)] {
        let out = redact_eng
            .evaluate(batch, &mut EvaluationScratch::new(), None)
            .unwrap()
            .records
            .unwrap();
        let view = LogsBatchView::try_new(&out).unwrap().unwrap();
        assert_eq!(
            view.extract_field_at_row(&LogFieldSelector::RecordField(LogRecordField::Body), 0),
            Some(ExtractedValue::Str(Cow::Borrowed(""))),
            "{label}: LogRedact must not overwrite empty-string Body"
        );
        assert_eq!(
            view.extract_field_at_row(&LogFieldSelector::RecordField(LogRecordField::Body), 1),
            Some(ExtractedValue::Str(Cow::Borrowed("[REDACTED]"))),
            "{label}: LogRedact must mask non-empty Body"
        );
    }

    // LogAdd(Body, upsert=false) populates "" Body without overwriting "real" Body
    let add_body_eng = compile_engine(&[Policy::log_transform(
        "fill_body",
        LogTransformPolicy {
            matches: vec![],
            rename: vec![],
            add: vec![LogAdd {
                target: LogTargetSelector::RecordField(LogRecordField::Body),
                value: ScalarValue::String("filled".into()),
                upsert: false,
            }],
            remove: vec![],
            redact: vec![],
        },
    )]);
    for (label, batch) in [("struct", &struct_batch), ("dict", &dict_batch)] {
        let out = add_body_eng
            .evaluate(batch, &mut EvaluationScratch::new(), None)
            .unwrap()
            .records
            .unwrap();
        let view = LogsBatchView::try_new(&out).unwrap().unwrap();
        assert_eq!(
            view.extract_field_at_row(&LogFieldSelector::RecordField(LogRecordField::Body), 0),
            Some(ExtractedValue::Str(Cow::Borrowed("filled"))),
            "{label}: LogAdd(upsert=false) must fill empty-string Body"
        );
        assert_eq!(
            view.extract_field_at_row(&LogFieldSelector::RecordField(LogRecordField::Body), 1),
            Some(ExtractedValue::Str(Cow::Borrowed("real"))),
            "{label}: LogAdd(upsert=false) must not overwrite non-empty Body"
        );
    }

    // BodyField write initializes "" Body as a Map without touching "real" Body
    let add_bf_eng = compile_engine(&[Policy::log_transform(
        "init_bf",
        LogTransformPolicy {
            matches: vec![],
            rename: vec![],
            add: vec![LogAdd {
                target: LogTargetSelector::BodyField(AttributePath::nested(["req", "id"])),
                value: ScalarValue::Int(7),
                upsert: false,
            }],
            remove: vec![],
            redact: vec![],
        },
    )]);
    for (label, batch) in [("struct", &struct_batch), ("dict", &dict_batch)] {
        let out = add_bf_eng
            .evaluate(batch, &mut EvaluationScratch::new(), None)
            .unwrap()
            .records
            .unwrap();
        let view = LogsBatchView::try_new(&out).unwrap().unwrap();
        assert_eq!(
            view.extract_field_at_row(
                &LogFieldSelector::BodyField(AttributePath::nested(["req", "id"])),
                0
            ),
            Some(ExtractedValue::Int(7)),
            "{label}: BodyField write must initialize empty-string Body as Map"
        );
        assert_eq!(
            view.extract_field_at_row(&LogFieldSelector::RecordField(LogRecordField::Body), 1),
            Some(ExtractedValue::Str(Cow::Borrowed("real"))),
            "{label}: BodyField write must fail-open on non-empty string Body"
        );
    }
}

/// Scenario: Run two `LogTransformPolicy` rules where policy A mutates the attribute matched by policy B, plus filter/transform evaluation over a multi-resource, multi-scope transport-optimized OTAP batch.
/// Guarantees: Transform matchers evaluate once against the pre-transform record (`policy.md` "Execution Order"), and transport-optimized multi-resource/multi-scope delta IDs decode cleanly before filtering and transforming.
#[test]
fn test_transform_pre_mutation_matcher_isolation_and_multi_res_scope_transport_opt() {
    // 1. Pre-transform matcher isolation: policy `a_set_flag` adds `flag = "yes"` and removes `orig`,
    //    while `b_match_flag` matches `flag == "yes"` (must NOT fire) and `c_match_orig` matches
    //    `orig == "present"` (MUST fire even though `a_set_flag` removes `orig` in Stage 3).
    let batch = build_custom_test_batch(
        vec![Some("hello")],
        vec![9],
        vec!["INFO"],
        vec![[1u8; 16]],
        vec![[1u8; 8]],
        vec![(0, "orig", ScalarValue::String("present".into()))],
        vec![],
    );
    let iso_eng = compile_engine(&[
        Policy::log_transform(
            "a_mutate",
            LogTransformPolicy {
                matches: vec![],
                rename: vec![],
                add: vec![LogAdd {
                    target: LogTargetSelector::LogAttribute(AttributePath::single("flag")),
                    value: ScalarValue::String("yes".into()),
                    upsert: true,
                }],
                remove: vec![LogRemove {
                    target: LogAttributeSelector::LogAttribute(AttributePath::single("orig")),
                }],
                redact: vec![],
            },
        ),
        Policy::log_transform(
            "b_match_added_flag",
            LogTransformPolicy {
                matches: vec![LogMatcher {
                    field: LogFieldSelector::LogAttribute(AttributePath::single("flag")),
                    predicate: Predicate::Equals(ScalarValue::String("yes".into())),
                    negate: false,
                }],
                rename: vec![],
                add: vec![LogAdd {
                    target: LogTargetSelector::LogAttribute(AttributePath::single(
                        "should_not_exist",
                    )),
                    value: ScalarValue::Bool(true),
                    upsert: true,
                }],
                remove: vec![],
                redact: vec![],
            },
        ),
        Policy::log_transform(
            "c_match_removed_orig",
            LogTransformPolicy {
                matches: vec![LogMatcher {
                    field: LogFieldSelector::LogAttribute(AttributePath::single("orig")),
                    predicate: Predicate::Equals(ScalarValue::String("present".into())),
                    negate: false,
                }],
                rename: vec![],
                add: vec![LogAdd {
                    target: LogTargetSelector::LogAttribute(AttributePath::single(
                        "matched_pre_transform",
                    )),
                    value: ScalarValue::Bool(true),
                    upsert: true,
                }],
                remove: vec![],
                redact: vec![],
            },
        ),
    ]);
    let out = iso_eng
        .evaluate(&batch, &mut EvaluationScratch::new(), None)
        .unwrap()
        .records
        .unwrap();
    let view = LogsBatchView::try_new(&out).unwrap().unwrap();
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::single("should_not_exist")),
            0
        ),
        None
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::single("matched_pre_transform")),
            0
        ),
        Some(ExtractedValue::Bool(true))
    );

    // 2. Multi-resource, multi-scope transport-optimized OTAP batch parity with plain batch.
    use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{
        AnyValue, InstrumentationScope, KeyValue,
    };
    use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::{
        LogRecord, LogsData, ResourceLogs, ScopeLogs,
    };
    use otel_arrow_dfe_pdata::proto::opentelemetry::resource::v1::Resource;

    let logs_data = LogsData {
        resource_logs: vec![
            ResourceLogs {
                resource: Some(Resource {
                    attributes: vec![KeyValue::new("service.name", AnyValue::new_string("svc-a"))],
                    ..Default::default()
                }),
                scope_logs: vec![
                    ScopeLogs {
                        scope: Some(InstrumentationScope {
                            name: "scope-1".into(),
                            attributes: vec![KeyValue::new("tier", AnyValue::new_string("core"))],
                            ..Default::default()
                        }),
                        log_records: vec![
                            LogRecord::build().body(AnyValue::new_string("a1")).finish(),
                            LogRecord::build().body(AnyValue::new_string("a2")).finish(),
                        ],
                        ..Default::default()
                    },
                    ScopeLogs {
                        scope: Some(InstrumentationScope {
                            name: "scope-2".into(),
                            attributes: vec![KeyValue::new("tier", AnyValue::new_string("edge"))],
                            ..Default::default()
                        }),
                        log_records: vec![
                            LogRecord::build().body(AnyValue::new_string("a3")).finish(),
                        ],
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            ResourceLogs {
                resource: Some(Resource {
                    attributes: vec![KeyValue::new("service.name", AnyValue::new_string("svc-b"))],
                    ..Default::default()
                }),
                scope_logs: vec![ScopeLogs {
                    scope: Some(InstrumentationScope {
                        name: "scope-3".into(),
                        attributes: vec![KeyValue::new("tier", AnyValue::new_string("core"))],
                        ..Default::default()
                    }),
                    log_records: vec![LogRecord::build().body(AnyValue::new_string("b1")).finish()],
                    ..Default::default()
                }],
                ..Default::default()
            },
        ],
    };
    let plain = encode_otlp_logs(&logs_data);
    let mut transport = plain.clone();
    transport.encode_transport_optimized().unwrap();

    let eng = compile_engine(&[
        Policy::log_filter(
            "drop_svc_a_edge",
            vec![
                LogMatcher {
                    field: LogFieldSelector::ResourceAttribute(AttributePath::single(
                        "service.name",
                    )),
                    predicate: Predicate::Equals(ScalarValue::String("svc-a".into())),
                    negate: false,
                },
                LogMatcher {
                    field: LogFieldSelector::ScopeAttribute(AttributePath::single("tier")),
                    predicate: Predicate::Equals(ScalarValue::String("edge".into())),
                    negate: false,
                },
            ],
            Action::Drop,
        ),
        Policy::log_transform(
            "tag_svc_b_core",
            LogTransformPolicy {
                matches: vec![
                    LogMatcher {
                        field: LogFieldSelector::ResourceAttribute(AttributePath::single(
                            "service.name",
                        )),
                        predicate: Predicate::Equals(ScalarValue::String("svc-b".into())),
                        negate: false,
                    },
                    LogMatcher {
                        field: LogFieldSelector::ScopeAttribute(AttributePath::single("tier")),
                        predicate: Predicate::Equals(ScalarValue::String("core".into())),
                        negate: false,
                    },
                ],
                rename: vec![],
                add: vec![LogAdd {
                    target: LogTargetSelector::LogAttribute(AttributePath::single("tagged")),
                    value: ScalarValue::String("yes".into()),
                    upsert: true,
                }],
                remove: vec![],
                redact: vec![],
            },
        ),
    ]);

    let mut scratch = EvaluationScratch::new();
    let plain_out = eng.evaluate(&plain, &mut scratch, None).unwrap();
    let trans_out = eng.evaluate(&transport, &mut scratch, None).unwrap();
    assert_eq!(plain_out.kept_rows, 3);
    assert_eq!(plain_out.dropped_rows, 1);
    assert_eq!(trans_out.kept_rows, 3);
    assert_eq!(trans_out.dropped_rows, 1);

    let plain_otlp = decode_otlp_logs(plain_out.records.unwrap());
    let trans_otlp = decode_otlp_logs(trans_out.records.unwrap());
    let collect_bodies_and_tags = |d: &LogsData| -> Vec<(String, bool)> {
        let mut v: Vec<(String, bool)> = d
            .resource_logs
            .iter()
            .flat_map(|rl| rl.scope_logs.iter())
            .flat_map(|sl| sl.log_records.iter())
            .map(|lr| {
                let body = format!("{:?}", lr.body);
                let tagged = lr.attributes.iter().any(|kv| kv.key == "tagged");
                (body, tagged)
            })
            .collect();
        v.sort();
        v
    };
    assert_eq!(
        collect_bodies_and_tags(&plain_otlp),
        collect_bodies_and_tags(&trans_otlp)
    );
    assert_eq!(
        collect_bodies_and_tags(&trans_otlp)
            .iter()
            .filter(|(_, tagged)| *tagged)
            .count(),
        1
    );
}

/// Scenario: Evaluate batches whose dictionary-encoded `ATTRIBUTE_BYTES` (`Dict<U8, FixedSizeBinary(4)>` and `Dict<U16, Binary>`), `ATTRIBUTE_SER` (`Dict<U16, Binary>`), and `ATTRIBUTE_STR` (`Dict<U16, Utf8>`) contain NULL dictionary value slots backed by non-empty masked byte/UTF-8 payloads across both low-selectivity and high-selectivity paths.
/// Guarantees: Dictionary-value validity is always respected: `Dict<U8, FixedSizeBinary(4)>` null dictionary slots decode as `&[]` with exact low- vs. high-selectivity parity, null-dict `Map`/`Slice` bodies evaluate `Exists` to `false` without leaking masked CBOR, and null-dict `Str` slots evaluate as `""` without leaking masked text.
#[test]
fn test_null_dict_values_on_bytes_ser_and_str_columns_eval_parity() {
    // 1. Direct parity check for `Dict<U8, FixedSizeBinary(4)>` with masked bytes behind null slot 0.
    let fsb_vals = FixedSizeBinaryArray::try_new(
        4,
        Buffer::from_slice_ref([0xdeu8, 0xad, 0xbe, 0xef, 0x01, 0x02, 0x03, 0x04]),
        Some(NullBuffer::from(vec![false, true])),
    )
    .unwrap();
    let fsb_dict = DictionaryArray::<UInt8Type>::try_new(
        UInt8Array::from(vec![Some(0u8), Some(1u8)]),
        Arc::new(fsb_vals),
    )
    .unwrap();
    let fsb_attr_rb = RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
            Field::new(consts::ATTRIBUTE_BYTES, fsb_dict.data_type().clone(), true),
        ])),
        vec![
            Arc::new(UInt8Array::from(vec![
                AttributeValueType::Bytes as u8,
                AttributeValueType::Bytes as u8,
            ])),
            Arc::new(fsb_dict.clone()),
        ],
    )
    .unwrap();
    assert_eq!(
        extract_anyval_from_attr_batch(&fsb_attr_rb, 0),
        Some(ExtractedValue::Bytes(Cow::Borrowed(&[])))
    );
    assert_eq!(
        extract_anyval_from_attr_batch(&fsb_attr_rb, 1),
        Some(ExtractedValue::Bytes(Cow::Borrowed(&[
            0x01, 0x02, 0x03, 0x04
        ])))
    );
    let mut pool = BitsetBufferPool::default();
    let mask = DenseBitset::full(2);
    let mut eq_empty = DenseBitset::new();
    eval_anyval_bytes_eq_into(Some(&fsb_dict), &[], &mask, &mut pool, &mut eq_empty);
    assert_eq!(eq_empty, DenseBitset::from_indices([0]));
    let mut eq_masked = DenseBitset::new();
    eval_anyval_bytes_eq_into(
        Some(&fsb_dict),
        &[0xde, 0xad, 0xbe, 0xef],
        &mask,
        &mut pool,
        &mut eq_masked,
    );
    assert!(eq_masked.is_empty());
    pool.recycle(eq_empty);
    pool.recycle(eq_masked);
    pool.recycle(mask);

    // 2. End-to-end OTAP batch with masked non-empty payloads behind NULL dictionary slots on
    //    `logs.body` and `LogAttrs` across both low-selectivity (>20%) and high-selectivity (<=20%) keys.
    let masked_map_cbor = encode_cbor(&ciborium::Value::Map(vec![(
        ciborium::Value::Text("secret".into()),
        ciborium::Value::Text("leaked".into()),
    )]));
    let valid_map_cbor = encode_cbor(&ciborium::Value::Map(vec![(
        ciborium::Value::Text("secret".into()),
        ciborium::Value::Text("ok".into()),
    )]));
    let masked_slice_cbor = encode_cbor(&ciborium::Value::Array(vec![ciborium::Value::Text(
        "leaked".into(),
    )]));
    let mut ser_buf = Vec::new();
    ser_buf.extend_from_slice(&masked_map_cbor);
    ser_buf.extend_from_slice(&masked_slice_cbor);
    ser_buf.extend_from_slice(&valid_map_cbor);
    let ser_dict_vals = BinaryArray::try_new(
        OffsetBuffer::from_lengths([
            masked_map_cbor.len(),
            masked_slice_cbor.len(),
            valid_map_cbor.len(),
        ]),
        Buffer::from_slice_ref(&ser_buf),
        Some(NullBuffer::from(vec![false, false, true])),
    )
    .unwrap();

    let mut bytes_buf = Vec::new();
    bytes_buf.extend_from_slice(&[0xde, 0xad, 0xbe, 0xef]);
    bytes_buf.extend_from_slice(&[0xaa, 0xbb]);
    let bytes_dict_vals = BinaryArray::try_new(
        OffsetBuffer::from_lengths([4, 2]),
        Buffer::from_slice_ref(&bytes_buf),
        Some(NullBuffer::from(vec![false, true])),
    )
    .unwrap();

    let str_dict_vals = StringArray::try_new(
        OffsetBuffer::from_lengths(["SECRET_MASKED".len(), "visible".len()]),
        Buffer::from_slice_ref(b"SECRET_MASKEDvisible"),
        Some(NullBuffer::from(vec![false, true])),
    )
    .unwrap();

    // 4 log rows:
    // row 0: body Map -> null dict slot 0 (masked {"secret":"leaked"})
    // row 1: body Slice -> null dict slot 1 (masked ["leaked"])
    // row 2: body Bytes -> null dict slot 0 (masked [0xde, 0xad, 0xbe, 0xef]) -> logical &[]
    // row 3: body Str -> null dict slot 0 (masked "SECRET_MASKED") -> logical ""
    let body_struct = StructArray::try_new(
        vec![
            Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
            Field::new(
                consts::ATTRIBUTE_STR,
                DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                true,
            ),
            Field::new(
                consts::ATTRIBUTE_BYTES,
                DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Binary)),
                true,
            ),
            Field::new(
                consts::ATTRIBUTE_SER,
                DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Binary)),
                true,
            ),
        ]
        .into(),
        vec![
            Arc::new(UInt8Array::from(vec![
                AttributeValueType::Map as u8,
                AttributeValueType::Slice as u8,
                AttributeValueType::Bytes as u8,
                AttributeValueType::Str as u8,
            ])),
            Arc::new(
                DictionaryArray::<UInt16Type>::try_new(
                    UInt16Array::from(vec![None, None, None, Some(0u16)]),
                    Arc::new(str_dict_vals.clone()),
                )
                .unwrap(),
            ),
            Arc::new(
                DictionaryArray::<UInt16Type>::try_new(
                    UInt16Array::from(vec![None, None, Some(0u16), None]),
                    Arc::new(bytes_dict_vals.clone()),
                )
                .unwrap(),
            ),
            Arc::new(
                DictionaryArray::<UInt16Type>::try_new(
                    UInt16Array::from(vec![Some(0u16), Some(1u16), None, None]),
                    Arc::new(ser_dict_vals.clone()),
                )
                .unwrap(),
            ),
        ],
        None,
    )
    .unwrap();

    let root = RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new(consts::ID, DataType::UInt16, false).with_plain_encoding(),
            Field::new(consts::BODY, body_struct.data_type().clone(), true),
        ])),
        vec![
            Arc::new(UInt16Array::from(vec![0u16, 1, 2, 3])),
            Arc::new(body_struct),
        ],
    )
    .unwrap();

    // Build 70 LogAttrs rows so `common_k` hits low-selectivity (>20%, >=64 rows) and `rare_k` hits
    // high-selectivity (<=20%). Both attach masked null-dict Bytes/Map/Str to rows 0..3.
    let mut a_pids = Vec::new();
    let mut a_keys = Vec::new();
    let mut a_types = Vec::new();
    let mut a_str_idx: Vec<Option<u16>> = Vec::new();
    let mut a_bytes_idx: Vec<Option<u16>> = Vec::new();
    let mut a_ser_idx: Vec<Option<u16>> = Vec::new();
    for i in 0..64u16 {
        a_pids.push(i % 4);
        a_keys.push("common_k");
        a_types.push(AttributeValueType::Bytes as u8);
        a_str_idx.push(None);
        a_bytes_idx.push(Some(0)); // null dict slot 0 (masked 0xdeadbeef -> logical &[])
        a_ser_idx.push(None);
    }
    // High-selectivity rare_k rows (1 per log):
    // log 0: Bytes null-dict 0; log 1: Map null-dict 0; log 2: Slice null-dict 1; log 3: Str null-dict 0
    for (pid, vt, si, bi, seri) in [
        (0u16, AttributeValueType::Bytes, None, Some(0), None),
        (1, AttributeValueType::Map, None, None, Some(0)),
        (2, AttributeValueType::Slice, None, None, Some(1)),
        (3, AttributeValueType::Str, Some(0), None, None),
    ] {
        a_pids.push(pid);
        a_keys.push("rare_k");
        a_types.push(vt as u8);
        a_str_idx.push(si);
        a_bytes_idx.push(bi);
        a_ser_idx.push(seri);
    }
    let attrs = RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new(consts::PARENT_ID, DataType::UInt16, false).with_plain_encoding(),
            Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
            Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
            Field::new(
                consts::ATTRIBUTE_STR,
                DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                true,
            ),
            Field::new(
                consts::ATTRIBUTE_BYTES,
                DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Binary)),
                true,
            ),
            Field::new(
                consts::ATTRIBUTE_SER,
                DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Binary)),
                true,
            ),
        ])),
        vec![
            Arc::new(UInt16Array::from(a_pids)),
            Arc::new(StringArray::from(a_keys)),
            Arc::new(UInt8Array::from(a_types)),
            Arc::new(
                DictionaryArray::<UInt16Type>::try_new(
                    UInt16Array::from(a_str_idx),
                    Arc::new(str_dict_vals),
                )
                .unwrap(),
            ),
            Arc::new(
                DictionaryArray::<UInt16Type>::try_new(
                    UInt16Array::from(a_bytes_idx),
                    Arc::new(bytes_dict_vals),
                )
                .unwrap(),
            ),
            Arc::new(
                DictionaryArray::<UInt16Type>::try_new(
                    UInt16Array::from(a_ser_idx),
                    Arc::new(ser_dict_vals),
                )
                .unwrap(),
            ),
        ],
    )
    .unwrap();

    let mut batch = OtapArrowRecords::Logs(Logs::default());
    batch.set(ArrowPayloadType::Logs, root).unwrap();
    batch.set(ArrowPayloadType::LogAttrs, attrs).unwrap();

    // No masked payload may ever match on body, low-selectivity attr, or high-selectivity attr.
    let no_leak_eng = compile_engine(&[
        Policy::log_filter(
            "body_map_leak",
            vec![LogMatcher {
                field: LogFieldSelector::BodyField(AttributePath::single("secret")),
                predicate: Predicate::Exists,
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "body_slice_leak",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Contains(ScalarValue::String("leaked".into())),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "body_bytes_leak",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Contains(ScalarValue::Bytes(vec![0xde, 0xad])),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "body_str_leak",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Equals(ScalarValue::String("SECRET_MASKED".into())),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "attr_low_sel_bytes_leak",
            vec![LogMatcher {
                field: LogFieldSelector::LogAttribute(AttributePath::single("common_k")),
                predicate: Predicate::Contains(ScalarValue::Bytes(vec![0xde, 0xad])),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "attr_high_sel_bytes_leak",
            vec![LogMatcher {
                field: LogFieldSelector::LogAttribute(AttributePath::single("rare_k")),
                predicate: Predicate::Equals(ScalarValue::Bytes(vec![0xde, 0xad, 0xbe, 0xef])),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "attr_high_sel_map_leak",
            vec![LogMatcher {
                field: LogFieldSelector::LogAttribute(AttributePath::nested(["rare_k", "secret"])),
                predicate: Predicate::Exists,
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "attr_high_sel_slice_leak",
            vec![LogMatcher {
                field: LogFieldSelector::LogAttribute(AttributePath::single("rare_k")),
                predicate: Predicate::Contains(ScalarValue::String("leaked".into())),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "attr_high_sel_str_leak",
            vec![LogMatcher {
                field: LogFieldSelector::LogAttribute(AttributePath::single("rare_k")),
                predicate: Predicate::Equals(ScalarValue::String("SECRET_MASKED".into())),
                negate: false,
            }],
            Action::Drop,
        ),
    ]);
    let kept = no_leak_eng
        .evaluate_filter(&batch, &mut EvaluationScratch::new(), None)
        .unwrap()
        .into_bitmap();
    assert_eq!(kept, DenseBitset::from_indices([0, 1, 2, 3]));

    // Only row 2 (Body Bytes -> logical &[]) has a present Body; null-dict Map (0), Slice (1), and
    // Str (3, logical "") are all absent for Body `Exists`.
    let body_exists_eng = compile_engine(&[Policy::log_filter(
        "drop_body_exists",
        vec![LogMatcher {
            field: LogFieldSelector::RecordField(LogRecordField::Body),
            predicate: Predicate::Exists,
            negate: false,
        }],
        Action::Drop,
    )]);
    let kept_exists = body_exists_eng
        .evaluate_filter(&batch, &mut EvaluationScratch::new(), None)
        .unwrap()
        .into_bitmap();
    assert_eq!(kept_exists, DenseBitset::from_indices([0, 1, 3]));

    // Logical default `&[]` matches on both low-selectivity `common_k` (all rows) and
    // high-selectivity `rare_k` (row 0), and logical default `""` matches on `rare_k` (row 3).
    let default_eq_eng = compile_engine(&[
        Policy::log_filter(
            "drop_rare_bytes_empty",
            vec![LogMatcher {
                field: LogFieldSelector::LogAttribute(AttributePath::single("rare_k")),
                predicate: Predicate::Equals(ScalarValue::Bytes(vec![])),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "drop_rare_str_empty",
            vec![LogMatcher {
                field: LogFieldSelector::LogAttribute(AttributePath::single("rare_k")),
                predicate: Predicate::Equals(ScalarValue::String(String::new())),
                negate: false,
            }],
            Action::Drop,
        ),
    ]);
    let kept_defaults = default_eq_eng
        .evaluate_filter(&batch, &mut EvaluationScratch::new(), None)
        .unwrap()
        .into_bitmap();
    assert_eq!(kept_defaults, DenseBitset::from_indices([1, 2]));
}
