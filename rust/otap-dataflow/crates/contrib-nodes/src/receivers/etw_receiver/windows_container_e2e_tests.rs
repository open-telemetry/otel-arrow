// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Windows-only end-to-end test for collecting ETW events from
//! process-isolated Windows containers (`enable_in_containers`).
//!
//! The test starts a real ETW session through the receiver on the host, then
//! runs this same test binary inside a process-isolated Nano Server container.
//! Inside the container the test re-enters in emitter mode (selected by the
//! `OTAP_ETW_CONTAINER_EMITTER_RUN_ID` environment variable) and writes events
//! from:
//!
//! - a TraceLogging static provider (`tracelogging`),
//! - a TraceLogging dynamic provider (`tracelogging_dynamic`) subscribed at
//!   `level: information`, which also writes verbose events that the level
//!   filter must reject,
//! - the `Microsoft-Windows-DNS-Client` manifest provider, triggered by a name
//!   lookup and subscribed with an `event_ids` allow-list of only the "DNS
//!   query is called" event, so the event-ID filter must reject the provider's
//!   other events for the lookup, and
//! - a TraceLogging control provider configured with
//!   `enable_in_containers: false`. The host test process also emits from it,
//!   proving it is subscribed while staying host-only.
//!
//! The binary's directory and a copy of the host's `vcruntime140.dll` are
//! mounted read-only into the container; Nano Server does not ship the VC++
//! runtime.
//!
//! Requirements:
//!
//! - Administrator privileges.
//! - Docker using the Windows container engine, with process isolation
//!   available for the container image (the image must match the host Windows
//!   version). Set `DOCKER_HOST` to use a non-default engine, and
//!   `OTAP_ETW_CONTAINER_IMAGE` to override the image.
//!
//! Run this test in its own process, separately from other ETW end-to-end
//! tests: `one_collect` uses a fixed session GUID and a receiver session lives
//! until its process exits, so two ETW end-to-end tests cannot share a process.
//!
//! ```pwsh
//! cargo test -p otel-arrow-dfe-contrib-nodes --features etw -- --ignored --exact `
//!     receivers::etw_receiver::windows_container_e2e_tests::etw_receiver_collects_events_from_process_isolated_container_end_to_end
//! ```

use super::windows_e2e_tests::{
    AttrSnapshot, cleanup_stale_otap_etw_sessions, collect_attributes, is_process_elevated,
    print_running_etw_sessions, string_column, u16_column,
};
use super::*;
use otel_arrow_dfe_config::node::NodeUserConfig;
use otel_arrow_dfe_engine::receiver::ReceiverWrapper;
use otel_arrow_dfe_engine::testing::receiver::{NotSendValidateContext, TestContext};
use otel_arrow_dfe_engine::testing::{receiver::TestRuntime, test_node, test_pipeline_ctx};
use otel_arrow_dfe_pdata::otap::OtapArrowRecords;
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_dfe_pdata::schema::consts;
use std::collections::BTreeSet;
use std::future::Future;
use std::io::Read;
use std::net::ToSocketAddrs;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};
use tokio::sync::oneshot;
use tokio::time;
use tracelogging_dynamic as tld;

tracelogging::define_provider!(SILO_STATIC_PROVIDER, "OtapEtwE2E.Silo.Static");

/// Must match the name passed to `define_provider!` above.
const SILO_STATIC_PROVIDER_NAME: &str = "OtapEtwE2E.Silo.Static";
/// Must match the event name literal passed to `write_event!` in
/// [`run_container_emitter`].
const STATIC_EVENT_NAME: &str = "SiloStaticEvent";
const DYNAMIC_EVENT_NAME: &str = "SiloDynamicEvent";
/// Written at verbose level, below the dynamic provider's configured
/// `information` level, so the level filter must reject every instance.
const DYNAMIC_VERBOSE_EVENT_NAME: &str = "SiloDynamicVerboseEvent";
const CONTROL_EVENT_NAME: &str = "SiloControlEvent";
const DNS_CLIENT_PROVIDER_NAME: &str = "Microsoft-Windows-DNS-Client";
/// `Microsoft-Windows-DNS-Client` "DNS query is called" event, emitted in the
/// process that performs the lookup. It is the only ID in the provider's
/// `event_ids` allow-list; the same process also logs 3008 ("DNS query is
/// completed") for every lookup, which the filter must reject.
const DNS_QUERY_EVENT_ID: i64 = 3006;
const EVENTS_PER_PROVIDER: u32 = 3;
const ORIGIN_CONTAINER: &str = "container";
const ORIGIN_HOST: &str = "host";

const TEST_FN_NAME: &str =
    "etw_receiver_collects_events_from_process_isolated_container_end_to_end";
