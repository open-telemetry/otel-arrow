// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Errors for the columnar query engine.

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {}
