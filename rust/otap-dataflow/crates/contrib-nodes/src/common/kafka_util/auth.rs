//! Shared authentication types for Kafka receiver and exporter.

use serde::Deserialize;

/// Supported SASL authentication mechanisms.
///
/// Serde renames ensure the enum deserializes from the exact user-facing
/// config strings (e.g., `"PLAIN"`, `"SCRAM-SHA-256"`).  Unknown strings
/// are rejected at deserialization time — no separate runtime validation
/// is needed for the mechanism itself.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Deserialize)]
pub enum SaslMechanism {
    /// SASL PLAIN (username + password).
    #[serde(rename = "PLAIN")]
    Plain,

    /// SASL SCRAM-SHA-256 (username + password).
    #[serde(rename = "SCRAM-SHA-256")]
    ScramSha256,

    /// SASL SCRAM-SHA-512 (username + password).
    #[serde(rename = "SCRAM-SHA-512")]
    ScramSha512,

    /// AWS MSK IAM via OAUTHBEARER token refresh.
    #[serde(rename = "AWS_MSK_IAM_OAUTHBEARER")]
    AwsMskIamOauthbearer,
}

impl SaslMechanism {
    /// Returns the mechanism string expected by librdkafka's `sasl.mechanism`
    /// configuration property.
    ///
    /// For AWS MSK IAM this returns `"OAUTHBEARER"` because the actual SASL
    /// handshake uses the OAUTHBEARER protocol; the MSK-specific token refresh
    /// is handled by the client context callback.
    #[must_use]
    pub fn as_rdkafka_str(&self) -> &'static str {
        match self {
            Self::Plain => "PLAIN",
            Self::ScramSha256 => "SCRAM-SHA-256",
            Self::ScramSha512 => "SCRAM-SHA-512",
            Self::AwsMskIamOauthbearer => "OAUTHBEARER",
        }
    }

    /// Returns `true` if the mechanism uses username/password credentials.
    #[must_use]
    pub fn is_username_password(&self) -> bool {
        matches!(self, Self::Plain | Self::ScramSha256 | Self::ScramSha512)
    }
}

/// Configuration for Kafka Auth.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Auth {
    /// SASL authentication.
    Sasl(SaslAuth),
}

/// Configuration for SASL Auth.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SaslAuth {
    /// SASL mechanism.
    mechanism: SaslMechanism,

    /// Username for PLAIN / SCRAM authentication.
    #[serde(default)]
    username: Option<String>,

    /// Password for PLAIN / SCRAM authentication.
    #[serde(default)]
    password: Option<String>,

    /// Optional AWS MSK-specific configuration.
    #[serde(default)]
    aws_msk: Option<AwsMskAuth>,
}

impl SaslAuth {
    /// Create a new SASL authentication configuration for AWS MSK or other
    /// mechanisms that do not use username/password.
    #[must_use]
    pub fn new(mechanism: SaslMechanism, aws_msk: Option<AwsMskAuth>) -> Self {
        Self {
            mechanism,
            username: None,
            password: None,
            aws_msk,
        }
    }

    /// Create a SASL authentication configuration for username/password-based
    /// mechanisms (PLAIN, SCRAM-SHA-256, SCRAM-SHA-512).
    #[must_use]
    pub fn new_username_password(
        mechanism: SaslMechanism,
        username: String,
        password: String,
    ) -> Self {
        Self {
            mechanism,
            username: Some(username),
            password: Some(password),
            aws_msk: None,
        }
    }

    /// The SASL mechanism.
    #[must_use]
    pub fn mechanism(&self) -> SaslMechanism {
        self.mechanism
    }

    /// Username for PLAIN / SCRAM authentication.
    #[must_use]
    pub fn username(&self) -> Option<&str> {
        self.username.as_deref()
    }

    /// Password for PLAIN / SCRAM authentication.
    #[must_use]
    pub fn password(&self) -> Option<&str> {
        self.password.as_deref()
    }

    /// Optional AWS MSK-specific configuration.
    #[must_use]
    pub fn aws_msk(&self) -> Option<&AwsMskAuth> {
        self.aws_msk.as_ref()
    }

