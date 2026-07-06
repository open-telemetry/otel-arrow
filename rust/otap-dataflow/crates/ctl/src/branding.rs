// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configurable CLI identity for library embedders.
//!
//! The standalone `dfctl` binary always uses the default identity. A downstream
//! binary that embeds this crate's library entrypoints can override the identity
//! so that help text, shell completions, and machine-readable output envelopes
//! reflect the embedding binary instead of `dfctl`.
//!
//! Branding is process-global and set-once, mirroring the crypto-provider
//! bootstrap in [`crate::crypto`]: a process runs as exactly one binary identity.
//! The branded entrypoints install the branding before executing a command; all
//! other entrypoints leave it unset and observe [`Branding::default`].

use std::sync::OnceLock;

/// The single `dfctl` default binary-name literal shared by the standalone
/// `BIN_NAME` constant and [`Branding::default`].
pub(crate) const DEFAULT_BIN_NAME: &str = "dfctl";

/// Identity strings used in the CLI's user-visible and machine-readable output.
///
/// Defaults reproduce the standalone `dfctl` identity. The fields are
/// `&'static str` because a binary's identity is fixed for the life of the
/// process.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Branding {
    /// Installed command name used in help, completions, shell completions,
    /// generated command metadata, and diagnostics (e.g. `dfctl`). Use
    /// [`crate::Cli::command_branded`] to produce a clap `Command` with your
    /// own name before parsing, and [`crate::run_with_terminal_and_diagnostics_branded`]
    /// to install the branding for runtime output.
    pub bin_name: &'static str,
    /// Schema-version identifier stamped into machine-readable output envelopes
    /// (e.g. `dfctl/v1`).
    pub schema_version: &'static str,
}

impl Default for Branding {
    fn default() -> Self {
        Self {
            bin_name: DEFAULT_BIN_NAME,
            schema_version: "dfctl/v1",
        }
    }
}

/// Process-global active branding. Unset means [`Branding::default`].
static ACTIVE: OnceLock<Branding> = OnceLock::new();

/// Install the process-wide branding.
///
/// Idempotent and set-once (like the crypto provider): the first call wins and
/// subsequent calls are ignored. Standalone `dfctl` never calls this and so
/// observes [`Branding::default`].
pub(crate) fn set_branding(branding: Branding) {
    // First-writer-wins; a binary has a single identity for its lifetime.
    let _ = ACTIVE.set(branding);
}

/// Return the active branding, or the default if none was installed.
pub(crate) fn active() -> Branding {
    ACTIVE.get().copied().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_dfctl() {
        let b = Branding::default();
        assert_eq!(b.bin_name, "dfctl");
        assert_eq!(b.schema_version, "dfctl/v1");
    }
}
