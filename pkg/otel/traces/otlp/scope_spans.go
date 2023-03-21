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
	otlp "github.com/f5/otel-arrow-adapter/pkg/otel/common/otlp"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
)

type ScopeSpansIds struct {
	Id        int
	SchemaUrl int
	ScopeIds  *otlp.ScopeIds
	SpansIds  *SpansIds
}

func NewScopeSpansIds(dt *arrow.StructType) (*ScopeSpansIds, error) {
	id, scopeSpansDT, err := arrowutils.ListOfStructsFieldIDFromStruct(dt, constants.ScopeSpans)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	schemaId, _ := arrowutils.FieldIDFromStruct(scopeSpansDT, constants.SchemaUrl)

	scopeIds, err := otlp.NewScopeIds(scopeSpansDT)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	spansIds, err := NewSpansIds(scopeSpansDT)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	return &ScopeSpansIds{
		Id:        id,
		SchemaUrl: schemaId,
		ScopeIds:  scopeIds,
		SpansIds:  spansIds,
	}, nil
}

func AppendScopeSpansInto(resSpans ptrace.ResourceSpans, arrowResSpans *arrowutils.ListOfStructs, resSpansIdx int, ids *ScopeSpansIds) error {
	if arrowResSpans == nil {
		return nil
	}

	arrowScopeSpans, err := arrowResSpans.ListOfStructsById(resSpansIdx, ids.Id)
	if err != nil {
		return werror.Wrap(err)
	}

	if arrowScopeSpans == nil {
		// No scope spans
		return nil
	}

	scopeSpansSlice := resSpans.ScopeSpans()
	scopeSpansSlice.EnsureCapacity(arrowScopeSpans.End() - arrowResSpans.Start())

	for scopeSpansIdx := arrowScopeSpans.Start(); scopeSpansIdx < arrowScopeSpans.End(); scopeSpansIdx++ {
		scopeSpans := scopeSpansSlice.AppendEmpty()

		if err = otlp.UpdateScopeWith(scopeSpans.Scope(), arrowScopeSpans, scopeSpansIdx, ids.ScopeIds); err != nil {
			return werror.Wrap(err)
		}

		schemaUrl, err := arrowScopeSpans.StringFieldByID(ids.SchemaUrl, scopeSpansIdx)
		if err != nil {
			return werror.Wrap(err)
		}
		scopeSpans.SetSchemaUrl(schemaUrl)

		arrowSpans, err := arrowScopeSpans.ListOfStructsById(scopeSpansIdx, ids.SpansIds.Id)
		if err != nil {
			return werror.Wrap(err)
		}
		spansSlice := scopeSpans.Spans()
		spansSlice.EnsureCapacity(arrowSpans.End() - arrowSpans.Start())
		for entityIdx := arrowSpans.Start(); entityIdx < arrowSpans.End(); entityIdx++ {
			err = AppendSpanInto(spansSlice, arrowSpans, entityIdx, ids.SpansIds)
			if err != nil {
				return werror.Wrap(err)
			}
		}
	}

	return nil
}
