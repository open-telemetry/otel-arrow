package otlp

import (
	"go.opentelemetry.io/collector/pdata/pcommon"
)

type SharedData struct {
	StartTime  *pcommon.Timestamp
	Time       *pcommon.Timestamp
	Attributes pcommon.Map
}
