// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use std::path::{Path, PathBuf};

fn test_path(path: &str) -> PathBuf {
    if cfg!(windows) && path.starts_with('/') {
        PathBuf::from(format!("C:{path}"))
    } else {
        PathBuf::from(path)
    }
}

fn parsed(extra: &str) -> Config {
    let include = if cfg!(windows) {
        "C:/var/log/*.log"
    } else {
        "/var/log/*.log"
    };
    serde_yaml::from_str(&format!("include: ['{include}']\n{extra}")).unwrap()
}
fn compiled(extra: &str) -> Result<CompiledConfig, ConfigError> {
    parsed(extra).compile("default", "main", "filelog", test_path("/work").as_path())
}

/// Scenario: Only the required include is supplied through YAML.
/// Guarantees: Documented defaults compile into immutable primitive and store inputs.
#[test]
fn defaults() {
    let c = compiled("").unwrap();
    assert_eq!(
        c.inputs().checkpoint_id(),
        "auto-f646ab9be8acd63559b3cee62718d6802d11c0ec67131c27cb91eebc3a2ddc47"
    );
    let s = c.inputs().settings();
    assert!(s.recursive);
    assert!(!s.follow_symlinks);
    assert_eq!(s.max_recursion_depth, 64);
    assert_eq!(s.include_file_metadata, FileMetadata::None);
    assert_eq!(s.start_at, StartAt::End);
    assert_eq!(s.discovery.reconcile_interval, Duration::from_secs(5));
    assert_eq!(s.discovery.reconcile_jitter_percent, 10);
    assert_eq!(s.reader.eof_reprobe_interval, Duration::from_millis(250));
    assert_eq!(s.identity.fingerprint_bytes, 1000);
    assert_eq!(s.identity.ignored_header_bytes, 0);
    assert_eq!(s.identity.on_recovery_mismatch, RecoveryMismatch::Beginning);
    assert_eq!(s.encoding, Encoding::Utf8);
    assert_eq!(s.on_decode_error, OnDecodeError::PreserveRaw);
    assert_eq!(s.framing.max_line_bytes, 1024 * 1024);
    assert_eq!(s.framing.max_record_bytes, 1024 * 1024);
    assert_eq!(s.framing.max_log_size_behavior, OversizeBehavior::Split);
    assert_eq!(s.framing.max_multiline_lines, 500);
    assert!(s.framing.force_flush_period.is_zero());
    assert!(s.ignore_older_than.is_zero());
    assert_eq!(s.limits.max_tracked_files, 10000);
    assert_eq!(s.limits.max_pending_candidates, 10000);
    assert_eq!(s.limits.max_open_files, 512);
    assert_eq!(s.limits.max_read_bytes_per_turn, 128 * 1024);
    assert_eq!(s.limits.max_partial_state_bytes, 256 * 1024 * 1024);
    assert_eq!(s.batch.max_records, 1024);
    assert_eq!(s.batch.max_bytes, 8 * 1024 * 1024);
    assert_eq!(s.batch.max_flush_period, Duration::from_secs(1));
    assert_eq!(s.rotation.rotate_wait, Duration::from_secs(5));
    assert_eq!(s.rotation.on_truncate, TruncatePolicy::Fail);
    assert!(s.checkpoint.sync_interval.is_zero());
    assert_eq!(s.checkpoint.compact_after_bytes, 64 * 1024 * 1024);
    assert_eq!(s.checkpoint.compact_after_transactions, 10000);
    assert_eq!(s.checkpoint.retention, Duration::from_secs(7 * 86400));
    assert_eq!(s.checkpoint.ownership_timeout, Duration::from_secs(30));
    assert_eq!(s.checkpoint.max_consecutive_failures, 5);
    assert_eq!(s.retry.max_attempts, 8);
    assert_eq!(s.retry.initial_backoff, Duration::from_millis(100));
    assert_eq!(s.retry.max_backoff, Duration::from_secs(5));
    assert_eq!(s.on_retry_exhaustion, RetryExhaustion::Pause);
    assert_eq!(s.drain_timeout, Duration::from_secs(10));
    assert_eq!(s.worker_shutdown_timeout, Duration::from_secs(5));
    assert!(c.inputs().boundary().is_none());
    assert!(c.inputs().grouping().is_none());
    #[cfg(target_os = "linux")]
    assert_eq!(
        c.inputs().store_config().unwrap().replay.max_tracked_files,
        10000
    );
}

/// Scenario: Configuration supplies unknown fields, enum variants, null sizes or malformed values.
/// Guarantees: The schema rejects mistakes at every nested level without accepting obsolete aliases.
#[test]
fn strict_schema() {
    for extra in [
        "on_nack: pause",
        "include_file_metadata: all",
        "encoding: utf16",
        "on_decode_error: ignore",
        "start_at: middle",
        "discovery: {unknown: 1}",
        "reader: {unknown: 1}",
        "identity: {unknown: 1}",
        "framing: {unknown: 1}",
        "framing: {multiline: {regex_profile: pcre}}",
        "limits: {unknown: 1}",
        "batch: {unknown: 1}",
        "rotation: {unknown: 1}",
        "checkpoint: {unknown: 1}",
        "retry: {unknown: 1}",
        "batch: {max_bytes: null}",
        "batch: {max_bytes: -1}",
        "drain_timeout: nonsense",
    ] {
        assert!(
            serde_yaml::from_str::<Config>(&format!("include: ['/x']\n{extra}")).is_err(),
            "{extra}"
        );
    }
    assert!(
        serde_yaml::from_str::<Config>("{}")
            .unwrap()
            .compile("g", "p", "n", test_path("/work").as_path())
            .is_err()
    );
}

