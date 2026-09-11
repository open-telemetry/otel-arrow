// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Consumer callbacks served inline by `recv()`.
//!
//! `RebalancingConsumerContext`'s `ConsumerContext` callbacks
//! (pre/post-rebalance and the commit callback) are served inline by
//! `consumer.recv()` (rdkafka 0.38.0: `MessageStream::poll_next` ->
//! `BaseConsumer::poll_queue` runs any queued rebalance/commit event on the
//! calling thread). Because the receive loop is the only caller of `recv()`,
//! they run on the loop's single-threaded pipeline thread, interleaved between
//! records -- not on a separate librdkafka poll thread. Only the
//! `ClientContext` OAUTHBEARER token refresh (AWS MSK IAM variant) may run on
//! a librdkafka-internal thread.
//!
//! The context records partition assignments/revocations and folds commit
//! results into the shared [`RebalanceState`], commits owned partitions before
//! they are revoked (commit-before-revoke), and, for the AWS MSK IAM variant,
//! refreshes the OAUTHBEARER token. It never touches the receive loop's state
//! directly.
//!
//! NOTE: because these callbacks run on the pipeline thread, the synchronous
//! commit-before-revoke in `RebalanceState::handle_revoke` (a `CommitMode::Sync`
//! broker round-trip) can block the receive loop during a rebalance. It is
//! bounded by librdkafka's internal commit timeout; moving it off the pipeline
//! thread is future work.

use super::RebalanceState;
#[cfg(feature = "aws")]
use crate::common::kafka::aws::AwsMskAuthClientContext;
use rdkafka::ClientContext;
use rdkafka::client::OAuthToken;
use rdkafka::consumer::{BaseConsumer, ConsumerContext, Rebalance};
use rdkafka::topic_partition_list::TopicPartitionList;

/// A [`ConsumerContext`] that records partition assignments and commits offsets
/// before partitions are revoked.
///
/// Wraps either the default (no-auth) context or the AWS MSK IAM auth context
/// so that both authentication modes get rebalance handling. The variant is
/// chosen at consumer-creation time based on the receiver's auth config.
pub(crate) enum RebalancingConsumerContext {
    /// No special authentication.
    Default(std::sync::Arc<RebalanceState>),
    /// AWS MSK IAM OAUTHBEARER authentication, delegating token refresh to the
    /// wrapped context.
    #[cfg(feature = "aws")]
    AwsMsk {
        /// Inner context providing the OAUTHBEARER token.
        inner: AwsMskAuthClientContext,
        /// Shared rebalance state.
        state: std::sync::Arc<RebalanceState>,
    },
}

impl RebalancingConsumerContext {
    fn state(&self) -> &RebalanceState {
        match self {
            RebalancingConsumerContext::Default(state) => state,
            #[cfg(feature = "aws")]
            RebalancingConsumerContext::AwsMsk { state, .. } => state,
        }
    }
}

impl ClientContext for RebalancingConsumerContext {
    // Mirror `AwsMskAuthClientContext`: the AWS variant needs periodic
    // OAUTHBEARER token refresh.
    //
    // This constant is inert unless SASL OAUTHBEARER is configured: librdkafka
    // only emits the token-refresh event (and thus only calls
    // `generate_oauth_token`) for the OAUTHBEARER mechanism. For the default,
    // non-AWS variant (plaintext/SSL/SCRAM) the event never fires, so leaving
    // it `true` is harmless; the `Err(..)` returned below is a defensive
    // fallback that should be unreachable in practice.
    const ENABLE_REFRESH_OAUTH_TOKEN: bool = true;

    fn generate_oauth_token(
        &self,
        oauthbearer_config: Option<&str>,
    ) -> Result<OAuthToken, Box<dyn std::error::Error>> {
        // `oauthbearer_config` is only consumed by the AWS MSK variant, which
        // is compiled out when the `aws` feature is disabled.
        let _ = oauthbearer_config;
        match self {
            RebalancingConsumerContext::Default(_) => {
                Err("OAUTH token refresh is not configured for this consumer".into())
            }
            #[cfg(feature = "aws")]
            RebalancingConsumerContext::AwsMsk { inner, .. } => {
                inner.generate_oauth_token(oauthbearer_config)
            }
        }
    }
}

impl ConsumerContext for RebalancingConsumerContext {
    // Served inline by `consumer.recv()`, so this runs on the pipeline thread.
    // The `Revoke` arm calls `handle_revoke`, which issues a synchronous
    // (`CommitMode::Sync`) commit-before-revoke and can therefore block the
    // receive loop for the broker round-trip during a rebalance. Bounded by
    // librdkafka's internal commit timeout; off-thread commit is future work.
    fn pre_rebalance(&self, base_consumer: &BaseConsumer<Self>, rebalance: &Rebalance<'_>) {
        let state = self.state();
        if state.is_auto_commit() {
            return;
        }
        match rebalance {
            Rebalance::Revoke(tpl) => {
                state.handle_revoke(base_consumer, tpl);
            }
            Rebalance::Assign(_) => {
                // Assignment is recorded in post_rebalance once it is in effect.
            }
            Rebalance::Error(err) => {
                otel_warn!(
                    "kafka.rebalance.error",
                    error = %err,
                );
            }
        }
    }

    fn post_rebalance(&self, base_consumer: &BaseConsumer<Self>, rebalance: &Rebalance<'_>) {
        let state = self.state();
        if state.is_auto_commit() {
            return;
        }
        match rebalance {
            Rebalance::Assign(tpl) => {
                state.handle_assign(base_consumer, tpl);
            }
            Rebalance::Revoke(_) => {
                // Revocation bookkeeping already happened in pre_rebalance.
            }
            Rebalance::Error(err) => {
                otel_warn!(
                    "kafka.rebalance.error",
                    error = %err,
                );
            }
        }
    }

    fn commit_callback(
        &self,
        result: rdkafka::error::KafkaResult<()>,
        _offsets: &TopicPartitionList,
    ) {
        let state = self.state();
        // In auto-commit mode librdkafka manages offsets itself; keep manual-mode
        // commit metrics clean by ignoring those callbacks.
        if state.is_auto_commit() {
            return;
        }
        // Single source of truth for commit success/failure metrics: the
        // receiver's steady-state commits are asynchronous, so the broker
        // outcome is only known here (not at commit-issue time).
        state.record_commit_result(&result);
    }
}
