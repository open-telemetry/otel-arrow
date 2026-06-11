// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Utilities for connecting to AWS Managed Kafka (MSK)

use aws_config::Region;
use aws_msk_iam_sasl_signer::generate_auth_token;
use rdkafka::ClientContext;
use rdkafka::client::{DefaultClientContext, OAuthToken};
use rdkafka::config::RDKafkaLogLevel;
use rdkafka::consumer::ConsumerContext;
use rdkafka::error::KafkaError;
use rdkafka::statistics::Statistics;

/// [`ClientContext`] implementation that can provide auth token for connecting to kafka using
/// AWS MSK IAM SASL Signer
pub struct AwsMskAuthClientContext {
    region: Region,
}

impl AwsMskAuthClientContext {
    /// Create a new instance of the client context
    #[must_use]
    pub fn new(region: Region) -> Self {
        Self { region }
    }
}

impl ConsumerContext for AwsMskAuthClientContext {}

impl ClientContext for AwsMskAuthClientContext {
    /// Periodically refresh the SASL `OAUTHBEARER` token. This parameter is relevant when using
    /// the `OAUTHBEARER` SASL mechanism (see trait documentation for more details).
    const ENABLE_REFRESH_OAUTH_TOKEN: bool = true;

    /// Fetch OAUTH token
    fn generate_oauth_token(
        &self,
        _oauthbearer_config: Option<&str>,
    ) -> Result<OAuthToken, Box<dyn std::error::Error>> {
        let region = self.region.clone();

        let (token, lifetime_ms) = std::thread::spawn(move || {
            // create new Tokio runtime so as not to block the current if running in a single
            // threaded runtime.
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to create tokio runtime");
            rt.block_on(async { generate_auth_token(region).await })
        })
        .join()
        .map_err(|e| format!("Token generation thread panicked: {e:?}"))??;

        Ok(OAuthToken {
            token,
            lifetime_ms,
            principal_name: String::default(),
        })
    }
}

/// Client context enum for Kafka producers, dispatching to either the default
/// context or the AWS MSK IAM auth context.
///
/// This allows the exporter to use a single concrete producer type
/// (`ExporterFutureProducer<ProducerClientContext>`) regardless of whether
/// authentication is configured.
pub enum ProducerClientContext {
    /// No special authentication — uses the default rdkafka client context.
    Default(DefaultClientContext),
    /// AWS MSK IAM OAUTHBEARER authentication.
    AwsMsk(AwsMskAuthClientContext),
}

impl ClientContext for ProducerClientContext {
    const ENABLE_REFRESH_OAUTH_TOKEN: bool = true;

    fn log(&self, level: RDKafkaLogLevel, fac: &str, log_message: &str) {
        match self {
            ProducerClientContext::Default(ctx) => ctx.log(level, fac, log_message),
            ProducerClientContext::AwsMsk(ctx) => ctx.log(level, fac, log_message),
        }
    }

    fn stats(&self, statistics: Statistics) {
        match self {
            ProducerClientContext::Default(ctx) => ctx.stats(statistics),
            ProducerClientContext::AwsMsk(ctx) => ctx.stats(statistics),
        }
    }

    fn stats_raw(&self, statistics: &[u8]) {
        match self {
            ProducerClientContext::Default(ctx) => ctx.stats_raw(statistics),
            ProducerClientContext::AwsMsk(ctx) => ctx.stats_raw(statistics),
        }
    }

    fn error(&self, error: KafkaError, reason: &str) {
        match self {
            ProducerClientContext::Default(ctx) => ctx.error(error, reason),
            ProducerClientContext::AwsMsk(ctx) => ctx.error(error, reason),
        }
    }

    fn generate_oauth_token(
        &self,
        oauthbearer_config: Option<&str>,
    ) -> Result<OAuthToken, Box<dyn std::error::Error>> {
        match self {
            ProducerClientContext::Default(ctx) => ctx.generate_oauth_token(oauthbearer_config),
            ProducerClientContext::AwsMsk(ctx) => ctx.generate_oauth_token(oauthbearer_config),
        }
    }
}