/// Scenario: Invalid scalars and relationships are independently supplied.
/// Guarantees: Each rejection identifies the setting, including overflow and resource ceilings.
#[test]
fn invalid_settings() {
    for (extra, setting) in [
        ("max_recursion_depth: 0", "max_recursion_depth"),
        ("max_recursion_depth: 1025", "max_recursion_depth"),
        (
            "discovery: {reconcile_interval: 99ms}",
            "discovery.reconcile_interval",
        ),
        (
            "discovery: {reconcile_interval: 25h}",
            "discovery.reconcile_interval",
        ),
        (
            "discovery: {reconcile_jitter_percent: 26}",
            "discovery.reconcile_jitter_percent",
        ),
        (
            "reader: {eof_reprobe_interval: 9ms}",
            "reader.eof_reprobe_interval",
        ),
        (
            "reader: {eof_reprobe_interval: 3601s}",
            "reader.eof_reprobe_interval",
        ),
        (
            "identity: {fingerprint_bytes: 15}",
            "identity.fingerprint_bytes",
        ),
        (
            "identity: {fingerprint_bytes: 65536}",
            "identity.fingerprint_bytes",
        ),
        (
            "identity: {ignored_header_bytes: 4294967296}",
            "identity.ignored_header_bytes",
        ),
        ("framing: {max_line_bytes: 3}", "framing.max_line_bytes"),
        (
            "framing: {max_multiline_lines: 0}",
            "framing.max_multiline_lines",
        ),
        (
            "framing: {force_flush_period: 1ns}",
            "framing.force_flush_period",
        ),
        (
            "framing: {force_flush_period: 5s}",
            "framing.force_flush_period",
        ),
        ("limits: {max_tracked_files: 0}", "limits.max_tracked_files"),
        (
            "limits: {max_tracked_files: 4294967296}",
            "limits.max_tracked_files",
        ),
        (
            "limits: {max_pending_candidates: 0}",
            "limits.max_pending_candidates",
        ),
        ("limits: {max_open_files: 10001}", "limits.max_open_files"),
        (
            "limits: {max_read_bytes_per_turn: 0}",
            "limits.max_read_bytes_per_turn",
        ),
        (
            "limits: {max_partial_state_bytes: 1}",
            "limits.max_partial_state_bytes",
        ),
        (
            "limits: {max_pending_candidates: 18446744073709551615}",
            "limits",
        ),
        (
            "limits: {max_pending_candidates: 1000000}",
            "limits/identity",
        ),
        ("batch: {max_records: 65536}", "batch.max_records"),
        ("batch: {max_bytes: 1}", "batch.max_bytes"),
        ("batch: {max_flush_period: 0s}", "batch.max_flush_period"),
        ("rotation: {rotate_wait: 0s}", "rotation.rotate_wait"),
        (
            "checkpoint: {compact_after_bytes: 16777255}",
            "checkpoint.compact_after_bytes",
        ),
        (
            "checkpoint: {compact_after_transactions: 0}",
            "checkpoint.compact_after_transactions",
        ),
        (
            "checkpoint: {max_consecutive_failures: 0}",
            "checkpoint.max_consecutive_failures",
        ),
        (
            "checkpoint: {ownership_timeout: 0s}",
            "checkpoint.ownership_timeout",
        ),
        ("checkpoint: {retention: 1000years}", "checkpoint.retention"),
        ("retry: {max_attempts: 0}", "retry.max_attempts"),
        ("retry: {initial_backoff: 0s}", "retry.initial_backoff"),
        ("retry: {max_backoff: 99ms}", "retry.max_backoff"),
        ("drain_timeout: 0s", "drain_timeout"),
        ("worker_shutdown_timeout: 0s", "worker_shutdown_timeout"),
        (
            "framing: {max_record_bytes: 18446744073709551615}",
            "framing",
        ),
    ] {
        let err = compiled(extra).unwrap_err();
        assert!(err.setting.contains(setting), "{extra}: {err}");
    }
}

/// Scenario: Minimum and maximum accepted scalar limits are used with compatible populations.
/// Guarantees: Boundary values and shared byte-unit parsing remain accepted.
#[test]
fn accepted_boundaries() {
    let _ = compiled("max_recursion_depth: 1024\ndiscovery: {reconcile_interval: 24h, reconcile_jitter_percent: 25}\nreader: {eof_reprobe_interval: 1h}\nidentity: {fingerprint_bytes: 65535, ignored_header_bytes: 4294967295}\nlimits: {max_tracked_files: 1, max_pending_candidates: 1, max_open_files: 1}\nbatch: {max_records: 65535}\ncheckpoint: {compact_after_bytes: 16777256, compact_after_transactions: 18446744073709551615}").unwrap();
    let _ = compiled("max_recursion_depth: 1\ndiscovery: {reconcile_interval: 100ms, reconcile_jitter_percent: 0}\nreader: {eof_reprobe_interval: 10ms}\nidentity: {fingerprint_bytes: 16}\nframing: {max_line_bytes: 4, max_record_bytes: 4, force_flush_period: 1ms}\nlimits: {max_read_bytes_per_turn: 1KiB}\ncheckpoint: {retention: 0s}").unwrap();
}

/// Scenario: Encoding and decode policy select different smallest valid framing units.
/// Guarantees: Configuration reuses primitive validation, including ASCII replacement expansion.
#[test]
fn encoding_minima() {
    for (encoding, policy, minimum) in [
        ("raw", "preserve_raw", 1),
        ("ascii", "fail", 1),
        ("ascii", "replace", 3),
        ("ascii", "preserve_raw", 3),
        ("utf-8", "fail", 4),
        ("utf-16le", "replace", 4),
        ("utf-16be", "preserve_raw", 4),
    ] {
        let _ = compiled(&format!("encoding: {encoding}\non_decode_error: {policy}\nframing: {{max_line_bytes: {minimum}, max_record_bytes: {minimum}}}")).unwrap();
        assert!(compiled(&format!("encoding: {encoding}\non_decode_error: {policy}\nframing: {{max_record_bytes: {}}}", minimum - 1)).is_err());
        assert!(
            compiled(&format!(
                "encoding: {encoding}\non_decode_error: {policy}\nframing: {{max_line_bytes: {}}}",
                minimum - 1
            ))
            .is_err()
        );
    }
}

