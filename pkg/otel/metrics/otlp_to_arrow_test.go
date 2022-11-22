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

package metrics

import (
	"math/rand"
	"testing"

	"github.com/f5/otel-arrow-adapter/pkg/air"
	"github.com/f5/otel-arrow-adapter/pkg/air/config"
	"github.com/f5/otel-arrow-adapter/pkg/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/datagen"
)

func TestOtlpMetricsToArrowRecords(t *testing.T) {
	t.Parallel()

	cfg := config.NewUint16DefaultConfig()
	rr := air.NewRecordRepository(cfg)
	entropy := datagen.NewTestEntropy(int64(rand.Uint64())) //nolint:gosec // only used for testing

	lg := datagen.NewMetricsGenerator(entropy, entropy.NewStandardResourceAttributes(), entropy.NewStandardInstrumentationScopes())

	multivariateConf := MultivariateMetricsConfig{
		Metrics: make(map[string]string),
	}
	multivariateConf.Metrics["system.cpu.time"] = "state"
	multivariateConf.Metrics["system.memory.usage"] = "state"

	request := lg.Generate(10, 100)
	multiSchemaRecords, err := OtlpMetricsToArrowRecords(rr, request, &multivariateConf, cfg)
	if err != nil {
		t.Errorf("Unexpected error: %v", err)
	}
	if len(multiSchemaRecords) != 5 {
		t.Errorf("Expected 5 record, got %d", len(multiSchemaRecords))
	}
	for _, record := range multiSchemaRecords {
		schemaId := arrow.SchemaToID(record.Schema())
		switch schemaId {
		case "metrics:{system.cpu.load_average.1m:F64},resource:{attributes:{hostname:Dic<U16,Str>,ip:Dic<U16,Str>,status:I64,up:Bol,version:F64}},scope_metrics:{name:Dic<U16,Str>,version:Dic<U16,Str>},start_time_unix_nano:U64,time_unix_nano:U64":
			if record.NumCols() != 5 {
				t.Errorf("Expected 6 fields, got %d", record.NumCols())
			}
			if record.NumRows() != 10 {
				t.Errorf("Expected 10 rows, got %d", record.NumRows())
			}
		case "attributes:{cpu:I64},metrics:{system.cpu.time:{idle:F64,interrupt:F64,iowait:F64,system:F64,user:F64}},resource:{attributes:{hostname:Dic<U16,Str>,ip:Dic<U16,Str>,status:I64,up:Bol,version:F64}},scope_metrics:{name:Dic<U16,Str>,version:Dic<U16,Str>},start_time_unix_nano:U64,time_unix_nano:U64":
			if record.NumCols() != 6 {
				t.Errorf("Expected 5 fields, got %d", record.NumCols())
			}
			if record.NumRows() != 10 {
				t.Errorf("Expected 10 rows, got %d", record.NumRows())
			}
		case "metrics:{system.memory.usage:{free:I64,inactive:I64,used:I64}},resource:{attributes:{hostname:Dic<U16,Str>,ip:Dic<U16,Str>,status:I64,up:Bol,version:F64}},scope_metrics:{name:Dic<U16,Str>,version:Dic<U16,Str>},start_time_unix_nano:U64,time_unix_nano:U64":
			if record.NumCols() != 5 {
				t.Errorf("Expected 5 fields, got %d", record.NumCols())
			}
			if record.NumRows() != 10 {
				t.Errorf("Expected 10 rows, got %d", record.NumRows())
			}
		case "flags:U32,histogram_fake.histogram:{bucket_counts:[U64],count:U64,explicit_bounds:[F64],max:F64,min:F64,sum:F64},resource:{attributes:{hostname:Dic<U16,Str>,ip:Dic<U16,Str>,status:I64,up:Bol,version:F64}},scope_metrics:{name:Dic<U16,Str>,version:Dic<U16,Str>},start_time_unix_nano:U64,time_unix_nano:U64":
			if record.NumCols() != 6 {
				t.Errorf("Expected 6 fields, got %d", record.NumCols())
			}
			if record.NumRows() != 100 {
				t.Errorf("Expected 100 rows, got %d", record.NumRows())
			}
		case "exp_histogram_fake.exp_histogram:{count:U64,max:F64,min:F64,negative:{bucket_counts:[U64],offset:I32},positive:{bucket_counts:[U64],offset:I32},scale:I32,sum:F64,zero_count:U64},flags:U32,resource:{attributes:{hostname:Dic<U16,Str>,ip:Dic<U16,Str>,status:I64,up:Bol,version:F64}},scope_metrics:{name:Dic<U16,Str>,version:Dic<U16,Str>},start_time_unix_nano:U64,time_unix_nano:U64":
			if record.NumCols() != 6 {
				t.Errorf("Expected 6 fields, got %d", record.NumCols())
			}
			if record.NumRows() != 100 {
				t.Errorf("Expected 100 rows, got %d", record.NumRows())
			}
		default:
			t.Errorf("Unexpected schemaId: %s", schemaId)
		}
	}
}
