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

package trace_test

import (
	"testing"

	"github.com/davecgh/go-spew/spew"

	"otel-arrow-adapter/pkg/air"
	"otel-arrow-adapter/pkg/air/config"
	datagen2 "otel-arrow-adapter/pkg/datagen"
	"otel-arrow-adapter/pkg/otel/batch_event"
	"otel-arrow-adapter/pkg/otel/trace"
)

func TestOtlpTraceToArrowEvents(t *testing.T) {
	// t.Skip("TODO: implement")
	t.Parallel()

	cfg := config.NewDefaultConfig()
	rr := air.NewRecordRepository(cfg)
	lg := datagen2.NewTraceGenerator(datagen2.DefaultResourceAttributes(), datagen2.DefaultInstrumentationScope())

	request := lg.Generate(10, 100)
	evts, err := trace.OtlpTraceToEvents(rr, request)
	if err != nil {
		t.Errorf("Unexpected error: %v", err)
	}
	if len(evts) != 1 {
		t.Errorf("Expected 1 record, got %d", len(evts))
	}

	traceConsumer := batch_event.NewConsumer()
	for _, event := range evts {
		ibes, err := traceConsumer.Consume(event)
		if err != nil {
			t.Errorf("Unexpected error: %v", err)
		}
		spew.Dump(ibes)
	}
}