/// Scenario: Multiline patterns are compiled for decoded text or raw bytes.
/// Guarantees: Shared re2-v1 validation rejects incompatible syntax and conflicting boundaries.
#[test]
fn multiline_validation() {
    for body in [
        "{line_start_pattern: '^BEGIN', line_end_pattern: '^END'}",
        "{line_start_pattern: ''}",
        "{line_start_pattern: '(?=x)'}",
        "{line_start_pattern: 'a{1001}'}",
        "{line_start_pattern: '\\p{Greek}'}",
    ] {
        assert!(
            compiled(&format!("framing: {{multiline: {body}}}")).is_err(),
            "{body}"
        );
    }
    let text = compiled("framing: {multiline: {line_start_pattern: '^BEGIN'}}").unwrap();
    assert_eq!(
        text.inputs().grouping(),
        Some(super::super::multiline::GroupingMode::Start)
    );
    assert_eq!(
        text.inputs().boundary().unwrap().mode(),
        super::super::multiline_pattern::PatternMode::Text
    );
    let raw =
        compiled("encoding: raw\nframing: {multiline: {line_end_pattern: '\\xFF$'}}").unwrap();
    let mut matcher = raw.inputs().boundary().unwrap().matcher();
    assert!(
        matcher
            .is_match(super::super::multiline_pattern::PatternInput::Raw(&[255]))
            .unwrap()
    );
}

/// Scenario: Logical IDs have ambiguous concatenations and explicit IDs vary by case.
/// Guarantees: Automatic IDs are length-delimited and explicit selection preserves exact bytes.
#[test]
fn checkpoint_selection() {
    let a = parsed("")
        .compile("ab", "c", "d", test_path("/work").as_path())
        .unwrap();
    let b = parsed("")
        .compile("a", "bc", "d", test_path("/work").as_path())
        .unwrap();
    assert_ne!(a.inputs().checkpoint_id(), b.inputs().checkpoint_id());
    let same = parsed("limits: {max_open_files: 1}")
        .compile("ab", "c", "d", test_path("/elsewhere").as_path())
        .unwrap();
    assert_eq!(a.inputs().checkpoint_id(), same.inputs().checkpoint_id());
    let explicit = parsed("checkpoint: {id: AppLogs}")
        .compile("new", "key", "names", test_path("/work").as_path())
        .unwrap();
    assert_eq!(explicit.inputs().checkpoint_id(), "AppLogs");
    assert_eq!(explicit.inputs().namespace_component(), "4170704c6f6773");
    for id in ["", "a/b", "a b", "\u{e9}"] {
        let mut config = parsed("");
        config.checkpoint.id = Some(id.into());
        assert!(
            config
                .compile("g", "p", "n", test_path("/work").as_path())
                .is_err()
        );
    }
    let mut config = parsed("");
    config.checkpoint.id = Some("x".repeat(127));
    assert_eq!(
        config
            .compile("g", "p", "n", test_path("/work").as_path())
            .unwrap()
            .inputs()
            .namespace_component()
            .len(),
        254
    );
    let mut config = parsed("");
    config.checkpoint.id = Some("x".repeat(128));
    assert!(
        config
            .compile("g", "p", "n", test_path("/work").as_path())
            .is_err()
    );
}

/// Scenario: Native paths, escaped literal roots and recursive wildcards are matched repeatedly.
/// Guarantees: Separators stay literal, root escapes are removed, and exclusions retain full patterns.
#[cfg(unix)]
#[test]
fn path_policy() {
    let policy = PathPolicy::compile(
        &[r"/logs/a\*/**/*.log".into()],
        &["/alias/private/*.log".into()],
        test_path("/work").as_path(),
    )
    .unwrap();
    assert_eq!(policy.include_roots(), &[test_path("/logs/a*").as_path()]);
    assert!(policy.includes(test_path("/logs/a*/one.log").as_path()));
    assert!(policy.includes(test_path("/logs/a*/sub/one.log").as_path()));
    assert!(!policy.includes(test_path("/logs/abc/one.log").as_path()));
    assert!(policy.may_descend(test_path("/logs").as_path()));
    assert!(!policy.may_descend(test_path("/logs/other").as_path()));
    assert!(policy.excludes_mapped(0, test_path("secret.log").as_path()));
    assert!(!policy.excludes_mapped(0, test_path("visible.txt").as_path()));
    assert!(!policy.excludes_mapped(0, test_path("nested/secret.log").as_path()));
    assert!(policy.excludes_mapped(5, test_path("secret.log").as_path()));
    assert!(policy.excludes_mapped(0, test_path("../secret.log").as_path()));
    assert!(!policy.excludes_subtree(test_path("/alias/private").as_path()));
    let relative =
        PathPolicy::compile(&["*.log".into()], &[], test_path("/work[x]").as_path()).unwrap();
    assert!(relative.includes(test_path("/work[x]/a.log").as_path()));
    assert!(!relative.includes(test_path("/work[x]/nested/a.log").as_path()));
}

/// Scenario: Unix filenames contain invalid UTF-8 bytes.
/// Guarantees: Include and mapped exclusion matching use original native bytes without replacement.
#[cfg(unix)]
#[test]
fn native_bytes() {
    use std::os::unix::ffi::OsStrExt;
    let path = Path::new(std::ffi::OsStr::from_bytes(b"/logs/\xff.log"));
    let p = PathPolicy::compile(
        &["/logs/?.log".into()],
        &["/alias/?.log".into()],
        test_path("/work").as_path(),
    )
    .unwrap();
    assert!(p.includes(path));
    assert!(p.excludes_mapped(0, Path::new(std::ffi::OsStr::from_bytes(b"\xff.log"))));
}

