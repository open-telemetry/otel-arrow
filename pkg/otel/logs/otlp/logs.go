package otlp

import (
	"github.com/apache/arrow/go/v10/arrow"
	"go.opentelemetry.io/collector/pdata/plog"
)

type LogsIds struct {
	ResourceLogs *ResourceLogsIds
}

// LogsFrom creates a [plog.Logs] from the given Arrow Record.
func LogsFrom(record arrow.Record) (plog.Logs, error) {
	logs := plog.NewLogs()

	ids, err := SchemaToIds(record.Schema())
	if err != nil {
		return logs, err
	}

	// TODO there is probably two nested lists that could be replaced by a single list (traces, resource spans). This could simplify a future query layer.

	err = AppendResourceLogsInto(logs, record, ids)
	return logs, err
}

func SchemaToIds(schema *arrow.Schema) (*LogsIds, error) {
	resLogsIds, err := NewResourceLogsIds(schema)
	if err != nil {
		return nil, err
	}
	return &LogsIds{
		ResourceLogs: resLogsIds,
	}, nil
}
