// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Generated command catalog for humans, scripts, and AI agents.
//!
//! `dfctl --help` is optimized for a person reading one command at a time. It
//! is intentionally free-form text, localized around the selected subcommand,
//! and does not provide stable fields for questions such as "does this command
//! mutate state?", "which output modes are valid?", "can this command run
//! without an admin connection?", or "which schema should an agent expect?".
//! The command catalog fills that gap with a versioned, machine-readable view
//! of the entire command tree.
//!
//! The catalog is generated from Clap's command model instead of being a
//! second hand-written command list. Command paths, aliases, global arguments,
//! local arguments, defaults, value ranges, environment variables, and possible
//! values are read from `Cli::command()`. This keeps the structural part of the
//! catalog synchronized when a new command or argument is added in `args.rs`.
//!
//! Some metadata is necessarily semantic and cannot be inferred reliably from
//! Clap alone: whether a command is high-impact, whether it needs an admin
//! client, which schema best describes its output, whether stdin is accepted,
//! and which examples are useful. When adding a new command or subcommand:
//!
//! - define the command and arguments in `args.rs`;
//! - decide whether it needs entries in `requires_admin_client`, `is_mutation`,
//!   `stdin_support`, `safety`, `output_schema`, `stream_schema`, and
//!   `curated_examples`;
//! - extend catalog tests when the command introduces a new behavior category
//!   or a new contract that agents should depend on.
//!
//! Drift detection is split accordingly. Structural drift is avoided by
//! generating from Clap on every `dfctl commands` invocation. Semantic drift is
//! covered by focused tests at the bottom of this module: they assert that known
//! commands appear, aliases and output modes are visible, local and
//! long-running commands are classified correctly, and high-impact mutations
//! expose safety, dry-run, wait/watch, stdin, and schema metadata.

use crate::Cli;
use crate::args::CommandsArgs;
use crate::commands::output::emit_read;
use crate::commands::schemas::{
    AGENT_ENVELOPE_SCHEMA, COMMAND_CATALOG_SCHEMA, DIAGNOSE_REPORT_SCHEMA, ERROR_SCHEMA,
    JSON_SCHEMA_SCHEMA, MUTATION_OUTCOME_SCHEMA, SCHEMA_CATALOG_SCHEMA, STREAM_EVENT_SCHEMA,
    SUPPORT_BUNDLE_SCHEMA,
};
use crate::error::CliError;
use crate::style::HumanStyle;
use clap::builder::ValueRange;
use clap::{Arg, ArgAction, Command as ClapCommand, CommandFactory};
use serde::Serialize;
use std::collections::BTreeSet;
use std::io::Write;
use std::time::SystemTime;

const SCHEMA_VERSION: &str = "dfctl-command-catalog/v1";
const BIN_NAME: &str = "dfctl";

/// Runs `dfctl commands` and emits either a compact human table or the full
/// machine-readable catalog, depending on the selected output mode.
pub(crate) fn run(
    stdout: &mut dyn Write,
    human_style: HumanStyle,
    args: CommandsArgs,
) -> Result<(), CliError> {
    let catalog = build_catalog();
    emit_read(stdout, args.output.output, &catalog, || {
        Ok(render_human(&human_style, &catalog))
    })
}

/// Builds a complete catalog snapshot from the current Clap command tree.
///
/// This is intentionally recomputed instead of cached or generated at build
/// time so the catalog always reflects the binary being executed.
fn build_catalog() -> CommandCatalog {
    let mut root = Cli::command();
    root.build();

    let global_arguments = visible_arguments(&root)
        .into_iter()
        .filter(|argument| argument.global)
        .collect::<Vec<_>>();

    let mut commands = Vec::new();
    let mut path = Vec::new();
    for command in root.get_subcommands() {
        collect_command(command, &mut path, &mut commands);
    }

    CommandCatalog {
        schema_version: SCHEMA_VERSION,
        generated_at: humantime::format_rfc3339_seconds(SystemTime::now()).to_string(),
        binary: BIN_NAME,
        version: env!("CARGO_PKG_VERSION"),
        global_arguments: global_arguments.clone(),
        commands,
    }
}