/// Scenario: Pattern count, individual length and aggregate text exceed their respective limits.
/// Guarantees: Compilation rejects bounded-input violations with setting-specific errors.
#[test]
fn pattern_limits() {
    let error = PathPolicy::compile(
        &["logs/x".into()],
        &["[".into()],
        test_path("/work").as_path(),
    )
    .unwrap_err();
    assert_eq!(error.setting, "exclude[0]");
    for (includes, excludes) in [
        (vec![], vec![]),
        (vec!["".into()], vec![]),
        (vec!["/x".into(); 1025], vec![]),
        (vec!["x".repeat(4097)], vec![]),
        (vec!["x".repeat(4096); 257], vec![]),
        (vec!["logs/x".into()], vec!["[".into()]),
    ] {
        assert!(PathPolicy::compile(&includes, &excludes, test_path("/work").as_path()).is_err());
    }
    let _ = PathPolicy::compile(
        &vec!["logs/x".into(); 1024],
        &[],
        test_path("/work").as_path(),
    )
    .unwrap();
}

/// Scenario: Record admission is evaluated at its exact bound, with and without exported paths.
/// Guarantees: Framing bounds are not added together and metadata is included in batch sizing.
#[test]
fn record_admission() {
    let base = compiled("").unwrap().inputs().resources().record_bytes;
    let _ = compiled(&format!("batch: {{max_bytes: {base}}}")).unwrap();
    assert!(compiled(&format!("batch: {{max_bytes: {}}}", base - 1)).is_err());
    assert!(
        compiled(&format!(
            "include_file_metadata: native\nbatch: {{max_bytes: {base}}}"
        ))
        .is_err()
    );
    assert!(compiled("framing: {max_line_bytes: 16MiB, max_record_bytes: 1KiB}").is_ok());
    assert!(compiled("framing: {max_line_bytes: 1KiB, max_record_bytes: 16MiB, multiline: {line_start_pattern: '^x'}}").is_err());
}

/// Scenario: The engine passes its opaque node JSON into receiver deserialization.
/// Guarantees: Shared units and nested defaults survive the actual node-schema boundary.
#[test]
fn engine_node_schema() {
    let node: otel_arrow_dfe_config::node::NodeUserConfig = serde_yaml::from_str(
        "type: urn:otel:receiver:filelog\nconfig:\n  include: ['/logs/*.log']\n  batch: {max_bytes: 8MiB}\n  reader: {eof_reprobe_interval: 20ms}"
    ).unwrap();
    let mut value = node.config;
    if cfg!(windows) {
        value["include"] = serde_json::json!(["C:/logs/*.log"]);
    }
    let config = Config::from_json(value).unwrap();
    let compiled = config
        .compile("g", "p", "n", test_path("/work").as_path())
        .unwrap();
    assert_eq!(
        compiled.inputs().settings().reader.eof_reprobe_interval,
        Duration::from_millis(20)
    );
    assert_eq!(
        compiled.inputs().settings().batch.max_bytes,
        8 * 1024 * 1024
    );
}

/// Scenario: An exclusion uses a recursive subtree, while the base has glob metacharacters.
/// Guarantees: Explicit recursive exclusions prune safely and literal bases do not broaden roots.
#[test]
fn subtree_and_literal_base() {
    let policy = PathPolicy::compile(
        &["*.log".into()],
        &["private/**".into()],
        test_path("/work[x]").as_path(),
    )
    .unwrap();
    assert_eq!(policy.include_roots(), &[test_path("/work[x]").as_path()]);
    assert_eq!(
        policy.exclude_roots(),
        &[test_path("/work[x]/private").as_path()]
    );
    assert!(policy.excludes_subtree(test_path("/work[x]/private").as_path()));
    assert!(policy.excludes_subtree(test_path("/work[x]/private/deeper").as_path()));
    assert!(!policy.excludes_subtree(test_path("/work[x]").as_path()));
    assert!(policy.excludes_mapped(0, test_path("secret").as_path()));
}

/// Scenario: Checkpoint thresholds allow products above u64 but cap reachable WAL bytes.
/// Guarantees: The store's saturating-intermediate rule is reused and oversized recovery is rejected.
#[test]
fn checkpoint_resource_bounds() {
    let _ = compiled("checkpoint: {compact_after_transactions: 18446744073709551615}").unwrap();
    let error = compiled("checkpoint: {compact_after_bytes: 2GiB}").unwrap_err();
    assert!(error.reason.contains("ceiling=1073741824"), "{error}");
}

/// Scenario: Runtime durations and allocation sizes approach machine limits.
/// Guarantees: Unrepresentable deadlines and buffer capacities fail before workers start.
#[test]
fn machine_limits() {
    let mut c = parsed("");
    c.checkpoint.sync_interval = Duration::MAX;
    assert!(
        c.compile("g", "p", "n", test_path("/work").as_path())
            .unwrap_err()
            .setting
            .contains("sync_interval")
    );
    let mut c = parsed("");
    c.batch.max_bytes = u64::MAX;
    assert!(
        c.compile("g", "p", "n", test_path("/work").as_path())
            .unwrap_err()
            .setting
            .contains("batch.max_bytes")
    );
    let mut c = parsed("");
    c.framing.force_flush_period = Duration::from_secs(u64::MAX / 1000 + 1);
    c.rotation.rotate_wait = Duration::MAX;
    assert!(
        c.compile("g", "p", "n", test_path("/work").as_path())
            .is_err()
    );
}

/// Scenario: Metadata and path selection change without changing record-boundary policy.
/// Guarantees: The checkpoint compatibility digest tracks framing rather than export settings.
#[test]
fn compatibility_profile() {
    let original = compiled("").unwrap();
    let metadata = compiled("include_file_metadata: native").unwrap();
    assert_eq!(
        original.inputs().profile().digest().unwrap(),
        metadata.inputs().profile().digest().unwrap()
    );
    let changed = compiled("framing: {max_record_bytes: 2MiB}").unwrap();
    assert_ne!(
        original.inputs().profile().digest().unwrap(),
        changed.inputs().profile().digest().unwrap()
    );
}

