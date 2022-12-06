package otlp

import (
	"github.com/apache/arrow/go/v10/arrow"
	"go.opentelemetry.io/collector/pdata/ptrace"
)

type TraceIds struct {
	ResourceSpans *ResourceSpansIds
}

// TracesFrom creates a [ptrace.Traces] from the given Arrow Record.
func TracesFrom(record arrow.Record) (ptrace.Traces, error) {
	traces := ptrace.NewTraces()

	traceIds, err := SchemaToIds(record.Schema())
	if err != nil {
		return traces, err
	}

	resSpansSlice := traces.ResourceSpans()
	resSpansCount := int(record.NumRows())
	resSpansSlice.EnsureCapacity(resSpansCount)

	// TODO there is probably two nested lists that could be replaced by a single list (traces, resource spans). This could simplify a future query layer.

	err = AppendResourceSpansInto(traces, record, traceIds)
	return traces, err
}

func SchemaToIds(schema *arrow.Schema) (*TraceIds, error) {
	resSpansIds, err := NewResourceSpansIds(schema)
	if err != nil {
		return nil, err
	}
	return &TraceIds{
		ResourceSpans: resSpansIds,
	}, nil
}