/// Recursively converts one Clap command and its children into catalog entries.
///
/// Non-leaf commands are included so agents can understand grouping nodes, but
/// examples and output schemas are only attached when the command is runnable.
fn collect_command(
    command: &ClapCommand,
    path: &mut Vec<String>,
    commands: &mut Vec<CommandEntry>,
) {
    if command.is_hide_set() || command.get_name() == "help" {
        return;
    }

    path.push(command.get_name().to_string());
    let subcommands = command
        .get_subcommands()
        .filter(|subcommand| !subcommand.is_hide_set() && subcommand.get_name() != "help")
        .count();
    let leaf = subcommands == 0;
    let arguments = visible_arguments(command)
        .into_iter()
        .filter(|argument| !argument.global)
        .collect::<Vec<_>>();
    let output_modes = output_modes(&arguments);
    let default_output = default_output(&arguments);
    let agent_default_output = agent_default_output(path, &output_modes);
    let command_line = command_line(path);
    let runnable = leaf
        || arguments
            .iter()
            .any(|argument| argument.kind == ArgumentKind::Positional);
    let examples = if runnable {
        examples_for(path, &arguments, &output_modes)
    } else {
        Vec::new()
    };

    commands.push(CommandEntry {
        id: command_id(path),
        name: command.get_name().to_string(),
        path: path.clone(),
        command_line,
        usage: usage_for(path, &arguments, subcommands > 0),
        about: styled_to_string(command.get_about()),
        long_about: styled_to_string(command.get_long_about()),
        aliases: command.get_all_aliases().map(str::to_string).collect(),
        leaf,
        runnable,
        requires_admin_client: requires_admin_client(path),
        interactive: is_interactive(path),
        long_running: is_long_running(path),
        mutation: is_mutation(path),
        output_modes,
        default_output,
        agent_default_output,
        output_schema: if runnable { output_schema(path) } else { None },
        stream_schema: if runnable { stream_schema(path) } else { None },
        error_schema: error_schema(path),
        stdin: stdin_support(path),
        safety: safety(path),
        idempotent: is_idempotent(path),
        supports_dry_run: supports_dry_run(&arguments),
        supports_wait: supports_flag(&arguments, "wait"),
        supports_watch: supports_flag(&arguments, "watch"),
        exit_codes: exit_codes(path),
        arguments: arguments.clone(),
        examples,
    });

    for subcommand in command.get_subcommands() {
        collect_command(subcommand, path, commands);
    }
    let _ = path.pop();
}

/// Returns all non-hidden, non-generated arguments Clap exposes for a command.
fn visible_arguments(command: &ClapCommand) -> Vec<ArgumentEntry> {
    command
        .get_arguments()
        .filter(|argument| !argument.is_hide_set())
        .filter(|argument| !is_generated_help_argument(argument))
        .map(argument_entry)
        .collect()
}

/// Filters out Clap's generated help arguments so the catalog only describes
/// arguments that are meaningful to invoke explicitly.
fn is_generated_help_argument(argument: &Arg) -> bool {
    matches!(
        argument.get_action(),
        &ArgAction::Help | &ArgAction::HelpShort | &ArgAction::HelpLong
    )
}

/// Converts a Clap argument into the stable argument shape exposed in JSON.
fn argument_entry(argument: &Arg) -> ArgumentEntry {
    let action = argument.get_action();
    let value_range = argument.get_num_args().map(value_range_entry);
    ArgumentEntry {
        id: argument.get_id().to_string(),
        kind: argument_kind(argument, action),
        long: argument.get_long().map(str::to_string),
        short: argument.get_short().map(|short| short.to_string()),
        aliases: argument
            .get_all_aliases()
            .unwrap_or_default()
            .into_iter()
            .map(str::to_string)
            .collect(),
        value_names: argument
            .get_value_names()
            .unwrap_or_default()
            .iter()
            .map(ToString::to_string)
            .collect(),
        possible_values: if argument.is_hide_possible_values_set() {
            Vec::new()
        } else {
            argument
                .get_possible_values()
                .into_iter()
                .filter(|value| !value.is_hide_set())
                .map(|value| PossibleValueEntry {
                    name: value.get_name().to_string(),
                    aliases: value
                        .get_name_and_aliases()
                        .skip(1)
                        .map(str::to_string)
                        .collect(),
                    help: styled_to_string(value.get_help()),
                })
                .collect()
        },
        default_values: if argument.is_hide_default_value_set() {
            Vec::new()
        } else {
            argument
                .get_default_values()
                .iter()
                .map(|value| value.to_string_lossy().to_string())
                .collect()
        },
        env: if argument.is_hide_env_set() {
            None
        } else {
            argument
                .get_env()
                .map(|value| value.to_string_lossy().to_string())
        },
        required: argument.is_required_set(),
        global: argument.is_global_set(),
        multiple: is_multiple(argument, action),
        value_range,
        action: format_action(action),
        help: styled_to_string(argument.get_help()),
        help_heading: argument.get_help_heading().map(str::to_string),
    }
}

/// Classifies a Clap argument into the simpler categories used by the catalog.
fn argument_kind(argument: &Arg, action: &ArgAction) -> ArgumentKind {
    if argument.is_positional() {
        ArgumentKind::Positional
    } else if action.takes_values() {
        ArgumentKind::Option
    } else {
        ArgumentKind::Flag
    }
}

