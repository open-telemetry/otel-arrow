// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Context, Result, ensure};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Duration;

/// Candidate executable or locally available container image.
pub enum Engine {
    /// Path to the candidate df_engine executable.
    Binary(PathBuf),
    /// Engine image to run through Docker on the local host.
    Image(String),
}

impl Engine {
    fn command(
        &self,
        config: &Path,
        container: Option<&str>,
        env: &[(&str, String)],
    ) -> Result<Command> {
        let mut cmd = match self {
            Self::Binary(path) => {
                let mut cmd = Command::new(path);
                let _ = cmd.envs(env.iter().map(|(k, v)| (*k, v)));
                cmd
            }
            Self::Image(image) => {
                let dir = config
                    .parent()
                    .context("config needs a directory")?
                    .canonicalize()?;
                let id = |flag: &str| -> Result<String> {
                    let out = Command::new("id").arg(flag).output()?;
                    ensure!(out.status.success(), "could not determine container user");
                    Ok(String::from_utf8(out.stdout)?.trim().to_owned())
                };
                let mut cmd = Command::new("docker");
                let _ = cmd.args([
                    "run",
                    "--rm",
                    "--read-only",
                    "--user",
                    &format!("{}:{}", id("-u")?, id("-g")?),
                ]);
                if let Some(name) = container {
                    let _ = cmd.args(["--name", name, "--network", "host"]);
                } else {
                    let _ = cmd.args(["--network", "none"]);
                }
                for (key, value) in env {
                    let _ = cmd.args(["--env", &format!("{key}={value}")]);
                }
                let mode = if container.is_some() { "rw" } else { "ro" };
                let _ = cmd.args(["--volume", &format!("{0}:{0}:{mode}", dir.display()), image]);
                cmd
            }
        };
        let _ = cmd.arg("--config").arg(config.canonicalize()?);
        Ok(cmd)
    }

    /// Run configuration preflight and check acceptance or the expected rejection text.
    pub async fn validate(&self, config: &Path, rejection: Option<&str>) -> Result<()> {
        let mut cmd: tokio::process::Command = self.command(config, None, &[])?.into();
        let output = tokio::time::timeout(
            Duration::from_secs(20),
            cmd.arg("--validate-and-exit").kill_on_drop(true).output(),
        )
        .await??;
        let diagnostic = String::from_utf8_lossy(&output.stderr);
        let valid = match rejection {
            Some(reason) => {
                output.status.code().is_some_and(|code| code > 0) && diagnostic.contains(reason)
            }
            None => output.status.success(),
        };
        ensure!(
            valid,
            "{}: {}: {diagnostic}",
            config.display(),
            output.status
        );
        Ok(())
    }

    pub(super) fn start(&self, config: &Path, env: &[(&str, String)]) -> Result<Running> {
        let name = matches!(self, Self::Image(_)).then(|| {
            format!(
                "otel-conformance-{}-{}",
                std::process::id(),
                config
                    .parent()
                    .and_then(Path::file_name)
                    .unwrap_or_default()
                    .to_string_lossy()
            )
        });
        let log = config.with_extension("log");
        let file = File::create(&log)?;
        let child = self
            .command(config, name.as_deref(), env)?
            .stdout(file.try_clone()?)
            .stderr(file)
            .spawn()?;
        Ok(Running { child, name, log })
    }
}

pub(super) struct Running {
    child: Child,
    name: Option<String>,
    log: PathBuf,
}

impl Running {
    pub(super) fn check(&mut self) -> Result<()> {
        if let Some(status) = self.child.try_wait()? {
            anyhow::bail!(
                "engine exited ({status}): {}",
                std::fs::read_to_string(&self.log).unwrap_or_default()
            );
        }
        Ok(())
    }
}

impl Drop for Running {
    fn drop(&mut self) {
        if let Some(name) = &self.name {
            let _ = Command::new("docker")
                .args(["rm", "--force", name])
                .stdout(Stdio::null())
                .status();
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
