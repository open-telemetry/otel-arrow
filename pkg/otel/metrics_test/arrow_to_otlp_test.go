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

package metrics_test

import (
	"testing"

	"github.com/davecgh/go-spew/spew"

	"otel-arrow-adapter/pkg/air"
	"otel-arrow-adapter/pkg/air/config"
	datagen2 "otel-arrow-adapter/pkg/datagen"
	"otel-arrow-adapter/pkg/otel/metrics"
)

func TestArrowToOtlpMetrics(t *testing.T) {
	t.Parallel()

	cfg := config.NewDefaultConfig()
	rr := air.NewRecordRepository(cfg)
	lg := datagen2.NewMetricsGenerator(datagen2.DefaultResourceAttributes(), datagen2.DefaultInstrumentationScopes())

	multivariateConf := metrics.MultivariateMetricsConfig{
		Metrics: make(map[string]string),
	}
	multivariateConf.Metrics["system.cpu.time"] = "state"
	multivariateConf.Metrics["system.memory.usage"] = "state"

	request := lg.Generate(1, 100)
	multiSchemaRecords, err := metrics.OtlpMetricsToArrowRecords(rr, request, &multivariateConf)
	if err != nil {
		t.Errorf("Unexpected error: %v", err)
	}

	for _, record := range multiSchemaRecords {
		req, err := metrics.ArrowRecordsToOtlpMetrics(record)
		if err != nil {
			t.Errorf("Unexpected error: %v", err)
		}
		spew.Dump(req)
	}
}
