// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Human-output styling policy and terminal-safe text helpers.
//!
//! `dfctl` must be pleasant in an interactive terminal without leaking ANSI
//! control sequences into pipes, logs, or machine-readable output. This module
//! centralizes color enablement, severity/status coloring, and defensive text
//! escaping so all human renderers follow the same `--color`, terminal-detection,
//! and `NO_COLOR` behavior.

use crate::args::ColorChoice;

/// Lightweight ANSI styling policy for human-readable CLI output.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct HumanStyle {
    enabled: bool,
}

impl HumanStyle {
    /// Resolves the effective color policy from CLI choice, terminal detection, and `NO_COLOR`.
    pub fn resolve(choice: ColorChoice, stdout_is_terminal: bool) -> Self {
        Self::resolve_with_no_color(
            choice,
            stdout_is_terminal,
            std::env::var_os("NO_COLOR").is_some(),
        )
    }

    fn resolve_with_no_color(
        choice: ColorChoice,
        stdout_is_terminal: bool,
        no_color_present: bool,
    ) -> Self {
        let enabled = match choice {
            ColorChoice::Always => true,
            ColorChoice::Never => false,
            ColorChoice::Auto => stdout_is_terminal && !no_color_present,
        };
        Self { enabled }
    }

    /// Returns true when human output should include ANSI styling.
    pub const fn is_enabled(self) -> bool {
        self.enabled
    }

    /// Styles section headers.
    pub fn header(self, text: impl AsRef<str>) -> String {
        self.paint(text, &[AnsiCode::Bold, AnsiCode::Cyan])
    }

    /// Styles field labels.
    pub fn label(self, text: impl AsRef<str>) -> String {
        self.paint(text, &[AnsiCode::Bold, AnsiCode::Cyan])
    }

    /// Styles low-emphasis text.
    pub fn dim(self, text: impl AsRef<str>) -> String {
        self.paint(text, &[AnsiCode::Dim])
    }

    /// Styles target identifiers such as URLs or pipeline names.
    pub fn target(self, text: impl AsRef<str>) -> String {
        self.paint(text, &[AnsiCode::Dim, AnsiCode::Cyan])
    }

    /// Styles status-like values based on success, warning, or failure keywords.
    pub fn state(self, text: impl AsRef<str>) -> String {
        let text = text.as_ref();
        match classify_state(text) {
            StateClass::Success => self.paint(text, &[AnsiCode::Bold, AnsiCode::Green]),
            StateClass::Warning => self.paint(text, &[AnsiCode::Bold, AnsiCode::Yellow]),
            StateClass::Failure => self.paint(text, &[AnsiCode::Bold, AnsiCode::Red]),
            StateClass::Neutral => text.to_string(),
        }
    }

    /// Styles log-level labels using severity-aware colors.
    pub fn log_level(self, text: impl AsRef<str>) -> String {
        let text = text.as_ref();
        match text.to_ascii_uppercase().as_str() {
            "TRACE" => self.paint(text, &[AnsiCode::Dim]),
            "DEBUG" => self.paint(text, &[AnsiCode::Bold, AnsiCode::Blue]),
            "INFO" => self.paint(text, &[AnsiCode::Bold, AnsiCode::Green]),
            "WARN" | "WARNING" => self.paint(text, &[AnsiCode::Bold, AnsiCode::Yellow]),
            "ERROR" | "FATAL" => self.paint(text, &[AnsiCode::Bold, AnsiCode::Red]),
            _ => text.to_string(),
        }
    }

