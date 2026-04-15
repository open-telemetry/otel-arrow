// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Create and run a multi-core pipeline

use clap::Parser;
use otap_df_config::config_provider::{ConfigFormat, resolve_config};
use otap_df_config::engine::OtelDataflowSpec;
use otap_df_config::policy::{CoreAllocation, CoreRange};
// Keep this side-effect import so the crate is linked and its `linkme`
// distributed-slice registrations (contrib nodes) are visible
// in `OTAP_PIPELINE_FACTORY` at runtime.
use otap_df_contrib_nodes as _;
use otap_df_controller::Controller;
use otap_df_controller::startup;
// Keep this side-effect import so the crate is linked and its `linkme`
// distributed-slice registrations (core nodes) are visible
// in `OTAP_PIPELINE_FACTORY` at runtime.
use cfg_if::cfg_if;
use otap_df_core_nodes as _;
use otap_df_otap::OTAP_PIPELINE_FACTORY;
/// Project license text (Apache-2.0), embedded at compile time.
const LICENSE_TEXT: &str = include_str!("../LICENSE");

/// Third-party notices, embedded at compile time from the repository root.
const THIRD_PARTY_NOTICES: &str = include_str!("../../../THIRD_PARTY_NOTICES.txt");

fn memory_allocator_name() -> &'static str {
    if cfg!(feature = "mimalloc") {
        "mimalloc"
    } else if cfg!(all(feature = "jemalloc", not(windows))) {
        "jemalloc"
    } else {
        "system"
    }
}

// -----------------------------------------------------------------------------
// Feature guard: jemalloc + mimalloc + dhat-heap any two together should fail.
// -----------------------------------------------------------------------------
#[cfg(all(
    not(any(test, doc)),
    not(clippy),
    any(
        all(feature = "dhat-heap", feature = "mimalloc"),
        all(feature = "dhat-heap", feature = "jemalloc"),
        all(feature = "jemalloc", feature = "mimalloc"),
    )
))]
compile_error!(
    "Allocator features are mutually exclusive. Enable only one allocator: `dhat-heap`, `mimalloc`, `jemalloc`. \
    Example: \
        (mimalloc): cargo build --release --no-default-features --features mimalloc. \
        (jemalloc): cargo build --release --no-default-features --features jemalloc. \
        (dhat): cargo build --profile profiling --no-default-features --features dhat-heap."
);

#[cfg(feature = "dhat-heap")]
use {
    dhat::Profiler,
    std::sync::{LazyLock, Mutex},
};

#[cfg(all(not(clippy), feature = "mimalloc"))]
use mimalloc::MiMalloc;

#[cfg(all(not(clippy), not(windows), feature = "jemalloc"))]
use tikv_jemallocator::Jemalloc;

// -----------------------------------------------------------------------------
// Global allocator selection.
// -----------------------------------------------------------------------------
cfg_if! {
    // dhat (profiling) — wins everywhere when enabled
    if #[cfg(all(not(tarpaulin_include), feature = "dhat-heap"))] {
        #[global_allocator]
        static GLOBAL: dhat::Alloc = dhat::Alloc;
        static DHAT_PROFILER: LazyLock<Mutex<Option<Profiler>>> = LazyLock::new(|| Mutex::new(None));

        fn dhat_start() {
                let mut profiler = DHAT_PROFILER.lock().unwrap();
                *profiler = Some(dhat::Profiler::new_heap());
        }

        fn dhat_finish() {
                let mut profiler = DHAT_PROFILER.lock().unwrap();
                let _ = profiler.take();
        }

    // Windows default: mimalloc
    } else if #[cfg(feature = "mimalloc")] {
        #[global_allocator]
        static GLOBAL: MiMalloc = MiMalloc;

    // Linux default: jemalloc
    } else if #[cfg(all(not(windows), feature = "jemalloc"))] {
        #[global_allocator]
        static GLOBAL: Jemalloc = Jemalloc;
    }
}

