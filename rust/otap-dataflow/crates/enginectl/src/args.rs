// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Clap argument definitions and effective-default handling for `dfctl`.
//!
//! This module defines the public command tree, shared flags, output modes,
//! validation conflicts, and parsers used by both human and agent-oriented
//! command execution. Keeping the argument model centralized lets the command
//! catalog, shell completions, generated help, and runtime dispatch stay aligned
//! with the same Clap source of truth.

use crate::BIN_NAME;
use clap::parser::ValueSource;
use clap::{
    ArgAction, ArgMatches, Args, CommandFactory, FromArgMatches, Parser, Subcommand, ValueEnum,
};
use clap_complete::Shell;
use std::ffi::OsString;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Parser, Debug, Clone)]
#[command(
    name = BIN_NAME,
    version,
    about = "Control CLI for the OTAP Dataflow Engine admin API"
)]
/// Top-level parsed CLI arguments for `dfctl`.
pub struct Cli {
    /// Enable automation-safe defaults for output, errors, and colors.
    #[arg(
        long,
        env = "DFCTL_AGENT_MODE",
        global = true,
        action = ArgAction::SetTrue,
        default_value_t = false,
        value_parser = clap::value_parser!(bool)
    )]
    pub agent: bool,

    /// Shared connection and transport configuration.
    #[command(flatten)]
    pub connection: ConnectionArgs,

    /// Increase diagnostic output on stderr. Repeat for more detail.
    #[arg(
        short,
        long,
        global = true,
        action = ArgAction::Count,
        conflicts_with = "quiet"
    )]
    pub verbose: u8,

    /// Suppress non-error diagnostic output.
    #[arg(short, long, global = true, default_value_t = false)]
    pub quiet: bool,

    /// Format used when reporting runtime errors on stderr.
    #[arg(long, global = true, value_enum, default_value_t = ErrorFormat::Text)]
    pub error_format: ErrorFormat,

    /// Terminal color policy for human-readable output.
    #[arg(long, global = true, value_enum, default_value_t = ColorChoice::Auto)]
    pub color: ColorChoice,

    /// Top-level command to execute.
    #[command(subcommand)]
    pub command: Command,
}

impl Cli {
    /// Parses command-line arguments and applies agent-mode defaults.
    #[must_use]
    pub fn parse_effective() -> Self {
        let matches = Self::command().get_matches();
        Self::from_arg_matches_with_effective_defaults(&matches).unwrap_or_else(|err| err.exit())
    }

    /// Parses supplied arguments and applies agent-mode defaults.
    pub fn try_parse_effective_from<I, T>(itr: I) -> Result<Self, clap::Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let matches = Self::command().try_get_matches_from(itr)?;
        Self::from_arg_matches_with_effective_defaults(&matches)
    }

    fn from_arg_matches_with_effective_defaults(matches: &ArgMatches) -> Result<Self, clap::Error> {
        let error_format_is_default =
            matches.value_source("error_format") == Some(ValueSource::DefaultValue);
        let color_is_default = matches.value_source("color") == Some(ValueSource::DefaultValue);
        let output_is_default =
            leaf_value_source(matches, "output") == Some(ValueSource::DefaultValue);
        let mut cli = Self::from_arg_matches(matches)?;

        if cli.agent {
            if error_format_is_default {
                cli.error_format = ErrorFormat::AgentJson;
            }
            if color_is_default {
                cli.color = ColorChoice::Never;
            }
            if output_is_default {
                cli.command.apply_agent_output_default();
            }
        }

        Ok(cli)
    }

    /// Returns true when the parsed command should launch the interactive UI.
    #[must_use]
    pub const fn is_ui(&self) -> bool {
        matches!(self.command, Command::Ui(_))
    }

    /// Returns true when non-error diagnostics should be suppressed.
    #[must_use]
    pub const fn is_quiet(&self) -> bool {
        self.quiet
    }

    /// Returns the configured runtime error output format.
    #[must_use]
    pub const fn error_format(&self) -> ErrorFormat {
        self.error_format
    }
}

fn leaf_value_source(matches: &ArgMatches, id: &str) -> Option<ValueSource> {
    let mut current = matches;
    while let Some((_, subcommand)) = current.subcommand() {
        current = subcommand;
    }
    current
        .ids()
        .any(|candidate| candidate.as_str() == id)
        .then(|| current.value_source(id))
        .flatten()
}

/// Global connection flags shared by commands that contact the admin API.
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