    fn paint(self, text: impl AsRef<str>, codes: &[AnsiCode]) -> String {
        let text = text.as_ref();
        if !self.enabled || text.is_empty() {
            return text.to_string();
        }

        let mut rendered = String::from("\u{1b}[");
        for (index, code) in codes.iter().enumerate() {
            if index > 0 {
                rendered.push(';');
            }
            rendered.push_str(code.as_str());
        }
        rendered.push('m');
        rendered.push_str(text);
        rendered.push_str("\u{1b}[0m");
        rendered
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum StateClass {
    Success,
    Warning,
    Failure,
    Neutral,
}

fn classify_state(text: &str) -> StateClass {
    let lowered = text.trim().to_ascii_lowercase();
    let lowered = lowered.trim_matches('"');

    match lowered {
        "accepted" | "completed" | "healthy" | "live" | "ok" | "ready" | "running" | "serving"
        | "started" | "succeeded" | "success" | "true" | "up" => StateClass::Success,
        "accepted_async" | "degraded" | "draining" | "draining_old" | "pending"
        | "rollback_started" | "starting" | "unknown" | "waiting" => StateClass::Warning,
        "down" | "failed" | "failure" | "false" | "rollback_failed" | "timeout" | "timed_out"
        | "unhealthy" | "unready" => StateClass::Failure,
        _ => {
            if lowered.contains("fail") || lowered.contains("timeout") || lowered.contains("error")
            {
                StateClass::Failure
            } else if lowered.contains("drain")
                || lowered.contains("pending")
                || lowered.contains("wait")
                || lowered.contains("rollback")
            {
                StateClass::Warning
            } else {
                StateClass::Neutral
            }
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum AnsiCode {
    Bold,
    Dim,
    Red,
    Green,
    Yellow,
    Blue,
    Cyan,
}

impl AnsiCode {
    fn as_str(self) -> &'static str {
        match self {
            AnsiCode::Bold => "1",
            AnsiCode::Dim => "2",
            AnsiCode::Red => "31",
            AnsiCode::Green => "32",
            AnsiCode::Yellow => "33",
            AnsiCode::Blue => "34",
            AnsiCode::Cyan => "36",
        }
    }
}

/// Remove terminal control sequences from engine-provided text before it is
/// interpolated into human-readable output.
pub(crate) fn terminal_safe(text: impl AsRef<str>) -> String {
    let mut safe = String::with_capacity(text.as_ref().len());
    let mut chars = text.as_ref().chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '\u{1b}' => skip_escape_sequence(&mut chars),
            '\n' | '\t' => safe.push(ch),
            ch if ch.is_control() => {
                safe.push_str("\\u{");
                safe.push_str(&format!("{:x}", ch as u32));
                safe.push('}');
            }
            _ => safe.push(ch),
        }
    }

    safe
}

fn skip_escape_sequence<I>(chars: &mut std::iter::Peekable<I>)
where
    I: Iterator<Item = char>,
{
    match chars.peek().copied() {
        Some('[') => {
            let _ = chars.next();
            for ch in chars.by_ref() {
                if ('@'..='~').contains(&ch) {
                    break;
                }
            }
        }
        Some(']') => {
            let _ = chars.next();
            let mut previous_was_escape = false;
            for ch in chars.by_ref() {
                if ch == '\u{7}' || (previous_was_escape && ch == '\\') {
                    break;
                }
                previous_was_escape = ch == '\u{1b}';
            }
        }
        Some(_) => {
            let _ = chars.next();
        }
        None => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: color mode is left on auto while terminal detection and
    /// `NO_COLOR` vary.
    /// Guarantees: automatic human styling respects both terminal presence and
    /// the global no-color override.
    #[test]
    fn auto_respects_terminal_and_no_color() {
        assert!(!HumanStyle::resolve_with_no_color(ColorChoice::Auto, false, false).enabled);
        assert!(HumanStyle::resolve_with_no_color(ColorChoice::Auto, true, false).enabled);
        assert!(!HumanStyle::resolve_with_no_color(ColorChoice::Auto, true, true).enabled);
    }

    /// Scenario: a caller explicitly selects always or never for human color
    /// rendering.
    /// Guarantees: explicit color choices override the automatic terminal
    /// heuristics.
    #[test]
    fn explicit_color_choice_overrides_auto_rules() {
        assert!(HumanStyle::resolve_with_no_color(ColorChoice::Always, false, true).enabled);
        assert!(!HumanStyle::resolve_with_no_color(ColorChoice::Never, true, false).enabled);
    }

    /// Scenario: semantic state labels are rendered with color enabled.
    /// Guarantees: known success and failure states receive ANSI styling while
    /// unknown labels stay untouched.
    #[test]
    fn state_uses_semantic_colors() {
        let style = HumanStyle::resolve_with_no_color(ColorChoice::Always, false, false);
        assert!(style.state("succeeded").contains("\u{1b}["));
        assert!(style.state("failed").contains("\u{1b}["));
        assert_eq!(style.state("custom"), "custom");
    }

    /// Scenario: engine-provided text contains terminal control sequences.
    /// Guarantees: human renderers can neutralize escape sequences before
    /// writing the text to a terminal.
    #[test]
    fn terminal_safe_strips_escape_sequences() {
        assert_eq!(terminal_safe("ok\u{1b}[2Jsecret"), "oksecret");
        assert_eq!(terminal_safe("line\rrewrite"), "line\\u{d}rewrite");
    }
}
