// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::args::ColorChoice;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct HumanStyle {
    enabled: bool,
}

impl HumanStyle {
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

    pub const fn is_enabled(self) -> bool {
        self.enabled
    }

    pub fn header(self, text: impl AsRef<str>) -> String {
        self.paint(text, &[AnsiCode::Bold, AnsiCode::Cyan])
    }

    pub fn label(self, text: impl AsRef<str>) -> String {
        self.paint(text, &[AnsiCode::Bold, AnsiCode::Cyan])
    }

    pub fn dim(self, text: impl AsRef<str>) -> String {
        self.paint(text, &[AnsiCode::Dim])
    }

    pub fn target(self, text: impl AsRef<str>) -> String {
        self.paint(text, &[AnsiCode::Dim, AnsiCode::Cyan])
    }

    pub fn state(self, text: impl AsRef<str>) -> String {
        let text = text.as_ref();
        match classify_state(text) {
            StateClass::Success => self.paint(text, &[AnsiCode::Bold, AnsiCode::Green]),
            StateClass::Warning => self.paint(text, &[AnsiCode::Bold, AnsiCode::Yellow]),
            StateClass::Failure => self.paint(text, &[AnsiCode::Bold, AnsiCode::Red]),
            StateClass::Neutral => text.to_string(),
        }
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_respects_terminal_and_no_color() {
        assert!(!HumanStyle::resolve_with_no_color(ColorChoice::Auto, false, false).enabled);
        assert!(HumanStyle::resolve_with_no_color(ColorChoice::Auto, true, false).enabled);
        assert!(!HumanStyle::resolve_with_no_color(ColorChoice::Auto, true, true).enabled);
    }

    #[test]
    fn explicit_color_choice_overrides_auto_rules() {
        assert!(HumanStyle::resolve_with_no_color(ColorChoice::Always, false, true).enabled);
        assert!(!HumanStyle::resolve_with_no_color(ColorChoice::Never, true, false).enabled);
    }

    #[test]
    fn state_uses_semantic_colors() {
        let style = HumanStyle::resolve_with_no_color(ColorChoice::Always, false, false);
        assert!(style.state("succeeded").contains("\u{1b}["));
        assert!(style.state("failed").contains("\u{1b}["));
        assert_eq!(style.state("custom"), "custom");
    }
}
