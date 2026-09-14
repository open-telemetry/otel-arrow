// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Consumer-side adapter over a bound `agent_fed_auth_provider` capability.

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use async_trait::async_trait;
use http::{HeaderName, HeaderValue};
use otel_arrow_dfe_engine::{
    capability::auth::{
        agent_fed_credential_provider::AgentFedCredentialSnapshot,
        bearer_token_provider::TOKEN_USABLE_MARGIN,
    },
    local::capability::auth::agent_fed_credential_provider::AgentFedCredentialProvider,
};
use rand::RngExt;

use crate::http_client_auth_provider::*;

/// Base reschedule delay after a failed acquisition. Consecutive failures grow
/// this exponentially (with jitter) up to `MAX_TOKEN_REFRESH_RETRY_SECS`.
const TOKEN_REFRESH_RETRY_SECS: u64 = 10;
/// Upper bound on the retry backoff after repeated failures.
const MAX_TOKEN_REFRESH_RETRY_SECS: u64 = 300;

const NAME: &str = "AgentFedAuth";

/// Consumer-side bearer-token authenticator: subscribes to a provider's agent
/// fed credentials caches the built `Authorization` header, and reports
/// usability.
///
/// All credential/expiry state lives here, so an exporter holds one of these
/// and never touches a credential directly.
pub struct AgentFedAuth {
    provider: Box<dyn AgentFedCredentialProvider>,
    cached_credential: Option<CachedCredential>,
    generation: u64,
    rejected_generation: Option<u64>,
}

#[derive(Debug)]
struct CachedCredential {
    snapshot: Arc<AgentFedCredentialSnapshot>,
    header: HeaderValue,
    expires_on: Option<Instant>,
    generation: u64,
}

impl AgentFedAuth {
    #[must_use]
    pub fn new(provider: Box<dyn AgentFedCredentialProvider>) -> Self {
        Self {
            provider,
            cached_credential: None,
            generation: 0,
            rejected_generation: None,
        }
    }

    fn try_accept_snapshot(
        &mut self,
        snapshot: Arc<AgentFedCredentialSnapshot>,
        events: &HttpClientAuthProviderEvents,
    ) -> Result<bool, ()> {
        if let Some(cached) = &self.cached_credential
            && Arc::ptr_eq(&cached.snapshot, &snapshot)
        {
            if self.rejected_generation == Some(cached.generation) {
                // Credentials returned were already rejected. Return Err to signal caller to retry.
                return Err(());
            }
            // Continue using the cached credential
            return Ok(false);
        }

        let token = snapshot.token();
        if token.expose_token().trim().is_empty() {
            events.emit_invalid(self, "Malformed token: Empty");
            // Keep using the previously cached token (if any)
            return Ok(false);
        }

        match HeaderValue::from_str(&format!("Bearer {}", token.expose_token())) {
            Ok(mut header) => {
                header.set_sensitive(true);
                let expires_on = token.expires_on();
                self.generation = self.generation.wrapping_add(1);
                self.cached_credential = Some(CachedCredential {
                    snapshot,
                    header,
                    expires_on,
                    generation: self.generation,
                });
                Ok(true)
            }
            Err(e) => {
                // Keep using the previously cached token (if any)
                events.emit_invalid(self, &format!("Malformed token: {e}"));
                Ok(false)
            }
        }
    }
}

#[async_trait(?Send)]
impl HttpClientAuthProvider for AgentFedAuth {
    fn name(&self) -> HttpClientAuthProviderName {
        NAME.into()
    }

    fn is_active(&self) -> bool {
        true
    }

    fn is_ready(&self) -> bool {
        self.cached_credential.as_ref().is_some_and(|credential| {
            self.rejected_generation != Some(credential.generation)
                && credential
                    .expires_on
                    .is_none_or(|expires_on| expires_on > Instant::now() + TOKEN_USABLE_MARGIN)
        })
    }

    fn not_ready_reason(&self) -> &'static str {
        match self.cached_credential.as_ref() {
            Some(credential) if self.rejected_generation == Some(credential.generation) => {
                "agent-fed bearer token was rejected; awaiting a different snapshot"
            }
            Some(credential)
                if credential.expires_on.is_some_and(|expires_on| {
                    expires_on <= Instant::now() + TOKEN_USABLE_MARGIN
                }) =>
            {
                "agent-fed bearer token at/near expiry; awaiting refresh"
            }
            Some(_) => "agent-fed credential refresh pending",
            None => "agent-fed bearer token unavailable",
        }
    }

    fn header(&self) -> Option<(HeaderName, HeaderValue, u64)> {
        self.cached_credential.as_ref().map(|credential| {
            (
                http::header::AUTHORIZATION,
                credential.header.clone(),
                credential.generation,
            )
        })
    }

    fn refresh_deadline(&self) -> Option<Instant> {
        if !self.is_ready() {
            return None;
        }
        self.cached_credential
            .as_ref()
            .and_then(|credential| credential.expires_on)
            .and_then(|expires_on| expires_on.checked_sub(TOKEN_USABLE_MARGIN))
    }

    fn invalidate(&mut self, generation: u64) {
        if self
            .cached_credential
            .as_ref()
            .is_some_and(|credential| credential.generation == generation)
        {
            self.rejected_generation = Some(generation);
        }
    }

    async fn poll_refresh(&mut self, events: &HttpClientAuthProviderEvents) -> bool {
        let mut consecutive_failures = 0;
        loop {
            match self.provider.get_credential().await {
                Ok(credential) => {
                    match self.try_accept_snapshot(credential, events) {
                        Err(_) => {
                            // Previously rejected credential encountered. Need to
                            // wait for a new credential to arrive.

                            events.emit_retry(
                                self,
                                "A previously rejected credential was retrieved; operation will be retried",
                                consecutive_failures
                            );

                            let backoff =
                                jittered_backoff(retry_backoff_secs(consecutive_failures));

                            tokio::time::sleep(backoff).await;

                            consecutive_failures += 1;

                            continue;
                        }
                        Ok(r) => {
                            return r;
                        }
                    }
                }
                Err(e) => {
                    // Retrieval error: Keep using the last cached token (if any).
                    events.emit_error(self, &format!("Error retrieving token: {e}"));
                    return false;
                }
            }
        }
    }
}

/// Base (un-jittered) backoff before retrying after a failed acquisition.
///
/// Grows exponentially with the number of consecutive prior failures, from
/// `TOKEN_REFRESH_RETRY_SECS` up to `MAX_TOKEN_REFRESH_RETRY_SECS`, so a
/// sustained token-endpoint outage settles into infrequent retries instead of a
/// tight loop.
fn retry_backoff_secs(consecutive_failures: u32) -> u64 {
    // Cap the shift so `1 << shift` cannot overflow; the value is clamped to the
    // max below long before the shift approaches that bound.
    let shift = consecutive_failures.min(16);
    TOKEN_REFRESH_RETRY_SECS
        .saturating_mul(1u64 << shift)
        .min(MAX_TOKEN_REFRESH_RETRY_SECS)
}

/// Applies "equal jitter" to a backoff: half the delay is a fixed floor and the
/// other half is randomized, yielding a delay in `[base/2, base]`. This keeps
/// per-core extensions from retrying in lockstep during an outage.
fn jittered_backoff(base_secs: u64) -> Duration {
    let half = base_secs / 2;
    let jitter = if half == 0 {
        0
    } else {
        rand::rng().random_range(0..=half)
    };
    Duration::from_secs(half + jitter)
}
