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

package batch_event

import (
	"github.com/apache/arrow/go/v9/arrow"

	v1 "otel-arrow-adapter/api/go.opentelemetry.io/proto/otlp/collector/events/v1"
	"otel-arrow-adapter/pkg/air"
)

type InternalBatchEvent struct {
	batchId      string
	subStreamId  string
	recordType   v1.OtlpArrowPayloadType
	record       arrow.Record
	deliveryType v1.DeliveryType
}

func NewBatchEventOfMetrics(record arrow.Record, deliveryType v1.DeliveryType) *InternalBatchEvent {
	return &InternalBatchEvent{
		subStreamId:  air.SchemaToId(record.Schema()),
		recordType:   v1.OtlpArrowPayloadType_METRICS,
		record:       record,
		deliveryType: deliveryType,
	}
}

func NewBatchEventOfLogs(record arrow.Record, deliveryType v1.DeliveryType) *InternalBatchEvent {
	record.Schema()
	return &InternalBatchEvent{
		subStreamId:  air.SchemaToId(record.Schema()),
		recordType:   v1.OtlpArrowPayloadType_LOGS,
		record:       record,
		deliveryType: deliveryType,
	}
}

func NewBatchEventOfTraces(record arrow.Record, deliveryType v1.DeliveryType) *InternalBatchEvent {
	return &InternalBatchEvent{
		subStreamId:  air.SchemaToId(record.Schema()),
		recordType:   v1.OtlpArrowPayloadType_LOGS,
		record:       record,
		deliveryType: deliveryType,
	}
}

func (ibe *InternalBatchEvent) Record() arrow.Record {
	return ibe.record
}
