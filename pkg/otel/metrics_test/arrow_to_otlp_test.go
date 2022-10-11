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

	"github.com/lquerel/otel-arrow-adapter/pkg/air"
	"github.com/lquerel/otel-arrow-adapter/pkg/air/config"
	"github.com/lquerel/otel-arrow-adapter/pkg/datagen"
	"github.com/lquerel/otel-arrow-adapter/pkg/otel/metrics"
	"github.com/lquerel/otel-arrow-adapter/pkg/otel/oteltest"
)

func TestSystemCpuTimeConversion(t *testing.T) {
	t.Parallel()

	cfg := config.NewUint8DefaultConfig()
	rr := air.NewRecordRepository(cfg)
	lg := datagen.NewMetricsGenerator(datagen.DefaultResourceAttributes(), datagen.DefaultInstrumentationScopes())

	multivariateConf := metrics.MultivariateMetricsConfig{
		Metrics: make(map[string]string),
	}
	multivariateConf.Metrics["system.cpu.time"] = "state"
	multivariateConf.Metrics["system.memory.usage"] = "state"

	request := lg.GenerateSystemCpuTime(1, 100)

	multiSchemaRecords, err := metrics.OtlpMetricsToArrowRecords(rr, request, &multivariateConf, cfg)
	if err != nil {
		t.Errorf("Unexpected error: %v", err)
	}

	for _, record := range multiSchemaRecords {
		req, err := metrics.ArrowRecordsToOtlpMetrics(record)
		if err != nil {
			t.Errorf("Unexpected error: %v", err)
		}
		if diff := oteltest.DiffMetrics(request, req); diff != "" {
			t.Error("Unexpected diff: ", diff)
		}
	}
}

func TestSystemMemoryUsageConversion(t *testing.T) {
	t.Parallel()

	cfg := config.NewUint8DefaultConfig()
	rr := air.NewRecordRepository(cfg)
	lg := datagen.NewMetricsGenerator(datagen.DefaultResourceAttributes(), datagen.DefaultInstrumentationScopes())

	multivariateConf := metrics.MultivariateMetricsConfig{
		Metrics: make(map[string]string),
	}
	multivariateConf.Metrics["system.cpu.time"] = "state"
	multivariateConf.Metrics["system.memory.usage"] = "state"

	request := lg.GenerateSystemMemoryUsage(1, 100)

	multiSchemaRecords, err := metrics.OtlpMetricsToArrowRecords(rr, request, &multivariateConf, cfg)
	if err != nil {
		t.Errorf("Unexpected error: %v", err)
	}

	for _, record := range multiSchemaRecords {
		req, err := metrics.ArrowRecordsToOtlpMetrics(record)
		if err != nil {
			t.Errorf("Unexpected error: %v", err)
		}
		if diff := oteltest.DiffMetrics(request, req); diff != "" {
			t.Error("Unexpected diff: ", diff)
		}
	}
}

func TestSystemCpuLoadAverage1mConversion(t *testing.T) {
	t.Parallel()

	cfg := config.NewUint8DefaultConfig()
	rr := air.NewRecordRepository(cfg)
	lg := datagen.NewMetricsGenerator(datagen.DefaultResourceAttributes(), datagen.DefaultInstrumentationScopes())

	multivariateConf := metrics.MultivariateMetricsConfig{
		Metrics: make(map[string]string),
	}
	multivariateConf.Metrics["system.cpu.time"] = "state"
	multivariateConf.Metrics["system.memory.usage"] = "state"

	request := lg.GenerateSystemCpuLoadAverage1m(1, 100)

	multiSchemaRecords, err := metrics.OtlpMetricsToArrowRecords(rr, request, &multivariateConf, cfg)
	if err != nil {
		t.Errorf("Unexpected error: %v", err)
	}

	for _, record := range multiSchemaRecords {
		req, err := metrics.ArrowRecordsToOtlpMetrics(record)
		if err != nil {
			t.Errorf("Unexpected error: %v", err)
		}
		if diff := oteltest.DiffMetrics(request, req); diff != "" {
			t.Error("Unexpected diff: ", diff)
		}
	}
}
