// Copyright The OpenTelemetry Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package benchmark

import (
	"testing"
)

func TestSummary(t *testing.T) {
	t.Parallel()

	metric := NewMetric()

	// Sample
	metric.Record(3.0)
	metric.Record(2.0)
	metric.Record(5.0)
	metric.Record(1.0)
	metric.Record(4.0)

	summary := metric.ComputeSummary()

	if summary.Min != 1.0 {
		t.Errorf("expected min to be 1.0, got %f", summary.Min)
	}
	if summary.Max != 5.0 {
		t.Errorf("expected max to be 5.0, got %f", summary.Max)
	}
	if summary.Mean != 3.0 {
		t.Errorf("expected mean to be 3.0, got %f", summary.Mean)
	}

	if summary.Stddev != 1.5811388300841898 {
		t.Errorf("expected stddev to be 1.5811388300841898, got %f", summary.Stddev)
	}

	if summary.P50 != 3.0 {
		t.Errorf("expected p50 to be 3.0, got %f", summary.P50)
	}
	if summary.P90 != 4.6 {
		t.Errorf("expected p90 to be 4.6, got %f", summary.P95)
	}
	if summary.P95 != 4.8 {
		t.Errorf("expected p95 to be 4.8, got %f", summary.P95)
	}
	if summary.P99 != 4.96 {
		t.Errorf("expected p99 to be 4.96, got %f", summary.P95)
	}

	if len(summary.Values) != 5 {
		t.Errorf("expected len(values) to be 5, got %d", len(summary.Values))
	}
	if summary.Values[0] != 1.0 {
		t.Errorf("expected values[0] to be 1.0, got %f", summary.Values[0])
	}
	if summary.Values[1] != 2.0 {
		t.Errorf("expected values[1] to be 2.0, got %f", summary.Values[0])
	}
	if summary.Values[2] != 3.0 {
		t.Errorf("expected values[2] to be 3.0, got %f", summary.Values[0])
	}
	if summary.Values[3] != 4.0 {
		t.Errorf("expected values[3] to be 4.0, got %f", summary.Values[0])
	}
	if summary.Values[4] != 5.0 {
		t.Errorf("expected values[4] to be 5.0, got %f", summary.Values[4])
	}
}