/// Top-level `dfctl` command tree.
#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    Completions(CompletionArgs),
    Commands(CommandsArgs),
    Schemas(SchemasArgs),
    Config(ConfigArgs),
    Ui(UiArgs),
    Engine(EngineArgs),
    Groups(GroupsArgs),
    Pipelines(PipelinesArgs),
    Telemetry(TelemetryArgs),
}

impl Command {
    fn apply_agent_output_default(&mut self) {
        match self {
            Command::Completions(_) | Command::Ui(_) => {}
            Command::Commands(args) => args.output.output = ReadOutput::AgentJson,
            Command::Schemas(args) => args.output.output = ReadOutput::AgentJson,
            Command::Config(args) => match &mut args.command {
                ConfigCommand::View(output) => output.output = ReadOutput::AgentJson,
            },
            Command::Engine(args) => match &mut args.command {
                EngineCommand::Status(output)
                | EngineCommand::Livez(output)
                | EngineCommand::Readyz(output) => output.output = ReadOutput::AgentJson,
            },
            Command::Groups(args) => apply_group_agent_output_default(args),
            Command::Pipelines(args) => apply_pipeline_agent_output_default(args),
            Command::Telemetry(args) => apply_telemetry_agent_output_default(args),
        }
    }
}

fn apply_group_agent_output_default(args: &mut GroupsArgs) {
    match &mut args.command {
        GroupsCommand::Describe(output)
        | GroupsCommand::Status(output)
        | GroupsCommand::Diagnose(GroupDiagnoseArgs {
            command: GroupDiagnoseCommand::Shutdown(GroupDiagnoseShutdownArgs { output, .. }),
        }) => output.output = ReadOutput::AgentJson,
        GroupsCommand::Events(events) => match &mut events.command {
            GroupEventsCommand::Get(args) => args.output.output = ReadOutput::AgentJson,
            GroupEventsCommand::Watch(args) => args.output.output = StreamOutput::Ndjson,
        },
        GroupsCommand::Bundle(args) => args.output = BundleOutput::AgentJson,
        GroupsCommand::Shutdown(args) => {
            args.output = if args.watch {
                GroupShutdownOutput::Ndjson
            } else {
                GroupShutdownOutput::AgentJson
            };
        }
    }
}

fn apply_pipeline_agent_output_default(args: &mut PipelinesArgs) {
    match &mut args.command {
        PipelinesCommand::Get(args)
        | PipelinesCommand::Describe(args)
        | PipelinesCommand::Status(args)
        | PipelinesCommand::Livez(args)
        | PipelinesCommand::Readyz(args) => args.output.output = ReadOutput::AgentJson,
        PipelinesCommand::Events(events) => match &mut events.command {
            PipelineEventsCommand::Get(args) => args.output.output = ReadOutput::AgentJson,
            PipelineEventsCommand::Watch(args) => args.output.output = StreamOutput::Ndjson,
        },
        PipelinesCommand::Diagnose(diagnose) => match &mut diagnose.command {
            PipelineDiagnoseCommand::Rollout(args) => args.output.output = ReadOutput::AgentJson,
            PipelineDiagnoseCommand::Shutdown(args) => args.output.output = ReadOutput::AgentJson,
        },
        PipelinesCommand::Bundle(args) => args.output = BundleOutput::AgentJson,
        PipelinesCommand::Reconfigure(args) => {
            args.output = if args.watch {
                MutationOutput::Ndjson
            } else {
                MutationOutput::AgentJson
            };
        }
        PipelinesCommand::Shutdown(args) => {
            args.output = if args.watch {
                MutationOutput::Ndjson
            } else {
                MutationOutput::AgentJson
            };
        }
        PipelinesCommand::Rollouts(args) => match &mut args.command {
            RolloutCommand::Get(args) => args.output.output = ReadOutput::AgentJson,
            RolloutCommand::Watch(args) => args.output.output = StreamOutput::Ndjson,
        },
        PipelinesCommand::RolloutStatus(args) => args.output.output = ReadOutput::AgentJson,
        PipelinesCommand::Shutdowns(args) => match &mut args.command {
            ShutdownCommand::Get(args) => args.output.output = ReadOutput::AgentJson,
            ShutdownCommand::Watch(args) => args.output.output = StreamOutput::Ndjson,
        },
        PipelinesCommand::ShutdownStatus(args) => args.output.output = ReadOutput::AgentJson,
    }
}

