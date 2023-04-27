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
	"github.com/apache/arrow/go/v12/arrow"
	"go.opentelemetry.io/collector/pdata/pcommon"
	"go.opentelemetry.io/collector/pdata/ptrace"

	arrowutils "github.com/f5/otel-arrow-adapter/pkg/arrow"
	otlp "github.com/f5/otel-arrow-adapter/pkg/otel/common/otlp"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
)

type ScopeSpansIds struct {
	Id                      int
	SchemaUrl               int
	ScopeIds                *otlp.ScopeIds
	SpansIds                *SpansIds
	SharedAttributeIds      *otlp.AttributeIds
	SharedEventAttributeIds *otlp.AttributeIds
	SharedLinkAttributeIds  *otlp.AttributeIds
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

	sharedAttrIds := otlp.NewSharedAttributeIds(scopeSpansDT)
	sharedEventAttrIds := otlp.NewSharedEventAttributeIds(scopeSpansDT)
	sharedLinkAttrIds := otlp.NewSharedLinkAttributeIds(scopeSpansDT)

	return &ScopeSpansIds{
		Id:                      id,
		SchemaUrl:               schemaId,
		ScopeIds:                scopeIds,
		SpansIds:                spansIds,
		SharedAttributeIds:      sharedAttrIds,
		SharedEventAttributeIds: sharedEventAttrIds,
		SharedLinkAttributeIds:  sharedLinkAttrIds,
	}, nil
}

func AppendScopeSpansInto(
	resSpans ptrace.ResourceSpans,
	arrowResSpans *arrowutils.ListOfStructs,
	resSpansIdx int,
	ids *ScopeSpansIds,
	relatedData *RelatedData,
) error {
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

		if err = otlp.UpdateScopeWith(scopeSpans.Scope(), arrowScopeSpans, scopeSpansIdx, ids.ScopeIds, relatedData.ScopeAttrMapStore); err != nil {
			return werror.Wrap(err)
		}

		schemaUrl, err := arrowScopeSpans.StringFieldByID(ids.SchemaUrl, scopeSpansIdx)
		if err != nil {
			return werror.Wrap(err)
		}
		scopeSpans.SetSchemaUrl(schemaUrl)

		sharedAttrs := pcommon.NewMap()
		if ids.SharedAttributeIds != nil {
			err = otlp.AppendAttributesInto(sharedAttrs, arrowScopeSpans.Array(), resSpansIdx, ids.SharedAttributeIds)
			if err != nil {
				return werror.Wrap(err)
			}
		}

		sharedEventAttrs := pcommon.NewMap()
		if ids.SharedEventAttributeIds != nil {
			err = otlp.AppendAttributesInto(sharedEventAttrs, arrowScopeSpans.Array(), resSpansIdx, ids.SharedEventAttributeIds)
			if err != nil {
				return werror.Wrap(err)
			}
		}

		sharedLinkAttrs := pcommon.NewMap()
		if ids.SharedLinkAttributeIds != nil {
			err = otlp.AppendAttributesInto(sharedLinkAttrs, arrowScopeSpans.Array(), resSpansIdx, ids.SharedLinkAttributeIds)
			if err != nil {
				return werror.Wrap(err)
			}
		}

		arrowSpans, err := arrowScopeSpans.ListOfStructsById(scopeSpansIdx, ids.SpansIds.SpansID)
		if err != nil {
			return werror.Wrap(err)
		}
		spansSlice := scopeSpans.Spans()
		spansSlice.EnsureCapacity(arrowSpans.End() - arrowSpans.Start())
		for entityIdx := arrowSpans.Start(); entityIdx < arrowSpans.End(); entityIdx++ {
			err = AppendSpanInto(spansSlice, arrowSpans, entityIdx, ids.SpansIds, sharedAttrs, sharedEventAttrs, sharedLinkAttrs, relatedData)
			if err != nil {
				return werror.Wrap(err)
			}
		}
	}

	return nil
}
