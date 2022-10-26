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

package logs_test

import (
	"testing"

	"github.com/f5/otel-arrow-adapter/pkg/air"
	"github.com/f5/otel-arrow-adapter/pkg/air/config"
	"github.com/f5/otel-arrow-adapter/pkg/datagen"
	"github.com/f5/otel-arrow-adapter/pkg/otel/logs"
)

func TestOtlpLogsToArrowRecords(t *testing.T) {
	t.Parallel()

	cfg := config.NewUint8DefaultConfig()
	rr := air.NewRecordRepository(cfg)
	lg := datagen.NewLogsGenerator(datagen.DefaultResourceAttributes(), datagen.DefaultInstrumentationScopes())

	request := lg.Generate(10, 100)
	records, err := logs.OtlpLogsToArrowRecords(rr, request, cfg)
	if err != nil {
		t.Errorf("Unexpected error: %v", err)
	}
	if len(records) != 1 {
		t.Errorf("Expected 1 record, got %d", len(records))
	}
}