fn apply_telemetry_agent_output_default(args: &mut TelemetryArgs) {
    match &mut args.command {
        TelemetryCommand::Logs(args) => match &mut args.command {
            LogsCommand::Get(args) => args.output.output = ReadOutput::AgentJson,
            LogsCommand::Watch(args) => args.output.output = StreamOutput::Ndjson,
        },
        TelemetryCommand::Metrics(args) => match &mut args.command {
            MetricsCommand::Get(args) => args.output.output = ReadOutput::AgentJson,
            MetricsCommand::Watch(args) => args.output.output = StreamOutput::Ndjson,
        },
    }
}

/// Arguments for shell completion generation or installation.
#[derive(Args, Debug, Clone)]
#[command(args_conflicts_with_subcommands = true)]
pub struct CompletionArgs {
    #[command(subcommand)]
    pub command: Option<CompletionCommand>,

    #[arg(value_enum)]
    pub shell: Option<Shell>,
}

/// Shell completion subcommands.
#[derive(Subcommand, Debug, Clone)]
pub enum CompletionCommand {
    Install(CompletionInstallArgs),
}

/// Arguments for installing a generated completion script.
#[derive(Args, Debug, Clone)]
pub struct CompletionInstallArgs {
    #[arg(value_enum)]
    pub shell: Shell,

    #[arg(long, value_name = "DIR")]
    pub dir: Option<PathBuf>,
}

/// Arguments for the machine-readable command catalog command.
#[derive(Args, Debug, Clone)]
pub struct CommandsArgs {
    #[command(flatten)]
    pub output: ReadOutputArgs,
}

/// Arguments for schema catalog and schema document discovery.
#[derive(Args, Debug, Clone)]
pub struct SchemasArgs {
    pub name: Option<String>,

    #[command(flatten)]
    pub output: ReadOutputArgs,
}

/// Arguments for client-side configuration inspection commands.
#[derive(Args, Debug, Clone)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommand,
}

/// Client-side configuration inspection subcommands.
#[derive(Subcommand, Debug, Clone)]
pub enum ConfigCommand {
    View(ReadOutputArgs),
}

/// Arguments for launching the interactive TUI.
#[derive(Args, Debug, Clone)]
pub struct UiArgs {
    #[arg(long, value_enum, default_value_t = UiStartView::Pipelines)]
    pub start_view: UiStartView,

    #[arg(long, value_parser = parse_poll_interval_arg, default_value = "2s")]
    pub refresh_interval: Duration,

    #[arg(long, default_value_t = 200)]
    pub logs_tail: usize,
}

/// Arguments for engine-level health and status commands.
#[derive(Args, Debug, Clone)]
pub struct EngineArgs {
    #[command(subcommand)]
    pub command: EngineCommand,
}

/// Engine-level command set.
#[derive(Subcommand, Debug, Clone)]
pub enum EngineCommand {
    Status(ReadOutputArgs),
    Livez(ReadOutputArgs),
    Readyz(ReadOutputArgs),
}

/// Arguments for group-level commands that operate across pipeline groups.
#[derive(Args, Debug, Clone)]
pub struct GroupsArgs {
    #[command(subcommand)]
    pub command: GroupsCommand,
}

/// Group-level command set.
#[derive(Subcommand, Debug, Clone)]
pub enum GroupsCommand {
    Describe(ReadOutputArgs),
    Events(GroupEventsArgs),
    Diagnose(GroupDiagnoseArgs),
    Bundle(GroupBundleArgs),
    Status(ReadOutputArgs),
    Shutdown(GroupShutdownArgs),
}

/// Arguments for commands scoped to a single pipeline or pipeline operation.
#[derive(Args, Debug, Clone)]
pub struct PipelinesArgs {
    #[command(subcommand)]
    pub command: PipelinesCommand,
}

/// Pipeline command set, including status, diagnosis, mutation, and operation lookup commands.
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

/// Arguments for pipeline rollout operation lookup and watch commands.
#[derive(Args, Debug, Clone)]
pub struct PipelineRolloutsArgs {
    #[command(subcommand)]
    pub command: RolloutCommand,
}

/// Pipeline rollout operation command set.
#[derive(Subcommand, Debug, Clone)]
pub enum RolloutCommand {
    Get(RolloutLookupArgs),
    Watch(RolloutWatchArgs),
}

/// Arguments for pipeline shutdown operation lookup and watch commands.
#[derive(Args, Debug, Clone)]
pub struct PipelineShutdownsArgs {
    #[command(subcommand)]
    pub command: ShutdownCommand,
}

