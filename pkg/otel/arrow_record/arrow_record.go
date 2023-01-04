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

package arrow_record

import (
	"github.com/apache/arrow/go/v11/arrow"

	v1 "github.com/f5/otel-arrow-adapter/api/collector/arrow/v1"
	arrow2 "github.com/f5/otel-arrow-adapter/pkg/arrow"
)

type PayloadType = v1.OtlpArrowPayloadType

// RecordMessage wraps an Arrow Record with a set of metadata used to identify the batch, sub-stream, and few other
// properties.
type RecordMessage struct {
	batchId      string
	subStreamId  string
	payloadType  PayloadType
	record       arrow.Record
	deliveryType v1.DeliveryType
}

// NewMetricsMessage creates a reference to a new RecordMessage from a given Arrow Record representing a collection of
// metrics.
func NewMetricsMessage(record arrow.Record, deliveryType v1.DeliveryType) *RecordMessage {
	return &RecordMessage{
		subStreamId:  arrow2.SchemaToID(record.Schema()),
		payloadType:  v1.OtlpArrowPayloadType_METRICS,
		record:       record,
		deliveryType: deliveryType,
	}
}

// NewLogsMessage creates a reference to a new RecordMessage from a given Arrow Record representing a collection of
// logs.
func NewLogsMessage(record arrow.Record, deliveryType v1.DeliveryType) *RecordMessage {
	record.Schema()
	return &RecordMessage{
		subStreamId:  arrow2.SchemaToID(record.Schema()),
		payloadType:  v1.OtlpArrowPayloadType_LOGS,
		record:       record,
		deliveryType: deliveryType,
	}
}

// NewTraceMessage creates a reference to a new RecordMessage from a given Arrow Record representing a collection of
// traces.
func NewTraceMessage(record arrow.Record, deliveryType v1.DeliveryType) *RecordMessage {
	return &RecordMessage{
		subStreamId:  arrow2.SchemaToID(record.Schema()),
		payloadType:  v1.OtlpArrowPayloadType_SPANS,
		record:       record,
		deliveryType: deliveryType,
	}
}

// Record returns the Arrow Record associated with this RecordMessage.
func (ibe *RecordMessage) Record() arrow.Record {
	return ibe.record
}

// PayloadType returns the type of payload contained in this RecordMessage.
func (ibe *RecordMessage) PayloadType() PayloadType {
	return ibe.payloadType
}
