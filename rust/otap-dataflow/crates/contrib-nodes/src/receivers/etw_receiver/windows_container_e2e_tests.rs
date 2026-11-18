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
//! - a TraceLogging dynamic provider (`tracelogging_dynamic`),
//! - the `Microsoft-Windows-DNS-Client` manifest provider, triggered by a name
//!   lookup, and
//! - a TraceLogging control provider configured with
//!   `enable_in_containers: false`.
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
const CONTROL_EVENT_NAME: &str = "SiloControlEvent";
const DNS_CLIENT_PROVIDER_NAME: &str = "Microsoft-Windows-DNS-Client";
/// `Microsoft-Windows-DNS-Client` "DNS query is called" event, emitted in the
/// process that performs the lookup.
const DNS_QUERY_EVENT_ID: i64 = 3006;
const EVENTS_PER_PROVIDER: u32 = 3;

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

/// Upper bound for the in-container wait for the host session to enable the
/// opted-in providers.
const EMITTER_ENABLE_TIMEOUT: Duration = Duration::from_secs(30);
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
/// to a TraceLogging static, a TraceLogging dynamic, and a manifest provider
/// (DNS-Client, filtered to `event_ids: [3006]`), plus a TraceLogging control
/// provider with `enable_in_containers: false`; this binary then emits from all
/// of them inside a process-isolated container.
/// Guarantees: Events from every opted-in provider type are collected from the
/// container process with their payload fields decoded, the manifest
/// `event_ids` filter still applies, and the control provider is never enabled
/// inside the container, so none of its events are collected.
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
                "level": "verbose",
                "enable_in_containers": true,
            },
            {
                "name": dynamic_provider_name(&run_id),
                "kind": "tracelogging",
                "level": "verbose",
                "enable_in_containers": true,
            },
            {
                "name": control_provider_name(&run_id),
                "kind": "tracelogging",
                "level": "verbose",
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
        .run_test(container_scenario(docker_args, container_name, all_seen_rx))
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
        SILO_STATIC_PROVIDER.enabled(tracelogging::Level::Verbose, 0)
            && dynamic.enabled(tld::Level::Verbose, 0)
    };
    let deadline = Instant::now() + EMITTER_ENABLE_TIMEOUT;
    while !opted_in_enabled() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(50));
    }
    println!(
        "{STATE_LINE_PREFIX} static_enabled={} dynamic_enabled={} control_enabled={}",
        SILO_STATIC_PROVIDER.enabled(tracelogging::Level::Verbose, 0),
        dynamic.enabled(tld::Level::Verbose, 0),
        control.enabled(tld::Level::Verbose, 0),
    );

    let origin = "container";
    let dns_probe = format!("{}:80", dns_probe_name(run_id));
    let mut builder = tld::EventBuilder::new();
    for seq in 0..EVENTS_PER_PROVIDER {
        // Control first, so a wrongly enabled control provider would be
        // delivered no later than the events the host waits for.
        let _ = builder
            .reset(CONTROL_EVENT_NAME, tld::Level::Informational, 0x1, 0)
            .add_str8("run_id", run_id.as_bytes(), tld::OutType::Default, 0)
            .add_u32("seq", seq, tld::OutType::Default, 0)
            .write(&control, None, None);
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
    control_enabled: bool,
}

fn parse_container_state(stdout: &str) -> Option<ContainerProviderState> {
    let line = stdout
        .lines()
        .find_map(|l| l.trim().strip_prefix(STATE_LINE_PREFIX))?;
    let flag = |key: &str| {
        line.split_whitespace()
            .find_map(|kv| kv.strip_prefix(key)?.strip_prefix('='))
            .map(|v| v == "true")
    };
    Some(ContainerProviderState {
        static_enabled: flag("static_enabled")?,
        dynamic_enabled: flag("dynamic_enabled")?,
        control_enabled: flag("control_enabled")?,
    })
}

// -- Host side: scenario --------------------------------------------------

