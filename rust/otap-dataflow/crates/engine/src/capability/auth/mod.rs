// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Auth capabilities and their shared vocabulary.
//!
//! Groups the credential-facing capabilities (token provider on the outbound
//! side, authorizer on the inbound side) with the shared data types they
//! exchange. The capability traits live in [`agent_fed_credential_provider`],
//! [`bearer_token_provider`], and [`bearer_token_authorizer`]; the shared data
//! types ([`BearerToken`], [`AuthorizedIdentity`], [`AuthzDecision`],
//! [`DenyReason`]) live in [`models`] and are re-exported here so consumers
//! reach them at
//! `capability::auth::{BearerToken, AuthorizedIdentity, AuthzDecision, DenyReason}`.

mod models;

pub mod agent_fed_credential_provider;
pub mod api_key_provider;
pub mod bearer_token_authorizer;
pub mod bearer_token_provider;

pub use models::{ApiKey, AuthorizedIdentity, AuthzDecision, BearerToken, ClaimValue, DenyReason};
