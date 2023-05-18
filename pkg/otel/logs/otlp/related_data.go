/*
 * Copyright The OpenTelemetry Authors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

package otlp

import (
	colarspb "github.com/f5/otel-arrow-adapter/api/experimental/arrow/v1"
	"github.com/f5/otel-arrow-adapter/pkg/otel"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/otlp"
	"github.com/f5/otel-arrow-adapter/pkg/record_message"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
)

// Infrastructure used to process related records.

type (
	RelatedData struct {
		LogRecordID           uint16
		ResAttrMapStore       *otlp.Attributes16Store
		ScopeAttrMapStore     *otlp.Attributes16Store
		LogRecordAttrMapStore *otlp.Attributes16Store
	}
)

func NewRelatedData() *RelatedData {
	return &RelatedData{
		ResAttrMapStore:       otlp.NewAttributes16Store(),
		ScopeAttrMapStore:     otlp.NewAttributes16Store(),
		LogRecordAttrMapStore: otlp.NewAttributes16Store(),
	}
}

func (r *RelatedData) LogRecordIDFromDelta(delta uint16) uint16 {
	r.LogRecordID += delta
	return r.LogRecordID
}

func RelatedDataFrom(records []*record_message.RecordMessage) (relatedData *RelatedData, logsRecord *record_message.RecordMessage, err error) {
	relatedData = NewRelatedData()

	// Create the attribute map stores for all the attribute records.
	for _, record := range records {
		switch record.PayloadType() {
		case colarspb.OtlpArrowPayloadType_RESOURCE_ATTRS:
			err = otlp.Attributes16StoreFrom(record.Record(), relatedData.ResAttrMapStore)
			if err != nil {
				return nil, nil, werror.Wrap(err)
			}
		case colarspb.OtlpArrowPayloadType_SCOPE_ATTRS:
			err = otlp.Attributes16StoreFrom(record.Record(), relatedData.ScopeAttrMapStore)
			if err != nil {
				return nil, nil, werror.Wrap(err)
			}
		case colarspb.OtlpArrowPayloadType_LOG_ATTRS:
			err = otlp.Attributes16StoreFrom(record.Record(), relatedData.LogRecordAttrMapStore)
			if err != nil {
				return nil, nil, werror.Wrap(err)
			}
		case colarspb.OtlpArrowPayloadType_LOGS:
			if logsRecord != nil {
				return nil, nil, werror.Wrap(otel.ErrMultipleTracesRecords)
			}
			logsRecord = record
		default:
			return nil, nil, werror.Wrap(otel.UnknownPayloadType)
		}
	}

	return
}
