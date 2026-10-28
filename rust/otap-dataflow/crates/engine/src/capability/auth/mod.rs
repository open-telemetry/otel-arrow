// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Auth capabilities and their shared vocabulary.
//!
//! Groups the credential-facing capabilities (token provider on the outbound
//! side, authorizer on the inbound side) with the shared data types they
//! exchange. The capability traits live in [`agent_fed_credential_provider`],
//! [`bearer_token_provider`], [`bearer_token_authorizer`], and
//! [`sasl_credential_provider`]; shared data types live in [`models`] and are
//! re-exported here.

mod models;

pub mod agent_fed_credential_provider;
pub mod api_key_provider;
pub mod basic_auth_provider;
pub mod bearer_token_authorizer;
pub mod bearer_token_provider;
pub mod sasl_credential_provider;

pub use models::{
    ApiKey, ApiKeyAttributeError, ApiKeyAttributes, AuthorizedIdentity, AuthzDecision,
    BasicAuthCredential, BasicAuthCredentialError, BearerToken, ClaimValue, DenyReason,
    SaslCredential, SaslCredentialError,
};