// Crypto provider features are mutually exclusive.
// The `not(any(test, doc))` and `not(clippy)` guards mirror the jemalloc/mimalloc
// pattern so that `cargo test --all-features` (used in CI) does not fail.
// When all features are enabled (e.g. --all-features), crypto.rs uses a
// priority order (ring > aws-lc > openssl > symcrypt) so the binary still works.
#[cfg(all(
    feature = "crypto-ring",
    feature = "crypto-aws-lc",
    not(any(test, doc)),
    not(clippy)
))]
compile_error!(
    "Features `crypto-ring` and `crypto-aws-lc` are mutually exclusive. \
     Use --no-default-features to disable the default crypto provider, then enable exactly one."
);
#[cfg(all(
    feature = "crypto-ring",
    feature = "crypto-symcrypt",
    not(any(test, doc)),
    not(clippy)
))]
compile_error!(
    "Features `crypto-ring` and `crypto-symcrypt` are mutually exclusive. \
     Use --no-default-features to disable the default crypto provider, then enable exactly one."
);
#[cfg(all(
    feature = "crypto-ring",
    feature = "crypto-openssl",
    not(any(test, doc)),
    not(clippy)
))]
compile_error!(
    "Features `crypto-ring` and `crypto-openssl` are mutually exclusive. \
     Use --no-default-features to disable the default crypto provider, then enable exactly one."
);
#[cfg(all(
    feature = "crypto-aws-lc",
    feature = "crypto-symcrypt",
    not(any(test, doc)),
    not(clippy)
))]
compile_error!(
    "Features `crypto-aws-lc` and `crypto-symcrypt` are mutually exclusive. \
     Use --no-default-features to disable the default crypto provider, then enable exactly one."
);
#[cfg(all(
    feature = "crypto-aws-lc",
    feature = "crypto-openssl",
    not(any(test, doc)),
    not(clippy)
))]
compile_error!(
    "Features `crypto-aws-lc` and `crypto-openssl` are mutually exclusive. \
     Use --no-default-features to disable the default crypto provider, then enable exactly one."
);
#[cfg(all(
    feature = "crypto-symcrypt",
    feature = "crypto-openssl",
    not(any(test, doc)),
    not(clippy)
))]
compile_error!(
    "Features `crypto-symcrypt` and `crypto-openssl` are mutually exclusive. \
     Use --no-default-features to disable the default crypto provider, then enable exactly one."
);

#[cfg(all(
    feature = "crypto-symcrypt",
    not(any(target_os = "linux", target_os = "windows")),
    not(any(test, doc)),
    not(clippy)
))]
compile_error!(
    "Feature `crypto-symcrypt` is only supported on Linux and Windows targets. \
     Use a different crypto provider on this platform (e.g., crypto-ring)."
);

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about = None,
    after_help = startup::system_info(&OTAP_PIPELINE_FACTORY, memory_allocator_name()),
    after_long_help = concat!(
        "EXAMPLES:\n",
        "  ", env!("CARGO_BIN_NAME"), " --config file:/etc/config.yaml\n",
        "  ", env!("CARGO_BIN_NAME"), " --config env:MY_CONFIG_VAR\n",
        "  ", env!("CARGO_BIN_NAME"), " --config /path/to/config.yaml\n",
    )
)]
struct Args {
    /// Configuration URI (file:/path, env:VAR, or bare path). If omitted, config.yaml in the current directory is tried.
    #[arg(short = 'c', long, value_name = "URI")]
    config: Option<String>,

    /// Number of cores to use (0 for all available cores)
    #[arg(long, conflicts_with = "core_id_range")]
    num_cores: Option<usize>,

    /// Inclusive range of CPU core IDs to pin threads to (e.g. "0-3", "0..3,5", "0..=3,6-7")
    #[arg(long, value_name = "START..END", value_parser = parse_core_id_allocation, conflicts_with = "num_cores")]
    core_id_range: Option<CoreAllocation>,

    /// Address to bind the HTTP admin server to (e.g., "127.0.0.1:8080", "0.0.0.0:8080")
    #[arg(long)]
    http_admin_bind: Option<String>,

    /// Validate the provided configuration and exit without starting the engine.
    ///
    /// Checks performed:
    /// - Configuration file parsing (YAML/JSON schema)
    /// - Structural validation (version, policies, connections, node references)
    /// - Component existence (every node URN maps to a registered component in this binary)
    /// - Component-specific config validation (when supported by the component)
    #[arg(long)]
    validate_and_exit: bool,

    /// Print the project license (Apache-2.0) and third-party notices, then exit.
    #[arg(long)]
    license: bool,
}

