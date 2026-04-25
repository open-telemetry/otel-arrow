// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shell completion generation and user-local installation helpers.

use crate::args::{CompletionArgs, CompletionCommand, CompletionInstallArgs};
use crate::error::CliError;
use crate::{BIN_NAME, Cli};
use clap::CommandFactory;
use clap_complete::Shell;
use std::ffi::OsString;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub(crate) fn run(stdout: &mut dyn Write, args: CompletionArgs) -> Result<(), CliError> {
    match args.command {
        Some(CompletionCommand::Install(args)) => install(stdout, args),
        None => generate(stdout, args),
    }
}

fn generate(stdout: &mut dyn Write, args: CompletionArgs) -> Result<(), CliError> {
    let shell = args.shell.ok_or_else(|| {
        CliError::invalid_usage(format!(
            "missing shell; use `{BIN_NAME} completions <shell>` or `{BIN_NAME} completions install <shell>`"
        ))
    })?;

    let mut command = Cli::command();
    clap_complete::generate(shell, &mut command, BIN_NAME, stdout);
    Ok(())
}

fn install(stdout: &mut dyn Write, args: CompletionInstallArgs) -> Result<(), CliError> {
    let dir = args
        .dir
        .unwrap_or_else(|| default_install_dir(args.shell).unwrap_or_default());
    if dir.as_os_str().is_empty() {
        return Err(CliError::config(format!(
            "failed to resolve default completion directory for {}; set HOME or pass --dir",
            args.shell
        )));
    }

    fs::create_dir_all(&dir).map_err(|err| {
        CliError::config(format!(
            "failed to create completion directory '{}': {err}",
            dir.display()
        ))
    })?;

    let mut command = Cli::command();
    let path = clap_complete::generate_to(args.shell, &mut command, BIN_NAME, dir.clone())
        .map_err(|err| {
            CliError::config(format!(
                "failed to install {} completions in '{}': {err}",
                args.shell,
                dir.display()
            ))
        })?;

    writeln!(
        stdout,
        "installed {} completions to {}",
        args.shell,
        path.display()
    )?;
    if let Some(note) = activation_note(args.shell, &dir, &path) {
        writeln!(stdout, "{note}")?;
    }
    stdout.flush()?;
    Ok(())
}

fn default_install_dir(shell: Shell) -> Option<PathBuf> {
    let env = EnvPaths::read();
    default_install_dir_from(shell, &env)
}

fn default_install_dir_from(shell: Shell, env: &EnvPaths) -> Option<PathBuf> {
    match shell {
        Shell::Bash => Some(env.data_home()?.join("bash-completion/completions")),
        Shell::Fish => Some(env.config_home()?.join("fish/completions")),
        Shell::Zsh => Some(env.data_home()?.join("zsh/site-functions")),
        Shell::Elvish => Some(env.config_home()?.join("elvish/lib")),
        Shell::PowerShell => Some(env.config_home()?.join("powershell/Completions")),
        _ => None,
    }
}

fn activation_note(shell: Shell, dir: &Path, path: &Path) -> Option<String> {
    let path = shell_quote(path);
    let dir = shell_quote(dir);
    match shell {
        Shell::Bash => Some(format!(
            "Start a new bash shell, or source {path}. Ensure bash-completion is loaded."
        )),
        Shell::Fish => Some(
            "Fish loads completions from this directory automatically in new shells.".to_string(),
        ),
        Shell::Zsh => Some(format!(
            "If zsh does not load it, add this to .zshrc: fpath=({dir} $fpath); autoload -Uz compinit; compinit"
        )),
        Shell::Elvish => Some(format!("Load it from rc.elv with: use {BIN_NAME}")),
        Shell::PowerShell => Some(format!(
            "Load it from your PowerShell profile with: . {path}"
        )),
        _ => None,
    }
}

fn shell_quote(path: &Path) -> String {
    let raw = path.display().to_string();
    format!("'{}'", raw.replace('\'', "'\"'\"'"))
}

#[derive(Debug, Clone, Default)]
struct EnvPaths {
    xdg_config_home: Option<PathBuf>,
    xdg_data_home: Option<PathBuf>,
    home: Option<PathBuf>,
}

impl EnvPaths {
    fn read() -> Self {
        Self {
            xdg_config_home: non_empty_env_path("XDG_CONFIG_HOME"),
            xdg_data_home: non_empty_env_path("XDG_DATA_HOME"),
            home: non_empty_env_path("HOME").or_else(|| non_empty_env_path("USERPROFILE")),
        }
    }

    fn config_home(&self) -> Option<PathBuf> {
        self.xdg_config_home
            .clone()
            .or_else(|| self.home.as_ref().map(|home| home.join(".config")))
    }

    fn data_home(&self) -> Option<PathBuf> {
        self.xdg_data_home
            .clone()
            .or_else(|| self.home.as_ref().map(|home| home.join(".local/share")))
    }
}

fn non_empty_env_path(name: &str) -> Option<PathBuf> {
    std::env::var_os(name).and_then(non_empty_path)
}

fn non_empty_path(value: OsString) -> Option<PathBuf> {
    if value.is_empty() {
        None
    } else {
        Some(PathBuf::from(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    /// Scenario: completion installation runs with an explicit directory.
    /// Guarantees: the command writes the shell-specific completion file and
    /// reports the installed path without touching any shell startup file.
    #[test]
    fn install_writes_completion_file_to_explicit_dir() {
        let dir = tempdir().expect("tempdir");
        let mut stdout = Vec::new();

        install(
            &mut stdout,
            CompletionInstallArgs {
                shell: Shell::Fish,
                dir: Some(dir.path().to_path_buf()),
            },
        )
        .expect("install completions");

        let path = dir.path().join(format!("{BIN_NAME}.fish"));
        assert!(path.exists());
        let output = String::from_utf8(stdout).expect("stdout");
        assert!(output.contains(path.to_str().expect("path")));
    }

    /// Scenario: XDG paths are available for the supported Unix shells.
    /// Guarantees: default install paths stay user-local and do not require
    /// privileged system completion directories.
    #[test]
    fn default_install_dirs_are_user_local() {
        let env = EnvPaths {
            xdg_config_home: Some(PathBuf::from("/tmp/config")),
            xdg_data_home: Some(PathBuf::from("/tmp/data")),
            home: Some(PathBuf::from("/home/alice")),
        };

        assert_eq!(
            default_install_dir_from(Shell::Bash, &env),
            Some(PathBuf::from("/tmp/data/bash-completion/completions"))
        );
        assert_eq!(
            default_install_dir_from(Shell::Fish, &env),
            Some(PathBuf::from("/tmp/config/fish/completions"))
        );
        assert_eq!(
            default_install_dir_from(Shell::Zsh, &env),
            Some(PathBuf::from("/tmp/data/zsh/site-functions"))
        );
    }
}
