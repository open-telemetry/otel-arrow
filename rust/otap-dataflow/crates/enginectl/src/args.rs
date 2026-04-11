// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "dfctl",
    version,
    about = "Control CLI for the OTAP Dataflow Engine admin API"
)]
/// Top-level parsed CLI arguments for `dfctl`.
pub struct Cli {
    /// Shared connection and transport configuration.
    #[command(flatten)]
    pub connection: ConnectionArgs,

    /// Terminal color policy for human-readable output.
    #[arg(long, global = true, value_enum, default_value_t = ColorChoice::Auto)]
    pub color: ColorChoice,

    /// Top-level command to execute.
    #[command(subcommand)]
    pub command: Command,
}

impl Cli {
    /// Returns true when the parsed command should launch the interactive UI.
    #[must_use]
    pub const fn is_ui(&self) -> bool {
        matches!(self.command, Command::Ui(_))
    }
}

#[derive(Args, Debug, Clone, Default)]
pub struct ConnectionArgs {
    #[arg(long, env = "DFCTL_URL", global = true, value_name = "URL")]
    pub url: Option<String>,

    #[arg(long, env = "DFCTL_PROFILE_FILE", global = true, value_name = "PATH")]
    pub profile_file: Option<PathBuf>,

    #[arg(
        long,
        env = "DFCTL_CONNECT_TIMEOUT",
        global = true,
        value_parser = parse_duration_arg,
        value_name = "DURATION"
    )]
    pub connect_timeout: Option<Duration>,

    #[arg(
        long,
        env = "DFCTL_REQUEST_TIMEOUT",
        global = true,
        value_parser = parse_duration_arg,
        value_name = "DURATION",
        conflicts_with = "no_request_timeout"
    )]
    pub request_timeout: Option<Duration>,

    #[arg(
        long,
        env = "DFCTL_NO_REQUEST_TIMEOUT",
        global = true,
        default_value_t = false
    )]
    pub no_request_timeout: bool,

    #[arg(
        long,
        env = "DFCTL_TCP_NODELAY",
        global = true,
        value_parser = clap::value_parser!(bool),
        value_name = "BOOL"
    )]
    pub tcp_nodelay: Option<bool>,

    #[arg(
        long,
        env = "DFCTL_TCP_KEEPALIVE",
        global = true,
        value_parser = parse_duration_arg,
        value_name = "DURATION"
    )]
    pub tcp_keepalive: Option<Duration>,

    #[arg(
        long,
        env = "DFCTL_TCP_KEEPALIVE_INTERVAL",
        global = true,
        value_parser = parse_duration_arg,
        value_name = "DURATION"
    )]
    pub tcp_keepalive_interval: Option<Duration>,

    #[arg(long, env = "DFCTL_CA_FILE", global = true, value_name = "PATH")]
    pub ca_file: Option<PathBuf>,

    #[arg(
        long,
        env = "DFCTL_CLIENT_CERT_FILE",
        global = true,
        value_name = "PATH"
    )]
    pub client_cert_file: Option<PathBuf>,

    #[arg(
        long,
        env = "DFCTL_CLIENT_KEY_FILE",
        global = true,
        value_name = "PATH"
    )]
    pub client_key_file: Option<PathBuf>,

    #[arg(
        long,
        env = "DFCTL_INCLUDE_SYSTEM_CA_CERTS",
        global = true,
        value_parser = clap::value_parser!(bool),
        value_name = "BOOL"
    )]
    pub include_system_ca_certs: Option<bool>,

    #[arg(
        long,
        env = "DFCTL_INSECURE_SKIP_VERIFY",
        global = true,
        value_parser = clap::value_parser!(bool),
        value_name = "BOOL"
    )]
    pub insecure_skip_verify: Option<bool>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    Ui(UiArgs),
    Engine(EngineArgs),
    Groups(GroupsArgs),
    Pipelines(PipelinesArgs),
    Telemetry(TelemetryArgs),
}

#[derive(Args, Debug, Clone)]
pub struct UiArgs {
    #[arg(long, value_enum, default_value_t = UiStartView::Pipelines)]
    pub start_view: UiStartView,

    #[arg(long, value_parser = parse_duration_arg, default_value = "2s")]
    pub refresh_interval: Duration,

