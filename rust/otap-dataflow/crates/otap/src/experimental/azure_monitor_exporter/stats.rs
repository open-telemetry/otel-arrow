// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

const IDLE_THRESHOLD_SECS: f64 = 1.0;

/// Statistics for the Azure Monitor exporter.
#[derive(Debug)]
pub struct AzureMonitorExporterStats {
    processing_started_at: tokio::time::Instant,
    last_message_received_at: tokio::time::Instant,
    idle_duration: tokio::time::Duration,
    successful_row_count: f64,
    successful_batch_count: f64,
    successful_msg_count: f64,
    failed_row_count: f64,
    failed_batch_count: f64,
    failed_msg_count: f64,
    average_client_latency_secs: f64,
    latency_request_count: f64,
}

impl Default for AzureMonitorExporterStats {
    fn default() -> Self {
        Self::new()
    }
}

impl AzureMonitorExporterStats {
    /// Create a new stats tracker.
    #[must_use]
    pub fn new() -> Self {
        Self {
            processing_started_at: tokio::time::Instant::now(),
            last_message_received_at: tokio::time::Instant::now(),
            idle_duration: tokio::time::Duration::ZERO,
            successful_row_count: 0.0,
            successful_batch_count: 0.0,
            successful_msg_count: 0.0,
            failed_row_count: 0.0,
            failed_batch_count: 0.0,
            failed_msg_count: 0.0,
            average_client_latency_secs: 0.0,
            latency_request_count: 0.0,
        }
    }

    /// Get the instant when processing started.
    #[must_use]
    pub fn started_at(&self) -> tokio::time::Instant {
        self.processing_started_at
    }

    /// Get the instant when the last message was received.
    #[must_use]
    pub fn last_message_received_at(&self) -> tokio::time::Instant {
        self.last_message_received_at
    }

    /// Get the total idle duration.
    #[must_use]
    pub fn idle_duration(&self) -> tokio::time::Duration {
        self.idle_duration
    }

    /// Get the total number of successfully exported rows.
    #[must_use]
    pub fn successful_row_count(&self) -> f64 {
        self.successful_row_count
    }

    /// Get the total number of successfully exported batches.
    #[must_use]
    pub fn successful_batch_count(&self) -> f64 {
        self.successful_batch_count
    }

    /// Get the total number of successfully exported messages.
    #[must_use]
    pub fn successful_msg_count(&self) -> f64 {
        self.successful_msg_count
    }

    /// Get the total number of rows that failed to export.
    #[must_use]
    pub fn failed_row_count(&self) -> f64 {
        self.failed_row_count
    }

    /// Get the total number of batches that failed to export.
    #[must_use]
    pub fn failed_batch_count(&self) -> f64 {
        self.failed_batch_count
    }

    /// Get the total number of messages that failed to export.
    #[must_use]
    pub fn failed_msg_count(&self) -> f64 {
        self.failed_msg_count
    }

    /// Get the average client latency in seconds.
    #[must_use]
    pub fn average_client_latency_secs(&self) -> f64 {
        self.average_client_latency_secs
    }

    /// Get the total number of latency requests recorded.
    #[must_use]
    pub fn latency_request_count(&self) -> f64 {
        self.latency_request_count
    }

    /// Record that a message was received.
    /// Updates idle time calculations based on current in-flight exports.
    pub fn message_received(&mut self, in_flight_exports: usize) {
        self.update_idle(in_flight_exports)
    }

    /// Update idle duration based on time since last message.
    /// Only accumulates idle time if duration > threshold and no exports are in flight.
    pub fn update_idle(&mut self, in_flight_exports: usize) {
        let idle_duration =
            tokio::time::Instant::now().duration_since(self.last_message_received_at);
        if idle_duration.as_secs_f64() > IDLE_THRESHOLD_SECS && in_flight_exports == 0 {
            self.idle_duration += idle_duration;
        }

        self.last_message_received_at = tokio::time::Instant::now();
    }

    /// Calculate total active (non-idle) duration in seconds.
    pub fn get_active_duration_secs(&mut self, in_flight_exports: usize) -> f64 {
        self.update_idle(in_flight_exports);
        self.processing_started_at.elapsed().as_secs_f64() - self.idle_duration.as_secs_f64()
    }

