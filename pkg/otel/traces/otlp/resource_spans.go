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

package otlp

import (
	"github.com/apache/arrow/go/v11/arrow"
	"go.opentelemetry.io/collector/pdata/ptrace"

	arrowutils "github.com/f5/otel-arrow-adapter/pkg/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/otlp"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

type ResourceSpansIds struct {
	Id         int
	Resource   *otlp.ResourceIds
	SchemaUrl  int
	ScopeSpans *ScopeSpansIds
}

func NewResourceSpansIds(schema *arrow.Schema) (*ResourceSpansIds, error) {
	id, rsDT, err := arrowutils.ListOfStructsFieldIDFromSchema(schema, constants.ResourceSpans)
	if err != nil {
		return nil, err
	}

	schemaId, _, err := arrowutils.FieldIDFromStruct(rsDT, constants.SchemaUrl)
	if err != nil {
		return nil, err
	}

	scopeSpansIds, err := NewScopeSpansIds(rsDT)
	if err != nil {
		return nil, err
	}

	resourceIds, err := otlp.NewResourceIds(rsDT)
	if err != nil {
		return nil, err
	}

	return &ResourceSpansIds{
		Id:         id,
		Resource:   resourceIds,
		SchemaUrl:  schemaId,
		ScopeSpans: scopeSpansIds,
	}, nil
}

func AppendResourceSpansInto(traces ptrace.Traces, record arrow.Record, traceIds *TraceIds) error {
	resSpansSlice := traces.ResourceSpans()
	resSpansCount := int(record.NumRows())

	for traceIdx := 0; traceIdx < resSpansCount; traceIdx++ {
		arrowResEnts, err := arrowutils.ListOfStructsFromRecord(record, traceIds.ResourceSpans.Id, traceIdx)
		if err != nil {
			return err
		}
		resSpansSlice.EnsureCapacity(resSpansSlice.Len() + arrowResEnts.End() - arrowResEnts.Start())

		for resSpansIdx := arrowResEnts.Start(); resSpansIdx < arrowResEnts.End(); resSpansIdx++ {
			resSpans := resSpansSlice.AppendEmpty()

			if err = otlp.UpdateResourceWith(resSpans.Resource(), arrowResEnts, resSpansIdx, traceIds.ResourceSpans.Resource); err != nil {
				return err
			}

			schemaUrl, err := arrowResEnts.StringFieldByID(traceIds.ResourceSpans.SchemaUrl, resSpansIdx)
			if err != nil {
				return err
			}
			resSpans.SetSchemaUrl(schemaUrl)

			err = AppendScopeSpansInto(resSpans, arrowResEnts, resSpansIdx, traceIds.ResourceSpans.ScopeSpans)
			if err != nil {
				return err
			}
		}
	}

	return nil
}