    #[arg(long, default_value_t = 200)]
    pub logs_tail: usize,
}

#[derive(Args, Debug, Clone)]
pub struct EngineArgs {
    #[command(subcommand)]
    pub command: EngineCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum EngineCommand {
    Status(ReadOutputArgs),
    Livez(ReadOutputArgs),
    Readyz(ReadOutputArgs),
}

#[derive(Args, Debug, Clone)]
pub struct GroupsArgs {
    #[command(subcommand)]
    pub command: GroupsCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum GroupsCommand {
    Describe(ReadOutputArgs),
    Events(GroupEventsArgs),
    Diagnose(GroupDiagnoseArgs),
    Bundle(GroupBundleArgs),
    Status(ReadOutputArgs),
    Shutdown(GroupShutdownArgs),
}

#[derive(Args, Debug, Clone)]
pub struct PipelinesArgs {
    #[command(subcommand)]
    pub command: PipelinesCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum PipelinesCommand {
    #[command(alias = "details")]
    Get(PipelineReadArgs),
    Describe(PipelineReadArgs),
    Events(PipelineEventsArgs),
    Diagnose(PipelineDiagnoseArgs),
    Bundle(PipelineBundleArgs),
    Status(PipelineReadArgs),
    Livez(PipelineReadArgs),
    Readyz(PipelineReadArgs),
    Reconfigure(ReconfigureArgs),
    Shutdown(PipelineShutdownArgs),
    Rollouts(PipelineRolloutsArgs),
    RolloutStatus(RolloutLookupArgs),
    Shutdowns(PipelineShutdownsArgs),
    ShutdownStatus(ShutdownLookupArgs),
}

#[derive(Args, Debug, Clone)]
pub struct PipelineRolloutsArgs {
    #[command(subcommand)]
    pub command: RolloutCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum RolloutCommand {
    Get(RolloutLookupArgs),
    Watch(RolloutWatchArgs),
}

#[derive(Args, Debug, Clone)]
pub struct PipelineShutdownsArgs {
    #[command(subcommand)]
    pub command: ShutdownCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum ShutdownCommand {
    Get(ShutdownLookupArgs),
    Watch(ShutdownWatchArgs),
}

#[derive(Args, Debug, Clone)]
pub struct TelemetryArgs {
    #[command(subcommand)]
    pub command: TelemetryCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum TelemetryCommand {
    Logs(LogsArgs),
    Metrics(MetricsArgs),
}

#[derive(Args, Debug, Clone)]
pub struct LogsArgs {
    #[command(subcommand)]
    pub command: LogsCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum LogsCommand {
    Get(LogsGetArgs),
    Watch(LogsWatchArgs),
}

#[derive(Args, Debug, Clone)]
pub struct MetricsArgs {
    #[command(subcommand)]
    pub command: MetricsCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum MetricsCommand {
    Get(MetricsGetArgs),
    Watch(MetricsWatchArgs),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum UiStartView {
    Pipelines,
    Groups,
    Engine,
}

#[derive(Args, Debug, Clone)]
pub struct GroupEventsArgs {
    #[command(subcommand)]
    pub command: GroupEventsCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum GroupEventsCommand {
    Get(GroupEventsGetArgs),
    Watch(GroupEventsWatchArgs),
}

#[derive(Args, Debug, Clone)]
pub struct PipelineEventsArgs {
    #[command(subcommand)]
    pub command: PipelineEventsCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum PipelineEventsCommand {
    Get(PipelineEventsGetArgs),
    Watch(PipelineEventsWatchArgs),
}

#[derive(Args, Debug, Clone)]
pub struct GroupDiagnoseArgs {
    #[command(subcommand)]
    pub command: GroupDiagnoseCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum GroupDiagnoseCommand {
    Shutdown(GroupDiagnoseShutdownArgs),
}

#[derive(Args, Debug, Clone)]
pub struct PipelineDiagnoseArgs {
    #[command(subcommand)]
    pub command: PipelineDiagnoseCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum PipelineDiagnoseCommand {
    Rollout(PipelineDiagnoseRolloutArgs),
    Shutdown(PipelineDiagnoseShutdownArgs),
}

#[derive(Args, Debug, Clone)]
pub struct PipelineTargetArgs {
    pub pipeline_group_id: String,
    pub pipeline_id: String,
}

#[derive(Args, Debug, Clone)]
pub struct RolloutTargetArgs {
    pub pipeline_group_id: String,
    pub pipeline_id: String,
    pub rollout_id: String,
}

#[derive(Args, Debug, Clone)]
pub struct ShutdownTargetArgs {
    pub pipeline_group_id: String,
    pub pipeline_id: String,
    pub shutdown_id: String,
}

#[derive(Args, Debug, Clone)]
pub struct GroupEventsFilterArgs {
    #[arg(long = "kind", value_enum)]
    pub kinds: Vec<EventKind>,

    #[arg(long = "group")]
    pub pipeline_group_id: Option<String>,

    #[arg(long = "pipeline")]
    pub pipeline_id: Option<String>,

    #[arg(long = "node")]
    pub node_id: Option<String>,

    #[arg(long)]
    pub contains: Option<String>,

    #[arg(long)]
    pub tail: Option<usize>,
}

#[derive(Args, Debug, Clone)]
pub struct PipelineEventsFilterArgs {
    #[arg(long = "kind", value_enum)]
    pub kinds: Vec<EventKind>,

    #[arg(long = "node")]
    pub node_id: Option<String>,

    #[arg(long)]
    pub contains: Option<String>,

    #[arg(long)]
    pub tail: Option<usize>,
}

#[derive(Args, Debug, Clone, Default)]
pub struct LogsFilterArgs {
    #[arg(long)]
    pub level: Option<String>,

    #[arg(long)]
    pub target: Option<String>,

    #[arg(long)]
    pub event: Option<String>,

    #[arg(long = "group")]
    pub pipeline_group_id: Option<String>,

    #[arg(long = "pipeline")]
    pub pipeline_id: Option<String>,

    #[arg(long = "node")]
    pub node_id: Option<String>,

    #[arg(long)]
    pub contains: Option<String>,
}

#[derive(Args, Debug, Clone, Default)]
pub struct MetricsFilterArgs {
    #[arg(long = "metric-set")]
    pub metric_sets: Vec<String>,

    #[arg(long = "metric-name")]
    pub metric_names: Vec<String>,

    #[arg(long = "group")]
    pub pipeline_group_id: Option<String>,

    #[arg(long = "pipeline")]
    pub pipeline_id: Option<String>,

    #[arg(long = "core")]
    pub core_id: Option<usize>,

    #[arg(long = "node")]
    pub node_id: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct PipelineReadArgs {
    #[command(flatten)]
    pub target: PipelineTargetArgs,

    #[command(flatten)]
    pub output: ReadOutputArgs,
}

#[derive(Args, Debug, Clone)]
pub struct RolloutLookupArgs {
    #[command(flatten)]
    pub target: RolloutTargetArgs,

    #[command(flatten)]
    pub output: ReadOutputArgs,
}

#[derive(Args, Debug, Clone)]
pub struct ShutdownLookupArgs {
    #[command(flatten)]
    pub target: ShutdownTargetArgs,

    #[command(flatten)]
    pub output: ReadOutputArgs,
}

#[derive(Args, Debug, Clone)]
pub struct GroupShutdownArgs {
    #[arg(long, default_value_t = false, conflicts_with = "watch")]
    pub wait: bool,

    #[arg(long, value_parser = parse_duration_arg, default_value = "60s")]
    pub wait_timeout: Duration,

    #[arg(long, default_value_t = false, conflicts_with = "wait")]
    pub watch: bool,

    #[arg(long = "watch-interval", value_parser = parse_duration_arg, default_value = "2s")]
    pub watch_interval: Duration,

    #[arg(long, value_enum, default_value_t = GroupShutdownOutput::Human)]
    pub output: GroupShutdownOutput,
}

#[derive(Args, Debug, Clone)]
pub struct ReconfigureArgs {
    #[command(flatten)]
    pub target: PipelineTargetArgs,

    #[arg(long, value_name = "PATH", required = true)]
    pub file: PathBuf,

    #[arg(long, value_parser = parse_duration_arg, default_value = "60s")]
    pub step_timeout: Duration,

    #[arg(long, value_parser = parse_duration_arg, default_value = "60s")]
    pub drain_timeout: Duration,

    #[arg(long, default_value_t = false, conflicts_with = "watch")]
    pub wait: bool,

    #[arg(long, value_parser = parse_duration_arg, default_value = "60s")]
    pub wait_timeout: Duration,

    #[arg(long, default_value_t = false, conflicts_with = "wait")]
    pub watch: bool,

    #[arg(long, value_parser = parse_duration_arg, default_value = "2s")]
    pub watch_interval: Duration,

    #[arg(long, value_enum, default_value_t = MutationOutput::Human)]
    pub output: MutationOutput,
}

#[derive(Args, Debug, Clone)]
pub struct PipelineShutdownArgs {
    #[command(flatten)]
    pub target: PipelineTargetArgs,

    #[arg(long, default_value_t = false, conflicts_with = "watch")]
    pub wait: bool,

    #[arg(long, value_parser = parse_duration_arg, default_value = "60s")]
    pub wait_timeout: Duration,

    #[arg(long, default_value_t = false, conflicts_with = "wait")]
    pub watch: bool,

    #[arg(long, value_parser = parse_duration_arg, default_value = "2s")]
    pub watch_interval: Duration,

    #[arg(long, value_enum, default_value_t = MutationOutput::Human)]
    pub output: MutationOutput,
}

#[derive(Args, Debug, Clone)]
pub struct RolloutWatchArgs {
    #[command(flatten)]
    pub target: RolloutTargetArgs,

    #[arg(long, value_parser = parse_duration_arg, default_value = "2s")]
    pub interval: Duration,

    #[command(flatten)]
    pub output: StreamOutputArgs,
}

#[derive(Args, Debug, Clone)]
pub struct ShutdownWatchArgs {
    #[command(flatten)]
    pub target: ShutdownTargetArgs,

    #[arg(long, value_parser = parse_duration_arg, default_value = "2s")]
    pub interval: Duration,

    #[command(flatten)]
    pub output: StreamOutputArgs,
}

#[derive(Args, Debug, Clone)]
pub struct GroupEventsGetArgs {
    #[command(flatten)]
    pub filters: GroupEventsFilterArgs,

    #[command(flatten)]
    pub output: ReadOutputArgs,
}

#[derive(Args, Debug, Clone)]
pub struct GroupEventsWatchArgs {
    #[command(flatten)]
    pub filters: GroupEventsFilterArgs,

    #[arg(long = "watch-interval", value_parser = parse_duration_arg, default_value = "2s")]
    pub interval: Duration,

    #[command(flatten)]
    pub output: StreamOutputArgs,
}

#[derive(Args, Debug, Clone)]
pub struct PipelineEventsGetArgs {
    #[command(flatten)]
    pub target: PipelineTargetArgs,

    #[command(flatten)]
    pub filters: PipelineEventsFilterArgs,

    #[command(flatten)]
    pub output: ReadOutputArgs,
}

#[derive(Args, Debug, Clone)]
pub struct PipelineEventsWatchArgs {
    #[command(flatten)]
    pub target: PipelineTargetArgs,

    #[command(flatten)]
    pub filters: PipelineEventsFilterArgs,

    #[arg(long = "watch-interval", value_parser = parse_duration_arg, default_value = "2s")]
    pub interval: Duration,

    #[command(flatten)]
    pub output: StreamOutputArgs,
}

#[derive(Args, Debug, Clone)]
pub struct GroupDiagnoseShutdownArgs {
    #[arg(long, default_value_t = 200)]
    pub logs_limit: usize,

    #[command(flatten)]
    pub output: ReadOutputArgs,
}

#[derive(Args, Debug, Clone)]
pub struct PipelineDiagnoseRolloutArgs {
    #[command(flatten)]
    pub target: PipelineTargetArgs,

    #[arg(long)]
    pub rollout_id: Option<String>,

    #[arg(long, default_value_t = 200)]
    pub logs_limit: usize,

    #[command(flatten)]
    pub output: ReadOutputArgs,
}

#[derive(Args, Debug, Clone)]
pub struct PipelineDiagnoseShutdownArgs {
    #[command(flatten)]
    pub target: PipelineTargetArgs,

    #[arg(long)]
    pub shutdown_id: Option<String>,

    #[arg(long, default_value_t = 200)]
    pub logs_limit: usize,

    #[command(flatten)]
    pub output: ReadOutputArgs,
}

#[derive(Args, Debug, Clone)]
pub struct GroupBundleArgs {
    #[arg(long, value_name = "PATH")]
    pub file: Option<PathBuf>,

    #[arg(long, default_value_t = 200)]
    pub logs_limit: usize,

    #[arg(long, value_enum, default_value_t = MetricsShape::Compact)]
    pub metrics_shape: MetricsShape,

    #[arg(long, value_enum, default_value_t = BundleOutput::Json)]
    pub output: BundleOutput,
}

#[derive(Args, Debug, Clone)]
pub struct PipelineBundleArgs {
    #[command(flatten)]
    pub target: PipelineTargetArgs,

    #[arg(long)]
    pub rollout_id: Option<String>,

    #[arg(long)]
    pub shutdown_id: Option<String>,

    #[arg(long, value_name = "PATH")]
    pub file: Option<PathBuf>,

    #[arg(long, default_value_t = 200)]
    pub logs_limit: usize,

    #[arg(long, value_enum, default_value_t = MetricsShape::Compact)]
    pub metrics_shape: MetricsShape,

    #[arg(long, value_enum, default_value_t = BundleOutput::Json)]
    pub output: BundleOutput,
}

#[derive(Args, Debug, Clone)]
pub struct LogsGetArgs {
    #[arg(long)]
    pub after: Option<u64>,

    #[arg(long)]
    pub limit: Option<usize>,

    #[command(flatten)]
    pub filters: LogsFilterArgs,

    #[command(flatten)]
    pub output: ReadOutputArgs,
}

#[derive(Args, Debug, Clone)]
pub struct LogsWatchArgs {
    #[arg(long, conflicts_with = "tail")]
    pub after: Option<u64>,

    #[arg(long)]
    pub tail: Option<usize>,

    #[arg(long)]
    pub limit: Option<usize>,

    #[arg(long, value_parser = parse_duration_arg, default_value = "2s")]
    pub interval: Duration,

    #[command(flatten)]
    pub filters: LogsFilterArgs,

    #[command(flatten)]
    pub output: StreamOutputArgs,
}

#[derive(Args, Debug, Clone)]
pub struct MetricsGetArgs {
    #[arg(long, value_enum, default_value_t = MetricsShape::Compact)]
    pub shape: MetricsShape,

    #[arg(long, default_value_t = false)]
    pub reset: bool,

    #[arg(long, default_value_t = false)]
    pub keep_all_zeroes: bool,

    #[command(flatten)]
    pub filters: MetricsFilterArgs,

    #[command(flatten)]
    pub output: ReadOutputArgs,
}

#[derive(Args, Debug, Clone)]
pub struct MetricsWatchArgs {
    #[arg(long, value_enum, default_value_t = MetricsShape::Compact)]
    pub shape: MetricsShape,

    #[arg(long, default_value_t = false)]
    pub reset: bool,

    #[arg(long, default_value_t = false)]
    pub keep_all_zeroes: bool,

    #[arg(long, value_parser = parse_duration_arg, default_value = "2s")]
    pub interval: Duration,

    #[command(flatten)]
    pub filters: MetricsFilterArgs,

    #[command(flatten)]
    pub output: StreamOutputArgs,
}

#[derive(Args, Debug, Clone)]
pub struct ReadOutputArgs {
    #[arg(long, value_enum, default_value_t = ReadOutput::Human)]
    pub output: ReadOutput,
}

#[derive(Args, Debug, Clone)]
pub struct StreamOutputArgs {
    #[arg(long, value_enum, default_value_t = StreamOutput::Human)]
    pub output: StreamOutput,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum ReadOutput {
    Human,
    Json,
    Yaml,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum StreamOutput {
    Human,
    Ndjson,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum MutationOutput {
    Human,
    Json,
    Yaml,
    Ndjson,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum GroupShutdownOutput {
    Human,
    Json,
    Yaml,
    Ndjson,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum BundleOutput {
    Json,
    Yaml,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum MetricsShape {
    Compact,
    Full,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum EventKind {
    Request,
    Success,
    Error,
    Log,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum ColorChoice {
    Auto,
    Always,
    Never,
}

pub fn parse_duration_arg(raw: &str) -> Result<Duration, String> {
    humantime::parse_duration(raw).map_err(|err| err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn details_alias_parses_to_get() {
        let cli = Cli::try_parse_from(["dfctl", "pipelines", "details", "tenant-a", "ingest"])
            .expect("details alias should parse");

        match cli.command {
            Command::Pipelines(PipelinesArgs {
                command: PipelinesCommand::Get(args),
            }) => {
                assert_eq!(args.target.pipeline_group_id, "tenant-a");
                assert_eq!(args.target.pipeline_id, "ingest");
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn color_flag_parses() {
        let cli = Cli::try_parse_from(["dfctl", "--color", "always", "engine", "status"])
            .expect("color flag should parse");

        assert_eq!(cli.color, ColorChoice::Always);
    }

    #[test]
    fn ui_args_parse() {
        let cli = Cli::try_parse_from([
            "dfctl",
            "ui",
            "--start-view",
            "groups",
            "--refresh-interval",
            "5s",
            "--logs-tail",
            "250",
        ])
        .expect("ui args should parse");

        match cli.command {
            Command::Ui(args) => {
                assert_eq!(args.start_view, UiStartView::Groups);
                assert_eq!(args.refresh_interval, Duration::from_secs(5));
                assert_eq!(args.logs_tail, 250);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn groups_shutdown_watch_conflicts_with_wait() {
        let result = Cli::try_parse_from(["dfctl", "groups", "shutdown", "--wait", "--watch"]);

        assert!(result.is_err());
    }

    #[test]
    fn pipeline_events_watch_parses_filters() {
        let cli = Cli::try_parse_from([
            "dfctl",
            "pipelines",
            "events",
            "watch",
            "tenant-a",
            "ingest",
            "--kind",
            "error",
            "--node",
            "receiver",
            "--contains",
            "deadline",
            "--tail",
            "5",
            "--watch-interval",
            "3s",
            "--output",
            "ndjson",
        ])
        .expect("pipeline events watch should parse");

        match cli.command {
            Command::Pipelines(PipelinesArgs {
                command:
                    PipelinesCommand::Events(PipelineEventsArgs {
                        command: PipelineEventsCommand::Watch(args),
                    }),
            }) => {
                assert_eq!(args.target.pipeline_group_id, "tenant-a");
                assert_eq!(args.target.pipeline_id, "ingest");
                assert_eq!(args.filters.kinds, vec![EventKind::Error]);
                assert_eq!(args.filters.node_id.as_deref(), Some("receiver"));
                assert_eq!(args.filters.contains.as_deref(), Some("deadline"));
                assert_eq!(args.filters.tail, Some(5));
                assert_eq!(args.interval, Duration::from_secs(3));
                assert_eq!(args.output.output, StreamOutput::Ndjson);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn telemetry_filters_parse() {
        let cli = Cli::try_parse_from([
            "dfctl",
            "telemetry",
            "metrics",
            "get",
            "--metric-set",
            "engine.pipeline",
            "--metric-name",
            "pending.sends",
            "--group",
            "tenant-a",
            "--pipeline",
            "ingest",
            "--node",
            "receiver",
            "--core",
            "2",
        ])
        .expect("telemetry metric filters should parse");

        match cli.command {
            Command::Telemetry(TelemetryArgs {
                command:
                    TelemetryCommand::Metrics(MetricsArgs {
                        command: MetricsCommand::Get(args),
                    }),
            }) => {
                assert_eq!(args.filters.metric_sets, vec!["engine.pipeline"]);
                assert_eq!(args.filters.metric_names, vec!["pending.sends"]);
                assert_eq!(args.filters.pipeline_group_id.as_deref(), Some("tenant-a"));
                assert_eq!(args.filters.pipeline_id.as_deref(), Some("ingest"));
                assert_eq!(args.filters.node_id.as_deref(), Some("receiver"));
                assert_eq!(args.filters.core_id, Some(2));
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }
}