fn parse_core_id_allocation(s: &str) -> Result<CoreAllocation, String> {
    // Accept format (EBNF):
    //  S -> digit | CoreRange | S,",",S
    //  CoreRange -> digit,"..",digit | digit,"..=",digit | digit,"-",digit
    //  digit -> [0-9]+
    Ok(CoreAllocation::core_set(
        s.split(',')
            .map(|part| {
                part.trim()
                    .parse::<usize>()
                    // A single ID is a range with the same start and end.
                    .map(|n| CoreRange { start: n, end: n })
                    .or_else(|_| parse_core_id_range(part))
            })
            .collect::<Result<Vec<CoreRange>, String>>()?,
    ))
}

fn parse_core_id_range(s: &str) -> Result<CoreRange, String> {
    // Accept formats: "a..=b", "a..b", "a-b"
    let normalized = s.replace("..=", "-").replace("..", "-");
    let mut parts: std::str::Split<'_, char> = normalized.split('-');
    let start: usize = parts
        .next()
        .ok_or_else(|| "missing start of core id range".to_string())?
        .trim()
        .parse::<usize>()
        .map_err(|_| "invalid start (expected unsigned integer)".to_string())?;
    let end = parts
        .next()
        .ok_or_else(|| "missing end of core id range".to_string())?
        .trim()
        .parse::<usize>()
        .map_err(|_| "invalid end (expected unsigned integer)".to_string())?;
    if parts.next().is_some() {
        return Err("unexpected extra data after end of range".to_string());
    }
    Ok(CoreRange { start, end })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(all(not(tarpaulin_include), feature = "dhat-heap"))]
    {
        dhat_start();
    }
    // Install the rustls crypto provider selected by the crypto-* feature flag.
    // This must happen before any TLS connections (reqwest, tonic, etc.).
    otap_df_otap::crypto::install_crypto_provider()
        .map_err(|e| format!("Failed to install rustls crypto provider: {e}"))?;

    let Args {
        config,
        num_cores,
        core_id_range,
        http_admin_bind,
        validate_and_exit,
        license,
    } = Args::parse();

    if license {
        println!("{LICENSE_TEXT}");
        println!("\n--- Third-Party Notices ---\n");
        println!("{THIRD_PARTY_NOTICES}");
        std::process::exit(0);
    }

    println!(
        "{}",
        startup::system_info(&OTAP_PIPELINE_FACTORY, memory_allocator_name())
    );

    let resolved = resolve_config(config.as_deref())?;
    let mut engine_cfg = match resolved.format {
        ConfigFormat::Json => OtelDataflowSpec::from_json(&resolved.content)?,
        ConfigFormat::Yaml => OtelDataflowSpec::from_yaml(&resolved.content)?,
    };
    startup::apply_cli_overrides(&mut engine_cfg, num_cores, core_id_range, http_admin_bind);

    startup::validate_engine_components(&engine_cfg, &OTAP_PIPELINE_FACTORY)?;

    if validate_and_exit {
        println!("Configuration '{}' is valid.", resolved.source);
        std::process::exit(0);
    }

    let controller = Controller::new(&OTAP_PIPELINE_FACTORY);
    let result = controller.run_forever(engine_cfg);
    #[cfg(all(not(tarpaulin_include), feature = "dhat-heap"))]
    {
        dhat_finish();
    }

    match result {
        Ok(_) => {
            println!("Pipeline run successfully");
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("Pipeline failed to run: {e}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_core_range_ok() {
        assert_eq!(
            parse_core_id_range("0..=4"),
            Ok(CoreRange { start: 0, end: 4 })
        );
        assert_eq!(
            parse_core_id_range("0..4"),
            Ok(CoreRange { start: 0, end: 4 })
        );
        assert_eq!(
            parse_core_id_range("0-4"),
            Ok(CoreRange { start: 0, end: 4 })
        );
    }

    #[test]
    fn parse_core_allocation_ok() {
        assert_eq!(
            parse_core_id_allocation("0..=4,5,6-7"),
            Ok(CoreAllocation::core_set(vec![
                CoreRange { start: 0, end: 4 },
                CoreRange { start: 5, end: 5 },
                CoreRange { start: 6, end: 7 },
            ]))
        );
        assert_eq!(
            parse_core_id_allocation("0..4"),
            Ok(CoreAllocation::core_set(vec![CoreRange {
                start: 0,
                end: 4,
            }]))
        );
    }

    #[test]
    fn parse_core_range_errors() {
        assert_eq!(
            parse_core_id_range(""),
            Err("invalid start (expected unsigned integer)".to_string())
        );
        assert_eq!(
            parse_core_id_range("a..4"),
            Err("invalid start (expected unsigned integer)".to_string())
        );
        assert_eq!(
            parse_core_id_range("-1..4"),
            Err("invalid start (expected unsigned integer)".to_string())
        );
        assert_eq!(
            parse_core_id_range("1.."),
            Err("invalid end (expected unsigned integer)".to_string())
        );
        assert_eq!(
            parse_core_id_range("1..a"),
            Err("invalid end (expected unsigned integer)".to_string())
        );
        assert_eq!(
            parse_core_id_range("1..2a"),
            Err("invalid end (expected unsigned integer)".to_string())
        );
    }

    #[test]
    fn parse_validate_and_exit_flag() {
        let args = Args::parse_from([
            "df_engine",
            "--config",
            "config.yaml",
            "--validate-and-exit",
        ]);
        assert!(args.validate_and_exit);
        assert_eq!(args.config.as_deref(), Some("config.yaml"));
    }

    #[test]
    fn config_is_optional() {
        let args = Args::parse_from(["df_engine", "--validate-and-exit"]);
        assert!(args.config.is_none());
        assert!(args.validate_and_exit);
    }

    #[test]
    fn validate_unknown_component_rejected() {
        use otap_df_config::pipeline::PipelineConfig;
        use otap_df_config::{PipelineGroupId, PipelineId};

        let pipeline_group_id: PipelineGroupId = "test_group".into();
        let pipeline_id: PipelineId = "test_pipeline".into();
        let yaml = r#"
nodes:
  receiver:
    type: "urn:fake:receiver:unknown"
    config: {}
  exporter:
    type: exporter:noop
    config: {}
connections:
  - from: receiver
    to: exporter
"#;

        let pipeline_cfg =
            PipelineConfig::from_yaml(pipeline_group_id.clone(), pipeline_id.clone(), yaml)
                .expect("pipeline YAML should parse");

        let err = startup::validate_pipeline_components(
            &pipeline_group_id,
            &pipeline_id,
            &pipeline_cfg,
            &OTAP_PIPELINE_FACTORY,
        )
        .expect_err("semantic component validation should fail");
        assert!(err.to_string().contains("Unknown receiver component"));
    }

    #[test]
    fn parse_license_flag() {
        let args = Args::parse_from(["df_engine", "--license"]);
        assert!(args.license);
    }

    #[test]
    fn license_flag_is_false_by_default() {
        let args = Args::parse_from(["df_engine", "--validate-and-exit"]);
        assert!(!args.license);
    }

    #[test]
    #[allow(clippy::const_is_empty)]
    fn license_text_is_embedded() {
        assert!(
            !LICENSE_TEXT.is_empty(),
            "LICENSE should be embedded at compile time"
        );
        assert!(
            LICENSE_TEXT.contains("Apache License"),
            "LICENSE should contain Apache License text"
        );
    }

    #[test]
    #[allow(clippy::const_is_empty)]
    fn third_party_notices_are_embedded() {
        assert!(
            !THIRD_PARTY_NOTICES.is_empty(),
            "THIRD_PARTY_NOTICES should be embedded at compile time"
        );
    }

    #[test]
    fn args_reject_conflicting_core_allocation_flags() {
        let err = Args::try_parse_from([
            "df_engine",
            "--config",
            "config.yaml",
            "--num-cores",
            "2",
            "--core-id-range",
            "0-3",
        ])
        .expect_err("clap should reject conflicting options");
        let msg = err.to_string();
        assert!(msg.contains("--num-cores"));
        assert!(msg.contains("--core-id-range"));
    }

    #[test]
    fn args_accept_num_cores_and_http_admin_bind() {
        let args = Args::try_parse_from([
            "df_engine",
            "--config",
            "config.yaml",
            "--num-cores",
            "2",
            "--http-admin-bind",
            "0.0.0.0:28080",
        ])
        .expect("args should parse");
        assert_eq!(args.num_cores, Some(2));
        assert!(args.core_id_range.is_none());
        assert_eq!(args.http_admin_bind.as_deref(), Some("0.0.0.0:28080"));
    }

    #[test]
    fn args_accept_core_id_range() {
        let args = Args::try_parse_from([
            "df_engine",
            "--config",
            "config.yaml",
            "--core-id-range",
            "1..=3,7",
        ])
        .expect("args should parse");
        assert_eq!(
            args.core_id_range,
            Some(CoreAllocation::core_set(vec![
                CoreRange { start: 1, end: 3 },
                CoreRange { start: 7, end: 7 },
            ]))
        );
        assert_eq!(args.num_cores, None);
    }
}
