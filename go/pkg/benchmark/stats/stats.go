/*
 * Copyright The OpenTelemetry Authors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

package stats

import (
	"fmt"
	"math"
	"sort"
)

type Summary struct {
	Min    float64
	Max    float64
	Mean   float64
	Stddev float64
	P50    float64
	P90    float64
	P95    float64
	P99    float64
	Values []float64
}

type BatchSummary struct {
	BatchSize              int
	UncompressedSizeByte   *Summary
	CompressedSizeByte     *Summary
	OtlpArrowConversionSec *Summary
	ProcessingSec          *Summary
	SerializationSec       *Summary
	DeserializationSec     *Summary
	CompressionSec         *Summary
	DecompressionSec       *Summary
	TotalTimeSec           *Summary
	ProcessingResults      []string
	CpuMemUsage            *CpuMemUsage
	OtlpConversionSec      *Summary
}

type ProfilerResult struct {
	BenchName string
	Tags      string
	Summaries []BatchSummary
}

type Metric struct {
	values []float64
}

func NewMetric() *Metric {
	return &Metric{
		values: make([]float64, 0, 100),
	}
}

func (m *Metric) Record(value float64) {
	m.values = append(m.values, value)
}

func (m *Metric) ComputeSummary() *Summary {
	if len(m.values) == 0 {
		return &Summary{}
	}

	min := math.MaxFloat64
	max := -math.MaxFloat64
	sum := 0.0

	sort.Float64s(m.values)

	for _, value := range m.values {
		min = math.Min(min, value)
		max = math.Max(max, value)
		sum += value
	}

	mean := sum / float64(len(m.values))

	return &Summary{
		Min:    min,
		Max:    max,
		Mean:   mean,
		Stddev: m.Stddev(mean),
		P50:    m.Percentile(50.0),
		P90:    m.Percentile(90.0),
		P95:    m.Percentile(95.0),
		P99:    m.Percentile(99.0),
		Values: m.values,
	}
}

func (m *Metric) Var(mean float64) float64 {
	if len(m.values) < 2 {
		return 0.0
	}

	v := 0.0
	for _, value := range m.values {
		x := value - mean
		v += x * x
	}
	denom := float64(len(m.values) - 1)
	return v / denom
}

func (m *Metric) Stddev(mean float64) float64 {
	return math.Sqrt(m.Var(mean))
}

func (m *Metric) Percentile(pct float64) float64 {
	if len(m.values) == 0 {
		return 0.0
	}

	if len(m.values) == 1 {
		return m.values[0]
	}

	if pct < 0.0 {
		panic("percentile must be >= 0.0")
	}
	hundred := 100.0
	if pct > hundred {
		panic("percentile must be <= 100.0")
	}
	if pct == hundred {
		return m.values[len(m.values)-1]
	}
	length := float64(len(m.values) - 1)
	rank := (pct / hundred) * length
	lrank := math.Floor(rank)
	d := rank - lrank
	n := uint64(lrank)
	lo := m.values[n]
	hi := m.values[n+1]
	return lo + (hi-lo)*d
}

func (s *Summary) Total(maxIter uint64) float64 {
	sum := 0.0
	for _, value := range s.Values {
		sum += value
	}
	return sum / float64(maxIter)
}

func (s *Summary) ToString() string {
	return fmt.Sprintf("mean: %8.2f, min: %8.2f, max: %8.2f, std-dev: %8.2f, p50: %8.2f, p99: %8.2f",
		s.Mean,
		s.Min,
		s.Max,
		s.Stddev,
		s.P50,
		s.P99,
	)
}

// AddSummaries combines multiple Summaries by adding their metric values together.
// This method panics if the Summaries have different number of values.
func AddSummaries(summaries ...*Summary) *Summary {
	if len(summaries) == 0 {
		return nil
	}

	valueCount := len(summaries[0].Values)
	values := make([]float64, valueCount, valueCount)
	for _, summary := range summaries {
		if valueCount != len(summary.Values) {
			panic("summaries have different number of values")
		}
		for j, value := range summary.Values {
			values[j] += value
		}
	}

	metric := &Metric{
		values: values,
	}

	return metric.ComputeSummary()
}
