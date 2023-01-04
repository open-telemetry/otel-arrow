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
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/otlp"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"
)

type ResourceLogsIds struct {
	Id        int
	Resource  *otlp.ResourceIds
	SchemaUrl int
	ScopeLogs *ScopeLogsIds
}

func NewResourceLogsIds(schema *arrow.Schema) (*ResourceLogsIds, error) {
	id, dt, err := arrowutils.ListOfStructsFieldIDFromSchema(schema, constants.ResourceLogs)
	if err != nil {
		return nil, err
	}

	schemaId, _, err := arrowutils.FieldIDFromStruct(dt, constants.SchemaUrl)
	if err != nil {
		return nil, err
	}

	scopeLogsIds, err := NewScopeLogsIds(dt)
	if err != nil {
		return nil, err
	}

	resourceIds, err := otlp.NewResourceIds(dt)
	if err != nil {
		return nil, err
	}

	return &ResourceLogsIds{
		Id:        id,
		Resource:  resourceIds,
		SchemaUrl: schemaId,
		ScopeLogs: scopeLogsIds,
	}, nil
}

func AppendResourceLogsInto(logs plog.Logs, record arrow.Record, ids *LogsIds) error {
	resLogsSlice := logs.ResourceLogs()
	resLogsCount := int(record.NumRows())

	for traceIdx := 0; traceIdx < resLogsCount; traceIdx++ {
		arrowResLogs, err := arrowutils.ListOfStructsFromRecord(record, ids.ResourceLogs.Id, traceIdx)
		if err != nil {
			return err
		}
		resLogsSlice.EnsureCapacity(resLogsSlice.Len() + arrowResLogs.End() - arrowResLogs.Start())

		for resLogsIdx := arrowResLogs.Start(); resLogsIdx < arrowResLogs.End(); resLogsIdx++ {
			resLogs := resLogsSlice.AppendEmpty()

			if err = otlp.UpdateResourceWith(resLogs.Resource(), arrowResLogs, resLogsIdx, ids.ResourceLogs.Resource); err != nil {
				return err
			}

			schemaUrl, err := arrowResLogs.StringFieldByID(ids.ResourceLogs.SchemaUrl, resLogsIdx)
			if err != nil {
				return err
			}
			resLogs.SetSchemaUrl(schemaUrl)

			err = AppendScopeLogsInto(resLogs, arrowResLogs, resLogsIdx, ids.ResourceLogs.ScopeLogs)
			if err != nil {
				return err
			}
		}
	}

	return nil
}