const EMITTER_RUN_ID_ENV: &str = "OTAP_ETW_CONTAINER_EMITTER_RUN_ID";
const IMAGE_ENV: &str = "OTAP_ETW_CONTAINER_IMAGE";
/// Must match the host Windows version for process isolation. Coupled to the
/// `windows-latest` runner OS, like `Dockerfile.windows`.
const DEFAULT_IMAGE: &str = "mcr.microsoft.com/windows/nanoserver:ltsc2025@sha256:5c42e42b31816a3a7619cb71b4f14353793a47293e198dbde8fe4fce2c8935cb";
const CONTAINER_BIN_DIR: &str = r"C:\otap-etw-test-bin";
const CONTAINER_VCRT_DIR: &str = r"C:\otap-etw-vcrt";
const STATE_LINE_PREFIX: &str = "OTAP_ETW_SILO_STATE";

/// Upper bound for a registered provider to be enabled by the running host
/// session, inside the container and on the host.
const PROVIDER_ENABLE_TIMEOUT: Duration = Duration::from_secs(30);
/// Deadlines for Docker CLI commands. A command that misses its deadline is
/// killed and the test fails, so a stuck pull or container cannot hang CI.
const DOCKER_QUERY_TIMEOUT: Duration = Duration::from_secs(60);
const DOCKER_PULL_TIMEOUT: Duration = Duration::from_secs(600);
const DOCKER_RUN_TIMEOUT: Duration = Duration::from_secs(180);
const DOCKER_CLEANUP_TIMEOUT: Duration = Duration::from_secs(60);
/// Failure guard for the receiver to deliver every expected container event
/// after the container exits.
const DELIVERY_TIMEOUT: Duration = Duration::from_secs(120);
/// Failure guard for the receiver output channel to close. It exceeds the
/// bounded scenario (container run, cleanup, and delivery wait), so scenario
/// failures are reported before this guard fires.
const VALIDATION_TIMEOUT: Duration = DOCKER_RUN_TIMEOUT
    .saturating_add(DOCKER_CLEANUP_TIMEOUT)
    .saturating_add(DELIVERY_TIMEOUT)
    .saturating_add(Duration::from_secs(120));

/// Scenario: A host ETW receiver subscribes with `enable_in_containers: true`
/// to a TraceLogging static provider, a TraceLogging dynamic provider at
/// `level: information`, and the DNS-Client manifest provider with an
/// `event_ids` allow-list of only event 3006, plus a TraceLogging control
/// provider with `enable_in_containers: false`; this binary emits from all of
/// them inside a process-isolated container, and the host test process also
/// emits from the control provider.
/// Guarantees: Events from every opted-in provider type are collected from the
/// container process with their payload fields decoded; the configured level
/// still filters container events (the dynamic provider's verbose events are
/// never delivered while its informational events are); the `event_ids`
/// allow-list still filters container events (the lookup's event 3006 is
/// delivered and no other DNS-Client event ID for it is); and the control
/// provider is collected from the host but is never enabled or collected inside
/// the container.
#[test]
#[ignore = "requires Administrator privileges, Docker with process-isolated Windows containers, and creates a real ETW kernel session; run explicitly with `-- --ignored`"]
fn etw_receiver_collects_events_from_process_isolated_container_end_to_end() {
    // Re-entry from inside the container: act as the event producer only.
    if let Ok(run_id) = std::env::var(EMITTER_RUN_ID_ENV) {
        run_container_emitter(&run_id);
        return;
    }

    assert!(
        is_process_elevated(),
        "ETW container e2e test requires Administrator privileges. \
         Re-run from an elevated PowerShell."
    );
    ensure_windows_docker_engine();
    let image = std::env::var(IMAGE_ENV).unwrap_or_else(|_| DEFAULT_IMAGE.to_string());
    let _ = run_docker_checked(&["pull".to_string(), image.clone()], DOCKER_PULL_TIMEOUT);

    cleanup_stale_otap_etw_sessions();

    let pid = std::process::id();
    let run_id = format!(
        "{pid}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock is past Unix epoch")
            .as_millis()
    );
    let config_json = serde_json::json!({
        "session_name": format!("OtapEtwE2E-Silo-{pid}"),
        "providers": [
            {
                "name": DNS_CLIENT_PROVIDER_NAME,
                "kind": "manifest",
                "level": "verbose",
                "event_ids": [DNS_QUERY_EVENT_ID],
                "enable_in_containers": true,
            },
            {
                "name": SILO_STATIC_PROVIDER_NAME,
                "kind": "tracelogging",
                "level": "information",
                "enable_in_containers": true,
            },
            {
                "name": dynamic_provider_name(&run_id),
                "kind": "tracelogging",
                "level": "information",
                "enable_in_containers": true,
            },
            {
                "name": control_provider_name(&run_id),
                "kind": "tracelogging",
                "level": "information",
                "enable_in_containers": false,
            },
        ],
    });

    let current_exe = std::env::current_exe().expect("test binary path must be known");
    let bin_dir = current_exe
        .parent()
        .expect("test binary must have a parent directory")
        .to_path_buf();
    let vcrt_dir = stage_vc_runtime(&bin_dir);
    let container_name = format!("otap-etw-e2e-{run_id}");
    let docker_args = container_emitter_args(
        &image,
        &container_name,
        &run_id,
        &current_exe,
        vcrt_dir.path(),
    );

    let (pipeline_ctx, _registry) = test_pipeline_ctx();
    let receiver = EtwReceiver::from_config(pipeline_ctx, &config_json)
        .expect("EtwReceiver::from_config should succeed (Administrator required)");

    let test_runtime = TestRuntime::<OtapPdata>::new();
    let node_config = Arc::new(NodeUserConfig::new_receiver_config(ETW_RECEIVER_URN));
    let receiver_wrapper = ReceiverWrapper::local(
        receiver,
        test_node(test_runtime.config().name.clone()),
        node_config,
        test_runtime.config(),
    );

    let (all_seen_tx, all_seen_rx) = oneshot::channel();
    test_runtime
        .set_receiver(receiver_wrapper)
        .run_test(container_scenario(
            docker_args,
            container_name,
            run_id.clone(),
            all_seen_rx,
        ))
        .run_validation_concurrent(container_validation(run_id, all_seen_tx));
}

