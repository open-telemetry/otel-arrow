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

package otel_test

import (
	"bufio"
	"bytes"
	"testing"

	"github.com/apache/arrow/go/v9/arrow/ipc"
	"github.com/davecgh/go-spew/spew"

	"github.com/f5/otel-arrow-adapter/pkg/air/config"
	"github.com/f5/otel-arrow-adapter/pkg/datagen"
	"github.com/f5/otel-arrow-adapter/pkg/otel/traces"
)

func TestIPCWriter(t *testing.T) {
	//t.Skip("WIP")
	t.Parallel()

	cfg := config.NewUint8DefaultConfig()
	lg := datagen.NewTraceGenerator(datagen.DefaultResourceAttributes(), datagen.DefaultInstrumentationScopes())

	request := lg.Generate(10, 100)
	producer := traces.NewOtlpArrowProducerWith(cfg)
	records, err := producer.ProduceFrom(request)
	if err != nil {
		t.Errorf("Unexpected error: %v", err)
	}
	if len(records) != 1 {
		t.Errorf("Expected 1 record, got %d", len(records))
	}

	for _, record := range records {
		var b bytes.Buffer
		memWriter := bufio.NewWriter(&b)
		writer := ipc.NewWriter(memWriter, ipc.WithSchema(record.Schema()))
		_ = writer.Write(record)
		spew.Dump(b.Bytes())
		_ = memWriter.Flush()
		b.Reset()
		_ = writer.Write(record)
		spew.Dump(b.Bytes())
	}
}