/// Converts Clap's value-range model into a serializable min/max pair.
fn value_range_entry(range: ValueRange) -> ValueRangeEntry {
    ValueRangeEntry {
        min: range.min_values(),
        max: if range.max_values() == usize::MAX {
            None
        } else {
            Some(range.max_values())
        },
    }
}

/// Detects whether an argument may be provided more than once or with multiple
/// values.
fn is_multiple(argument: &Arg, action: &ArgAction) -> bool {
    matches!(action, &ArgAction::Append | &ArgAction::Count)
        || argument
            .get_num_args()
            .map(|range| range.min_values() != range.max_values() || range.max_values() > 1)
            .unwrap_or(false)
}

/// Extracts the supported values of a command-local `--output` argument.
fn output_modes(arguments: &[ArgumentEntry]) -> Vec<String> {
    arguments
        .iter()
        .find(|argument| argument.long.as_deref() == Some("output"))
        .map(|argument| {
            argument
                .possible_values
                .iter()
                .map(|value| value.name.clone())
                .collect()
        })
        .unwrap_or_default()
}

/// Extracts the default value of a command-local `--output` argument.
fn default_output(arguments: &[ArgumentEntry]) -> Option<String> {
    arguments
        .iter()
        .find(|argument| argument.long.as_deref() == Some("output"))
        .and_then(|argument| argument.default_values.first().cloned())
}

/// Reports the output mode agent mode will select when the caller did not pass
/// an explicit output flag.
fn agent_default_output(path: &[String], output_modes: &[String]) -> Option<String> {
    if output_modes.is_empty() {
        return None;
    }
    if is_long_running(path) && output_modes.iter().any(|mode| mode == "ndjson") {
        return Some("ndjson".to_string());
    }
    if output_modes.iter().any(|mode| mode == "agent-json") {
        return Some("agent-json".to_string());
    }
    if output_modes.iter().any(|mode| mode == "json") {
        return Some("json".to_string());
    }
    output_modes.first().cloned()
}

/// Returns true when a command exposes a command-local flag by long name.
fn supports_flag(arguments: &[ArgumentEntry], long: &str) -> bool {
    arguments
        .iter()
        .any(|argument| argument.long.as_deref() == Some(long))
}

/// Returns true when the command exposes the common preflight flag.
fn supports_dry_run(arguments: &[ArgumentEntry]) -> bool {
    supports_flag(arguments, "dry-run")
}

/// Builds a stable, dot-separated command identifier.
fn command_id(path: &[String]) -> String {
    format!("dfctl.{}", path.join(".").replace('-', "_"))
}

/// Builds the shortest command-line prefix for a command path.
fn command_line(path: &[String]) -> String {
    format!("{BIN_NAME} {}", path.join(" "))
}

/// Builds a compact usage string from catalog argument metadata.
fn usage_for(path: &[String], local_arguments: &[ArgumentEntry], has_subcommands: bool) -> String {
    let mut parts = vec![command_line(path)];
    if local_arguments
        .iter()
        .any(|argument| argument.kind != ArgumentKind::Positional)
    {
        parts.push("[OPTIONS]".to_string());
    }
    for argument in local_arguments
        .iter()
        .filter(|argument| argument.kind == ArgumentKind::Positional)
    {
        let placeholder = value_placeholder(argument);
        if argument.required {
            parts.push(format!("<{placeholder}>"));
        } else {
            parts.push(format!("[{placeholder}]"));
        }
    }
    if has_subcommands {
        parts.push("<COMMAND>".to_string());
    }
    parts.join(" ")
}

/// Builds examples from required arguments, curated examples, and output modes.
fn examples_for(
    path: &[String],
    arguments: &[ArgumentEntry],
    output_modes: &[String],
) -> Vec<String> {
    let mut examples = vec![baseline_example(path, arguments)];
    examples.extend(curated_examples(path));

    if output_modes.iter().any(|mode| mode == "json") {
        examples.push(format!("{} --output json", command_line(path)));
    } else if output_modes.iter().any(|mode| mode == "ndjson") {
        examples.push(format!("{} --output ndjson", command_line(path)));
    }

    dedupe(examples)
}

