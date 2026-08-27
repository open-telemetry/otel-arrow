// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Translator for AMCS (Azure Monitor Configuration Service) third-party configuration.
//!
//! AMCS delivers the Azure Monitor Agent every customer-authored Data Collection Rule (DCR) that
//! applies to a host, as a single JSON document. This module turns that document into an
//! [`OtelDataflowSpec`] the OTAP dataflow engine can run.
//!
//! This is a port of `AMCSParser.ExtractConfiguration` from the .NET `AMCSConfiguration`
//! project. The input is byte-for-byte the same payload the .NET agent consumes; only the output
//! differs -- a pipeline specification rather than a list of in-memory endpoint bindings.
//!
//! It additionally supports the **Agent Settings DCR** (`content.kind: "AgentSettings"`), which
//! the .NET parser does not read -- `Content.kind` and `Content.settings` are commented out in
//! `Configurations.cs`. That behaviour follows
//! `Telemetry-Collection-Spec/AMACoreAgent/otel-port-configuration.md` (owner: Ragu Marimuthu):
//! listener ports resolve from the environment first, then the Agent Settings rule, then the
//! built-in defaults. See [`listener`] for the full precedence chain.
//!
//!
//! # Stages
//!
//! | Module | Responsibility |
//! |---|---|
//! | [`schema`] | Serde model of the AMCS JSON payload |

pub mod schema;

/// The dialect name reported by the AMCS translator.
pub const AMCS_DIALECT: &str = "amcs";