    /// Validate the SASL authentication configuration.
    ///
    /// The mechanism itself is already validated by serde deserialization
    /// (unknown variants are rejected).  This method checks that the
    /// mechanism-specific fields are present and non-empty, and rejects
    /// fields that are incompatible with the chosen mechanism.
    ///
    /// # Errors
    ///
    /// Returns a human-readable description if:
    /// - A username/password mechanism is missing `username` or `password`.
    /// - A username/password mechanism has an `aws_msk` block.
    /// - `AwsMskIamOauthbearer` is missing the `aws_msk` block or region.
    /// - `AwsMskIamOauthbearer` has `username` or `password` set.
    pub fn validate(&self) -> Result<(), String> {
        if self.mechanism.is_username_password() {
            match (&self.username, &self.password) {
                (Some(u), Some(p)) if !u.is_empty() && !p.is_empty() => {}
                _ => {
                    return Err(format!(
                        "SASL mechanism '{}' requires non-empty 'username' and 'password'",
                        self.mechanism.as_rdkafka_str()
                    ));
                }
            }

            // Reject aws_msk block for username/password mechanisms.
            if self.aws_msk.is_some() {
                return Err(format!(
                    "SASL mechanism '{}' does not support 'aws_msk' configuration",
                    self.mechanism.as_rdkafka_str()
                ));
            }
        }

        if self.mechanism == SaslMechanism::AwsMskIamOauthbearer {
            match &self.aws_msk {
                Some(msk) if !msk.region().is_empty() => {}
                _ => {
                    return Err(
                        "SASL mechanism 'AWS_MSK_IAM_OAUTHBEARER' requires 'aws_msk' \
                         with a non-empty 'region'"
                            .to_string(),
                    );
                }
            }

            // Reject username/password for OAUTHBEARER.
            if self.username.is_some() || self.password.is_some() {
                return Err("SASL mechanism 'AWS_MSK_IAM_OAUTHBEARER' does not support \
                     'username' or 'password'"
                    .to_string());
            }
        }

        Ok(())
    }
}

/// AWS MSK IAM authentication configuration.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AwsMskAuth {
    /// AWS region for the MSK cluster.
    region: String,
}

impl AwsMskAuth {
    /// Create a new AWS MSK authentication configuration.
    #[must_use]
    pub fn new(region: String) -> Self {
        Self { region }
    }

    /// The AWS region for the MSK cluster.
    #[must_use]
    pub fn region(&self) -> &str {
        &self.region
    }
}

impl Auth {
    /// Convenience constructor for AWS MSK IAM OAUTHBEARER authentication.
    #[must_use]
    pub fn new_aws_msk_iam(region: String) -> Self {
        Self::Sasl(SaslAuth::new(
            SaslMechanism::AwsMskIamOauthbearer,
            Some(AwsMskAuth::new(region)),
        ))
    }

    /// Convenience constructor for username/password-based SASL mechanisms
    /// (PLAIN, SCRAM-SHA-256, SCRAM-SHA-512).
    #[must_use]
    pub fn new_sasl(mechanism: SaslMechanism, username: String, password: String) -> Self {
        Self::Sasl(SaslAuth::new_username_password(
            mechanism, username, password,
        ))
    }