/// Creates a generic runnable command example from required options and
/// positionals.
fn baseline_example(path: &[String], arguments: &[ArgumentEntry]) -> String {
    let mut parts = vec![command_line(path)];

    for argument in arguments.iter().filter(|argument| {
        !argument.global && argument.required && argument.kind == ArgumentKind::Option
    }) {
        if let Some(long) = &argument.long {
            parts.push(format!("--{long}"));
            parts.push(format!("<{}>", value_placeholder(argument)));
        }
    }

    for argument in arguments
        .iter()
        .filter(|argument| !argument.global && argument.kind == ArgumentKind::Positional)
    {
        let placeholder = value_placeholder(argument);
        if argument.required {
            parts.push(format!("<{placeholder}>"));
        } else {
            parts.push(format!("[{placeholder}]"));
        }
    }

    parts.join(" ")
}

/// Provides hand-curated examples for common commands where placeholders alone
/// would not be useful enough for humans or agents.
fn curated_examples(path: &[String]) -> Vec<String> {
    match path
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["commands"] => vec!["dfctl commands --output json".to_string()],
        ["schemas"] => vec![
            "dfctl schemas --output json".to_string(),
            "dfctl schemas dfctl.error.v1 --output json".to_string(),
        ],
        ["completions"] => vec!["dfctl completions bash".to_string()],
        ["completions", "install"] => vec!["dfctl completions install zsh".to_string()],
        ["config", "view"] => vec!["dfctl config view --output json".to_string()],
        ["ui"] => vec!["dfctl ui --start-view pipelines".to_string()],
        ["engine", "status"] => vec!["dfctl engine status --output json".to_string()],
        ["engine", "livez"] => vec!["dfctl engine livez".to_string()],
        ["engine", "readyz"] => vec!["dfctl engine readyz".to_string()],
        ["groups", "status"] => vec!["dfctl groups status".to_string()],
        ["groups", "describe"] => vec!["dfctl groups describe".to_string()],
        ["groups", "events", "get"] => vec!["dfctl groups events get --tail 20".to_string()],
        ["groups", "events", "watch"] => {
            vec!["dfctl groups events watch --kind error --tail 20".to_string()]
        }
        ["groups", "diagnose", "shutdown"] => vec!["dfctl groups diagnose shutdown".to_string()],
        ["groups", "bundle"] => vec!["dfctl groups bundle --file bundle.json".to_string()],
        ["groups", "shutdown"] => vec!["dfctl groups shutdown --watch".to_string()],
        ["pipelines", "get"] => vec!["dfctl pipelines get tenant-a ingest".to_string()],
        ["pipelines", "describe"] => {
            vec!["dfctl pipelines describe tenant-a ingest".to_string()]
        }
        ["pipelines", "status"] => {
            vec!["dfctl pipelines status tenant-a ingest --output json".to_string()]
        }
        ["pipelines", "livez"] => vec!["dfctl pipelines livez tenant-a ingest".to_string()],
        ["pipelines", "readyz"] => vec!["dfctl pipelines readyz tenant-a ingest".to_string()],
        ["pipelines", "events", "get"] => {
            vec!["dfctl pipelines events get tenant-a ingest --tail 20".to_string()]
        }
        ["pipelines", "events", "watch"] => {
            vec!["dfctl pipelines events watch tenant-a ingest --kind error --tail 20".to_string()]
        }
        ["pipelines", "diagnose", "rollout"] => {
            vec!["dfctl pipelines diagnose rollout tenant-a ingest".to_string()]
        }
        ["pipelines", "diagnose", "shutdown"] => {
            vec!["dfctl pipelines diagnose shutdown tenant-a ingest".to_string()]
        }
        ["pipelines", "bundle"] => {
            vec!["dfctl pipelines bundle tenant-a ingest --file bundle.json".to_string()]
        }
        ["pipelines", "reconfigure"] => vec![
            "dfctl pipelines reconfigure tenant-a ingest --file pipeline.yaml --wait".to_string(),
        ],
        ["pipelines", "shutdown"] => {
            vec!["dfctl pipelines shutdown tenant-a ingest --watch".to_string()]
        }
        ["pipelines", "rollouts", "get"] => {
            vec!["dfctl pipelines rollouts get tenant-a ingest rollout-1".to_string()]
        }
        ["pipelines", "rollouts", "watch"] => {
            vec!["dfctl pipelines rollouts watch tenant-a ingest rollout-1".to_string()]
        }
        ["pipelines", "rollout-status"] => {
            vec!["dfctl pipelines rollout-status tenant-a ingest rollout-1".to_string()]
        }
        ["pipelines", "shutdowns", "get"] => {
            vec!["dfctl pipelines shutdowns get tenant-a ingest shutdown-1".to_string()]
        }
        ["pipelines", "shutdowns", "watch"] => {
            vec!["dfctl pipelines shutdowns watch tenant-a ingest shutdown-1".to_string()]
        }
        ["pipelines", "shutdown-status"] => {
            vec!["dfctl pipelines shutdown-status tenant-a ingest shutdown-1".to_string()]
        }
        ["telemetry", "logs", "get"] => {
            vec!["dfctl telemetry logs get --limit 50 --output json".to_string()]
        }
        ["telemetry", "logs", "watch"] => {
            vec!["dfctl telemetry logs watch --tail 50".to_string()]
        }
        ["telemetry", "metrics", "get"] => {
            vec!["dfctl telemetry metrics get --shape compact --output json".to_string()]
        }
        ["telemetry", "metrics", "watch"] => {
            vec!["dfctl telemetry metrics watch --shape compact --output ndjson".to_string()]
        }
        _ => Vec::new(),
    }
}

