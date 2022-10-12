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

	v1 "github.com/lquerel/otel-arrow-adapter/api/collector/arrow/v1"
	"github.com/lquerel/otel-arrow-adapter/pkg/air"
)

type PayloadType = v1.OtlpArrowPayloadType

type RecordMessage struct {
	batchId      string
	subStreamId  string
	payloadType  PayloadType
	record       arrow.Record
	deliveryType v1.DeliveryType
}

func NewMetricsMessage(record arrow.Record, deliveryType v1.DeliveryType) *RecordMessage {
	return &RecordMessage{
		subStreamId:  air.SchemaToId(record.Schema()),
		payloadType:  v1.OtlpArrowPayloadType_METRICS,
		record:       record,
		deliveryType: deliveryType,
	}
}

func NewLogsMessage(record arrow.Record, deliveryType v1.DeliveryType) *RecordMessage {
	record.Schema()
	return &RecordMessage{
		subStreamId:  air.SchemaToId(record.Schema()),
		payloadType:  v1.OtlpArrowPayloadType_LOGS,
		record:       record,
		deliveryType: deliveryType,
	}
}

func NewTraceMessage(record arrow.Record, deliveryType v1.DeliveryType) *RecordMessage {
	return &RecordMessage{
		subStreamId:  air.SchemaToId(record.Schema()),
		payloadType:  v1.OtlpArrowPayloadType_SPANS,
		record:       record,
		deliveryType: deliveryType,
	}
}

func (ibe *RecordMessage) Record() arrow.Record {
	return ibe.record
}

func (ibe *RecordMessage) PayloadType() PayloadType {
	return ibe.payloadType
}
