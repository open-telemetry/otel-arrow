// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared auth data types.
//!
//! The credential ([`BearerToken`]) and the scheme-agnostic authorization
//! outcome vocabulary ([`AuthorizedIdentity`], [`AuthzDecision`],
//! [`DenyReason`]) exchanged by the auth capabilities, one type per file. The
//! submodules are private; each type is re-exported here and again from
//! [`super`], so consumers reach them at `capability::auth::<Type>`.

mod authorized_identity;
mod authz_decision;
mod bearer_token;
mod deny_reason;

pub use authorized_identity::AuthorizedIdentity;
pub use authz_decision::AuthzDecision;
pub use bearer_token::BearerToken;
pub use deny_reason::DenyReason;
