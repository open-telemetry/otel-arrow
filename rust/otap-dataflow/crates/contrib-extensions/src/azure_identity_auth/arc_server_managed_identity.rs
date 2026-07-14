// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::{
    collections::HashMap,
    env,
    fs::{self, File},
    io::Read,
    path::Path,
    sync::Arc,
};

use async_lock::RwLock;
use azure_core::{
    Error,
    credentials::{AccessToken, Secret, TokenCredential, TokenRequestOptions},
    error::ErrorKind,
    http::{
        ClientOptions, ExponentialRetryOptions, Method, Pipeline, PipelineOptions,
        PipelineSendOptions, Request, RetryOptions, StatusCode, Url, headers::HeaderName,
    },
    json::from_json,
    time::{Duration, OffsetDateTime},
};
use azure_identity::{ManagedIdentityCredentialOptions, UserAssignedId};
use serde::{Deserialize, Deserializer, de};

pub const AUTHORIZATION: HeaderName = HeaderName::from_static_standard("authorization");
pub const WWW_AUTHENTICATE: HeaderName = HeaderName::from_static_standard("www-authenticate");

const DEFAULT_ENDPOINT: &str = "http://localhost:40342/metadata/identity/oauth2/token";
const API_VERSION: &str = "2021-02-01";

// ArcServerManagedIdentity is very similar to ImdsManagedIdentityCredential, but with Arc-connected servers support.
// I tried to contribute this code to the Azure SDK but they were not willing to accept contributions of identity providers,
// hence the need to re-implement it here until the MSAL/SDK team can add Arc support to the Rust SDK.
#[derive(Debug)]
pub(crate) struct ArcServerManagedIdentity {
    pipeline: Pipeline,
    endpoint: Url,
    api_version: String,
    user_assigned_id: Option<UserAssignedId>,
    cache: TokenCache,
}

#[async_trait::async_trait]
impl TokenCredential for ArcServerManagedIdentity {
    async fn get_token(
        &self,
        scopes: &[&str],
        options: Option<TokenRequestOptions<'_>>,
    ) -> azure_core::Result<AccessToken> {
        self.cache
            .get_token(scopes, options, |s, o| self.get_token(s, o))
            .await
    }
}

impl ArcServerManagedIdentity {
    pub(crate) fn new(
        options: Option<ManagedIdentityCredentialOptions>,
    ) -> azure_core::Result<Arc<Self>> {
        let options = options.unwrap_or_default();

        let endpoint = match (
            env::var("IDENTITY_ENDPOINT").ok(),
            env::var("IMDS_ENDPOINT").ok(),
        ) {
            (Some(identity_endpoint), Some(_)) => identity_endpoint,
            _ => DEFAULT_ENDPOINT.to_owned(),
        };

        let endpoint = Url::parse(&endpoint)?;

        // these settings approximate the recommendations at
        // https://learn.microsoft.com/entra/identity/managed-identities-azure-resources/how-to-use-vm-token#retry-guidance
        let client_options = ClientOptions {
            retry: RetryOptions::exponential(ExponentialRetryOptions {
                initial_delay: Duration::milliseconds(1340),
                max_retries: 6,
                max_total_elapsed: Duration::seconds(72),
                ..Default::default()
            }),
            ..options.client_options
        };

        let pipeline_options = Some(PipelineOptions {
            // https://learn.microsoft.com/entra/identity/managed-identities-azure-resources/how-to-use-vm-token#error-handling
            retry_status_codes: Vec::from([
                StatusCode::NotFound,
                StatusCode::Gone,
                StatusCode::TooManyRequests,
                StatusCode::InternalServerError,
                StatusCode::NotImplemented,
                StatusCode::BadGateway,
                StatusCode::ServiceUnavailable,
                StatusCode::GatewayTimeout,
                StatusCode::HttpVersionNotSupported,
                StatusCode::VariantAlsoNegotiates,
                StatusCode::InsufficientStorage,
                StatusCode::LoopDetected,
                StatusCode::NotExtended,
                StatusCode::NetworkAuthenticationRequired,
            ]),
            ..Default::default()
        });

        let pipeline = Pipeline::new(
            option_env!("CARGO_PKG_NAME"),
            option_env!("CARGO_PKG_VERSION"),
            client_options,
            Vec::default(),
            Vec::default(),
            pipeline_options,
        );
        Ok(Arc::new(Self {
            pipeline,
            endpoint,
            user_assigned_id: options.user_assigned_id,
            api_version: API_VERSION.to_owned(),
            cache: TokenCache::new(),
        }))
    }

