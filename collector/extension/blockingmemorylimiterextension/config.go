package blockingmemorylimiterextension // import "github.com/open-telemetry/otel-arrow/collector/blockingmemorylimiterextension"

import (
	"time"
)

type Config struct {
	MemoryLimitMiB uint32 `mapstructure:"memory_limit_mib"`
	// Timeout is the amount of time to wait for request to be accepted.
	Timeout        time.Duration `mapstructure:timeout`
}