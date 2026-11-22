// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Data-plane observations for syslog input and JSONL file output.
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{fs, path::Path};
use tokio::{io::AsyncWriteExt, net::TcpStream};

/// Send a marked syslog record over a new TCP connection.
pub async fn send(port: u16, marker: &str) -> Result<()> {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).await?;
    stream
        .write_all(format!("<14>1 2025-01-01T00:00:00Z host app - - - {marker}\n").as_bytes())
        .await?;
    stream.shutdown().await?;
    Ok(())
}

/// Find the marker in complete JSONL log records and verify its optional attribute.
pub fn delivered(
    dir: &Path,
    prefix: &str,
    marker: &str,
    attribute: Option<(&str, Option<&str>)>,
) -> Result<bool> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if !entry
            .file_name()
            .to_string_lossy()
            .starts_with(&format!("{prefix}-logs-"))
        {
            continue;
        }
        let data = fs::read(entry.path())?;
        for line in data
            .split_inclusive(|&b| b == b'\n')
            .filter(|line| line.ends_with(b"\n"))
        {
            let payload: Value = serde_json::from_slice(line)?;
            for resource in payload["resourceLogs"]
                .as_array()
                .context("missing resourceLogs")?
            {
                for scope in resource["scopeLogs"]
                    .as_array()
                    .context("missing scopeLogs")?
                {
                    for record in scope["logRecords"]
                        .as_array()
                        .context("missing logRecords")?
                    {
                        let attrs = record["attributes"]
                            .as_array()
                            .context("missing attributes")?;
                        let attr = |key: &str| {
                            attrs
                                .iter()
                                .find(|a| a["key"] == key)
                                .and_then(|a| a["value"]["stringValue"].as_str())
                        };
                        if attr("syslog.message") == Some(marker) {
                            if let Some((key, expected)) = attribute {
                                let actual = attrs.iter().find(|a| a["key"] == key);
                                ensure!(
                                    actual.map(|a| &a["value"])
                                        == expected.map(|s| json!({"stringValue":s})).as_ref(),
                                    "{marker}: unexpected {key}: {actual:?}"
                                );
                            }
                            return Ok(true);
                        }
                    }
                }
            }
        }
    }
    Ok(false)
}

/// Send a marker and wait for the matching exported record.
pub async fn assert_delivery(
    port: u16,
    dir: &Path,
    prefix: &str,
    marker: &str,
    attribute: Option<(&str, Option<&str>)>,
) -> Result<()> {
    send(port, marker).await?;
    super::wait_for(marker, async || delivered(dir, prefix, marker, attribute)).await
}