fn dynamic_provider_name(run_id: &str) -> String {
    format!("OtapEtwE2E.Silo.Dynamic.{run_id}")
}

fn control_provider_name(run_id: &str) -> String {
    format!("OtapEtwE2E.Silo.Control.{run_id}")
}

fn dns_probe_name(run_id: &str) -> String {
    format!("otap-etw-silo-{run_id}.invalid")
}

// -- Container emitter ---------------------------------------------------

/// Runs inside the container. Registers the providers, reports which ones the
/// host session enabled inside this container, then emits events.
#[allow(clippy::print_stdout)]
fn run_container_emitter(run_id: &str) {
    let dynamic = Box::pin(tld::Provider::new(
        &dynamic_provider_name(run_id),
        &tld::Provider::options(),
    ));
    let control = Box::pin(tld::Provider::new(
        &control_provider_name(run_id),
        &tld::Provider::options(),
    ));
    // SAFETY: both dynamic providers stay pinned until this function returns
    // and unregister on drop. The static provider is unregistered below and
    // on process exit.
    #[allow(unsafe_code)]
    let results = unsafe {
        [
            SILO_STATIC_PROVIDER.register(),
            dynamic.as_ref().register(),
            control.as_ref().register(),
        ]
    };
    assert_eq!(results, [0, 0, 0], "provider registration failed");

    // The host session is already running, so ETW enables opted-in providers
    // as they register. Poll that state with a bounded deadline in case the
    // enable notification arrives asynchronously.
    let opted_in_enabled = || {
        SILO_STATIC_PROVIDER.enabled(tracelogging::Level::Informational, 0)
            && dynamic.enabled(tld::Level::Informational, 0)
    };
    let deadline = Instant::now() + PROVIDER_ENABLE_TIMEOUT;
    while !opted_in_enabled() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(50));
    }
    println!(
        "{STATE_LINE_PREFIX} static_enabled={} dynamic_enabled={} dynamic_verbose_enabled={} \
         control_enabled={}",
        SILO_STATIC_PROVIDER.enabled(tracelogging::Level::Informational, 0),
        dynamic.enabled(tld::Level::Informational, 0),
        dynamic.enabled(tld::Level::Verbose, 0),
        control.enabled(tld::Level::Informational, 0),
    );

    let origin = ORIGIN_CONTAINER;
    let dns_probe = format!("{}:80", dns_probe_name(run_id));
    let mut builder = tld::EventBuilder::new();
    for seq in 0..EVENTS_PER_PROVIDER {
        // Events that must be rejected are written first, so a wrongly
        // delivered one arrives no later than the events the host waits for.
        write_control_event(&mut builder, &control, run_id, seq, origin);
        let _ = builder
            .reset(DYNAMIC_VERBOSE_EVENT_NAME, tld::Level::Verbose, 0x1, 0)
            .add_str8("run_id", run_id.as_bytes(), tld::OutType::Default, 0)
            .add_u32("seq", seq, tld::OutType::Default, 0)
            .add_str8("origin", origin.as_bytes(), tld::OutType::Default, 0)
            .write(&dynamic, None, None);
        let _ = tracelogging::write_event!(
            SILO_STATIC_PROVIDER,
            "SiloStaticEvent", // == STATIC_EVENT_NAME (macro requires a string literal)
            level(Informational),
            keyword(0x1),
            str8("run_id", run_id.as_bytes()),
            u32("seq", &seq),
            str8("origin", origin.as_bytes()),
        );
        let _ = builder
            .reset(DYNAMIC_EVENT_NAME, tld::Level::Informational, 0x1, 0)
            .add_str8("run_id", run_id.as_bytes(), tld::OutType::Default, 0)
            .add_u32("seq", seq, tld::OutType::Default, 0)
            .add_str8("origin", origin.as_bytes(), tld::OutType::Default, 0)
            .write(&dynamic, None, None);
        // The lookup fails (no network), but DNS-Client still emits its
        // manifest events in this process.
        let _ = dns_probe.to_socket_addrs();
    }

    let _ = SILO_STATIC_PROVIDER.unregister();
}