/// Scenario: The engine supplies malformed nested values through its JSON node boundary.
/// Guarantees: Parsing errors name the actual setting and retain the underlying reason.
#[test]
fn parse_error_paths() {
    for (field, value) in [
        (
            "framing.max_line_bytes",
            serde_json::json!({"framing": {"max_line_bytes": "oops"}}),
        ),
        (
            "reader.eof_reprobe_interval",
            serde_json::json!({"reader": {"eof_reprobe_interval": "oops"}}),
        ),
        (
            "rotation.on_truncate",
            serde_json::json!({"rotation": {"on_truncate": "oops"}}),
        ),
    ] {
        let error = Config::from_json(value).unwrap_err();
        assert_eq!(error.setting, field);
        assert!(!error.reason.is_empty());
    }
}

/// Scenario: Nonrecursive glob syntax limits which directory depths can contain matches.
/// Guarantees: Flat and literal includes do not scan unrelated child directories.
#[test]
fn traversal_depth() {
    let c = compiled("").unwrap();
    assert!(
        !c.paths()
            .may_descend(test_path("/var/log/private").as_path())
    );
    let base = test_path("/work");
    let p = PathPolicy::compile(&["logs/*/*.log".into()], &[], &base).unwrap();
    assert!(p.may_descend(&base.join("logs/app")));
    assert!(!p.may_descend(&base.join("logs/app/private")));
    let p = PathPolicy::compile(&["logs/{a/b,c}/*.log".into()], &[], &base).unwrap();
    assert!(p.may_descend(&base.join("logs/a/b")));
    assert!(!p.may_descend(&base.join("logs/a/b/c")));
    let p = PathPolicy::compile(&["logs/**/*.log".into()], &[], &base).unwrap();
    assert!(p.may_descend(&base.join("logs/a/b/c")));
}

/// Scenario: A Unix base contains literal backslashes, and a file glob contains escaped stars.
/// Guarantees: Escaping preserves the real native path and does not enable recursive traversal.
#[cfg(unix)]
#[test]
fn literal_backslashes_and_stars() {
    let decorated_base = Path::new("/work[x]/./logs");
    let decorated = PathPolicy::compile(&["*.log".into()], &[], decorated_base).unwrap();
    assert!(decorated.includes(&decorated_base.join("a.log")));
    assert!(PathPolicy::compile(&[r"/logs/\../*.log".into()], &[], Path::new("/work")).is_err());
    let base = Path::new(r"/work\logs");
    let p = PathPolicy::compile(&["*.log".into()], &[], base).unwrap();
    assert!(p.includes(&base.join("a.log")));
    assert!(!p.includes(Path::new("/worklogs/a.log")));
    let p = PathPolicy::compile(&[r"logs/\**.log".into()], &[], Path::new("/work")).unwrap();
    assert!(!p.may_descend(Path::new("/work/logs/private")));
}

/// Scenario: A pattern contains parent traversal or an impossible post-wildcard component.
/// Guarantees: Errors name the configured entry before scanner construction.
#[test]
fn unsupported_components() {
    for pattern in [
        "../logs/*.log",
        "logs/*/../x.log",
        "logs/*/./x.log",
        "logs/*//x.log",
        "logs/",
    ] {
        let err =
            PathPolicy::compile(&[pattern.into()], &[], test_path("/work").as_path()).unwrap_err();
        assert_eq!(err.setting, "include[0]", "{pattern}: {err}");
    }
}

/// Scenario: A multiline configuration retains path programs, boundary programs and scratch.
/// Guarantees: Known matching charges and source remainder are included exactly once in their sums.
#[test]
fn resource_composition() {
    let c = compiled("framing: {multiline: {line_start_pattern: '^START'}}").unwrap();
    let m = c.inputs().resources();
    assert!(m.path_matcher_bytes > 0);
    assert!(m.multiline_program_bytes > 0);
    assert!(m.multiline_scratch_bytes > 0);
    assert_eq!(
        m.matching_bytes,
        m.path_matcher_bytes + m.multiline_program_bytes + m.multiline_scratch_bytes
    );
    assert_eq!(
        m.partial_state_floor_bytes,
        m.framer_payload_bytes + c.inputs().settings().limits.max_read_bytes_per_turn
    );
    assert_eq!(
        m.batch_and_carry_over_bytes,
        c.inputs().settings().batch.max_bytes + m.record_bytes
    );
    let newline = compiled("").unwrap();
    let old_floor = newline.inputs().resources().framer_payload_bytes;
    let error = compiled(&format!("limits: {{max_partial_state_bytes: {old_floor}}}")).unwrap_err();
    let exact_floor = newline.inputs().resources().partial_state_floor_bytes;
    let _ = compiled(&format!(
        "limits: {{max_partial_state_bytes: {exact_floor}}}"
    ))
    .unwrap();
    assert!(error.reason.contains("limits.max_read_bytes_per_turn"));
    assert!(error.reason.contains("framing.max_line_bytes"));
}

/// Scenario: Source lengths reach exactly the per-pattern and aggregate configured bounds.
/// Guarantees: Inclusive text bounds are accepted rather than accidentally made exclusive.
#[test]
fn exact_pattern_limits() {
    let patterns: Vec<_> = (0..256)
        .map(|index| {
            let mut pattern = format!("logs/{index:03}/");
            pattern.push_str(&"a".repeat(4096 - pattern.len()));
            pattern
        })
        .collect();
    let p = PathPolicy::compile(&patterns, &[], test_path("/work").as_path()).unwrap();
    assert_eq!(p.include_roots().len(), 256);
}