/// Pipeline shutdown operation command set.
#[derive(Subcommand, Debug, Clone)]
pub enum ShutdownCommand {
    Get(ShutdownLookupArgs),
    Watch(ShutdownWatchArgs),
}

/// Arguments for telemetry commands exposed by the admin API.
#[derive(Args, Debug, Clone)]
pub struct TelemetryArgs {
    #[command(subcommand)]
    pub command: TelemetryCommand,
}

/// Telemetry command set.
#[derive(Subcommand, Debug, Clone)]
pub enum TelemetryCommand {
    Logs(LogsArgs),
    Metrics(MetricsArgs),
}

/// Arguments for retained-log commands.
#[derive(Args, Debug, Clone)]
pub struct LogsArgs {
    #[command(subcommand)]
    pub command: LogsCommand,
}

/// Retained-log command set.
#[derive(Subcommand, Debug, Clone)]
pub enum LogsCommand {
    Get(LogsGetArgs),
    Watch(LogsWatchArgs),
}

/// Arguments for metrics snapshot and watch commands.
#[derive(Args, Debug, Clone)]
pub struct MetricsArgs {
    #[command(subcommand)]
    pub command: MetricsCommand,
}

/// Metrics command set.
#[derive(Subcommand, Debug, Clone)]
pub enum MetricsCommand {
    Get(MetricsGetArgs),
    Watch(MetricsWatchArgs),
}

/// Initial top-level view selected when the TUI starts.
#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum UiStartView {
    Pipelines,
    Groups,
    Engine,
}

/// Arguments for group-level event commands.
#[derive(Args, Debug, Clone)]
pub struct GroupEventsArgs {
    #[command(subcommand)]
    pub command: GroupEventsCommand,
}

/// Group-level event command set.
#[derive(Subcommand, Debug, Clone)]
pub enum GroupEventsCommand {
    Get(GroupEventsGetArgs),
    Watch(GroupEventsWatchArgs),
}

/// Arguments for pipeline-scoped event commands.
#[derive(Args, Debug, Clone)]
pub struct PipelineEventsArgs {
    #[command(subcommand)]
    pub command: PipelineEventsCommand,
}

/// Pipeline-scoped event command set.
#[derive(Subcommand, Debug, Clone)]
pub enum PipelineEventsCommand {
    Get(PipelineEventsGetArgs),
    Watch(PipelineEventsWatchArgs),
}

/// Arguments for group-level diagnosis commands.
#[derive(Args, Debug, Clone)]
pub struct GroupDiagnoseArgs {
    #[command(subcommand)]
    pub command: GroupDiagnoseCommand,
}

/// Group-level diagnosis command set.
#[derive(Subcommand, Debug, Clone)]
pub enum GroupDiagnoseCommand {
    Shutdown(GroupDiagnoseShutdownArgs),
}

/// Arguments for pipeline-level diagnosis commands.
#[derive(Args, Debug, Clone)]
pub struct PipelineDiagnoseArgs {
    #[command(subcommand)]
    pub command: PipelineDiagnoseCommand,
}

/// Pipeline-level diagnosis command set.
#[derive(Subcommand, Debug, Clone)]
pub enum PipelineDiagnoseCommand {
    Rollout(PipelineDiagnoseRolloutArgs),
    Shutdown(PipelineDiagnoseShutdownArgs),
}

/// Positional coordinates for a pipeline within a group.
#[derive(Args, Debug, Clone)]
pub struct PipelineTargetArgs {
    pub pipeline_group_id: String,
    pub pipeline_id: String,
}

/// Positional coordinates for a rollout operation.
#[derive(Args, Debug, Clone)]
pub struct RolloutTargetArgs {
    pub pipeline_group_id: String,
    pub pipeline_id: String,
    pub rollout_id: String,
}

/// Positional coordinates for a shutdown operation.
#[derive(Args, Debug, Clone)]
pub struct ShutdownTargetArgs {
    pub pipeline_group_id: String,
    pub pipeline_id: String,
    pub shutdown_id: String,
}

/// Client-side filters applied to group event snapshots or watches.
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

/// Client-side filters applied to pipeline event snapshots or watches.
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

/// Client-side filters applied to retained admin logs.
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

/// Client-side filters applied to metrics snapshots or watches.
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

/// Common arguments for read-only pipeline commands.
#[derive(Args, Debug, Clone)]
pub struct PipelineReadArgs {
    #[command(flatten)]
    pub target: PipelineTargetArgs,