/// Returns the best display placeholder for an argument value.
fn value_placeholder(argument: &ArgumentEntry) -> String {
    argument
        .value_names
        .first()
        .cloned()
        .unwrap_or_else(|| argument.id.to_ascii_uppercase().replace('-', "_"))
}

/// Removes duplicate examples while preserving the first occurrence order.
fn dedupe(values: Vec<String>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut deduped = Vec::new();
    for value in values {
        if seen.insert(value.clone()) {
            deduped.push(value);
        }
    }
    deduped
}

/// Classifies commands that can run without resolving an admin endpoint.
fn requires_admin_client(path: &[String]) -> bool {
    !matches!(
        path.first().map(String::as_str),
        Some("completions" | "commands" | "schemas" | "config")
    )
}

/// Classifies commands that take over the terminal for interactive operation.
fn is_interactive(path: &[String]) -> bool {
    matches!(path.first().map(String::as_str), Some("ui"))
}

/// Classifies commands that keep running until completion or interruption.
fn is_long_running(path: &[String]) -> bool {
    is_interactive(path) || path.iter().any(|part| part == "watch")
}

/// Classifies commands that can change local or remote state.
fn is_mutation(path: &[String]) -> bool {
    matches!(
        path.iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .as_slice(),
        ["completions", "install"]
            | ["groups", "shutdown"]
            | ["pipelines", "reconfigure"]
            | ["pipelines", "shutdown"]
    )
}

/// Maps runnable commands to the stable schema name that best describes their
/// one-shot output.
fn output_schema(path: &[String]) -> Option<&'static str> {
    match path
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["commands"] => Some(COMMAND_CATALOG_SCHEMA),
        ["schemas"] => Some(SCHEMA_CATALOG_SCHEMA),
        ["schemas", _] => Some(JSON_SCHEMA_SCHEMA),
        ["groups", "bundle"] | ["pipelines", "bundle"] => Some(SUPPORT_BUNDLE_SCHEMA),
        ["groups", "diagnose", _] | ["pipelines", "diagnose", _] => Some(DIAGNOSE_REPORT_SCHEMA),
        ["groups", "shutdown"] | ["pipelines", "reconfigure"] | ["pipelines", "shutdown"] => {
            Some(MUTATION_OUTCOME_SCHEMA)
        }
        _ if is_long_running(path) => None,
        _ if has_read_output(path) => Some(AGENT_ENVELOPE_SCHEMA),
        _ => None,
    }
}

/// Maps long-running commands to the stable stream event schema.
fn stream_schema(path: &[String]) -> Option<&'static str> {
    if is_long_running(path) {
        Some(STREAM_EVENT_SCHEMA)
    } else {
        None
    }
}

/// Maps commands to the structured error schema they may emit on stderr.
fn error_schema(path: &[String]) -> Option<&'static str> {
    if !path.is_empty() {
        Some(ERROR_SCHEMA)
    } else {
        None
    }
}

/// Returns true when a command follows the common read-output contract.
fn has_read_output(path: &[String]) -> bool {
    !matches!(path.first().map(String::as_str), Some("completions" | "ui"))
}

/// Classifies whether a command reads data from stdin.
fn stdin_support(path: &[String]) -> StdinSupport {
    if matches!(
        path.iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .as_slice(),
        ["pipelines", "reconfigure"]
    ) {
        StdinSupport::Optional
    } else {
        StdinSupport::None
    }
}

/// Classifies the expected operational impact of running a command.
fn safety(path: &[String]) -> SafetyLevel {
    match path
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["groups", "shutdown"] | ["pipelines", "reconfigure"] | ["pipelines", "shutdown"] => {
            SafetyLevel::HighImpact
        }
        ["completions", "install"] => SafetyLevel::LowImpact,
        _ => SafetyLevel::Read,
    }
}

/// Reports whether retrying a command is expected to be safe without changing
/// remote state more than once.
fn is_idempotent(path: &[String]) -> bool {
    match safety(path) {
        SafetyLevel::Read => true,
        SafetyLevel::LowImpact => true,
        SafetyLevel::HighImpact | SafetyLevel::Destructive => false,
    }
}