    /// Validate the authentication configuration.
    ///
    /// Delegates to the inner variant's validation method.
    ///
    /// # Errors
    ///
    /// Returns a human-readable description of the first validation error.
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Auth::Sasl(sasl) => sasl.validate(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── SaslMechanism ───────────────────────────────────────

    #[test]
    fn mechanism_as_rdkafka_str() {
        assert_eq!(SaslMechanism::Plain.as_rdkafka_str(), "PLAIN");
        assert_eq!(SaslMechanism::ScramSha256.as_rdkafka_str(), "SCRAM-SHA-256");
        assert_eq!(SaslMechanism::ScramSha512.as_rdkafka_str(), "SCRAM-SHA-512");
        assert_eq!(
            SaslMechanism::AwsMskIamOauthbearer.as_rdkafka_str(),
            "OAUTHBEARER"
        );
    }

    #[test]
    fn mechanism_is_username_password() {
        assert!(SaslMechanism::Plain.is_username_password());
        assert!(SaslMechanism::ScramSha256.is_username_password());
        assert!(SaslMechanism::ScramSha512.is_username_password());
        assert!(!SaslMechanism::AwsMskIamOauthbearer.is_username_password());
    }

    // ── SaslAuth::validate ──────────────────────────────────

    #[test]
    fn validate_plain_with_credentials_succeeds() {
        let sasl = SaslAuth::new_username_password(
            SaslMechanism::Plain,
            "user".to_string(),
            "pass".to_string(),
        );
        assert!(sasl.validate().is_ok());
    }

    #[test]
    fn validate_scram_sha256_with_credentials_succeeds() {
        let sasl = SaslAuth::new_username_password(
            SaslMechanism::ScramSha256,
            "user".to_string(),
            "pass".to_string(),
        );
        assert!(sasl.validate().is_ok());
    }

    #[test]
    fn validate_scram_sha512_with_credentials_succeeds() {
        let sasl = SaslAuth::new_username_password(
            SaslMechanism::ScramSha512,
            "user".to_string(),
            "pass".to_string(),
        );
        assert!(sasl.validate().is_ok());
    }

    #[test]
    fn validate_plain_missing_username_fails() {
        let sasl = SaslAuth {
            mechanism: SaslMechanism::Plain,
            username: None,
            password: Some("pass".to_string()),
            aws_msk: None,
        };
        let err = sasl.validate().unwrap_err();
        assert!(err.contains("username"), "unexpected error: {err}");
    }

    #[test]
    fn validate_plain_missing_password_fails() {
        let sasl = SaslAuth {
            mechanism: SaslMechanism::Plain,
            username: Some("user".to_string()),
            password: None,
            aws_msk: None,
        };
        let err = sasl.validate().unwrap_err();
        assert!(err.contains("password"), "unexpected error: {err}");
    }

    #[test]
    fn validate_plain_empty_username_fails() {
        let sasl = SaslAuth::new_username_password(
            SaslMechanism::Plain,
            "".to_string(),
            "pass".to_string(),
        );
        let err = sasl.validate().unwrap_err();
        assert!(err.contains("non-empty"), "unexpected error: {err}");
    }

    #[test]
    fn validate_plain_empty_password_fails() {
        let sasl = SaslAuth::new_username_password(
            SaslMechanism::ScramSha512,
            "user".to_string(),
            "".to_string(),
        );
        let err = sasl.validate().unwrap_err();
        assert!(err.contains("non-empty"), "unexpected error: {err}");
    }

    #[test]
    fn validate_plain_with_aws_msk_block_fails() {
        let sasl = SaslAuth {
            mechanism: SaslMechanism::Plain,
            username: Some("user".to_string()),
            password: Some("pass".to_string()),
            aws_msk: Some(AwsMskAuth::new("us-east-1".to_string())),
        };
        let err = sasl.validate().unwrap_err();
        assert!(
            err.contains("does not support 'aws_msk'"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn validate_scram_sha256_with_aws_msk_block_fails() {
        let sasl = SaslAuth {
            mechanism: SaslMechanism::ScramSha256,
            username: Some("user".to_string()),
            password: Some("pass".to_string()),
            aws_msk: Some(AwsMskAuth::new("us-east-1".to_string())),
        };
        let err = sasl.validate().unwrap_err();
        assert!(
            err.contains("does not support 'aws_msk'"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn validate_aws_msk_with_username_fails() {
        let sasl = SaslAuth {
            mechanism: SaslMechanism::AwsMskIamOauthbearer,
            username: Some("user".to_string()),
            password: None,
            aws_msk: Some(AwsMskAuth::new("us-east-1".to_string())),
        };
        let err = sasl.validate().unwrap_err();
        assert!(
            err.contains("does not support 'username' or 'password'"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn validate_aws_msk_with_password_fails() {
        let sasl = SaslAuth {
            mechanism: SaslMechanism::AwsMskIamOauthbearer,
            username: None,
            password: Some("pass".to_string()),
            aws_msk: Some(AwsMskAuth::new("us-east-1".to_string())),
        };
        let err = sasl.validate().unwrap_err();
        assert!(
            err.contains("does not support 'username' or 'password'"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn validate_aws_msk_with_both_credentials_fails() {
        let sasl = SaslAuth {
            mechanism: SaslMechanism::AwsMskIamOauthbearer,
            username: Some("user".to_string()),
            password: Some("pass".to_string()),
            aws_msk: Some(AwsMskAuth::new("us-east-1".to_string())),
        };
        let err = sasl.validate().unwrap_err();
        assert!(
            err.contains("does not support 'username' or 'password'"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn validate_aws_msk_with_region_succeeds() {
        let sasl = SaslAuth::new(
            SaslMechanism::AwsMskIamOauthbearer,
            Some(AwsMskAuth::new("us-east-1".to_string())),
        );
        assert!(sasl.validate().is_ok());
    }

    #[test]
    fn validate_aws_msk_missing_aws_msk_block_fails() {
        let sasl = SaslAuth::new(SaslMechanism::AwsMskIamOauthbearer, None);
        let err = sasl.validate().unwrap_err();
        assert!(err.contains("aws_msk"), "unexpected error: {err}");
    }

    #[test]
    fn validate_aws_msk_empty_region_fails() {
        let sasl = SaslAuth::new(
            SaslMechanism::AwsMskIamOauthbearer,
            Some(AwsMskAuth::new("".to_string())),
        );
        let err = sasl.validate().unwrap_err();
        assert!(err.contains("region"), "unexpected error: {err}");
    }

    // ── Auth::validate (delegates) ──────────────────────────

    #[test]
    fn auth_validate_delegates_to_sasl() {
        let auth = Auth::new_aws_msk_iam("us-west-2".to_string());
        assert!(auth.validate().is_ok());
    }

    #[test]
    fn auth_validate_sasl_credentials() {
        let auth = Auth::new_sasl(SaslMechanism::Plain, "user".to_string(), "pass".to_string());
        assert!(auth.validate().is_ok());
    }

    // ── Deserialization ─────────────────────────────────────

    #[test]
    fn deserialize_mechanism_plain() {
        let json = r#""PLAIN""#;
        let m: SaslMechanism = serde_json::from_str(json).expect("valid mechanism");
        assert_eq!(m, SaslMechanism::Plain);
    }

    #[test]
    fn deserialize_mechanism_scram_sha256() {
        let json = r#""SCRAM-SHA-256""#;
        let m: SaslMechanism = serde_json::from_str(json).expect("valid mechanism");
        assert_eq!(m, SaslMechanism::ScramSha256);
    }

    #[test]
    fn deserialize_mechanism_scram_sha512() {
        let json = r#""SCRAM-SHA-512""#;
        let m: SaslMechanism = serde_json::from_str(json).expect("valid mechanism");
        assert_eq!(m, SaslMechanism::ScramSha512);
    }

    #[test]
    fn deserialize_mechanism_aws_msk() {
        let json = r#""AWS_MSK_IAM_OAUTHBEARER""#;
        let m: SaslMechanism = serde_json::from_str(json).expect("valid mechanism");
        assert_eq!(m, SaslMechanism::AwsMskIamOauthbearer);
    }

    #[test]
    fn deserialize_unknown_mechanism_fails() {
        let json = r#""KERBEROS""#;
        let result = serde_json::from_str::<SaslMechanism>(json);
        assert!(result.is_err());
    }

    #[test]
    fn deserialize_plain_sasl() {
        let json = r#"{
            "sasl": {
                "mechanism": "PLAIN",
                "username": "myuser",
                "password": "mypass"
            }
        }"#;
        let auth: Auth = serde_json::from_str(json).expect("valid auth");
        match &auth {
            Auth::Sasl(sasl) => {
                assert_eq!(sasl.mechanism(), SaslMechanism::Plain);
                assert_eq!(sasl.username(), Some("myuser"));
                assert_eq!(sasl.password(), Some("mypass"));
                assert!(sasl.aws_msk().is_none());
            }
        }
        assert!(auth.validate().is_ok());
    }

    #[test]
    fn deserialize_scram_sha512_sasl() {
        let json = r#"{
            "sasl": {
                "mechanism": "SCRAM-SHA-512",
                "username": "myuser",
                "password": "mypass"
            }
        }"#;
        let auth: Auth = serde_json::from_str(json).expect("valid auth");
        match &auth {
            Auth::Sasl(sasl) => {
                assert_eq!(sasl.mechanism(), SaslMechanism::ScramSha512);
                assert_eq!(sasl.username(), Some("myuser"));
                assert_eq!(sasl.password(), Some("mypass"));
            }
        }
        assert!(auth.validate().is_ok());
    }

    #[test]
    fn deserialize_aws_msk_backward_compatible() {
        let json = r#"{
            "sasl": {
                "mechanism": "AWS_MSK_IAM_OAUTHBEARER",
                "aws_msk": {
                    "region": "us-east-1"
                }
            }
        }"#;
        let auth: Auth = serde_json::from_str(json).expect("valid auth");
        match &auth {
            Auth::Sasl(sasl) => {
                assert_eq!(sasl.mechanism(), SaslMechanism::AwsMskIamOauthbearer);
                assert!(sasl.username().is_none());
                assert!(sasl.password().is_none());
                assert_eq!(sasl.aws_msk().unwrap().region(), "us-east-1");
            }
        }
        assert!(auth.validate().is_ok());
    }

    #[test]
    fn deserialize_unknown_mechanism_in_auth_fails() {
        let json = r#"{
            "sasl": {
                "mechanism": "KERBEROS"
            }
        }"#;
        let result = serde_json::from_str::<Auth>(json);
        assert!(result.is_err());
    }
}
