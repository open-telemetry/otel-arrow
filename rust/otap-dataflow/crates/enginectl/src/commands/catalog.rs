// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Generated command catalog for humans, scripts, and AI agents.

use crate::Cli;
use crate::args::CommandsArgs;
use crate::commands::output::emit_read;
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
        arguments: arguments.clone(),
        examples,
    });

    for subcommand in command.get_subcommands() {
        collect_command(subcommand, path, commands);
    }
    let _ = path.pop();
}

fn visible_arguments(command: &ClapCommand) -> Vec<ArgumentEntry> {
    command
        .get_arguments()
        .filter(|argument| !argument.is_hide_set())
        .filter(|argument| !is_generated_help_argument(argument))
        .map(argument_entry)
        .collect()
}

fn is_generated_help_argument(argument: &Arg) -> bool {
    matches!(
        argument.get_action(),
        &ArgAction::Help | &ArgAction::HelpShort | &ArgAction::HelpLong
    )
}

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

fn argument_kind(argument: &Arg, action: &ArgAction) -> ArgumentKind {
    if argument.is_positional() {
        ArgumentKind::Positional
    } else if action.takes_values() {
        ArgumentKind::Option
    } else {
        ArgumentKind::Flag
    }
}

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

fn is_multiple(argument: &Arg, action: &ArgAction) -> bool {
    matches!(action, &ArgAction::Append | &ArgAction::Count)
        || argument
            .get_num_args()
            .map(|range| range.min_values() != range.max_values() || range.max_values() > 1)
            .unwrap_or(false)
}

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

fn command_line(path: &[String]) -> String {
    format!("{BIN_NAME} {}", path.join(" "))
}

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

fn curated_examples(path: &[String]) -> Vec<String> {
    match path
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["commands"] => vec!["dfctl commands --output json".to_string()],
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

fn value_placeholder(argument: &ArgumentEntry) -> String {
    argument
        .value_names
        .first()
        .cloned()
        .unwrap_or_else(|| argument.id.to_ascii_uppercase().replace('-', "_"))
}

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

fn requires_admin_client(path: &[String]) -> bool {
    !matches!(
        path.first().map(String::as_str),
        Some("completions" | "commands" | "config")
    )
}

fn is_interactive(path: &[String]) -> bool {
    matches!(path.first().map(String::as_str), Some("ui"))
}

fn is_long_running(path: &[String]) -> bool {
    is_interactive(path) || path.iter().any(|part| part == "watch")
}

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

fn format_action(action: &ArgAction) -> String {
    format!("{action:?}").to_ascii_lowercase()
}

fn styled_to_string(value: Option<&clap::builder::StyledStr>) -> Option<String> {
    value.map(ToString::to_string)
}

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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CommandCatalog {
    schema_version: &'static str,
    generated_at: String,
    binary: &'static str,
    version: &'static str,
    global_arguments: Vec<ArgumentEntry>,
    commands: Vec<CommandEntry>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CommandEntry {
    name: String,
    path: Vec<String>,
    command_line: String,
    usage: String,
    about: Option<String>,
    long_about: Option<String>,
    aliases: Vec<String>,
    leaf: bool,
    runnable: bool,
    requires_admin_client: bool,
    interactive: bool,
    long_running: bool,
    mutation: bool,
    output_modes: Vec<String>,
    arguments: Vec<ArgumentEntry>,
    examples: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ArgumentEntry {
    id: String,
    kind: ArgumentKind,
    long: Option<String>,
    short: Option<String>,
    aliases: Vec<String>,
    value_names: Vec<String>,
    possible_values: Vec<PossibleValueEntry>,
    default_values: Vec<String>,
    env: Option<String>,
    required: bool,
    global: bool,
    multiple: bool,
    value_range: Option<ValueRangeEntry>,
    action: String,
    help: Option<String>,
    help_heading: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum ArgumentKind {
    Flag,
    Option,
    Positional,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PossibleValueEntry {
    name: String,
    aliases: Vec<String>,
    help: Option<String>,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ValueRangeEntry {
    min: usize,
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
        assert_eq!(logs_watch.output_modes, vec!["human", "ndjson"]);
    }
}