    #[command(flatten)]
    pub output: ReadOutputArgs,
}

/// Arguments for fetching one rollout status snapshot.
#[derive(Args, Debug, Clone)]
pub struct RolloutLookupArgs {
    #[command(flatten)]
    pub target: RolloutTargetArgs,

    #[command(flatten)]
    pub output: ReadOutputArgs,
}

/// Arguments for fetching one shutdown status snapshot.
#[derive(Args, Debug, Clone)]
pub struct ShutdownLookupArgs {
    #[command(flatten)]
    pub target: ShutdownTargetArgs,

    #[command(flatten)]
    pub output: ReadOutputArgs,
}

/// Arguments for initiating and optionally observing a group shutdown.
#[derive(Args, Debug, Clone)]
pub struct GroupShutdownArgs {
    #[arg(long, default_value_t = false, conflicts_with = "watch")]
    pub dry_run: bool,

    #[arg(long, default_value_t = false, conflicts_with = "watch")]
    pub wait: bool,

    #[arg(long, value_parser = parse_duration_arg, default_value = "60s")]
    pub wait_timeout: Duration,

    #[arg(long, default_value_t = false, conflicts_with = "wait")]
    pub watch: bool,

    #[arg(long = "watch-interval", value_parser = parse_poll_interval_arg, default_value = "2s")]
    pub watch_interval: Duration,

    #[arg(long, value_enum, default_value_t = GroupShutdownOutput::Human)]
    pub output: GroupShutdownOutput,
}

/// Arguments for redeploying a pipeline from a local configuration file.
#[derive(Args, Debug, Clone)]
pub struct ReconfigureArgs {
    #[command(flatten)]
    pub target: PipelineTargetArgs,

    #[arg(long, value_name = "PATH", required = true)]
    pub file: PathBuf,

    #[arg(long, default_value_t = false, conflicts_with = "watch")]
    pub dry_run: bool,

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

    #[arg(long, value_parser = parse_poll_interval_arg, default_value = "2s")]
    pub watch_interval: Duration,

    #[arg(long, value_enum, default_value_t = MutationOutput::Human)]
    pub output: MutationOutput,
}

/// Arguments for initiating and optionally observing one pipeline shutdown.
#[derive(Args, Debug, Clone)]
pub struct PipelineShutdownArgs {
    #[command(flatten)]
    pub target: PipelineTargetArgs,

    #[arg(long, default_value_t = false, conflicts_with = "watch")]
    pub dry_run: bool,

    #[arg(long, default_value_t = false, conflicts_with = "watch")]
    pub wait: bool,

    #[arg(long, value_parser = parse_duration_arg, default_value = "60s")]
    pub wait_timeout: Duration,

    #[arg(long, default_value_t = false, conflicts_with = "wait")]
    pub watch: bool,

    #[arg(long, value_parser = parse_poll_interval_arg, default_value = "2s")]
    pub watch_interval: Duration,

    #[arg(long, value_enum, default_value_t = MutationOutput::Human)]
    pub output: MutationOutput,
}

/// Arguments for watching one rollout operation until interrupted or terminal.
#[derive(Args, Debug, Clone)]
pub struct RolloutWatchArgs {
    #[command(flatten)]
    pub target: RolloutTargetArgs,

    #[arg(long, value_parser = parse_poll_interval_arg, default_value = "2s")]
    pub interval: Duration,

    #[command(flatten)]
    pub output: StreamOutputArgs,
}

/// Arguments for watching one shutdown operation until interrupted or terminal.
#[derive(Args, Debug, Clone)]
pub struct ShutdownWatchArgs {
    #[command(flatten)]
    pub target: ShutdownTargetArgs,

    #[arg(long, value_parser = parse_poll_interval_arg, default_value = "2s")]
    pub interval: Duration,

    #[command(flatten)]
    pub output: StreamOutputArgs,
}

/// Arguments for reading a filtered group event snapshot.
#[derive(Args, Debug, Clone)]
pub struct GroupEventsGetArgs {
    #[command(flatten)]
    pub filters: GroupEventsFilterArgs,

    #[command(flatten)]
    pub output: ReadOutputArgs,
}

/// Arguments for watching group events by polling group status.
#[derive(Args, Debug, Clone)]
pub struct GroupEventsWatchArgs {
    #[command(flatten)]
    pub filters: GroupEventsFilterArgs,