    async fn get_token(
        &self,
        scopes: &[&str],
        options: Option<TokenRequestOptions<'_>>,
    ) -> Result<AccessToken, Error> {
        let resource = scopes_to_resource(scopes)?;

        let mut query_items = vec![
            ("api-version", self.api_version.as_str()),
            ("resource", resource),
        ];

        if let Some(ref user_assigned_id) = self.user_assigned_id {
            match user_assigned_id {
                UserAssignedId::ClientId(client_id) => query_items.push(("client_id", client_id)),
                UserAssignedId::ObjectId(object_id) => query_items.push(("object_id", object_id)),
                UserAssignedId::ResourceId(msi_res_id) => {
                    query_items.push(("msi_res_id", msi_res_id))
                }
                // this part is a consequence of UserAssignedId being marked as non_exhaustive.
                _ => {
                    return Err(Error::with_message(
                        ErrorKind::Credential,
                        "Unsupported user assigned identity type provided",
                    ));
                }
            }
        }

        let mut url = self.endpoint.clone();
        let _ = url.query_pairs_mut().extend_pairs(query_items);

        let mut req = Request::new(url, Method::Get);

        req.insert_header("metadata", "true");

        let options = options.unwrap_or_default();
        let ctx = options.method_options.context.to_borrowed();
        let mut rsp = self
            .pipeline
            .send(
                &ctx,
                &mut req,
                Some(PipelineSendOptions {
                    skip_checks: true,
                    ..Default::default()
                }),
            )
            .await?;

        let mut status = rsp.status();

        if status == StatusCode::Unauthorized {
            if let Ok(challenge) = rsp.headers().get_str(&WWW_AUTHENTICATE) {
                if let Some(challenge_location) = challenge
                    .split_once('=')
                    .and_then(|(_, location)| location.trim().trim_matches('"').splitn(2, '"').next())
                {
                    let challenge_response =
                        self.retrieve_challenge_response(challenge_location)?;
                    req.insert_header(AUTHORIZATION, format!("Basic {challenge_response}"));

                    // try the request again with the challenge response header. Then, drop through to the usual error handling and token extraction
                    rsp = self
                        .pipeline
                        .send(
                            &ctx,
                            &mut req,
                            Some(PipelineSendOptions {
                                skip_checks: true,
                                ..Default::default()
                            }),
                        )
                        .await?;
                    status = rsp.status();
                }
            }
        }

        if !status.is_success() {
            let message = match status {
                StatusCode::BadRequest => {
                    "The requested identity has not been assigned to this resource".to_string()
                }
                StatusCode::BadGateway | StatusCode::GatewayTimeout => {
                    "The request failed due to a gateway error".to_string()
                }
                _ => {
                    let body = String::from_utf8_lossy(rsp.body());
                    format!("The request failed: {body}")
                }
            };
            return Err(Error::new(
                ErrorKind::HttpResponse {
                    error_code: None,
                    raw_response: Some(Box::new(rsp)),
                    status,
                },
                message,
            ));
        }

        let token_response: MsiTokenResponse = from_json(rsp.into_body())?;
        Ok(AccessToken::new(
            token_response.access_token,
            token_response.expires_on,
        ))
    }

    // This is used for Arc for server's flavour of IMDS, where a challenge-response protocol is implemented.
    fn retrieve_challenge_response(&self, challenge: &str) -> Result<String, Error> {
        let challenge_path = Path::new(challenge).canonicalize()?;
        let expected_challenge_base = if cfg!(windows) {
            let program_data_dir = env::var("PROGRAMDATA").map_err(|e| {
                Error::with_error(
                    ErrorKind::Io,
                    e,
                    "Could not find %PROGRAMDATA% variable to perform Arc challenge-response",
                )
            })?;
            Path::new(&program_data_dir).join("AzureConnectedMachineAgent\\Tokens\\")
        } else {
            Path::new("/var/opt/azcmagent/tokens/").to_path_buf()
        };

        if !(challenge_path.starts_with(expected_challenge_base)
            && challenge_path.extension().is_some_and(|ext| ext == "key"))
        {
            if !cfg!(test) {
                return Err(Error::with_message(
                    ErrorKind::Credential,
                    format!("Challenge received was invalid: {challenge}"),
                ));
            } else {
                // for tests, it's okay if the challenge file is not in the expected location
            }
        }

        let challenge_file = File::open(challenge_path)?;
        let mut challenge_response = String::new();
        let _ = challenge_file
            .take(4096) // avoid slurping a huge file - the challenge response will not be > 4KiB
            .read_to_string(&mut challenge_response)?;

        Ok(challenge_response.trim().to_owned())
    }
}

