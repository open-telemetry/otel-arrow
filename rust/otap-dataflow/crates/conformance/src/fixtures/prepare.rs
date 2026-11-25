// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Create the local files declared by a fixture before applying its configuration.
use super::render_text;
use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    fs,
    path::{Component, Path, PathBuf},
};

#[derive(Deserialize, Default)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum Prepare {
    #[default]
    None,
    Journal,
    Wasm,
    #[serde(untagged)]
    Files {
        files: BTreeMap<PathBuf, FileSource>,
    },
}

/// An embedded asset or a short UTF-8 file, rendered for the current stage.
#[derive(Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub(crate) enum FileSource {
    Asset { asset: String },
    Text { text: String },
}

impl Prepare {
    pub(crate) fn install(&self, root: &Path, vars: &minijinja::Value) -> Result<()> {
        fs::create_dir_all(root)?;
        match self {
            Self::None => (),
            Self::Wasm => fs::write(
                root.join("pass.wasm"),
                wat::parse_str(include_str!("../../fixtures/pass.wat"))?,
            )?,
            Self::Journal => {
                let path = root.join("var/log/journal/22222222222222222222222222222222");
                fs::create_dir_all(&path)?;
                let mut reader = flate2::read::GzDecoder::new(
                    &include_bytes!("../../fixtures/sample.journal.gz")[..],
                );
                let _ = std::io::copy(
                    &mut reader,
                    &mut fs::File::create(path.join("system.journal"))?,
                )?;
                fs::create_dir_all(root.join("etc"))?;
                fs::write(
                    root.join("etc/machine-id"),
                    "22222222222222222222222222222222\n",
                )?;
            }
            Self::Files { files } => {
                static ASSETS: include_dir::Dir<'_> =
                    include_dir::include_dir!("$CARGO_MANIFEST_DIR/fixtures/assets");
                for (destination, source) in files {
                    ensure!(
                        !destination.as_os_str().is_empty()
                            && destination
                                .components()
                                .all(|c| matches!(c, Component::Normal(_))),
                        "prepared file must be relative to the stage directory: {}",
                        destination.display()
                    );
                    let path = root.join(destination);
                    fs::create_dir_all(path.parent().context("missing stage directory")?)?;
                    match source {
                        FileSource::Asset { asset } => {
                            let name = render_text(asset, vars)?;
                            let file = ASSETS
                                .get_file(&name)
                                .with_context(|| format!("unknown fixture asset {name}"))?;
                            fs::write(path, file.contents())?;
                        }
                        FileSource::Text { text } => fs::write(path, render_text(text, vars)?)?,
                    }
                }
            }
        }
        Ok(())
    }
}