    #[arg(long = "watch-interval", value_parser = parse_poll_interval_arg, default_value = "2s")]
    pub interval: Duration,

    #[command(flatten)]
    pub output: StreamOutputArgs,
}

/// Arguments for reading a filtered pipeline event snapshot.
#[derive(Args, Debug, Clone)]
pub struct PipelineEventsGetArgs {
    #[command(flatten)]
    pub target: PipelineTargetArgs,

    #[command(flatten)]
    pub filters: PipelineEventsFilterArgs,

    #[command(flatten)]
    pub output: ReadOutputArgs,
}

/// Arguments for watching pipeline events by polling pipeline status.
#[derive(Args, Debug, Clone)]
pub struct PipelineEventsWatchArgs {
    #[command(flatten)]
    pub target: PipelineTargetArgs,

    #[command(flatten)]
    pub filters: PipelineEventsFilterArgs,

    #[arg(long = "watch-interval", value_parser = parse_poll_interval_arg, default_value = "2s")]
    pub interval: Duration,

    #[command(flatten)]
    pub output: StreamOutputArgs,
}

/// Arguments for diagnosing coordinated group shutdown state.
#[derive(Args, Debug, Clone)]
pub struct GroupDiagnoseShutdownArgs {
    #[arg(long, default_value_t = 200)]
    pub logs_limit: usize,

    #[command(flatten)]
    pub output: ReadOutputArgs,
}

/// Arguments for diagnosing a pipeline rollout, optionally by operation id.
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

/// Arguments for diagnosing a pipeline shutdown, optionally by operation id.
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

/// Arguments for collecting a group-scoped troubleshooting bundle.
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

/// Arguments for collecting a pipeline-scoped troubleshooting bundle.
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

/// Arguments for reading retained logs once.
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

/// Arguments for following retained logs by polling the admin API.
#[derive(Args, Debug, Clone)]
pub struct LogsWatchArgs {
    #[arg(long, conflicts_with = "tail")]
    pub after: Option<u64>,

    #[arg(long)]
    pub tail: Option<usize>,

    #[arg(long)]
    pub limit: Option<usize>,

    #[arg(long, value_parser = parse_poll_interval_arg, default_value = "2s")]
    pub interval: Duration,

    #[command(flatten)]
    pub filters: LogsFilterArgs,

    #[command(flatten)]
    pub output: StreamOutputArgs,
}

/// Arguments for reading one metrics snapshot.
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

/// Arguments for following metrics snapshots by polling the admin API.
#[derive(Args, Debug, Clone)]
pub struct MetricsWatchArgs {
    #[arg(long, value_enum, default_value_t = MetricsShape::Compact)]
    pub shape: MetricsShape,

    #[arg(long, default_value_t = false)]
    pub reset: bool,

    #[arg(long, default_value_t = false)]
    pub keep_all_zeroes: bool,

    #[arg(long, value_parser = parse_poll_interval_arg, default_value = "2s")]
    pub interval: Duration,

    #[command(flatten)]
    pub filters: MetricsFilterArgs,

    #[command(flatten)]
    pub output: StreamOutputArgs,
}

/// Shared output selector for finite read-only command responses.
#[derive(Args, Debug, Clone)]
pub struct ReadOutputArgs {
    #[arg(long, value_enum, default_value_t = ReadOutput::Human)]
    pub output: ReadOutput,
}

/// Shared output selector for long-running stream commands.
#[derive(Args, Debug, Clone)]
pub struct StreamOutputArgs {
    #[arg(long, value_enum, default_value_t = StreamOutput::Human)]
    pub output: StreamOutput,
}

/// Output modes supported by finite read-only command responses.
#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum ReadOutput {
    Human,
    Json,
    Yaml,
    AgentJson,
}

/// Output modes supported by long-running stream commands.
#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum StreamOutput {
    Human,
    Ndjson,
}

/// Output modes supported by pipeline mutation commands.
#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum MutationOutput {
    Human,
    Json,
    Yaml,
    Ndjson,
    AgentJson,
}

/// Output modes supported by group shutdown commands.
#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum GroupShutdownOutput {
    Human,
    Json,
    Yaml,
    Ndjson,
    AgentJson,
}

/// Output modes supported by support bundle commands.
#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum BundleOutput {
    Json,
    Yaml,
    AgentJson,
}

/// Runtime error output format for stderr diagnostics.
#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum ErrorFormat {
    /// Human-readable single-line errors.
    Text,
    /// Compact JSON error object for scripts.
    Json,
    /// Agent-oriented JSON envelope with schema version and timestamp.
    AgentJson,
}

