package memorylimiterextension // import "go.opentelemetry.io/collector/extension/memorylimiterextension"

import (
	"time"
)

type Config struct {
	MemoryLimitMiB uint32 `mapstructure:"memory_limit_mib"`
	// Timeout is the amount of time to wait for request to be accepted.
	Timeout        time.Duration `mapstructure:timeout`
}
