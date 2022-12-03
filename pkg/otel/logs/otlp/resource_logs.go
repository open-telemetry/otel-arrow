package otlp

import (
	"github.com/apache/arrow/go/v10/arrow"
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
	id, dt, err := arrowutils.ListOfStructsFieldIDFromSchema(schema, constants.RESOURCE_LOGS)
	if err != nil {
		return nil, err
	}

	schemaId, _, err := arrowutils.FieldIDFromStruct(dt, constants.SCHEMA_URL)
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
		arrowResLogs, err := arrowutils.ListOfStructsFromRecordBis(record, ids.ResourceLogs.Id, traceIdx)
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