/// Runs the container emitter, checks the provider state it reports, waits
/// until validation has observed every expected container event, then drains
/// and shuts the receiver down.
fn container_scenario(
    docker_args: Vec<String>,
    container_name: String,
    all_seen: oneshot::Receiver<()>,
) -> impl FnOnce(TestContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
    use futures::FutureExt;
    use std::panic::AssertUnwindSafe;

    move |ctx| {
        Box::pin(async move {
            let outcome = AssertUnwindSafe(async move {
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
                        control_enabled: false,
                    },
                    "only providers with enable_in_containers: true must be enabled inside \
                     the container; stdout:\n{stdout}"
                );

                match time::timeout(DELIVERY_TIMEOUT, all_seen).await {
                    Ok(Ok(())) => {}
                    Ok(Err(_)) => panic!("validation ended before observing all container events"),
                    Err(_) => {
                        print_running_etw_sessions();
                        panic!(
                            "receiver did not deliver every expected container event within \
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

/// One decoded container event: the emitting process and its attributes.
#[derive(Debug)]
struct ObservedEvent {
    pid: Option<i64>,
    attrs: Vec<(String, AttrSnapshot)>,
}

#[derive(Debug, Default)]
struct ContainerObservations {
    static_events: Vec<ObservedEvent>,
    dynamic_events: Vec<ObservedEvent>,
    control_events: Vec<ObservedEvent>,
    dns_probe_events: Vec<ObservedEvent>,
    dns_event_ids: BTreeSet<i64>,
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
                if let Some(id) = int_attr(&event.attrs, "etw.event.id") {
                    let _ = self.dns_event_ids.insert(id);
                }
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
                Some(CONTROL_EVENT_NAME) => self.control_events.push(event),
                _ => {}
            }
        }
    }

    fn all_expected_seen(&self) -> bool {
        self.static_events.len() >= EVENTS_PER_PROVIDER as usize
            && self.dynamic_events.len() >= EVENTS_PER_PROVIDER as usize
            && !self.dns_probe_events.is_empty()
    }

    fn summary(&self) -> String {
        format!(
            "static={} dynamic={} control={} dns_probe={} dns_event_ids={:?}",
            self.static_events.len(),
            self.dynamic_events.len(),
            self.control_events.len(),
            self.dns_probe_events.len(),
            self.dns_event_ids
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
                str_attr(&event.attrs, "origin"),
                Some("container"),
                "{kind} event must carry decoded `origin`: {event:?}"
            );
        }
    }

    assert!(
        !observations.dns_probe_events.is_empty(),
        "expected {DNS_CLIENT_PROVIDER_NAME} events for QueryName={dns_probe} from the container; \
         observed {summary}"
    );
    for event in &observations.dns_probe_events {
        assert_eq!(
            int_attr(&event.attrs, "etw.event.id"),
            Some(DNS_QUERY_EVENT_ID),
            "manifest event must decode its event ID: {event:?}"
        );
    }
    assert_eq!(
        observations.dns_event_ids,
        BTreeSet::from([DNS_QUERY_EVENT_ID]),
        "event_ids: [{DNS_QUERY_EVENT_ID}] must filter every delivered {DNS_CLIENT_PROVIDER_NAME} event"
    );

    // Every container event comes from the single emitter process, which is
    // not the host test process.
    let pids: BTreeSet<Option<i64>> = observations
        .static_events
        .iter()
        .chain(&observations.dynamic_events)
        .chain(&observations.dns_probe_events)
        .map(|e| e.pid)
        .collect();
    assert_eq!(
        pids.len(),
        1,
        "all container events must share one process ID, got {pids:?}"
    );
    let container_pid = pids.into_iter().next().flatten();
    assert!(
        container_pid.is_some() && container_pid != Some(host_pid),
        "container events must come from the container process, not the host test process \
         ({host_pid}); got {container_pid:?}"
    );

    assert!(
        observations.control_events.is_empty(),
        "provider with enable_in_containers: false must not deliver container events; \
         observed {summary}"
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