/// Lists the documented process exit codes for a command.
fn exit_codes(path: &[String]) -> Vec<ExitCodeEntry> {
    if requires_admin_client(path) {
        vec![
            ExitCodeEntry::new(0, "success"),
            ExitCodeEntry::new(2, "invalid CLI usage"),
            ExitCodeEntry::new(3, "requested admin resource was not found"),
            ExitCodeEntry::new(4, "admin API rejected the request"),
            ExitCodeEntry::new(5, "accepted operation failed or timed out"),
            ExitCodeEntry::new(
                6,
                "configuration, I/O, transport, decode, or internal error",
            ),
        ]
    } else {
        vec![
            ExitCodeEntry::new(0, "success"),
            ExitCodeEntry::new(2, "invalid CLI usage"),
            ExitCodeEntry::new(6, "configuration or I/O error"),
        ]
    }
}

/// Normalizes Clap actions into simple strings for machine readers.
fn format_action(action: &ArgAction) -> String {
    format!("{action:?}").to_ascii_lowercase()
}

/// Converts Clap's styled help text into plain text suitable for JSON output.
fn styled_to_string(value: Option<&clap::builder::StyledStr>) -> Option<String> {
    value.map(ToString::to_string)
}

/// Renders the compact human view of the catalog.
fn render_human(style: &HumanStyle, catalog: &CommandCatalog) -> String {
    let rows = catalog
        .commands
        .iter()
        .filter(|command| command.runnable)
        .map(|command| {
            vec![
                command.command_line.clone(),
                command.output_modes.join(","),
                command_hint(command),
            ]
        })
        .collect::<Vec<_>>();

    let lines = [
        style.header("dfctl command catalog"),
        format!("{}: {}", style.label("schema"), catalog.schema_version),
        format!("{}: {}", style.label("commands"), catalog.commands.len()),
        String::new(),
        render_table(
            &[
                style.header("command"),
                style.header("output"),
                style.header("hints"),
            ],
            rows,
        ),
        String::new(),
        "Use `dfctl commands --output json` for the machine-readable catalog.".to_string(),
    ];
    lines.join("\n")
}

/// Produces the terse human hint column for one command row.
fn command_hint(command: &CommandEntry) -> String {
    let mut hints = Vec::new();
    if !command.requires_admin_client {
        hints.push("local");
    }
    if command.interactive {
        hints.push("interactive");
    }
    if command.long_running {
        hints.push("long-running");
    }
    if command.mutation {
        hints.push("mutation");
    }
    if hints.is_empty() {
        "read".to_string()
    } else {
        hints.join(",")
    }
}

/// Renders an aligned text table while accounting for ANSI escape sequences.
fn render_table(headers: &[String], rows: Vec<Vec<String>>) -> String {
    let mut widths = headers
        .iter()
        .map(|header| visible_width(header))
        .collect::<Vec<_>>();

    for row in &rows {
        for (index, cell) in row.iter().enumerate() {
            if let Some(width) = widths.get_mut(index) {
                *width = (*width).max(visible_width(cell));
            }
        }
    }

    let mut rendered = Vec::with_capacity(rows.len().saturating_add(2));
    rendered.push(join_row(headers, &widths));
    rendered.push(join_row(
        &widths
            .iter()
            .map(|width| "-".repeat(*width))
            .collect::<Vec<_>>(),
        &widths,
    ));
    for row in rows {
        rendered.push(join_row(&row, &widths));
    }
    rendered.join("\n")
}

/// Joins one table row using precomputed visible widths.
fn join_row(cells: &[String], widths: &[usize]) -> String {
    cells
        .iter()
        .enumerate()
        .map(|(index, cell)| {
            let padding = widths
                .get(index)
                .copied()
                .unwrap_or_default()
                .saturating_sub(visible_width(cell));
            format!("{cell}{}", " ".repeat(padding))
        })
        .collect::<Vec<_>>()
        .join("  ")
}

/// Computes printable width after ignoring ANSI escape sequences.
fn visible_width(text: &str) -> usize {
    let mut width = 0;
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' && chars.peek() == Some(&'[') {
            let _ = chars.next();
            for code in chars.by_ref() {
                if ('@'..='~').contains(&code) {
                    break;
                }
            }
        } else {
            width += 1;
        }
    }
    width
}

/// Top-level JSON document emitted by `dfctl commands --output json`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CommandCatalog {
    /// Schema identifier for consumers that validate or version the catalog.
    schema_version: &'static str,
    /// Timestamp indicating when this binary generated the catalog snapshot.
    generated_at: String,
    /// Binary name used in generated command lines.
    binary: &'static str,
    /// Cargo package version embedded in this binary.
    version: &'static str,
    /// Global arguments inherited by commands, such as connection settings.
    global_arguments: Vec<ArgumentEntry>,
    /// Flattened command tree, including grouping and runnable commands.
    commands: Vec<CommandEntry>,
}

