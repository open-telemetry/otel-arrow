// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Deterministic SDK transport fixtures; never contact Azure or resolve DNS.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use azure_core::credentials::{AccessToken, TokenCredential, TokenRequestOptions};
use azure_core::http::{
    AsyncRawResponse, HttpClient, Request, RetryOptions, StatusCode, Transport,
    headers::{AUTHORIZATION, Headers, WWW_AUTHENTICATE},
};
use azure_core::time::{Duration, OffsetDateTime};
use azure_security_keyvault_secrets::{SecretClient, SecretClientOptions};

use super::Source;
use super::config::Config;

pub(crate) const TOKEN: &str = "offline-sensitive-token-marker";
pub(crate) const PAYLOAD: &str = "offline-sensitive-body-marker";

pub(crate) enum Reply {
    Json(u16, serde_json::Value),
    Pending,
}

pub(crate) fn secret(value: &str) -> Reply {
    Reply::Json(200, serde_json::json!({ "value": value }))
}

pub(crate) fn failure(status: u16) -> Reply {
    Reply::Json(
        status,
        serde_json::json!({
            "error": { "code": PAYLOAD, "message": format!("{PAYLOAD} {TOKEN}") }
        }),
    )
}

pub(crate) struct FakeTransport {
    replies: Mutex<VecDeque<Reply>>,
    pub requests: Mutex<Vec<(String, bool)>>,
    pub in_flight: AtomicUsize,
    pub challenge_resource: &'static str,
}

impl std::fmt::Debug for FakeTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FakeTransport").finish_non_exhaustive()
    }
}

impl FakeTransport {
    pub fn new(replies: Vec<Reply>) -> Arc<Self> {
        Arc::new(Self {
            replies: Mutex::new(replies.into()),
            requests: Mutex::new(Vec::new()),
            in_flight: AtomicUsize::new(0),
            challenge_resource: "https://vault.azure.net",
        })
    }

    pub fn request_count(&self) -> usize {
        self.requests.lock().unwrap().len()
    }

    pub fn with_challenge(resource: &'static str) -> Arc<Self> {
        let mut transport = Self::new(vec![]);
        Arc::get_mut(&mut transport).unwrap().challenge_resource = resource;
        transport
    }
}

struct Flight<'a>(&'a AtomicUsize);

impl Drop for Flight<'_> {
    fn drop(&mut self) {
        let _ = self.0.fetch_sub(1, Ordering::SeqCst);
    }
}

#[async_trait]
impl HttpClient for FakeTransport {
    async fn execute_request(&self, request: &Request) -> azure_core::Result<AsyncRawResponse> {
        let _ = self.in_flight.fetch_add(1, Ordering::SeqCst);
        let _flight = Flight(&self.in_flight);
        let authorization = request.headers().get_optional_str(&AUTHORIZATION);
        assert_eq!(request.url().host_str(), Some("test-vault.vault.azure.net"));
        if let Some(authorization) = authorization {
            assert_eq!(authorization, format!("Bearer {TOKEN}"));
        }
        self.requests
            .lock()
            .unwrap()
            .push((request.url().path().to_owned(), authorization.is_some()));
        if authorization.is_none() {
            let mut headers = Headers::new();
            headers.insert(
                WWW_AUTHENTICATE,
                format!(
                    "Bearer authorization=\"https://login.microsoftonline.com/test\", resource=\"{}\"",
                    self.challenge_resource
                ),
            );
            return Ok(AsyncRawResponse::from_bytes(
                StatusCode::Unauthorized,
                headers,
                "",
            ));
        }
        let reply = self
            .replies
            .lock()
            .unwrap()
            .pop_front()
            .expect("expected SDK request");
        match reply {
            Reply::Json(status, body) => Ok(AsyncRawResponse::from_bytes(
                StatusCode::from(status),
                Headers::new(),
                serde_json::to_vec(&body).unwrap(),
            )),
            Reply::Pending => std::future::pending().await,
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct FakeCredential {
    pub scopes: Mutex<Vec<Vec<String>>>,
    pub fail: bool,
}

#[async_trait]
impl TokenCredential for FakeCredential {
    async fn get_token(
        &self,
        scopes: &[&str],
        _options: Option<TokenRequestOptions<'_>>,
    ) -> azure_core::Result<AccessToken> {
        self.scopes
            .lock()
            .unwrap()
            .push(scopes.iter().map(|scope| (*scope).to_owned()).collect());
        if self.fail {
            return Err(azure_core::Error::with_message(
                azure_core::error::ErrorKind::Credential,
                format!("{TOKEN} {PAYLOAD}"),
            ));
        }
        Ok(AccessToken {
            token: TOKEN.into(),
            expires_on: OffsetDateTime::now_utc() + Duration::hours(1),
        })
    }
}

pub(crate) fn source(
    config: Config,
    transport: Arc<FakeTransport>,
    credential: Arc<FakeCredential>,
    retry: RetryOptions,
) -> Source {
    let mut options = SecretClientOptions::default();
    options.client_options.transport = Some(Transport::new(transport));
    options.client_options.retry = retry;
    let client = SecretClient::new(&config.vault_url, credential, Some(options)).unwrap();
    Source::with_client(config, client)
}

pub(crate) fn config() -> Config {
    serde_json::from_value(config_value()).unwrap()
}

pub(crate) fn config_value() -> serde_json::Value {
    serde_json::json!({
        "vault_url": "https://test-vault.vault.azure.net",
        "username_secret": { "name": "sasl-username" },
        "password_secret": { "name": "sasl-password" }
    })
}
