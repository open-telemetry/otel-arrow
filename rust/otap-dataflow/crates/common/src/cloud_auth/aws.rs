// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use object_store::aws::{AmazonS3Builder, AmazonS3ConfigKey};
use serde::{Deserialize, Serialize};

use crate::cloud_auth::opaque_string::OpaqueString;

/// AWS authentication methods. This can be leveraged in component
/// configuration objects for a consistent way to specify AWS auth information.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuthMethod {
    /// Load credentials from the standard AWS credential chain
    /// (env vars, shared config files, instance metadata, etc.).
    Default,

    /// Static credentials.
    StaticCredentials {
        /// AWS access key id.
        access_key_id: String,

        /// AWS secret access key.
        secret_access_key: OpaqueString,

        /// Optional session token.
        session_token: Option<OpaqueString>,
    },

    /// Assume role with web identity token, commonly used in EKS.
    WebIdentity {
        /// IAM role ARN. If omitted, use `AWS_ROLE_ARN`.
        role_arn: Option<String>,

        /// Path to web identity token file. If omitted, use
        /// `AWS_WEB_IDENTITY_TOKEN_FILE`.
        token_file_path: Option<String>,
    },

    /// Assume a role using STS parameters supported by object_store.
    ///
    /// Notes:
    /// - `external_id` is accepted in config for forward compatibility.
    /// - Current object_store S3 config keys do not expose an explicit
    ///   external-id setting, so this field is currently not applied.
    AssumeRole {
        /// IAM role ARN to assume.
        role_arn: String,

        /// External ID for third-party role assumption.
        external_id: Option<String>,

        /// Optional STS session name.
        session_name: Option<String>,
    },
}

/// Configure an [AmazonS3Builder] based on [AuthMethod].
pub fn configure_builder(builder: AmazonS3Builder, auth: &AuthMethod) -> AmazonS3Builder {
    match auth {
        AuthMethod::Default => builder,
        AuthMethod::StaticCredentials {
            access_key_id,
            secret_access_key,
            session_token,
        } => {
            let mut configured = builder
                .with_access_key_id(access_key_id)
                .with_secret_access_key(secret_access_key.as_ref());
            if let Some(token) = session_token {
                configured = configured.with_token(token.as_ref());
            }
            configured
        }
        AuthMethod::WebIdentity {
            role_arn,
            token_file_path,
        } => {
            let mut configured = builder;
            if let Some(value) = role_arn {
                configured = configured.with_config(AmazonS3ConfigKey::RoleArn, value);
            }
            if let Some(value) = token_file_path {
                configured = configured.with_config(AmazonS3ConfigKey::WebIdentityTokenFile, value);
            }
            configured
        }
        AuthMethod::AssumeRole {
            role_arn,
            external_id: _,
            session_name,
        } => {
            let mut configured = builder.with_config(AmazonS3ConfigKey::RoleArn, role_arn);
            if let Some(value) = session_name {
                configured = configured.with_config(AmazonS3ConfigKey::RoleSessionName, value);
            }
            configured
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_default() {
        let json = json!({
            "type": "default"
        })
        .to_string();

        let method: AuthMethod = serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(method, AuthMethod::Default);
    }

    #[test]
    fn test_static_credentials() {
        let json = json!({
            "type": "static_credentials",
            "access_key_id": "test-access-key",
            "secret_access_key": "test-secret-key",
            "session_token": "test-session-token"
        })
        .to_string();

        let method: AuthMethod = serde_json::from_str(&json).expect("Failed to deserialize");
        let expected = AuthMethod::StaticCredentials {
            access_key_id: "test-access-key".to_string(),
            secret_access_key: "test-secret-key".into(),
            session_token: Some("test-session-token".into()),
        };

        assert_eq!(method, expected);
    }

    #[test]
    fn test_static_credentials_debug_redacts_secrets() {
        let method = AuthMethod::StaticCredentials {
            access_key_id: "AKIA...".to_string(),
            secret_access_key: "super-secret".into(),
            session_token: Some("token-value".into()),
        };
        let debug = format!("{method:?}");
        assert!(!debug.contains("super-secret"));
        assert!(!debug.contains("token-value"));
    }

    #[test]
    fn test_web_identity() {
        let json = json!({
            "type": "web_identity",
            "role_arn": "arn:aws:iam::123456789012:role/TestRole",
            "token_file_path": "/var/run/secrets/eks.amazonaws.com/serviceaccount/token"
        })
        .to_string();

        let method: AuthMethod = serde_json::from_str(&json).expect("Failed to deserialize");
        let expected = AuthMethod::WebIdentity {
            role_arn: Some("arn:aws:iam::123456789012:role/TestRole".to_string()),
            token_file_path: Some(
                "/var/run/secrets/eks.amazonaws.com/serviceaccount/token".to_string(),
            ),
        };

        assert_eq!(method, expected);
    }

    #[test]
    fn test_assume_role() {
        let json = json!({
            "type": "assume_role",
            "role_arn": "arn:aws:iam::123456789012:role/CrossAccountRole",
            "external_id": "my-external-id",
            "session_name": "otap-session"
        })
        .to_string();

        let method: AuthMethod = serde_json::from_str(&json).expect("Failed to deserialize");
        let expected = AuthMethod::AssumeRole {
            role_arn: "arn:aws:iam::123456789012:role/CrossAccountRole".to_string(),
            external_id: Some("my-external-id".to_string()),
            session_name: Some("otap-session".to_string()),
        };

        assert_eq!(method, expected);
    }
}