/// Metrics payload shape requested from the admin API.
#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum MetricsShape {
    Compact,
    Full,
}

/// Normalized event kind filters accepted by event commands.
#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum EventKind {
    Request,
    Success,
    Error,
    Log,
}

/// Terminal color policy for human-readable output.
#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum ColorChoice {
    Auto,
    Always,
    Never,
}

/// Parses human-friendly duration arguments accepted by CLI flags.
pub fn parse_duration_arg(raw: &str) -> Result<Duration, String> {
    humantime::parse_duration(raw).map_err(|err| err.to_string())
}

/// Parses and validates polling intervals for watch-style commands.
pub fn parse_poll_interval_arg(raw: &str) -> Result<Duration, String> {
    let duration = parse_duration_arg(raw)?;
    let minimum = Duration::from_millis(100);
    if duration < minimum {
        return Err("poll intervals must be at least 100ms".to_string());
    }
    Ok(duration)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: the legacy `pipelines details` alias is used instead of the
    /// canonical `pipelines get` command.
    /// Guarantees: clap still maps the alias onto the same get subcommand and
    /// preserves the target coordinates.
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

    /// Scenario: a caller passes an explicit global color flag before the
    /// selected command.
    /// Guarantees: the parsed CLI stores the requested color policy so later
    /// rendering decisions see the exact override.
    #[test]
    fn color_flag_parses() {
        let cli = Cli::try_parse_from(["dfctl", "--color", "always", "engine", "status"])
            .expect("color flag should parse");

        assert_eq!(cli.color, ColorChoice::Always);
    }

    /// Scenario: the interactive UI command is launched with non-default
    /// startup and refresh arguments.
    /// Guarantees: the parsed UI args preserve the selected start view,
    /// refresh interval, and retained log tail size.
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

    /// Scenario: the interactive UI command is parsed through the effective
    /// parser used by the real binary entrypoint.
    /// Guarantees: agent-default output detection ignores commands without an
    /// `--output` argument instead of asking clap for an unknown argument id.
    #[test]
    fn ui_effective_parse_ignores_missing_output_arg() {
        let cli = Cli::try_parse_effective_from(["dfctl", "ui"])
            .expect("ui command should parse through the effective parser");

        assert!(cli.is_ui());
        assert!(!cli.agent);
    }

    /// Scenario: an operator requests shell completion generation.
    /// Guarantees: the completion command parses without requiring a runtime
    /// admin connection.
    #[test]
    fn completions_command_parses_shell() {
        let cli = Cli::try_parse_from(["dfctl", "completions", "bash"])
            .expect("completion command should parse");
        match cli.command {
            Command::Completions(args) => {
                assert_eq!(args.shell, Some(Shell::Bash));
                assert!(args.command.is_none());
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    /// Scenario: an operator installs completions into the default directory
    /// for a specific shell.
    /// Guarantees: clap treats `install` as a subcommand while preserving the
    /// existing direct `dfctl completions <shell>` generation syntax.
    #[test]
    fn completions_install_command_parses_shell() {
        let cli = Cli::try_parse_from(["dfctl", "completions", "install", "zsh"])
            .expect("completion install command should parse");
        match cli.command {
            Command::Completions(args) => match args.command {
                Some(CompletionCommand::Install(install)) => {
                    assert_eq!(install.shell, Shell::Zsh);
                    assert!(install.dir.is_none());
                    assert!(args.shell.is_none());
                }
                other => panic!("unexpected completion command: {other:?}"),
            },
            other => panic!("unexpected command: {other:?}"),
        }
    }

    /// Scenario: an agent asks for the generated command catalog in JSON mode.
    /// Guarantees: the top-level discovery command parses locally and carries
    /// the standard read-output selection.
    #[test]
    fn commands_catalog_command_parses_output() {
        let cli = Cli::try_parse_from(["dfctl", "commands", "--output", "json"])
            .expect("commands catalog command should parse");

        match cli.command {
            Command::Commands(args) => assert_eq!(args.output.output, ReadOutput::Json),
            other => panic!("unexpected command: {other:?}"),
        }
    }

    /// Scenario: an automation client enables agent mode without per-command
    /// output flags.
    /// Guarantees: effective parsing switches default data, error, and color
    /// modes to automation-safe values while preserving the selected command.
    #[test]
    fn agent_mode_applies_effective_defaults() {
        let cli = Cli::try_parse_effective_from(["dfctl", "--agent", "engine", "status"])
            .expect("agent command should parse");

        assert!(cli.agent);
        assert_eq!(cli.error_format, ErrorFormat::AgentJson);
        assert_eq!(cli.color, ColorChoice::Never);
        match cli.command {
            Command::Engine(EngineArgs {
                command: EngineCommand::Status(output),
            }) => assert_eq!(output.output, ReadOutput::AgentJson),
            other => panic!("unexpected command: {other:?}"),
        }
    }

    /// Scenario: an automation client enables agent mode but also passes
    /// explicit output, error, and color flags.
    /// Guarantees: explicit command-line flags retain precedence over agent
    /// defaults.
    #[test]
    fn agent_mode_preserves_explicit_flags() {
        let cli = Cli::try_parse_effective_from([
            "dfctl",
            "--agent",
            "--error-format",
            "text",
            "--color",
            "always",
            "engine",
            "status",
            "--output",
            "json",
        ])
        .expect("agent command should parse");

        assert_eq!(cli.error_format, ErrorFormat::Text);
        assert_eq!(cli.color, ColorChoice::Always);
        match cli.command {
            Command::Engine(EngineArgs {
                command: EngineCommand::Status(output),
            }) => assert_eq!(output.output, ReadOutput::Json),
            other => panic!("unexpected command: {other:?}"),
        }
    }

    /// Scenario: agent mode is used with a long-running watch command.
    /// Guarantees: effective parsing selects newline-delimited JSON instead of
    /// the human stream renderer.
    #[test]
    fn agent_mode_defaults_watch_to_ndjson() {
        let cli = Cli::try_parse_effective_from(["dfctl", "--agent", "telemetry", "logs", "watch"])
            .expect("watch command should parse");

        match cli.command {
            Command::Telemetry(TelemetryArgs {
                command:
                    TelemetryCommand::Logs(LogsArgs {
                        command: LogsCommand::Watch(args),
                    }),
            }) => assert_eq!(args.output.output, StreamOutput::Ndjson),
            other => panic!("unexpected command: {other:?}"),
        }
    }

    /// Scenario: an agent discovers output schemas without resolving an admin
    /// endpoint.
    /// Guarantees: the schemas command parses as a local read-style command.
    #[test]
    fn schemas_command_parses_name_and_output() {
        let cli = Cli::try_parse_from(["dfctl", "schemas", "dfctl.error.v1", "--output", "json"])
            .expect("schemas command should parse");

        match cli.command {
            Command::Schemas(args) => {
                assert_eq!(args.name.as_deref(), Some("dfctl.error.v1"));
                assert_eq!(args.output.output, ReadOutput::Json);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    /// Scenario: a polling interval is configured as zero.
    /// Guarantees: the parser rejects intervals that would spin or panic in
    /// long-running watch and TUI loops.
    #[test]
    fn poll_intervals_reject_zero_duration() {
        let result = Cli::try_parse_from(["dfctl", "ui", "--refresh-interval", "0s"]);
        assert!(result.is_err());
    }

    /// Scenario: a caller asks the group shutdown command to both wait for
    /// completion and stream watch output.
    /// Guarantees: clap rejects the incompatible flag combination before any
    /// runtime request is attempted.
    #[test]
    fn groups_shutdown_watch_conflicts_with_wait() {
        let result = Cli::try_parse_from(["dfctl", "groups", "shutdown", "--wait", "--watch"]);

        assert!(result.is_err());
    }

    /// Scenario: a high-impact mutation is checked in preflight mode while
    /// also requesting stream output.
    /// Guarantees: clap rejects dry-run/watch combinations because dry-run
    /// does not create a long-running operation to watch.
    #[test]
    fn pipeline_shutdown_dry_run_conflicts_with_watch() {
        let result = Cli::try_parse_from([
            "dfctl",
            "pipelines",
            "shutdown",
            "tenant-a",
            "ingest",
            "--dry-run",
            "--watch",
        ]);

        assert!(result.is_err());
    }

    /// Scenario: pipeline event watch arguments include scope, kind, content,
    /// and streaming-output filters.
    /// Guarantees: the parsed command preserves every filter and stream option
    /// needed by the event watch implementation.
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

    /// Scenario: telemetry metric filters are supplied on the command line for
    /// metric set, metric name, scope, and core selection.
    /// Guarantees: the parsed filters capture every requested field so the
    /// client-side metric pruning logic sees the intended scope.
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
