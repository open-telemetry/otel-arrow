// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared table and field-formatting helpers for human rendering.

use crate::style::{HumanStyle, terminal_safe};
use otap_df_admin_api::{pipelines, telemetry};
use std::collections::BTreeMap;

/// Formats one label/value line for human output.
pub(super) fn field(style: &HumanStyle, label: &str, value: impl std::fmt::Display) -> String {
    format!(
        "{} {}",
        style.label(format!("{label}:")),
        terminal_safe(value.to_string())
    )
}

/// Formats one label/value line where the value should use status coloring.
pub(super) fn state_field(style: &HumanStyle, label: &str, value: impl AsRef<str>) -> String {
    format!(
        "{} {}",
        style.label(format!("{label}:")),
        style.state(terminal_safe(value.as_ref()))
    )
}

/// Formats one pipeline condition as a compact human-readable line.
pub(super) fn condition_line(style: &HumanStyle, condition: &pipelines::Condition) -> String {
    format!(
        "{} kind={} status={} reason={} message={}",
        style.label("condition:"),
        style.state(format!("{:?}", condition.kind)),
        style.state(format!("{:?}", condition.status)),
        terminal_safe(
            condition
                .reason
                .as_ref()
                .map(|value| value.as_str().to_string())
                .unwrap_or_else(|| "none".to_string()),
        ),
        terminal_safe(
            condition
                .message
                .clone()
                .unwrap_or_else(|| "none".to_string()),
        )
    )
}

/// Formats metric attributes while neutralizing terminal control sequences.
pub(super) fn format_metric_attributes(
    attributes: &BTreeMap<String, telemetry::AttributeValue>,
) -> String {
    if attributes.is_empty() {
        return "none".to_string();
    }

    attributes
        .iter()
        .map(|(key, value)| format!("{}={}", terminal_safe(key), attribute_value_string(value)))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Converts one admin API metric attribute value to display text.
pub(super) fn attribute_value_string(value: &telemetry::AttributeValue) -> String {
    match value {
        telemetry::AttributeValue::String(value) => terminal_safe(value),
        telemetry::AttributeValue::Int(value) => value.to_string(),
        telemetry::AttributeValue::UInt(value) => value.to_string(),
        telemetry::AttributeValue::Double(value) => value.to_string(),
        telemetry::AttributeValue::Boolean(value) => value.to_string(),
        telemetry::AttributeValue::Map(values) => format!(
            "{{{}}}",
            values
                .iter()
                .map(|(key, value)| format!(
                    "{}={}",
                    terminal_safe(key),
                    attribute_value_string(value)
                ))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

/// Converts one admin API metric value to display text.
pub(super) fn metric_value_string(value: &telemetry::MetricValue) -> String {
    match value {
        telemetry::MetricValue::U64(value) => value.to_string(),
        telemetry::MetricValue::F64(value) => value.to_string(),
        telemetry::MetricValue::Mmsc(value) => format!(
            "min={} max={} sum={} count={}",
            value.min, value.max, value.sum, value.count
        ),
    }
}

/// Renders an aligned plain-text table using visible terminal width.
pub(super) fn render_table(
    style: &HumanStyle,
    columns: &[TableColumn<'_>],
    rows: Vec<Vec<String>>,
) -> String {
    let mut widths = columns
        .iter()
        .map(|column| visible_width(column.header))
        .collect::<Vec<_>>();

    for row in &rows {
        for (index, cell) in row.iter().enumerate().take(widths.len()) {
            widths[index] = widths[index].max(visible_width(cell));
        }
    }

    let mut rendered = Vec::with_capacity(rows.len().saturating_add(2));
    rendered.push(join_table_row(
        columns
            .iter()
            .zip(widths.iter())
            .map(|(column, width)| pad_cell(&style.header(column.header), *width, TableAlign::Left))
            .collect(),
    ));
    rendered.push(join_table_row(
        widths.iter().map(|width| "-".repeat(*width)).collect(),
    ));

    for row in rows {
        rendered.push(join_table_row(
            columns
                .iter()
                .enumerate()
                .map(|(index, column)| {
                    pad_cell(
                        row.get(index).map(String::as_str).unwrap_or(""),
                        widths[index],
                        column.align,
                    )
                })
                .collect(),
        ));
    }

    rendered.join("\n")
}

fn join_table_row(cells: Vec<String>) -> String {
    cells.join("  ")
}

fn pad_cell(text: &str, width: usize, align: TableAlign) -> String {
    let padding = width.saturating_sub(visible_width(text));
    match align {
        TableAlign::Left => format!("{text}{}", " ".repeat(padding)),
        TableAlign::Right => format!("{}{text}", " ".repeat(padding)),
    }
}

/// Returns display width after removing ANSI styling sequences.
pub(super) fn visible_width(text: &str) -> usize {
    strip_ansi(text).chars().count()
}

fn strip_ansi(text: &str) -> String {
    let mut stripped = String::with_capacity(text.len());
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
            stripped.push(ch);
        }
    }

    stripped
}

/// Horizontal alignment applied when padding table cells.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum TableAlign {
    Left,
    Right,
}

/// Column definition used by the human table renderer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct TableColumn<'a> {
    pub(super) header: &'a str,
    pub(super) align: TableAlign,
}

impl<'a> TableColumn<'a> {
    /// Creates a left-aligned table column.
    pub(super) const fn left(header: &'a str) -> Self {
        Self {
            header,
            align: TableAlign::Left,
        }
    }

    /// Creates a right-aligned table column.
    pub(super) const fn right(header: &'a str) -> Self {
        Self {
            header,
            align: TableAlign::Right,
        }
    }
}
