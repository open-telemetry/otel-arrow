package metrics

import (
	"encoding/json"
	"math/rand"
	"testing"

	"github.com/apache/arrow/go/v11/arrow/memory"
	"go.opentelemetry.io/collector/pdata/pmetric/pmetricotlp"

	"github.com/f5/otel-arrow-adapter/pkg/datagen"
	"github.com/f5/otel-arrow-adapter/pkg/otel/assert"
	"github.com/f5/otel-arrow-adapter/pkg/otel/metrics/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/metrics/otlp"
)

// TestBackAndForthConversion tests the conversion of OTLP metrics to Arrow and back to OTLP.
// The initial OTLP metrics are generated from a synthetic dataset.
// This test is based on the JSON serialization of the initial generated OTLP metrics compared to the JSON serialization
// of the OTLP metrics generated from the Arrow records.
func TestBackAndForthConversion(t *testing.T) {
	t.Parallel()

	entropy := datagen.NewTestEntropy(int64(rand.Uint64()))

	dg := datagen.NewDataGenerator(entropy, entropy.NewStandardResourceAttributes(), entropy.NewStandardInstrumentationScopes()).
		WithConfig(datagen.Config{
			ProbMetricDescription: 0.5,
			ProbMetricUnit:        0.5,
			ProbHistogramHasSum:   0.5,
			ProbHistogramHasMin:   0.5,
			ProbHistogramHasMax:   0.5,
		})
	metricsGen := datagen.NewMetricsGeneratorWithDataGenerator(dg)

	// Generate a random OTLP metrics request.
	expectedRequest := pmetricotlp.NewExportRequestFromMetrics(metricsGen.Generate(100, 100))

	// Convert the OTLP metrics request to Arrow.
	pool := memory.NewCheckedAllocator(memory.NewGoAllocator())
	defer pool.AssertSize(t, 0)
	lb := arrow.NewMetricsBuilder(pool)
	err := lb.Append(expectedRequest.Metrics())
	if err != nil {
		t.Fatal(err)
	}
	record, err := lb.Build()
	if err != nil {
		t.Fatal(err)
	}
	defer record.Release()

	// Convert the Arrow records back to OTLP.
	metrics, err := otlp.MetricsFrom(record)
	if err != nil {
		t.Fatal(err)
	}
	assert.Equiv(t, []json.Marshaler{expectedRequest}, []json.Marshaler{pmetricotlp.NewExportRequestFromMetrics(metrics)})
}
