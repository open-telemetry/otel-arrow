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

	"otel-arrow-adapter/pkg/air"
	"otel-arrow-adapter/pkg/air/config"
	datagen2 "otel-arrow-adapter/pkg/datagen"
	"otel-arrow-adapter/pkg/otel/metrics"
)

func TestOtlpMetricsToArrowEvents(t *testing.T) {
	t.Parallel()

	cfg := config.NewDefaultConfig()
	rr := air.NewRecordRepository(cfg)
	lg := datagen2.NewMetricsGenerator(datagen2.DefaultResourceAttributes(), datagen2.DefaultInstrumentationScopes())

	multivariateConf := metrics.MultivariateMetricsConfig{
		Metrics: make(map[string]string),
	}
	multivariateConf.Metrics["system.cpu.time"] = "state"
	multivariateConf.Metrics["system.memory.usage"] = "state"

	request := lg.Generate(10, 100)
	multiSchemaRecords, err := metrics.OtlpMetricsToArrowRecords(rr, request, &multivariateConf)
	if err != nil {
		t.Errorf("Unexpected error: %v", err)
	}
	if len(multiSchemaRecords) != 3 {
		t.Errorf("Expected 3 record, got %d", len(multiSchemaRecords))
	}
	for _, record := range multiSchemaRecords {
		schemaId := air.SchemaToId(record.Schema())
		switch schemaId {
		case "metrics:{system.cpu.load_average.1m:F64},resource:{attributes:{hostname:Dic<U8,Str>,ip:Dic<U8,Str>,status:I64,up:Bol,version:F64}},scope_metrics:{name:Dic<U8,Str>,version:Dic<U8,Str>},start_time_unix_nano:U64,time_unix_nano:U64":
			if record.NumCols() != 5 {
				t.Errorf("Expected 6 fields, got %d", record.NumCols())
			}
			if record.NumRows() != 10 {
				t.Errorf("Expected 10 rows, got %d", record.NumRows())
			}
		case "attributes:{cpu:I64},metrics:{system.cpu.time:{idle:F64,interrupt:F64,iowait:F64,system:F64,user:F64}},resource:{attributes:{hostname:Dic<U8,Str>,ip:Dic<U8,Str>,status:I64,up:Bol,version:F64}},scope_metrics:{name:Dic<U8,Str>,version:Dic<U8,Str>},start_time_unix_nano:U64,time_unix_nano:U64":
			if record.NumCols() != 6 {
				t.Errorf("Expected 5 fields, got %d", record.NumCols())
			}
			if record.NumRows() != 10 {
				t.Errorf("Expected 10 rows, got %d", record.NumRows())
			}
		case "metrics:{system.memory.usage:{free:I64,inactive:I64,used:I64}},resource:{attributes:{hostname:Dic<U8,Str>,ip:Dic<U8,Str>,status:I64,up:Bol,version:F64}},scope_metrics:{name:Dic<U8,Str>,version:Dic<U8,Str>},start_time_unix_nano:U64,time_unix_nano:U64":
			if record.NumCols() != 5 {
				t.Errorf("Expected 5 fields, got %d", record.NumCols())
			}
			if record.NumRows() != 10 {
				t.Errorf("Expected 10 rows, got %d", record.NumRows())
			}
		default:
			t.Errorf("Unexpected schemaId: %s", schemaId)
		}
	}
}