fn write_control_event(
    builder: &mut tld::EventBuilder,
    control: &tld::Provider,
    run_id: &str,
    seq: u32,
    origin: &str,
) {
    let _ = builder
        .reset(CONTROL_EVENT_NAME, tld::Level::Informational, 0x1, 0)
        .add_str8("run_id", run_id.as_bytes(), tld::OutType::Default, 0)
        .add_u32("seq", seq, tld::OutType::Default, 0)
        .add_str8("origin", origin.as_bytes(), tld::OutType::Default, 0)
        .write(control, None, None);
}

/// Emits the control provider's events from the host test process. The
/// provider must be enabled here, proving it is subscribed even though it is
/// not enabled inside the container.
async fn emit_host_control_events(run_id: &str) {
    let control = Box::pin(tld::Provider::new(
        &control_provider_name(run_id),
        &tld::Provider::options(),
    ));
    // SAFETY: the provider stays pinned until this function returns and
    // unregisters on drop, including during unwinding.
    #[allow(unsafe_code)]
    let r = unsafe { control.as_ref().register() };
    assert_eq!(r, 0, "host control Provider::register returned errno {r}");

    // Enablement is external ETW state with no notification here, so poll it
    // against a bounded deadline.
    let deadline = Instant::now() + PROVIDER_ENABLE_TIMEOUT;
    while !control.enabled(tld::Level::Informational, 0) {
        assert!(
            Instant::now() < deadline,
            "control provider was not enabled on the host within {PROVIDER_ENABLE_TIMEOUT:?}; \
             it must be subscribed for host events"
        );
        time::sleep(Duration::from_millis(50)).await;
    }

    let mut builder = tld::EventBuilder::new();
    for seq in 0..EVENTS_PER_PROVIDER {
        write_control_event(&mut builder, &control, run_id, seq, ORIGIN_HOST);
    }
}

// -- Host side: Docker ---------------------------------------------------

fn ensure_windows_docker_engine() {
    let args = ["version", "--format", "{{.Server.Os}}"].map(String::from);
    let output = run_docker(&args, DOCKER_QUERY_TIMEOUT).unwrap_or_else(|e| {
        panic!("{e}\nthis test requires Docker with the Windows container engine")
    });
    let server_os = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert!(
        output.status.success() && server_os == "windows",
        "this test requires Docker with the Windows container engine (server OS: {server_os:?}, \
         stderr: {}). With Docker Desktop, switch to Windows containers, or set DOCKER_HOST \
         to a Windows engine.",
        String::from_utf8_lossy(&output.stderr).trim()
    );
}

/// Runs the Docker CLI with `args`, killing it if it does not exit within
/// `timeout`. Returns an error rather than panicking, so it is safe to call
/// from `Drop` during unwinding.
fn run_docker(args: &[String], timeout: Duration) -> Result<Output, String> {
    let mut child = Command::new("docker")
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to invoke docker {args:?}: {e}"))?;
    let stdout = read_to_end_in_background(child.stdout.take());
    let stderr = read_to_end_in_background(child.stderr.take());

    // Process exit has no completion signal on this thread, so poll it against
    // the deadline.
    let deadline = Instant::now() + timeout;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Ok(status),
            Ok(None) if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                break Err(format!(
                    "docker {args:?} did not exit within {timeout:?} and was killed"
                ));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(100)),
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                break Err(format!("failed to wait for docker {args:?}: {e}"));
            }
        }
    };
    let stdout = stdout.join().unwrap_or_default();
    let stderr = stderr.join().unwrap_or_default();
    match status {
        Ok(status) => Ok(Output {
            status,
            stdout,
            stderr,
        }),
        Err(message) => Err(format!(
            "{message}\n--- stdout ---\n{}\n--- stderr ---\n{}",
            String::from_utf8_lossy(&stdout),
            String::from_utf8_lossy(&stderr)
        )),
    }
}

fn read_to_end_in_background<R: Read + Send + 'static>(
    pipe: Option<R>,
) -> std::thread::JoinHandle<Vec<u8>> {
    std::thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(mut pipe) = pipe {
            let _ = pipe.read_to_end(&mut buf);
        }
        buf
    })
}