/// Scenario: Default compatibility inputs are serialized by the checkpoint codec.
/// Guarantees: The configuration matches the normative default profile digest vector.
#[test]
fn default_profile_vector() {
    assert_eq!(
        hex::encode(compiled("").unwrap().inputs().profile().digest().unwrap()),
        "6dd95711093f4fcffe680091011b1227aabfe8029010413aee7336c59304e393"
    );
}

/// Scenario: Windows matching receives drive-qualified patterns and native separators.
/// Guarantees: Portable fixtures exercise real Windows path matching rather than Unix-rooted strings.
#[cfg(windows)]
#[test]
fn windows_paths() {
    let p = PathPolicy::compile(
        &[r"C:\logs\*.log".into()],
        &[r"C:\logs\private*".into()],
        Path::new(r"C:\work"),
    )
    .unwrap();
    assert!(p.includes(Path::new(r"C:\logs\app.log")));
    assert!(p.excludes(Path::new(r"C:\logs\private.log")));
    assert!(!p.may_descend(Path::new(r"C:\logs\private")));
    let p = PathPolicy::compile(
        &[r"C:\logs\**".into()],
        &[r"C:\logs\private\**".into()],
        Path::new(r"C:\work"),
    )
    .unwrap();
    assert!(p.excludes_subtree(Path::new(r"C:\logs\private")));
    assert!(p.excludes_subtree(Path::new(r"C:\logs\private\sub")));
    let root = PathPolicy::compile(&["*.log".into()], &[], Path::new(r"C:\")).unwrap();
    assert!(root.includes(Path::new(r"C:\app.log")));
    assert!(PathPolicy::compile(&[r"\logs\*.log".into()], &[], Path::new(r"C:\work")).is_err());
}

/// Scenario: The identity model equals its specified 1 GiB ceiling and then exceeds it.
/// Guarantees: The boundary is inclusive and excess diagnostics report inputs and the computed value.
#[test]
fn exact_identity_ceiling() {
    let c = compiled(
        "limits: {max_tracked_files: 6248, max_pending_candidates: 24945, max_open_files: 512}",
    )
    .unwrap();
    assert_eq!(
        c.inputs().resources().identity_reconciliation_bytes,
        1 << 30
    );
    let e = compiled(
        "limits: {max_tracked_files: 6248, max_pending_candidates: 24946, max_open_files: 512}",
    )
    .unwrap_err();
    assert!(e.reason.contains("ceiling=1073741824"));
    assert!(e.reason.contains("max_pending_candidates=24946"));
}

/// Scenario: Scanner root capacity exactly equals a derived native root length.
/// Guarantees: Configuration identifies the root index and requires space for the terminating byte.
#[cfg(unix)]
#[test]
fn scanner_root_capacity() {
    let c = compiled("").unwrap();
    let len = c.paths().include_roots()[0].as_os_str().len();
    assert_eq!(
        c.paths().validate_root_lengths(len).unwrap_err().setting,
        "include[0]"
    );
    c.paths().validate_root_lengths(len + 1).unwrap();
}

/// Scenario: The explicit-cache glob engine matches syntax accepted by globset on Unix native paths.
/// Guarantees: Literals, classes, alternation, recursion, newlines and invalid UTF-8 retain reference semantics.
#[cfg(unix)]
#[test]
fn glob_reference_equivalence() {
    use std::os::unix::ffi::OsStrExt;
    let candidates = [
        Path::new("/logs/a.log"),
        Path::new("/logs/a"),
        Path::new("/logs/b"),
        Path::new("/logs/sub/a.log"),
        Path::new("/logs/a\n.log"),
        Path::new("/logs/.hidden.log"),
        Path::new("/logs/*abc.log"),
        Path::new("/logs/a/b/c.log"),
        Path::new("/logs/\u{e9}clair.log"),
        Path::new("/logs/\u{e9}"),
        Path::new(std::ffi::OsStr::from_bytes(b"/logs/\xc3")),
        Path::new(std::ffi::OsStr::from_bytes(b"/logs/\xa9")),
        Path::new(std::ffi::OsStr::from_bytes(b"/logs/\xff.log")),
    ];
    for pattern in [
        "/logs/*.log",
        "/logs/?.log",
        "/logs/**/*.log",
        "/logs/{a,sub}/**/*.log",
        r"/logs/\**.log",
        "/logs/[a-z]*.log",
        "/logs/[a/../b]",
        "/logs/\u{e9}*",
        "/logs/[\u{e9}]",
    ] {
        let reference = globset::GlobBuilder::new(pattern)
            .literal_separator(true)
            .backslash_escape(true)
            .build()
            .unwrap()
            .compile_matcher();
        let policy =
            PathPolicy::compile(&[pattern.into()], &[pattern.into()], Path::new("/work")).unwrap();
        for candidate in candidates {
            assert_eq!(
                policy.includes(candidate),
                reference.is_match(candidate),
                "{pattern} {candidate:?}"
            );
            assert_eq!(
                policy.excludes(candidate),
                reference.is_match(candidate),
                "{pattern} {candidate:?}"
            );
            if let Ok(suffix) = candidate.strip_prefix(&policy.exclude_roots()[0]) {
                assert_eq!(
                    policy.excludes_mapped(0, suffix),
                    reference.is_match(candidate),
                    "mapped {pattern} {candidate:?}"
                );
            }
        }
    }
}

/// Scenario: Intermediate directory names cannot satisfy a multi-component include.
/// Guarantees: Unrelated directories are pruned before their failures can invalidate discovery.
#[test]
fn directory_names_control_descent() {
    let base = test_path("/work");
    let p = PathPolicy::compile(&["logs/*/app/*.log".into()], &[], &base).unwrap();
    assert!(p.may_descend(&base.join("logs/service")));
    assert!(p.may_descend(&base.join("logs/service/app")));
    assert!(!p.may_descend(&base.join("logs/service/private")));
    let p = PathPolicy::compile(&["logs/{web,db}/**/*.log".into()], &[], &base).unwrap();
    assert!(!p.may_descend(&base.join("logs/private")));
    assert!(p.may_descend(&base.join("logs/web/any/depth")));
    let p = PathPolicy::compile(&["logs/{a,b/c}/*.log".into()], &[], &base).unwrap();
    assert!(p.may_descend(&base.join("logs/b/c")));
    assert!(!p.may_descend(&base.join("logs/private")));
    assert!(!p.may_descend(&base.join("logs/a/private")));
    let p = PathPolicy::compile(&["logs/{a/b,c}/*.log".into()], &[], &base).unwrap();
    assert!(p.may_descend(&base.join("logs/a/b")));
    assert!(p.may_descend(&base.join("logs/c")));
    assert!(!p.may_descend(&base.join("logs/private")));
    assert!(!p.may_descend(&base.join("logs/c/private")));
}

/// Scenario: Different canonical exclusion roots require different suffix patterns.
/// Guarantees: Indexed matching selects the requested rule rather than any rule in the shared program.
#[test]
fn indexed_exclusions_preserve_root_identity() {
    let p = PathPolicy::compile(
        &["logs/**".into()],
        &["a/*.log".into(), "b/*.txt".into()],
        test_path("/work").as_path(),
    )
    .unwrap();
    assert!(p.excludes_mapped(0, Path::new("one.log")));
    assert!(!p.excludes_mapped(0, Path::new("one.txt")));
    assert!(p.excludes_mapped(1, Path::new("one.txt")));
    assert!(!p.excludes_mapped(1, Path::new("one.log")));
}

/// Scenario: Discovery takes ownership of the path policy while startup retains validated inputs.
/// Guarantees: Settings, framing, resources, and store configuration remain usable after the policy moves to a worker.
#[test]
fn configuration_parts_have_independent_owners() {
    let config = compiled("framing: {multiline: {line_start_pattern: '^START'}}").unwrap();
    let path = test_path("/var/log/app.log");
    let (inputs, policy) = config.into_parts();
    let worker = std::thread::spawn(move || policy.includes(&path));
    assert_eq!(inputs.settings().limits.max_tracked_files, 10000);
    assert_eq!(inputs.line().encoding, Encoding::Utf8);
    assert_eq!(
        inputs.grouping(),
        Some(super::super::multiline::GroupingMode::Start)
    );
    assert!(!inputs.checkpoint_id().is_empty());
    assert!(!inputs.namespace_component().is_empty());
    assert!(inputs.resources().matching_bytes > 0);
    assert!(inputs.profile().digest().is_ok());
    assert!(
        inputs
            .boundary()
            .unwrap()
            .matcher()
            .is_match(super::super::multiline_pattern::PatternInput::Text(
                "START record"
            ))
            .unwrap()
    );
    #[cfg(target_os = "linux")]
    assert_eq!(
        inputs.store_config().unwrap().replay.max_tracked_files,
        10000
    );
    assert!(worker.join().unwrap());
}

/// Scenario: A configuration has a cheap semantic error as well as invalid glob syntax.
/// Guarantees: Checkpoint, framing, and multiline conflicts are rejected before compiling path matchers.
#[test]
fn cheap_validation_precedes_path_compilation() {
    for (settings, field) in [
        ("checkpoint: {id: 'bad/id'}", "checkpoint.id"),
        (
            "framing: {max_line_bytes: 1}",
            "framing.max_line_bytes/max_record_bytes",
        ),
        (
            "framing: {multiline: {line_start_pattern: '^a', line_end_pattern: '$'}}",
            "framing.multiline",
        ),
    ] {
        let mut config = parsed(settings);
        config.include = vec!["[".into()];
        let error = config
            .compile("g", "p", "n", test_path("/work").as_path())
            .unwrap_err();
        assert_eq!(error.setting, field);
    }
}

/// Scenario: Accepted files use classes, alternatives, escapes, and recursive directories.
/// Guarantees: Directory pruning never hides an accepted file in the generated candidate set.
#[cfg(unix)]
#[test]
fn accepted_files_have_traversable_ancestors() {
    let base = Path::new("/work");
    for pattern in [
        "logs/[ab]*/*.log",
        "logs/[!ab]*/*.log",
        r"logs/\*/*.log",
        "logs/{a/b,c}/*.log",
        "logs/{a,b/c}/[!x]*/*.log",
        "logs/[a/]*/*.log",
        "logs/**/app/*.log",
    ] {
        let policy = PathPolicy::compile(&[pattern.into()], &[], base).unwrap();
        let root = &policy.include_roots()[0];
        let names = ["a", "b", "c", "app", "private", "*", "abc", "x"];
        let mut candidates = vec![base.join("logs")];
        for _ in 0..3 {
            let next: Vec<_> = candidates
                .iter()
                .flat_map(|parent| names.iter().map(move |name| parent.join(name)))
                .collect();
            for directory in &next {
                let file = directory.join("one.log");
                if policy.includes(&file) {
                    for ancestor in directory.ancestors().take_while(|p| p.starts_with(root)) {
                        assert!(
                            policy.may_descend(ancestor),
                            "{pattern}: accepted {file:?}, pruned {ancestor:?}"
                        );
                    }
                }
            }
            candidates = next;
        }
    }
}

/// Scenario: The embedding integration has no explicit lexical base.
/// Guarantees: Absolute patterns compile, while relative includes and excludes fail without a CWD fallback.
#[test]
fn missing_base_requires_absolute_patterns() {
    let config = parsed("");
    assert!(config.compile("g", "p", "n", Path::new("")).is_ok());
    let mut config = parsed("");
    config.include = vec!["logs/*.log".into()];
    let error = config.compile("g", "p", "n", Path::new("")).unwrap_err();
    assert_eq!(error.setting, "include[0]");
    let mut config = parsed("");
    config.exclude = vec!["private/**".into()];
    let error = config.compile("g", "p", "n", Path::new("")).unwrap_err();
    assert_eq!(error.setting, "exclude[0]");
}

/// Scenario: Several include/exclude patterns overlap and contain an exact duplicate.
/// Guarantees: Set matching and deduplication preserve globset's union semantics for matching and nonmatching paths.
#[cfg(unix)]
#[test]
fn glob_set_reference_equivalence() {
    let patterns: Vec<String> = [
        "/logs/app/*.log",
        "/logs/{db,worker}/**/*.txt",
        "/logs/\u{e9}*",
        "/logs/app/*.log",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    let mut builder = globset::GlobSetBuilder::new();
    for pattern in &patterns {
        let _ = builder.add(
            globset::GlobBuilder::new(pattern)
                .literal_separator(true)
                .backslash_escape(true)
                .build()
                .unwrap(),
        );
    }
    let reference = builder.build().unwrap();
    let policy = PathPolicy::compile(&patterns, &patterns, Path::new("/work")).unwrap();
    for (candidate, expected) in [
        ("/logs/app/one.log", true),
        ("/logs/db/one.txt", true),
        ("/logs/worker/sub/one.txt", true),
        ("/logs/\u{e9}clair", true),
        ("/logs/app/sub/one.log", false),
        ("/logs/db/one.log", false),
        ("/logs/other/one.txt", false),
        ("/elsewhere/app/one.log", false),
    ] {
        let path = Path::new(candidate);
        assert_eq!(reference.is_match(path), expected, "{candidate}");
        assert_eq!(policy.includes(path), expected, "include {candidate}");
        assert_eq!(policy.excludes(path), expected, "exclude {candidate}");
    }
}

/// Scenario: A Windows directory contains an unpaired UTF-16 surrogate.
/// Guarantees: Both directory-rule forms conservatively permit traversal when matching is unrepresentable.
#[cfg(windows)]
#[test]
fn windows_unrepresentable_directory_traversal() {
    use std::{ffi::OsString, os::windows::ffi::OsStringExt};
    let base = Path::new(r"C:\work");
    let directory = base.join("logs").join(OsString::from_wide(&[0xD800]));
    assert!(directory.to_str().is_none());
    for pattern in ["logs/{a/b,c}/*.log", "logs/*/app/*.log"] {
        let policy = PathPolicy::compile(&[pattern.into()], &[], base).unwrap();
        assert!(policy.may_descend(&directory), "{pattern}");
        // This hardening does not claim support for matching native Windows
        // paths that the current byte adapter cannot represent.
        assert!(!policy.includes(&directory.join("one.log")));
    }
}

/// Scenario: Unix recursive exclusions use escaped separators or literal backslashes before separators.
/// Guarantees: Valid patterns compile and subtree pruning preserves literal backslashes in directory names.
#[cfg(unix)]
#[test]
fn escaped_recursive_exclusion_separators() {
    for (pattern, root) in [
        (r"/logs/private\/**", "/logs/private"),
        (r"/logs/private\\/**", r"/logs/private\"),
        (r"/logs/private\\\/**", r"/logs/private\"),
    ] {
        let policy =
            PathPolicy::compile(&["/logs/**".into()], &[pattern.into()], Path::new("/work"))
                .unwrap();
        let root = Path::new(root);
        assert!(policy.excludes_subtree(root), "{pattern}");
        assert!(policy.excludes(&root.join("sub/one.log")), "{pattern}");
        assert!(
            !policy.excludes_subtree(Path::new("/logs/private-other")),
            "{pattern}"
        );
    }
}

/// Scenario: Long literal and wildcard alternatives fit the pattern size and configured traversal limits.
/// Guarantees: Generated directory rules accept 300 and 1024 components without excessive nesting or lost matches.
#[test]
fn deep_separator_alternative() {
    for depth in [300, 1024] {
        let deep = vec!["a"; depth].join("/");
        for component in ["a", "a*"] {
            let alternative = vec![component; depth].join("/");
            let pattern = format!("logs/{{{alternative},b}}/*.log");
            assert!(pattern.len() < 4096);
            let mut config = parsed("max_recursion_depth: 1024");
            config.include = vec![pattern];
            let base = test_path("/work");
            let compiled = config.compile("g", "p", "n", &base).unwrap();
            let policy = compiled.paths();
            let root = base.join("logs");
            let file = root.join(&deep).join("one.log");
            assert!(policy.includes(&file));
            for ancestor in file
                .parent()
                .unwrap()
                .ancestors()
                .take_while(|p| p.starts_with(&root))
            {
                assert!(policy.may_descend(ancestor), "{ancestor:?}");
            }
            assert!(policy.includes(&root.join("b/one.log")));
            assert!(!policy.may_descend(&root.join("unrelated")));
            assert!(!policy.may_descend(&root.join("a/not-a")));
        }
    }
}

/// Scenario: A Unix fixed include prefix ends in an escaped separator.
/// Guarantees: Discovery receives the complete literal root and only the file glob as its suffix.
#[cfg(unix)]
#[test]
fn escaped_include_separator_keeps_fixed_root() {
    for (pattern, root) in [
        (r"/logs/private\/*.log", "/logs/private/"),
        (r"/logs/private\\\/*.log", "/logs/private\\/"),
    ] {
        let (prefix, suffix) = paths::prefix(pattern);
        assert_eq!(prefix, Path::new(root));
        assert_eq!(suffix, "*.log");
        let policy = PathPolicy::compile(&[pattern.into()], &[], Path::new("/work")).unwrap();
        assert_eq!(policy.include_roots(), &[Path::new(root)]);
        assert!(policy.includes(&Path::new(root).join("app.log")));
        assert!(!policy.may_descend(&Path::new(root).join("child")));
    }
}
