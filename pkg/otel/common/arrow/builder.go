package arrow

import (
	"github.com/apache/arrow/go/v11/arrow"
	"go.opentelemetry.io/collector/pdata/plog"
	"go.opentelemetry.io/collector/pdata/pmetric"
	"go.opentelemetry.io/collector/pdata/ptrace"
)

type EntityBuilder[T pmetric.Metrics | plog.Logs | ptrace.Traces] interface {
	Append(T) error
	Build() (arrow.Record, error)
}