    /// Get total accumulated idle duration in seconds.
    #[must_use]
    pub fn get_idle_duration_secs(&self) -> f64 {
        self.idle_duration.as_secs_f64()
    }

    /// Increment successful row count.
    pub fn add_rows(&mut self, row_count: f64) {
        self.successful_row_count += row_count;
    }

    /// Increment successful batch count.
    pub fn add_batch(&mut self) {
        self.successful_batch_count += 1.0;
    }

    /// Increment successful message count.
    pub fn add_messages(&mut self, msg_count: f64) {
        self.successful_msg_count += msg_count;
    }

    /// Increment failed row count.
    pub fn add_failed_rows(&mut self, row_count: f64) {
        self.failed_row_count += row_count;
    }

    /// Increment failed batch count.
    pub fn add_failed_batch(&mut self) {
        self.failed_batch_count += 1.0;
    }

    /// Increment failed message count.
    pub fn add_failed_messages(&mut self, msg_count: f64) {
        self.failed_msg_count += msg_count;
    }

    /// Update average client latency with a new measurement.
    /// Uses a running average calculation.
    pub fn add_client_latency(&mut self, latency_secs: f64) {
        self.latency_request_count += 1.0;
        self.average_client_latency_secs = ((self.average_client_latency_secs
            * (self.latency_request_count - 1.0))
            + latency_secs)
            / self.latency_request_count;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_stats_initialization() {
        let stats = AzureMonitorExporterStats::new();
        assert_eq!(stats.successful_row_count(), 0.0);
        assert_eq!(stats.failed_row_count(), 0.0);
        assert_eq!(stats.average_client_latency_secs(), 0.0);
        assert_eq!(stats.idle_duration(), Duration::ZERO);
    }

    #[test]
    fn test_stats_counters() {
        let mut stats = AzureMonitorExporterStats::new();

        stats.add_rows(100.0);
        stats.add_batch();
        stats.add_messages(50.0);

        assert_eq!(stats.successful_row_count(), 100.0);
        assert_eq!(stats.successful_batch_count(), 1.0);
        assert_eq!(stats.successful_msg_count(), 50.0);

        stats.add_failed_rows(10.0);
        stats.add_failed_batch();
        stats.add_failed_messages(5.0);

        assert_eq!(stats.failed_row_count(), 10.0);
        assert_eq!(stats.failed_batch_count(), 1.0);
        assert_eq!(stats.failed_msg_count(), 5.0);
    }

    #[test]
    fn test_latency_calculation() {
        let mut stats = AzureMonitorExporterStats::new();

        // First request: 1.0s
        stats.add_client_latency(1.0);
        assert_eq!(stats.average_client_latency_secs(), 1.0);
        assert_eq!(stats.latency_request_count(), 1.0);

        // Second request: 2.0s -> avg 1.5s
        stats.add_client_latency(2.0);
        assert_eq!(stats.average_client_latency_secs(), 1.5);
        assert_eq!(stats.latency_request_count(), 2.0);

        // Third request: 0.0s -> avg 1.0s
        stats.add_client_latency(0.0);
        assert_eq!(stats.average_client_latency_secs(), 1.0);
        assert_eq!(stats.latency_request_count(), 3.0);
    }

    #[tokio::test]
    async fn test_idle_tracking() {
        let mut stats = AzureMonitorExporterStats::new();

        // Simulate processing
        stats.message_received(0);

        // Wait a bit (simulated)
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Update idle with 0 in-flight (should count as idle)
        stats.update_idle(0);

        // Update idle with 1 in-flight (should NOT count as idle)
        stats.update_idle(1);

        // Verify getters work
        assert!(stats.started_at().elapsed().as_secs_f64() < 2.0);
        assert!(stats.last_message_received_at().elapsed().as_secs_f64() < 2.0);
        assert_eq!(
            stats.get_active_duration_secs(0),
            stats.processing_started_at.elapsed().as_secs_f64() - stats.get_idle_duration_secs()
        );
        assert_eq!(
            stats.get_idle_duration_secs(),
            stats.idle_duration().as_secs_f64()
        );
    }
}
