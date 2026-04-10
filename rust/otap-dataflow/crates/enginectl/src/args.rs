// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "df_enginectl",
    version,
    about = "Control CLI for the OTAP Dataflow Engine admin API"
)]
/// Top-level parsed CLI arguments for `df_enginectl`.
pub struct Cli {
    /// Shared connection and transport configuration.
    #[command(flatten)]
    pub connection: ConnectionArgs,

    /// Top-level command to execute.
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Args, Debug, Clone, Default)]
pub struct ConnectionArgs {
    #[arg(long, env = "DF_ENGINECTL_URL", global = true, value_name = "URL")]
    pub url: Option<String>,

    #[arg(
        long,
        env = "DF_ENGINECTL_PROFILE_FILE",
        global = true,
        value_name = "PATH"
    )]
    pub profile_file: Option<PathBuf>,

    #[arg(
        long,
        env = "DF_ENGINECTL_CONNECT_TIMEOUT",
        global = true,
        value_parser = parse_duration_arg,
        value_name = "DURATION"
    )]
    pub connect_timeout: Option<Duration>,

    #[arg(
        long,
        env = "DF_ENGINECTL_REQUEST_TIMEOUT",
        global = true,
        value_parser = parse_duration_arg,
        value_name = "DURATION",
        conflicts_with = "no_request_timeout"
    )]
    pub request_timeout: Option<Duration>,

    #[arg(
        long,
        env = "DF_ENGINECTL_NO_REQUEST_TIMEOUT",
        global = true,
        default_value_t = false
    )]
    pub no_request_timeout: bool,

    #[arg(
        long,
        env = "DF_ENGINECTL_TCP_NODELAY",
        global = true,
        value_parser = clap::value_parser!(bool),
        value_name = "BOOL"
    )]
    pub tcp_nodelay: Option<bool>,

    #[arg(
        long,
        env = "DF_ENGINECTL_TCP_KEEPALIVE",
        global = true,
        value_parser = parse_duration_arg,
        value_name = "DURATION"
    )]
    pub tcp_keepalive: Option<Duration>,

    #[arg(
        long,
        env = "DF_ENGINECTL_TCP_KEEPALIVE_INTERVAL",
        global = true,
        value_parser = parse_duration_arg,
        value_name = "DURATION"
    )]
    pub tcp_keepalive_interval: Option<Duration>,

    #[arg(long, env = "DF_ENGINECTL_CA_FILE", global = true, value_name = "PATH")]
    pub ca_file: Option<PathBuf>,

    #[arg(
        long,
        env = "DF_ENGINECTL_CLIENT_CERT_FILE",
        global = true,
        value_name = "PATH"
    )]
    pub client_cert_file: Option<PathBuf>,

    #[arg(
        long,
        env = "DF_ENGINECTL_CLIENT_KEY_FILE",
        global = true,
        value_name = "PATH"
    )]
    pub client_key_file: Option<PathBuf>,

    #[arg(
        long,
        env = "DF_ENGINECTL_INCLUDE_SYSTEM_CA_CERTS",
        global = true,
        value_parser = clap::value_parser!(bool),
        value_name = "BOOL"
    )]
    pub include_system_ca_certs: Option<bool>,

    #[arg(
        long,
        env = "DF_ENGINECTL_INSECURE_SKIP_VERIFY",
        global = true,
        value_parser = clap::value_parser!(bool),
        value_name = "BOOL"
    )]
    pub insecure_skip_verify: Option<bool>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    Engine(EngineArgs),
    Groups(GroupsArgs),
    Pipelines(PipelinesArgs),
    Telemetry(TelemetryArgs),
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
    #[arg(long, default_value_t = false)]
    pub wait: bool,

    #[arg(long, value_parser = parse_duration_arg, default_value = "60s")]
    pub wait_timeout: Duration,

    #[command(flatten)]
    pub output: ReadOutputArgs,
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
pub struct LogsGetArgs {
    #[arg(long)]
    pub after: Option<u64>,

    #[arg(long)]
    pub limit: Option<usize>,

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
pub enum MetricsShape {
    Compact,
    Full,
}

pub fn parse_duration_arg(raw: &str) -> Result<Duration, String> {
    humantime::parse_duration(raw).map_err(|err| err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn details_alias_parses_to_get() {
        let cli =
            Cli::try_parse_from(["df_enginectl", "pipelines", "details", "tenant-a", "ingest"])
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
}
