// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Experimental single-file tailing, not a production receiver executable.

// This developer CLI reports its own command errors, not internal node events.
#![allow(clippy::print_stderr)]

fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::FAILURE
        }
    }
}

#[cfg(target_os = "linux")]
#[tokio::main(flavor = "current_thread")]
async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use otel_arrow_dfe_core_nodes::receivers::filelog_receiver::experimental;
    use std::{env, io, path::PathBuf};

    let mut arguments = env::args_os().skip(1);
    let (input, output, status) = match (
        arguments.next(),
        arguments.next(),
        arguments.next(),
        arguments.next(),
    ) {
        (Some(input), Some(output), status, None) => (
            PathBuf::from(input),
            PathBuf::from(output),
            status.map(PathBuf::from),
        ),
        _ => {
            return Err(io::Error::other(
                "usage: filelog_tail INPUT NEW_OUTPUT [NEW_STATUS] (experimental; stop with Ctrl-C)",
            )
            .into());
        }
    };
    experimental::tail(input, output, status, tokio::signal::ctrl_c()).await?;
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Err(std::io::Error::other("the experimental filelog_tail example requires Linux").into())
}
