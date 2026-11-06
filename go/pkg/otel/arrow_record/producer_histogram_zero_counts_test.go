/*
 * Copyright The OpenTelemetry Authors
 * SPDX-License-Identifier: Apache-2.0
 */

package arrow_record

import (
	"testing"

	"github.com/apache/arrow-go/v18/arrow/memory"
	"github.com/stretchr/testify/require"
	"go.opentelemetry.io/collector/pdata/pmetric"

	"github.com/open-telemetry/otel-arrow/go/pkg/config"
)

// histogramWithBucketCounts builds a single-metric payload holding one
// explicit histogram data point with explicit bounds [1, 5, 10], the given
// bucket counts, and count equal to the sum of the bucket counts.
func histogramWithBucketCounts(bucketCounts []uint64) pmetric.Metrics {
	md := pmetric.NewMetrics()
	m := md.ResourceMetrics().AppendEmpty().ScopeMetrics().AppendEmpty().Metrics().AppendEmpty()
	m.SetName("test.latency")
	dp := m.SetEmptyHistogram().DataPoints().AppendEmpty()
	dp.ExplicitBounds().FromRaw([]float64{1, 5, 10})
	dp.BucketCounts().FromRaw(bucketCounts)
	var count uint64
	for _, bc := range bucketCounts {
		count += bc
	}
	dp.SetCount(count)
	return md
}

// exponentialHistogramWithZeroPositiveBuckets builds a single-metric payload
// holding one exponential histogram data point with count 0, zero_count 0,
// and a non-empty all-zero positive bucket_counts list.
func exponentialHistogramWithZeroPositiveBuckets() pmetric.Metrics {
	md := pmetric.NewMetrics()
	m := md.ResourceMetrics().AppendEmpty().ScopeMetrics().AppendEmpty().Metrics().AppendEmpty()
	m.SetName("test.exponential_latency")
	dp := m.SetEmptyExponentialHistogram().DataPoints().AppendEmpty()
	dp.SetCount(0)
	dp.SetZeroCount(0)
	positive := dp.Positive()
	positive.SetOffset(-1)
	positive.BucketCounts().FromRaw([]uint64{0, 0})
	return md
}

// Scenario: An explicit histogram data point whose bucket_counts list is
// non-empty but contains only zeros (count == 0) is encoded, as produced by
// pipeline processors like deltatocumulative.
// Guarantees: Encoding the batch succeeds without panicking in the Arrow
// schema derivation and without dropping the whole batch with an error.
func TestHistogramWithAllZeroBucketCountsEncodes(t *testing.T) {
	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	producer := NewProducerWithOptions(config.WithAllocator(pool))
	defer func() {
		require.NoError(t, producer.Close())
	}()

	md := histogramWithBucketCounts([]uint64{0, 0, 0, 0})
	batch, err := producer.BatchArrowRecordsFromMetrics(md)
	require.NoError(t, err)
	require.NotNil(t, batch)
}

// Scenario: An explicit histogram data point with at least one non-zero
// bucket count is encoded.
// Guarantees: The non-degenerate histogram encoding path keeps working after
// any change to the optional list-field schema handling.
func TestHistogramWithNonZeroBucketCountsEncodes(t *testing.T) {
	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	producer := NewProducerWithOptions(config.WithAllocator(pool))
	defer func() {
		require.NoError(t, producer.Close())
	}()

	md := histogramWithBucketCounts([]uint64{0, 3, 1, 0})
	batch, err := producer.BatchArrowRecordsFromMetrics(md)
	require.NoError(t, err)
	require.NotNil(t, batch)
}

// Scenario: A zero-count explicit histogram data point is encoded and
// decoded back into pdata metrics.
// Guarantees: The data point survives the round trip with its name, type,
// count and explicit bounds preserved, and with no bucket count decoded as
// a non-zero value.
func TestHistogramWithAllZeroBucketCountsRoundTrips(t *testing.T) {
	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	producer := NewProducerWithOptions(config.WithAllocator(pool))
	defer func() {
		require.NoError(t, producer.Close())
	}()
	consumer := NewConsumer()
	defer func() {
		require.NoError(t, consumer.Close())
	}()

	md := histogramWithBucketCounts([]uint64{0, 0, 0, 0})
	batch, err := producer.BatchArrowRecordsFromMetrics(md)
	require.NoError(t, err)
	require.NotNil(t, batch)

	received, err := consumer.MetricsFrom(batch)
	require.NoError(t, err)
	require.Equal(t, 1, len(received))

	decoded := received[0]
	require.Equal(t, 1, decoded.ResourceMetrics().Len())
	hist := decoded.ResourceMetrics().At(0).ScopeMetrics().At(0).Metrics().At(0)
	require.Equal(t, pmetric.MetricTypeHistogram, hist.Type())
	require.Equal(t, "test.latency", hist.Name())
	dp := hist.Histogram().DataPoints().At(0)
	require.Equal(t, uint64(0), dp.Count())
	require.Equal(t, []float64{1, 5, 10}, dp.ExplicitBounds().AsRaw())
	// The bucket structure is preserved: the four zero bucket counts are
	// decoded as four zero bucket counts, not as a truncated or empty list.
	require.Equal(t, 4, dp.BucketCounts().Len())
	for i := 0; i < dp.BucketCounts().Len(); i++ {
		require.Equal(t, uint64(0), dp.BucketCounts().At(i))
	}
}

// Scenario: An exponential histogram data point with a non-empty all-zero
// positive bucket_counts list (count == 0, zero_count == 0) is encoded.
// Guarantees: The all-zero uint64 list handling that protects explicit
// histograms also protects exponential histograms, whose bucket_counts is
// the same optional uint64 list field.
func TestExponentialHistogramWithAllZeroBucketCountsEncodes(t *testing.T) {
	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)

	producer := NewProducerWithOptions(config.WithAllocator(pool))
	defer func() {
		require.NoError(t, producer.Close())
	}()

	md := exponentialHistogramWithZeroPositiveBuckets()
	batch, err := producer.BatchArrowRecordsFromMetrics(md)
	require.NoError(t, err)
	require.NotNil(t, batch)
}