/// One command or command-group entry in the flattened catalog.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CommandEntry {
    /// Stable command identifier, derived from the command path.
    id: String,
    /// Final command segment name.
    name: String,
    /// Command path without the binary name.
    path: Vec<String>,
    /// Runnable command prefix with the binary name.
    command_line: String,
    /// Compact usage string derived from local arguments and subcommands.
    usage: String,
    /// Short help text from Clap.
    about: Option<String>,
    /// Long help text from Clap.
    long_about: Option<String>,
    /// Alternate command names accepted by Clap.
    aliases: Vec<String>,
    /// True when the command has no visible child subcommands.
    leaf: bool,
    /// True when this entry can be directly invoked.
    runnable: bool,
    /// True when the command resolves and calls an admin endpoint.
    requires_admin_client: bool,
    /// True when the command owns the terminal interactively.
    interactive: bool,
    /// True when the command can run until interrupted or terminal state.
    long_running: bool,
    /// True when the command can change local or remote state.
    mutation: bool,
    /// Supported output modes for commands with an `--output` argument.
    output_modes: Vec<String>,
    /// Default output mode declared by Clap.
    default_output: Option<String>,
    /// Output mode selected by `--agent` when no explicit mode is passed.
    agent_default_output: Option<String>,
    /// Schema name for one-shot output, when available.
    output_schema: Option<&'static str>,
    /// Schema name for streamed events, when available.
    stream_schema: Option<&'static str>,
    /// Schema name for structured stderr errors.
    error_schema: Option<&'static str>,
    /// Whether the command reads stdin.
    stdin: StdinSupport,
    /// Operational impact classification.
    safety: SafetyLevel,
    /// Whether repeat execution should be safe without duplicated side effects.
    idempotent: bool,
    /// True when the command supports `--dry-run`.
    supports_dry_run: bool,
    /// True when the command supports `--wait`.
    supports_wait: bool,
    /// True when the command supports `--watch`.
    supports_watch: bool,
    /// Documented process exit codes.
    exit_codes: Vec<ExitCodeEntry>,
    /// Command-local arguments.
    arguments: Vec<ArgumentEntry>,
    /// Example invocations.
    examples: Vec<String>,
}

/// Stdin contract for a command.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
#[allow(dead_code)]
enum StdinSupport {
    /// The command does not read stdin.
    None,
    /// The command can read stdin when requested, usually via `--file -`.
    Optional,
    /// The command requires stdin input.
    Required,
}

/// Safety classification for operator and agent decision-making.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
#[allow(dead_code)]
enum SafetyLevel {
    /// Read-only command with no local or remote side effects.
    Read,
    /// Local or low-risk side effect, such as installing completions.
    LowImpact,
    /// Remote state mutation or operationally disruptive action.
    HighImpact,
    /// Destructive action. Reserved for future commands with stronger risk.
    Destructive,
}

/// One documented process exit code.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExitCodeEntry {
    code: u8,
    meaning: &'static str,
}

impl ExitCodeEntry {
    /// Creates a static exit-code entry.
    const fn new(code: u8, meaning: &'static str) -> Self {
        Self { code, meaning }
    }
}

/// One command-local or global argument exposed in the catalog.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ArgumentEntry {
    /// Clap argument id.
    id: String,
    /// Simplified argument kind.
    kind: ArgumentKind,
    /// Long flag name without leading dashes.
    long: Option<String>,
    /// Short flag name without leading dash.
    short: Option<String>,
    /// Alternate argument names accepted by Clap.
    aliases: Vec<String>,
    /// Value names shown in help and usage.
    value_names: Vec<String>,
    /// Enumerated possible values, when Clap exposes them.
    possible_values: Vec<PossibleValueEntry>,
    /// Default values visible to callers.
    default_values: Vec<String>,
    /// Environment variable name, when configured.
    env: Option<String>,
    /// True when Clap requires this argument.
    required: bool,
    /// True when this argument is inherited globally.
    global: bool,
    /// True when the argument accepts multiple occurrences or values.
    multiple: bool,
    /// Minimum and maximum number of values accepted.
    value_range: Option<ValueRangeEntry>,
    /// Normalized Clap action name.
    action: String,
    /// Argument help text.
    help: Option<String>,
    /// Help heading, when Clap groups this argument.
    help_heading: Option<String>,
}

/// Simplified argument categories for catalog consumers.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum ArgumentKind {
    /// Boolean or count-like flag.
    Flag,
    /// Named option that accepts one or more values.
    Option,
    /// Positional argument.
    Positional,
}

