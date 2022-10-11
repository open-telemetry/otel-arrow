/*
 * // Copyright The OpenTelemetry Authors
 * //
 * // Licensed under the Apache License, Version 2.0 (the "License");
 * // you may not use this file except in compliance with the License.
 * // You may obtain a copy of the License at
 * //
 * //       http://www.apache.org/licenses/LICENSE-2.0
 * //
 * // Unless required by applicable law or agreed to in writing, software
 * // distributed under the License is distributed on an "AS IS" BASIS,
 * // WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * // See the License for the specific language governing permissions and
 * // limitations under the License.
 *
 */

package batch_event_test

import (
	"fmt"
	"testing"

	v1 "github.com/lquerel/otel-arrow-adapter/api/collector/arrow/v1"
	"github.com/lquerel/otel-arrow-adapter/pkg/air"
	cfg "github.com/lquerel/otel-arrow-adapter/pkg/air/config"
	"github.com/lquerel/otel-arrow-adapter/pkg/air/rfield"
	"github.com/lquerel/otel-arrow-adapter/pkg/otel/batch_event"
)

func TestProducerConsumer(t *testing.T) {
	t.Parallel()

	producer := batch_event.NewProducer()
	consumer := batch_event.NewConsumer()
	config := cfg.NewUint8DefaultConfig()
	rr := air.NewRecordRepository(config)

	recordCount := 10
	tsValues := make([]int64, 0, recordCount)
	for i := 0; i < recordCount; i++ {
		tsValues = append(tsValues, int64(i))
	}
	for _, ts := range tsValues {
		rr.AddRecord(GenRecord(ts, int(ts%15), int(ts%2), int(ts)))
	}
	records, err := rr.BuildRecords()
	if err != nil {
		t.Fatal(err)
	}

	for _, record := range records {
		recordMesssage := batch_event.NewTraceMessage(record, v1.DeliveryType_BEST_EFFORT)
		batchEvent, err := producer.Produce(recordMesssage)
		if err != nil {
			t.Fatal(err)
		}
		recordMessages, err := consumer.Consume(batchEvent)
		if err != nil {
			t.Fatal(err)
		}
		if len(recordMessages) != 1 {
			t.Errorf("Expected 1 record message, got %d", len(recordMessages))
		}
		if recordMessages[0].Record().NumCols() != 5 {
			t.Errorf("Expected 5 columns, got %d", recordMessages[0].Record().NumCols())
		}
		if recordMessages[0].Record().NumRows() != int64(recordCount) {
			t.Errorf("Expected %d rows, got %d", recordCount, recordMessages[0].Record().NumRows())
		}
	}

	for _, ts := range tsValues {
		rr.AddRecord(GenRecord(ts, int(ts%15), int(ts%2), int(ts)))
	}
	records, err = rr.BuildRecords()
	if err != nil {
		t.Fatal(err)
	}

	for _, record := range records {
		recordMesssage := batch_event.NewTraceMessage(record, v1.DeliveryType_BEST_EFFORT)
		batchEvent, err := producer.Produce(recordMesssage)
		if err != nil {
			t.Fatal(err)
		}
		recordMessages, err := consumer.Consume(batchEvent)
		if err != nil {
			t.Fatal(err)
		}
		if len(recordMessages) != 1 {
			t.Errorf("Expected 1 record message, got %d", len(recordMessages))
		}
		if recordMessages[0].Record().NumCols() != 5 {
			t.Errorf("Expected 5 columns, got %d", recordMessages[0].Record().NumCols())
		}
		if recordMessages[0].Record().NumRows() != int64(recordCount) {
			t.Errorf("Expected %d rows, got %d", recordCount, recordMessages[0].Record().NumRows())
		}
	}
}

func GenRecord(ts int64, value_a, value_b, value_c int) *air.Record {
	record := air.NewRecord()
	record.I64Field("ts", ts)
	record.StringField("c", fmt.Sprintf("c_%d", value_c))
	record.StringField("a", fmt.Sprintf("a___%d", value_a))
	record.StringField("b", fmt.Sprintf("b__%d", value_b))
	record.StructField("d", rfield.Struct{
		Fields: []*rfield.Field{
			{Name: "a", Value: rfield.NewString(fmt.Sprintf("a_%d", value_a))},
			{Name: "b", Value: rfield.NewString(fmt.Sprintf("b_%d", value_b))},
			{Name: "c", Value: &rfield.List{Values: []rfield.Value{
				rfield.NewI64(1),
				rfield.NewI64(2),
				rfield.NewI64(3),
			}}},
			{Name: "d", Value: &rfield.List{Values: []rfield.Value{
				&rfield.Struct{Fields: []*rfield.Field{
					rfield.NewI64Field("a", 1),
					rfield.NewF64Field("b", 2.0),
					rfield.NewStringField("c", "3"),
				}},
				&rfield.Struct{Fields: []*rfield.Field{
					rfield.NewI64Field("a", 4),
					rfield.NewF64Field("b", 5.0),
					rfield.NewStringField("c", "6"),
				}},
				&rfield.Struct{Fields: []*rfield.Field{
					rfield.NewI64Field("a", 7),
					rfield.NewF64Field("b", 8.0),
					rfield.NewStringField("c", "9"),
				}},
			}}},
		},
	})
	return record
}