// NOTE: expires_on is a String version of unix epoch time, not an integer.
// https://learn.microsoft.com/azure/app-service/overview-managed-identity?tabs=dotnet#rest-protocol-examples
#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
struct MsiTokenResponse {
    pub access_token: Secret,
    #[serde(deserialize_with = "expires_on_string")]
    pub expires_on: OffsetDateTime,
    pub token_type: String,
    pub resource: String,
}

fn expires_on_string<'de, D>(deserializer: D) -> Result<OffsetDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let v = String::deserialize(deserializer)?;
    let as_i64 = v.parse::<i64>().map_err(de::Error::custom)?;
    OffsetDateTime::from_unix_timestamp(as_i64).map_err(de::Error::custom)
}

// this is copied from https://github.com/Azure/azure-sdk-for-rust/blob/f5fab74c3baf37653c4b5118bbbc1a485ceda2da/sdk/identity/azure_identity/src/cache.rs
#[derive(Debug)]
struct TokenCache(RwLock<HashMap<Vec<String>, AccessToken>>);

impl TokenCache {
    fn new() -> Self {
        Self(RwLock::new(HashMap::new()))
    }

    async fn get_token<'a, C, F>(
        &self,
        scopes: &'a [&'a str],
        options: Option<TokenRequestOptions<'a>>,
        callback: C,
    ) -> azure_core::Result<AccessToken>
    where
        C: FnOnce(&'a [&'a str], Option<TokenRequestOptions<'a>>) -> F + Send,
        F: Future<Output = azure_core::Result<AccessToken>> + Send,
    {
        let token_cache = self.0.read().await;
        let scopes_owned = scopes.iter().map(ToString::to_string).collect::<Vec<_>>();
        if let Some(token) = token_cache.get(&scopes_owned) {
            if !should_refresh(token) {
                return Ok(token.clone());
            }
        }

        // otherwise, drop the read lock and get a write lock to refresh the token
        drop(token_cache);
        let mut token_cache = self.0.write().await;

        // check again in case another thread refreshed the token while we were
        // waiting on the write lock
        if let Some(token) = token_cache.get(&scopes_owned) {
            if !should_refresh(token) {
                return Ok(token.clone());
            }
        }

        let token = callback(scopes, options).await?;
        let _ = token_cache.insert(scopes_owned, token.clone());
        Ok(token)
    }
}

impl Default for TokenCache {
    fn default() -> Self {
        TokenCache::new()
    }
}

fn should_refresh(token: &AccessToken) -> bool {
    token.expires_on <= OffsetDateTime::now_utc() + Duration::seconds(300)
}

pub(crate) fn is_arc_server_environment() -> bool {
    if cfg!(windows) {
        if let Ok(program_files_path) = env::var("PROGRAMFILES") {
            !program_files_path.is_empty()
                && fs::exists(format!(
                    "{program_files_path}\\AzureConnectedMachineAgent\\himds.exe"
                ))
                .unwrap_or(false)
        } else {
            // %PROGRAMFILES% should exist on Windows, but if it's not there,
            // we can't tell if we're on Arc or not, so just assume we aren't
            false
        }
    } else {
        fs::exists("/opt/azcmagent/bin/himds").unwrap_or(false)
    }
}

/// Convert a `AADv2` scope to an `AADv1` resource
/// copied from https://github.com/Azure/azure-sdk-for-rust/blob/f5fab74c3baf37653c4b5118bbbc1a485ceda2da/sdk/identity/azure_identity/src/imds_managed_identity_credential.rs#L206
fn scopes_to_resource<'a>(scopes: &'a [&'a str]) -> azure_core::Result<&'a str> {
    if scopes.len() != 1 {
        return Err(Error::with_message(
            ErrorKind::Credential,
            "only one scope is supported for IMDS authentication",
        ));
    }

    let Some(scope) = scopes.first() else {
        return Err(Error::with_message(
            ErrorKind::Credential,
            "no scopes were provided",
        ));
    };

    Ok(scope.strip_suffix("/.default").unwrap_or(*scope))
}