/// One possible value for an enumerated argument.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PossibleValueEntry {
    /// Canonical value name.
    name: String,
    /// Alternate names accepted by Clap.
    aliases: Vec<String>,
    /// Value-specific help text.
    help: Option<String>,
}

/// Serializable representation of Clap's value-count range.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ValueRangeEntry {
    /// Minimum accepted value count.
    min: usize,
    /// Maximum accepted value count, or `None` for unbounded.
    max: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: the generated catalog is inspected by an automation client.
    /// Guarantees: the catalog exposes a versioned schema and core command
    /// metadata without depending on a live admin endpoint.
    #[test]
    fn catalog_includes_schema_and_known_commands() {
        let catalog = build_catalog();

        assert_eq!(catalog.schema_version, SCHEMA_VERSION);
        assert!(
            catalog
                .commands
                .iter()
                .any(|command| command.path == ["engine", "status"])
        );
        assert!(
            catalog
                .commands
                .iter()
                .any(|command| command.path == ["pipelines", "reconfigure"])
        );
        assert!(
            catalog
                .commands
                .iter()
                .any(|command| command.path == ["completions", "install"])
        );
        assert!(
            catalog
                .commands
                .iter()
                .any(|command| command.path == ["schemas"])
        );
    }

    /// Scenario: command metadata is generated from clap argument definitions.
    /// Guarantees: global connection flags, output modes, aliases, and examples
    /// are discoverable without scraping human help text.
    #[test]
    fn catalog_includes_arguments_aliases_and_examples() {
        let catalog = build_catalog();
        let pipeline_get = catalog
            .commands
            .iter()
            .find(|command| command.path == ["pipelines", "get"])
            .expect("pipeline get command");

        assert!(pipeline_get.aliases.contains(&"details".to_string()));
        assert!(
            catalog
                .global_arguments
                .iter()
                .any(|argument| argument.long.as_deref() == Some("url") && argument.global)
        );
        assert_eq!(
            pipeline_get.output_modes,
            vec!["human", "json", "yaml", "agent-json"]
        );
        assert_eq!(pipeline_get.id, "dfctl.pipelines.get");
        assert_eq!(pipeline_get.default_output.as_deref(), Some("human"));
        assert_eq!(
            pipeline_get.agent_default_output.as_deref(),
            Some("agent-json")
        );
        assert_eq!(pipeline_get.output_schema, Some(AGENT_ENVELOPE_SCHEMA));
        assert_eq!(pipeline_get.error_schema, Some(ERROR_SCHEMA));
        assert!(!pipeline_get.examples.is_empty());
    }

    /// Scenario: local-only discovery commands are described in the catalog.
    /// Guarantees: agents can avoid resolving an admin connection before
    /// invoking local catalog, completion, and config commands.
    #[test]
    fn catalog_marks_local_and_long_running_commands() {
        let catalog = build_catalog();
        let commands = catalog
            .commands
            .iter()
            .find(|command| command.path == ["commands"])
            .expect("commands command");
        let logs_watch = catalog
            .commands
            .iter()
            .find(|command| command.path == ["telemetry", "logs", "watch"])
            .expect("logs watch command");

        assert!(!commands.requires_admin_client);
        assert!(logs_watch.requires_admin_client);
        assert!(logs_watch.long_running);
        assert_eq!(logs_watch.agent_default_output.as_deref(), Some("ndjson"));
        assert_eq!(logs_watch.stream_schema, Some(STREAM_EVENT_SCHEMA));
        assert_eq!(logs_watch.output_modes, vec!["human", "ndjson"]);
    }

    /// Scenario: mutation commands are inspected before an agent decides
    /// whether they can be safely run unattended.
    /// Guarantees: the catalog marks high-impact commands, dry-run support,
    /// wait/watch support, stdin support, and non-idempotent behavior.
    #[test]
    fn catalog_marks_mutation_safety_and_preflight_metadata() {
        let catalog = build_catalog();
        let reconfigure = catalog
            .commands
            .iter()
            .find(|command| command.path == ["pipelines", "reconfigure"])
            .expect("pipeline reconfigure command");

        assert!(reconfigure.mutation);
        assert_eq!(reconfigure.safety, SafetyLevel::HighImpact);
        assert_eq!(reconfigure.stdin, StdinSupport::Optional);
        assert!(!reconfigure.idempotent);
        assert!(reconfigure.supports_dry_run);
        assert!(reconfigure.supports_wait);
        assert!(reconfigure.supports_watch);
        assert_eq!(reconfigure.output_schema, Some(MUTATION_OUTCOME_SCHEMA));
    }
}
