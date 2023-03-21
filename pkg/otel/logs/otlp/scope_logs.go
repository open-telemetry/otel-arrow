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
	"go.opentelemetry.io/collector/pdata/plog"

	arrowutils "github.com/f5/otel-arrow-adapter/pkg/arrow"
	otlp "github.com/f5/otel-arrow-adapter/pkg/otel/common/otlp"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
)

type ScopeLogsIds struct {
	Id           int
	SchemaUrl    int
	ScopeIds     *otlp.ScopeIds
	LogRecordIds *LogRecordIds
}

func NewScopeLogsIds(dt *arrow.StructType) (*ScopeLogsIds, error) {
	if dt == nil {
		return nil, nil
	}

	id, scopeSpansDT, err := arrowutils.ListOfStructsFieldIDFromStruct(dt, constants.ScopeLogs)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	schemaId, _ := arrowutils.FieldIDFromStruct(scopeSpansDT, constants.SchemaUrl)

	scopeIds, err := otlp.NewScopeIds(scopeSpansDT)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	spansIds, err := NewLogRecordIds(scopeSpansDT)
	if err != nil {
		return nil, werror.Wrap(err)
	}

	return &ScopeLogsIds{
		Id:           id,
		SchemaUrl:    schemaId,
		ScopeIds:     scopeIds,
		LogRecordIds: spansIds,
	}, nil
}

func AppendScopeLogsInto(resLogs plog.ResourceLogs, arrowResLogs *arrowutils.ListOfStructs, resLogsIdx int, ids *ScopeLogsIds) error {
	arrowScopeLogs, err := arrowResLogs.ListOfStructsById(resLogsIdx, ids.Id)
	if err != nil {
		return werror.Wrap(err)
	}
	scopeLogsSlice := resLogs.ScopeLogs()
	scopeLogsSlice.EnsureCapacity(arrowScopeLogs.End() - arrowResLogs.Start())

	for scopeLogsIdx := arrowScopeLogs.Start(); scopeLogsIdx < arrowScopeLogs.End(); scopeLogsIdx++ {
		scopeLogs := scopeLogsSlice.AppendEmpty()

		if err = otlp.UpdateScopeWith(scopeLogs.Scope(), arrowScopeLogs, scopeLogsIdx, ids.ScopeIds); err != nil {
			return werror.Wrap(err)
		}

		schemaUrl, err := arrowScopeLogs.StringFieldByID(ids.SchemaUrl, scopeLogsIdx)
		if err != nil {
			return werror.Wrap(err)
		}
		scopeLogs.SetSchemaUrl(schemaUrl)

		arrowLogs, err := arrowScopeLogs.ListOfStructsById(scopeLogsIdx, ids.LogRecordIds.Id)
		if err != nil {
			return werror.Wrap(err)
		}
		logsSlice := scopeLogs.LogRecords()
		logsSlice.EnsureCapacity(arrowLogs.End() - arrowLogs.Start())
		for logIdx := arrowLogs.Start(); logIdx < arrowLogs.End(); logIdx++ {
			err = AppendLogRecordInto(logsSlice, arrowLogs, logIdx, ids.LogRecordIds)
			if err != nil {
				return werror.Wrap(err)
			}
		}
	}

	return nil
}