/// Runs the Docker CLI with a deadline and returns its stdout, panicking if the
/// command times out or exits unsuccessfully.
fn run_docker_checked(args: &[String], timeout: Duration) -> String {
    let output = run_docker(args, timeout).unwrap_or_else(|e| panic!("{e}"));
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    assert!(
        output.status.success(),
        "docker {args:?} failed with {}\n--- stdout ---\n{stdout}\n--- stderr ---\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    stdout
}

/// Force-removes a named container when dropped, including during unwinding.
/// Killing a timed-out `docker run` client does not stop its container, so the
/// container is removed by name.
struct ContainerCleanup {
    name: String,
}

impl Drop for ContainerCleanup {
    #[allow(clippy::print_stderr)]
    fn drop(&mut self) {
        // `--rm` usually removes the container first, so a "no such container"
        // exit status is expected. Only report failures to run the command.
        let args = ["rm".to_string(), "--force".to_string(), self.name.clone()];
        if let Err(e) = run_docker(&args, DOCKER_CLEANUP_TIMEOUT) {
            eprintln!("failed to remove container {}: {e}", self.name);
        }
    }
}

/// Copy the host's VC++ runtime into a temporary directory next to the test
/// binary, so it can be mounted into the container. The directory is created
/// under the build output so the container can read it.
fn stage_vc_runtime(bin_dir: &Path) -> tempfile::TempDir {
    let system32 =
        PathBuf::from(std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".into()))
            .join("System32");
    let staging_parent = bin_dir.parent().unwrap_or(bin_dir);
    let dir = tempfile::Builder::new()
        .prefix("otap-etw-vcrt-")
        .tempdir_in(staging_parent)
        .expect("failed to create VC++ runtime staging directory");
    let dll = "vcruntime140.dll";
    let _ = std::fs::copy(system32.join(dll), dir.path().join(dll)).unwrap_or_else(|e| {
        panic!(
            "failed to copy {dll} from {}: {e}; install the Visual C++ runtime",
            system32.display()
        )
    });
    dir
}

fn container_emitter_args(
    image: &str,
    container_name: &str,
    run_id: &str,
    current_exe: &Path,
    vcrt_dir: &Path,
) -> Vec<String> {
    let bin_dir = current_exe
        .parent()
        .expect("test binary must have a parent directory");
    let exe_name = current_exe
        .file_name()
        .expect("test binary must have a file name")
        .to_string_lossy();
    let module = module_path!()
        .split_once("::")
        .map_or(module_path!(), |(_, rest)| rest);
    vec![
        "run".into(),
        "--rm".into(),
        "--name".into(),
        container_name.into(),
        "--isolation=process".into(),
        "--network".into(),
        "none".into(),
        "--mount".into(),
        format!(
            "type=bind,source={},target={CONTAINER_BIN_DIR},readonly",
            bin_dir.display()
        ),
        "--mount".into(),
        format!(
            "type=bind,source={},target={CONTAINER_VCRT_DIR},readonly",
            vcrt_dir.display()
        ),
        "--env".into(),
        format!(r"PATH={CONTAINER_VCRT_DIR};C:\Windows\System32;C:\Windows"),
        "--env".into(),
        format!("{EMITTER_RUN_ID_ENV}={run_id}"),
        image.into(),
        format!(r"{CONTAINER_BIN_DIR}\{exe_name}"),
        "--exact".into(),
        format!("{module}::{TEST_FN_NAME}"),
        "--ignored".into(),
        "--nocapture".into(),
    ]
}

/// Provider enable state reported by the emitter inside the container.
#[derive(Debug, PartialEq, Eq)]
struct ContainerProviderState {
    static_enabled: bool,
    dynamic_enabled: bool,
    dynamic_verbose_enabled: bool,
    control_enabled: bool,
}

fn parse_container_state(stdout: &str) -> Option<ContainerProviderState> {
    // Find the marker anywhere in a line: single-threaded libtest prints
    // `test <name> ... ` before the test runs, so with `--nocapture` the
    // emitter's report can follow it on the same line.
    let line = stdout
        .lines()
        .find_map(|l| l.split_once(STATE_LINE_PREFIX).map(|(_, rest)| rest))?;
    let flag = |key: &str| {
        line.split_whitespace()
            .find_map(|kv| kv.strip_prefix(key)?.strip_prefix('='))
            .map(|v| v == "true")
    };
    Some(ContainerProviderState {
        static_enabled: flag("static_enabled")?,
        dynamic_enabled: flag("dynamic_enabled")?,
        dynamic_verbose_enabled: flag("dynamic_verbose_enabled")?,
        control_enabled: flag("control_enabled")?,
    })
}

/// Scenario: The container emitter's provider-state report is printed on its
/// own line, or after libtest's single-threaded `test <name> ... ` progress
/// prefix on the same line, and a report is missing a flag.
/// Guarantees: The host parses the report regardless of libtest's progress
/// formatting, and rejects an incomplete report instead of defaulting flags.
#[test]
fn parse_container_state_finds_report_after_libtest_progress_prefix() {
    let report = format!(
        "{STATE_LINE_PREFIX} static_enabled=true dynamic_enabled=true \
         dynamic_verbose_enabled=false control_enabled=false"
    );
    let expected = ContainerProviderState {
        static_enabled: true,
        dynamic_enabled: true,
        dynamic_verbose_enabled: false,
        control_enabled: false,
    };
    for stdout in [
        format!("\nrunning 1 test\n{report}\ntest {TEST_FN_NAME} ... ok\n"),
        format!("\nrunning 1 test\ntest {TEST_FN_NAME} ... {report}\nok\n"),
    ] {
        assert_eq!(
            parse_container_state(&stdout).as_ref(),
            Some(&expected),
            "stdout:\n{stdout}"
        );
    }

    let incomplete = format!("{STATE_LINE_PREFIX} static_enabled=true dynamic_enabled=true\n");
    assert_eq!(parse_container_state(&incomplete), None);
}

// -- Host side: scenario --------------------------------------------------

/// Emits the control provider's host events, runs the container emitter,
/// checks the provider state it reports, waits until validation has observed
/// every expected event, then drains and shuts the receiver down.
fn container_scenario(
    docker_args: Vec<String>,
    container_name: String,
    run_id: String,
    all_seen: oneshot::Receiver<()>,
) -> impl FnOnce(TestContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
    use futures::FutureExt;
    use std::panic::AssertUnwindSafe;

    move |ctx| {
        Box::pin(async move {
            let outcome = AssertUnwindSafe(async move {
                emit_host_control_events(&run_id).await;

                // The cleanup guard lives on the blocking thread, so the
                // container is removed after `docker run` returns, times out,
                // or panics, without blocking the receiver's runtime thread.
                let emitter = tokio::task::spawn_blocking(move || {
                    let _cleanup = ContainerCleanup {
                        name: container_name,
                    };
                    run_docker_checked(&docker_args, DOCKER_RUN_TIMEOUT)
                })
                .await;
                let stdout = match emitter {
                    Ok(stdout) => stdout,
                    Err(e) if e.is_panic() => std::panic::resume_unwind(e.into_panic()),
                    Err(e) => panic!("container emitter task failed: {e}"),
                };
                let state = parse_container_state(&stdout).unwrap_or_else(|| {
                    panic!("container emitter did not report provider state; stdout:\n{stdout}")
                });
                assert_eq!(
                    state,
                    ContainerProviderState {
                        static_enabled: true,
                        dynamic_enabled: true,
                        dynamic_verbose_enabled: false,
                        control_enabled: false,
                    },
                    "inside the container, only providers with enable_in_containers: true must \
                     be enabled, and only at their configured level; stdout:\n{stdout}"
                );

                match time::timeout(DELIVERY_TIMEOUT, all_seen).await {
                    Ok(Ok(())) => {}
                    Ok(Err(_)) => panic!("validation ended before observing all expected events"),
                    Err(_) => {
                        print_running_etw_sessions();
                        panic!(
                            "receiver did not deliver every expected event within \
                             {DELIVERY_TIMEOUT:?} after the container exited"
                        );
                    }
                }
            })
            .catch_unwind()
            .await;

            // Always drain and stop the receiver so validation observes a
            // closed output channel, even when an assertion above failed.
            let _ = ctx
                .send_control_msg(NodeControlMsg::DrainIngress {
                    deadline: Instant::now() + Duration::from_secs(5),
                    reason: "Test drain".to_owned(),
                })
                .await;
            let _ = ctx.send_shutdown(Instant::now(), "Test").await;

            if let Err(panic) = outcome {
                std::panic::resume_unwind(panic);
            }
        })
    }
}

// -- Host side: validation ------------------------------------------------

/// One decoded event from this run: the emitting process and its attributes.
#[derive(Debug)]
struct ObservedEvent {
    pid: Option<i64>,
    attrs: Vec<(String, AttrSnapshot)>,
}

impl ObservedEvent {
    fn origin(&self) -> Option<&str> {
        str_attr(&self.attrs, "origin")
    }
}

#[derive(Debug, Default)]
struct ContainerObservations {
    static_events: Vec<ObservedEvent>,
    dynamic_events: Vec<ObservedEvent>,
    dynamic_verbose_events: Vec<ObservedEvent>,
    control_events: Vec<ObservedEvent>,
    dns_probe_events: Vec<ObservedEvent>,
}

impl ContainerObservations {
    fn record(&mut self, records: &OtapArrowRecords, run_id: &str, dns_probe: &str) {
        let (Some(logs_rb), Some(attrs_rb)) = (
            records.get(ArrowPayloadType::Logs),
            records.get(ArrowPayloadType::LogAttrs),
        ) else {
            return;
        };
        let names = string_column(logs_rb, consts::EVENT_NAME);
        let log_ids = u16_column(logs_rb, consts::ID);
        for (row, name) in names.iter().enumerate() {
            let Some(log_id) = log_ids[row] else {
                continue;
            };
            let attrs = collect_attributes(attrs_rb, log_id);
            let event = ObservedEvent {
                pid: int_attr(&attrs, "etw.process.id"),
                attrs,
            };

            if str_attr(&event.attrs, "etw.provider.name") == Some(DNS_CLIENT_PROVIDER_NAME) {
                if str_attr(&event.attrs, "QueryName") == Some(dns_probe) {
                    self.dns_probe_events.push(event);
                }
                continue;
            }
            if str_attr(&event.attrs, "run_id") != Some(run_id) {
                continue;
            }
            match name.as_deref() {
                Some(STATIC_EVENT_NAME) => self.static_events.push(event),
                Some(DYNAMIC_EVENT_NAME) => self.dynamic_events.push(event),
                Some(DYNAMIC_VERBOSE_EVENT_NAME) => self.dynamic_verbose_events.push(event),
                Some(CONTROL_EVENT_NAME) => self.control_events.push(event),
                _ => {}
            }
        }
    }

    /// Control events written by the host test process itself.
    fn host_control_events(&self) -> impl Iterator<Item = &ObservedEvent> {
        let host_pid = i64::from(std::process::id());
        self.control_events
            .iter()
            .filter(move |e| e.origin() == Some(ORIGIN_HOST) && e.pid == Some(host_pid))
    }

    fn all_expected_seen(&self) -> bool {
        self.static_events.len() >= EVENTS_PER_PROVIDER as usize
            && self.dynamic_events.len() >= EVENTS_PER_PROVIDER as usize
            && !self.dns_probe_events.is_empty()
            && self.host_control_events().count() >= EVENTS_PER_PROVIDER as usize
    }

    fn summary(&self) -> String {
        format!(
            "static={} dynamic={} dynamic_verbose={} control={} (host={}) dns_probe={}",
            self.static_events.len(),
            self.dynamic_events.len(),
            self.dynamic_verbose_events.len(),
            self.control_events.len(),
            self.host_control_events().count(),
            self.dns_probe_events.len(),
        )
    }
}

/// Collects receiver output until the channel closes after shutdown, signals
/// the scenario once every expected container event has arrived, then
/// asserts on the complete set of delivered events.
fn container_validation(
    run_id: String,
    all_seen: oneshot::Sender<()>,
) -> impl FnOnce(NotSendValidateContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
    move |mut ctx| {
        Box::pin(async move {
            let dns_probe = dns_probe_name(&run_id);
            let mut observations = ContainerObservations::default();
            let mut all_seen = Some(all_seen);
            let deadline = time::Instant::now() + VALIDATION_TIMEOUT;

            loop {
                match time::timeout_at(deadline, ctx.recv()).await {
                    Ok(Ok(pdata)) => {
                        let records = pdata
                            .payload_ref()
                            .otap_ref()
                            .expect("expected native OTAP payload from ETW receiver");
                        observations.record(records, &run_id, &dns_probe);
                        if observations.all_expected_seen()
                            && let Some(tx) = all_seen.take()
                        {
                            let _ = tx.send(());
                        }
                    }
                    // The receiver dropped its output after shutdown.
                    Ok(Err(_)) => break,
                    Err(_) => panic!(
                        "receiver output did not close within {VALIDATION_TIMEOUT:?}; observed {}",
                        observations.summary()
                    ),
                }
            }

            assert_container_events(&observations, &dns_probe);
        })
    }
}

fn assert_container_events(observations: &ContainerObservations, dns_probe: &str) {
    let summary = observations.summary();
    let host_pid = i64::from(std::process::id());

    for (kind, events) in [
        ("TraceLogging static", &observations.static_events),
        ("TraceLogging dynamic", &observations.dynamic_events),
    ] {
        assert_eq!(
            events.len(),
            EVENTS_PER_PROVIDER as usize,
            "expected {EVENTS_PER_PROVIDER} {kind} events from the container; observed {summary}"
        );
        let seqs: BTreeSet<i64> = events
            .iter()
            .filter_map(|e| int_attr(&e.attrs, "seq"))
            .collect();
        assert_eq!(
            seqs,
            (0..i64::from(EVENTS_PER_PROVIDER)).collect(),
            "{kind} events must carry decoded `seq` values 0..{EVENTS_PER_PROVIDER}"
        );
        for event in events {
            assert_eq!(
                event.origin(),
                Some(ORIGIN_CONTAINER),
                "{kind} event must carry decoded `origin`: {event:?}"
            );
        }
    }

    assert!(
        observations.dynamic_verbose_events.is_empty(),
        "verbose events are below the dynamic provider's configured information level and \
         must be rejected for container events; observed {summary}"
    );

    assert!(
        !observations.dns_probe_events.is_empty(),
        "expected {DNS_CLIENT_PROVIDER_NAME} events for QueryName={dns_probe} from the container; \
         observed {summary}"
    );
    let dns_probe_ids: BTreeSet<i64> = observations
        .dns_probe_events
        .iter()
        .filter_map(|e| int_attr(&e.attrs, "etw.event.id"))
        .collect();
    assert!(
        dns_probe_ids.contains(&DNS_QUERY_EVENT_ID),
        "the container lookup must produce a decoded {DNS_CLIENT_PROVIDER_NAME} event \
         {DNS_QUERY_EVENT_ID}; got event IDs {dns_probe_ids:?}"
    );
    let filtered_out: Vec<&ObservedEvent> = observations
        .dns_probe_events
        .iter()
        .filter(|e| int_attr(&e.attrs, "etw.event.id") != Some(DNS_QUERY_EVENT_ID))
        .collect();
    assert!(
        filtered_out.is_empty(),
        "the event_ids allow-list [{DNS_QUERY_EVENT_ID}] must reject every other \
         {DNS_CLIENT_PROVIDER_NAME} event for container lookups; got event IDs \
         {dns_probe_ids:?}: {filtered_out:?}"
    );

    // The TraceLogging events come from the single emitter process, which is
    // not the host test process.
    let emitter_pids: BTreeSet<Option<i64>> = observations
        .static_events
        .iter()
        .chain(&observations.dynamic_events)
        .map(|e| e.pid)
        .collect();
    assert_eq!(
        emitter_pids.len(),
        1,
        "all TraceLogging container events must share one process ID, got {emitter_pids:?}"
    );
    let emitter_pid = emitter_pids.into_iter().next().flatten();
    assert!(
        emitter_pid.is_some() && emitter_pid != Some(host_pid),
        "container events must come from the container process, not the host test process \
         ({host_pid}); got {emitter_pid:?}"
    );
    // Event 3006 is logged by the process performing the lookup, and no probe
    // event may come from the host test process.
    assert!(
        observations.dns_probe_events.iter().any(|e| {
            int_attr(&e.attrs, "etw.event.id") == Some(DNS_QUERY_EVENT_ID) && e.pid == emitter_pid
        }),
        "{DNS_CLIENT_PROVIDER_NAME} event {DNS_QUERY_EVENT_ID} must come from the emitter process \
         {emitter_pid:?}; observed {summary}"
    );
    assert!(
        observations
            .dns_probe_events
            .iter()
            .all(|e| e.pid.is_some() && e.pid != Some(host_pid)),
        "probe {DNS_CLIENT_PROVIDER_NAME} events must come from container processes, not the host \
         test process ({host_pid})"
    );

    let host_control: Vec<&ObservedEvent> = observations.host_control_events().collect();
    let host_control_seqs: BTreeSet<i64> = host_control
        .iter()
        .filter_map(|e| int_attr(&e.attrs, "seq"))
        .collect();
    assert!(
        host_control.len() == EVENTS_PER_PROVIDER as usize
            && host_control_seqs == (0..i64::from(EVENTS_PER_PROVIDER)).collect(),
        "the control provider must be subscribed: expected {EVENTS_PER_PROVIDER} host control \
         events with seq 0..{EVENTS_PER_PROVIDER}; observed {summary}"
    );
    let non_host_control: Vec<&ObservedEvent> = observations
        .control_events
        .iter()
        .filter(|e| e.origin() != Some(ORIGIN_HOST) || e.pid != Some(host_pid))
        .collect();
    assert!(
        non_host_control.is_empty(),
        "provider with enable_in_containers: false must not deliver container events; \
         got {non_host_control:?}"
    );
}

fn str_attr<'a>(attrs: &'a [(String, AttrSnapshot)], key: &str) -> Option<&'a str> {
    match attrs.iter().find(|(k, _)| k == key).map(|(_, v)| v) {
        Some(AttrSnapshot::Str(s)) => Some(s.as_str()),
        _ => None,
    }
}

fn int_attr(attrs: &[(String, AttrSnapshot)], key: &str) -> Option<i64> {
    match attrs.iter().find(|(k, _)| k == key).map(|(_, v)| v) {
        Some(AttrSnapshot::Int(v)) => Some(*v),
        _ => None,
    }
}
